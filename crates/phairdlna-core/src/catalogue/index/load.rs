//! The reading of a catalogue out of the index.
//!
//! What: the rows of the index and the catalogue they make.
//! Why: the catalogue of the last run is what the server serves while it reads
//!       the library again. The reader has a connection of its own and sees the
//!       state that was committed last (`FUN-008`, FUN-009).
//! Example: `load` reads the catalogue of a library out of its index.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;

use super::super::library::ContainerNode;
use super::super::{Catalogue, ContentItem, MediaKind, RootSummary, SubtitleResource};
use super::rows::{read_children, read_containers, read_objects, read_roots, read_update_id};
use super::rows::{ChildRow, ObjectRow};
use super::IndexError;

/// Reads the catalogue out of the index.
///
/// The answer is `None` when the index holds no catalogue: the library has then
/// not been read yet.
pub(super) fn load(reader: &mut Connection, path: &Path) -> Result<Option<Catalogue>, IndexError> {
    let transaction = reader
        .transaction()
        .map_err(|error| unusable(path, error))?;
    let objects = read_objects(&transaction).map_err(|error| unusable(path, error))?;
    if objects.is_empty() {
        return Ok(None);
    }
    let containers = read_containers(&transaction).map_err(|error| unusable(path, error))?;
    let children = read_children(&transaction).map_err(|error| unusable(path, error))?;
    let roots = read_roots(&transaction).map_err(|error| unusable(path, error))?;
    let (fingerprint, generation) =
        read_update_id(&transaction).map_err(|error| unusable(path, error))?;
    Ok(Some(into_catalogue(
        objects,
        containers,
        children,
        roots,
        fingerprint,
        generation,
    )))
}

/// Builds the catalogue out of the rows of the index.
fn into_catalogue(
    objects: Vec<ObjectRow>,
    containers: HashMap<String, u32>,
    children: HashMap<String, Vec<ChildRow>>,
    roots: Vec<RootSummary>,
    fingerprint: u64,
    generation: u32,
) -> Catalogue {
    let mut catalogue = Catalogue::default();
    catalogue.roots = roots;
    catalogue.fingerprint = fingerprint;
    catalogue.system_update_id = generation;
    catalogue.items = items_of(&objects);
    catalogue.files = files_of(&objects);
    catalogue.containers = containers_of(&objects, &containers, &children, &catalogue.items);
    catalogue
}

/// The items of the tree: the objects that are not containers.
fn items_of(objects: &[ObjectRow]) -> BTreeMap<String, ContentItem> {
    let subtitles: HashMap<&str, &ObjectRow> = objects
        .iter()
        .filter(|object| object.kind == MediaKind::Subtitle)
        .map(|object| (object.id.as_str(), object))
        .collect();
    let mut items = BTreeMap::new();
    for object in objects {
        if object.kind.is_container() {
            continue;
        }
        let mut item = ContentItem::new(
            object.id.clone(),
            object.parent_id.clone(),
            object.title.clone(),
            object.kind,
        );
        item.metadata.mime_type = object.mime.clone();
        item.metadata.size = object.size;
        item.metadata.modified = object.modified;
        item.metadata.duration = object.duration.map(Duration::from_secs);
        item.metadata.charset = object.charset.clone();
        item.metadata.subtitle = subtitle_of(object, &subtitles);
        items.insert(object.id.clone(), item);
    }
    items
}

/// The additional resource of a media file.
fn subtitle_of(
    object: &ObjectRow,
    subtitles: &HashMap<&str, &ObjectRow>,
) -> Option<SubtitleResource> {
    let id = object.subtitle_id.as_deref()?;
    let sidecar = subtitles.get(id)?;
    Some(SubtitleResource {
        id: id.to_owned(),
        mime_type: sidecar.mime.clone().unwrap_or_default(),
        charset: sidecar.charset.clone().unwrap_or_default(),
        size: sidecar.size,
    })
}

/// The files of the tree: the objects that carry a path.
fn files_of(objects: &[ObjectRow]) -> BTreeMap<String, PathBuf> {
    let mut files = BTreeMap::new();
    for object in objects {
        if let Some(path) = &object.path {
            files.insert(object.id.clone(), path.into());
        }
    }
    files
}

/// The containers of the tree, with their children.
fn containers_of(
    objects: &[ObjectRow],
    containers: &HashMap<String, u32>,
    children: &HashMap<String, Vec<ChildRow>>,
    items: &BTreeMap<String, ContentItem>,
) -> BTreeMap<String, ContainerNode> {
    let titles: HashMap<&str, &ObjectRow> = objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect();
    let mut built: BTreeMap<String, ContainerNode> = BTreeMap::new();
    for object in objects {
        if !object.kind.is_container() {
            continue;
        }
        built.insert(
            object.id.clone(),
            ContainerNode {
                id: object.id.clone(),
                parent_id: object.parent_id.clone(),
                title: object.title.clone(),
                kind: object.kind,
                children: Vec::new(),
                update_id: containers.get(&object.id).copied().unwrap_or_default(),
            },
        );
    }
    let lists: Vec<(String, Vec<ContentItem>)> = built
        .keys()
        .map(|id| {
            let members = children.get(id);
            let list = members
                .map(|members| {
                    members
                        .iter()
                        .map(|child| child_item(child, id, &titles, items, children))
                        .collect()
                })
                .unwrap_or_default();
            (id.clone(), list)
        })
        .collect();
    for (id, list) in lists {
        if let Some(node) = built.get_mut(&id) {
            node.children = list;
        }
    }
    built
}

/// The item of one child of a container.
///
/// A child is a file of the tree, a folder, a playlist, or an entry that the
/// catalogue knows but that has no file of its own.
fn child_item(
    child: &ChildRow,
    container_id: &str,
    titles: &HashMap<&str, &ObjectRow>,
    items: &BTreeMap<String, ContentItem>,
    children: &HashMap<String, Vec<ChildRow>>,
) -> ContentItem {
    let title = child
        .title
        .clone()
        .or_else(|| {
            titles
                .get(child.object_id.as_str())
                .map(|object| object.title.clone())
        })
        .unwrap_or_default();
    if let Some(item) = items.get(&child.object_id) {
        let mut item = item.clone();
        item.title = title;
        item.parent_id = container_id.to_owned();
        return item;
    }
    let kind = titles
        .get(child.object_id.as_str())
        .map(|object| object.kind)
        .unwrap_or(MediaKind::Other);
    let count = children
        .get(&child.object_id)
        .map(|members| members.len() as u32)
        .unwrap_or_default();
    match kind {
        MediaKind::Playlist => {
            ContentItem::playlist(child.object_id.clone(), container_id, title, count)
        }
        MediaKind::Folder => {
            ContentItem::folder(child.object_id.clone(), container_id, title, count)
        }
        _ => ContentItem::new(child.object_id.clone(), container_id, title, kind),
    }
}

/// The error of a file that cannot be read.
fn unusable(path: &Path, error: rusqlite::Error) -> IndexError {
    IndexError::Unusable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
