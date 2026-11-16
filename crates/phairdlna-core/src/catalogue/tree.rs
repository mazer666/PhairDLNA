//! The assembly of the tree of the catalogue.
//!
//! What: the containers and the items that the scan produced, and the
//!       identifier that changes when the content of a container changes.
//! Why: the walk reads the disk and the builder decides what the catalogue is;
//!       the assembly of the tree is a third thing, and it is the same for
//!       every root (`F-08`).
//! Example: `add_container` turns the entries of one folder into a container
//!       with its children.

use std::time::UNIX_EPOCH;

use super::identifier::object_id;
use super::library::{
    sort_children, Catalogue, ContainerNode, CONTENT_DIRECTORY_OBJECT_ID, ROOT_OBJECT_ID,
};
use super::scanner::ScanEntry;
use super::{ContentItem, MediaKind};

/// What the assembly needs to know about the root it is building.
pub(super) struct RootContext<'a> {
    /// The entries the scan found below the root.
    pub(super) entries: &'a [ScanEntry],
    /// The volume the root lives on.
    pub(super) volume: &'a crate::platform::VolumeId,
    /// The key of the root in the identifiers.
    pub(super) root_key: &'a str,
}

/// Adds one container and everything below it to the catalogue.
///
/// The recursion walks the entries of the scan, not the disk: the scan has
/// already read the disk once.
pub(super) fn add_container(
    catalogue: &mut Catalogue,
    id: &str,
    parent_id: &str,
    title: &str,
    context: &RootContext<'_>,
    identity: &str,
) -> Vec<ContentItem> {
    let mut children = Vec::new();
    let mut fingerprint = Fnv::new();
    for entry in context
        .entries
        .iter()
        .filter(|entry| entry.parent_identity == identity)
    {
        let child_id = object_id(context.volume, context.root_key, &entry.identity);
        fingerprint.write(child_id.as_bytes());
        fingerprint.write(&size_of(entry).to_le_bytes());
        fingerprint.write(&modified_of(entry).to_le_bytes());
        if entry.is_folder {
            let below = add_container(
                catalogue,
                &child_id,
                id,
                &entry.title,
                context,
                &entry.identity,
            );
            let count = below.len() as u32;
            children.push(ContentItem::folder(child_id, id, &entry.title, count));
        } else {
            children.push(file_item(catalogue, entry, &child_id, id));
        }
    }
    sort_children(&mut children);
    let update_id = fingerprint.finish();
    catalogue.containers.insert(
        id.to_owned(),
        ContainerNode {
            id: id.to_owned(),
            parent_id: parent_id.to_owned(),
            title: title.to_owned(),
            children: children.clone(),
            update_id,
        },
    );
    children
}

/// Builds the item of a file.
fn file_item(
    catalogue: &mut Catalogue,
    entry: &ScanEntry,
    id: &str,
    parent_id: &str,
) -> ContentItem {
    let kind = entry
        .mime_type
        .as_deref()
        .map(super::media::kind_of)
        .unwrap_or(MediaKind::Other);
    let mut item = ContentItem::new(id, parent_id, &entry.title, kind);
    item.metadata.mime_type = entry.mime_type.clone();
    item.metadata.size = entry.size;
    catalogue.items.insert(id.to_owned(), item.clone());
    catalogue.files.insert(id.to_owned(), entry.path.clone());
    item
}

/// Adds the two standard containers every client starts at.
pub(super) fn add_standard_containers(catalogue: &mut Catalogue, root_items: Vec<ContentItem>) {
    let content_directory = ContentItem::folder(
        CONTENT_DIRECTORY_OBJECT_ID.to_owned(),
        ROOT_OBJECT_ID.to_owned(),
        "Content Directory",
        root_items.len() as u32,
    );
    let content_update_id = update_id_of(&root_items);
    catalogue.containers.insert(
        CONTENT_DIRECTORY_OBJECT_ID.to_owned(),
        ContainerNode {
            id: CONTENT_DIRECTORY_OBJECT_ID.to_owned(),
            parent_id: ROOT_OBJECT_ID.to_owned(),
            title: "Content Directory".to_owned(),
            children: root_items,
            update_id: content_update_id,
        },
    );
    let root_children = vec![content_directory];
    let root_update_id = update_id_of(&root_children);
    catalogue.containers.insert(
        ROOT_OBJECT_ID.to_owned(),
        ContainerNode {
            id: ROOT_OBJECT_ID.to_owned(),
            parent_id: "-1".to_owned(),
            title: "Root".to_owned(),
            children: root_children,
            update_id: root_update_id,
        },
    );
}

/// The size of an entry, for the fingerprint of a container.
fn size_of(entry: &ScanEntry) -> u64 {
    entry.size.unwrap_or_default()
}

/// The moment an entry was changed, in seconds since the epoch.
fn modified_of(entry: &ScanEntry) -> u64 {
    entry
        .modified
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

/// The identifier that changes when the content of a container changes.
fn update_id_of(children: &[ContentItem]) -> u32 {
    let mut fingerprint = Fnv::new();
    for child in children {
        fingerprint.write(child.id.as_bytes());
    }
    fingerprint.finish()
}

/// A small hash that mixes bytes into a number.
struct Fnv(u32);

impl Fnv {
    /// Starts a hash.
    fn new() -> Self {
        Self(0x811c_9dc5)
    }

    /// Mixes bytes into the hash.
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u32::from(*byte);
            self.0 = self.0.wrapping_mul(0x0100_0193);
        }
    }

    /// The number the hash ended at.
    fn finish(&self) -> u32 {
        self.0
    }
}
