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
use super::scanner::{PlaylistTarget, ScanEntry, TargetState};
use super::{ContentItem, MediaKind, SubtitleResource};

/// What the assembly needs to know about the root it is building.
pub(super) struct RootContext<'a> {
    /// The entries the scan found below the root.
    pub(super) entries: &'a [ScanEntry],
    /// The volume the root lives on.
    pub(super) volume: &'a crate::platform::VolumeId,
    /// The key of the root in the identifiers.
    pub(super) root_key: &'a str,
}

/// A playlist whose children are not known yet.
///
/// A playlist may name a file that lives in another folder of the root, and
/// that file is read after the playlist. The children are therefore gathered
/// when the whole tree of the root stands (`FUN-015`).
pub(super) struct PendingPlaylist {
    /// The identifier of the container of the playlist.
    pub(super) id: String,
    /// The identifier of the container the playlist lives in.
    pub(super) parent_id: String,
    /// The identity of the playlist, relative to the root.
    pub(super) identity: String,
    /// The name the client shows.
    pub(super) title: String,
    /// The entries of the playlist, in the order the playlist writes them.
    pub(super) targets: Vec<PlaylistTarget>,
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
    pending: &mut Vec<PendingPlaylist>,
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
                pending,
            );
            let count = below.len() as u32;
            children.push(ContentItem::folder(child_id, id, &entry.title, count));
        } else if let Some(plan) = &entry.playlist {
            let count = plan.offered() as u32;
            children.push(ContentItem::playlist(
                child_id.clone(),
                id,
                &entry.title,
                count,
            ));
            pending.push(PendingPlaylist {
                id: child_id,
                parent_id: id.to_owned(),
                identity: entry.identity.clone(),
                title: entry.title.clone(),
                targets: plan.targets.clone(),
            });
        } else {
            children.push(file_item(catalogue, entry, &child_id, id, context));
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
            kind: MediaKind::Folder,
            children: children.clone(),
            update_id,
        },
    );
    children
}

/// Fills the playlists of a root with the tracks they name.
///
/// The children of a playlist keep the order of the playlist: a list that is
/// sorted is a different list (`FUN-015`). An entry that names nothing is
/// kept as a child that cannot be played, so that the user learns about it
/// instead of wondering where the track went (`UX-004`).
pub(super) fn fill_playlists(
    catalogue: &mut Catalogue,
    context: &RootContext<'_>,
    pending: Vec<PendingPlaylist>,
) {
    for playlist in pending {
        let children = children_of_playlist(catalogue, context, &playlist);
        let update_id = update_id_of(&children);
        catalogue.containers.insert(
            playlist.id.clone(),
            ContainerNode {
                id: playlist.id.clone(),
                parent_id: playlist.parent_id.clone(),
                title: playlist.title.clone(),
                kind: MediaKind::Playlist,
                children: children.clone(),
                update_id,
            },
        );
    }
}

/// The children of one playlist.
///
/// A refused entry is not offered: the user is told about it in the
/// diagnostics, and the playlist shows the tracks that can be played. An entry
/// that names a file that is not there stays in the playlist, so that the user
/// learns where the track went (`UX-004`, `SEC-068`).
fn children_of_playlist(
    catalogue: &Catalogue,
    context: &RootContext<'_>,
    playlist: &PendingPlaylist,
) -> Vec<ContentItem> {
    playlist
        .targets
        .iter()
        .enumerate()
        .filter(|(_, target)| !matches!(target.state, TargetState::Refused(_)))
        .map(|(index, target)| child_of_target(catalogue, context, playlist, target, index))
        .collect()
}

/// The child of a playlist that names a file.
///
/// The child is the media object of the file, with the parent the playlist and
/// the title the playlist gives. A file that the catalogue does not know
/// becomes a child that carries no resource: it is listed, and it cannot be
/// played (`UX-004`).
fn child_of_target(
    catalogue: &Catalogue,
    context: &RootContext<'_>,
    playlist: &PendingPlaylist,
    target: &PlaylistTarget,
    index: usize,
) -> ContentItem {
    let id = object_id(context.volume, context.root_key, &target.identity);
    let item = catalogue.items.get(&id);
    match item {
        Some(item) => {
            let mut child = item.clone();
            child.parent_id = playlist.id.clone();
            if let Some(title) = &target.title {
                child.title = title.clone();
            }
            child
        }
        None => missing_child(context, playlist, target, index),
    }
}

/// The child of a playlist that names a file the catalogue does not know.
fn missing_child(
    context: &RootContext<'_>,
    playlist: &PendingPlaylist,
    target: &PlaylistTarget,
    index: usize,
) -> ContentItem {
    let identity = format!("{}/missing/{index}", playlist.identity);
    let id = object_id(context.volume, context.root_key, &identity);
    let title = target.title.clone().unwrap_or_else(|| target.name.clone());
    ContentItem::new(id, playlist.id.clone(), title, MediaKind::Other)
}

/// Builds the item of a file.
fn file_item(
    catalogue: &mut Catalogue,
    entry: &ScanEntry,
    id: &str,
    parent_id: &str,
    context: &RootContext<'_>,
) -> ContentItem {
    let kind = entry
        .mime_type
        .as_deref()
        .map(super::media::kind_of)
        .unwrap_or(MediaKind::Other);
    let mut item = ContentItem::new(id, parent_id, &entry.title, kind);
    item.metadata.mime_type = entry.mime_type.clone();
    item.metadata.size = entry.size;
    if let Some(sidecar) = &entry.subtitle {
        item.metadata.subtitle = register_subtitle(catalogue, sidecar, id, context);
    }
    catalogue.items.insert(id.to_owned(), item.clone());
    catalogue.files.insert(id.to_owned(), entry.path.clone());
    item
}

/// Registers the subtitle of a media file and answers its identifier.
///
/// The sidecar is not an item of the tree: it is an additional resource of
/// the media object, and the client asks for it by the identifier that is
/// written here (`FUN-016`).
fn register_subtitle(
    catalogue: &mut Catalogue,
    sidecar: &super::scanner::SubtitleEntry,
    parent_id: &str,
    context: &RootContext<'_>,
) -> Option<SubtitleResource> {
    let id = object_id(context.volume, context.root_key, &sidecar.identity);
    let path = sidecar.path.clone();
    let mut item = ContentItem::new(
        id.clone(),
        parent_id.to_owned(),
        &sidecar.name,
        MediaKind::Subtitle,
    );
    item.metadata.mime_type = Some(sidecar.mime_type.clone());
    item.metadata.charset = Some(sidecar.charset.to_owned());
    item.metadata.size = sidecar.size;
    catalogue.items.insert(id.clone(), item);
    catalogue.files.insert(id.clone(), path);
    Some(SubtitleResource {
        id,
        mime_type: sidecar.mime_type.clone(),
        charset: sidecar.charset.to_owned(),
        size: sidecar.size,
    })
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
            kind: MediaKind::Folder,
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
            kind: MediaKind::Folder,
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
