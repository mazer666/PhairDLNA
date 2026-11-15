//! The builder that turns the roots into the tree of the catalogue.
//!
//! What: the scan of every root and the assembly of the containers, the items
//!       and the identifiers of the catalogue.
//! Why: the catalogue is read-only for the protocol layer, so everything that
//!       writes into it happens here, in one place (`F-08`, `F-09`).
//! Example: `build` scans the roots and returns the catalogue the ContentDirectory
//!       service reads.

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::config::{MediaRoot, SymlinkPolicy};
use crate::platform::{FileSystem, VolumeId};

use super::identifier::object_id;
use super::library::{
    sort_children, Catalogue, ContainerNode, CONTENT_DIRECTORY_OBJECT_ID, ROOT_OBJECT_ID,
};
use super::scanner::{scan, RootSpec, ScanEntry, ScanOptions, ScanOutcome};
use super::update_id::{update_id_for, CatalogueFingerprint};
use super::{ContentItem, MediaKind};

/// What the catalogue knows about a root.
#[derive(Debug, Clone)]
pub struct RootSummary {
    /// The name the client shows.
    pub name: String,
    /// The path of the root on disk.
    pub path: PathBuf,
    /// The identifier of the container of the root.
    pub id: String,
    /// Whether the root could be read.
    pub present: bool,
    /// The number of entries below the root.
    pub entries: usize,
}

/// What the builder needs to know about the root it is building.
struct RootContext<'a> {
    entries: &'a [ScanEntry],
    volume: &'a VolumeId,
    root_key: &'a str,
}

/// Scans the roots and builds the catalogue.
pub fn build(
    roots: &[MediaRoot],
    symlink_policy: SymlinkPolicy,
    filesystem: &dyn FileSystem,
) -> Catalogue {
    let mut builder = Builder {
        catalogue: Catalogue::default(),
        filesystem,
    };
    builder.add_roots(roots, symlink_policy);
    builder.finish()
}

/// The builder that turns the roots into the tree.
struct Builder<'a> {
    catalogue: Catalogue,
    filesystem: &'a dyn FileSystem,
}

impl Builder<'_> {
    /// Scans every root and adds it to the tree.
    fn add_roots(&mut self, roots: &[MediaRoot], symlink_policy: SymlinkPolicy) {
        let mut root_items = Vec::new();
        for (index, root) in roots.iter().enumerate() {
            root_items.push(self.add_root(root, index, symlink_policy));
        }
        self.add_standard_containers(root_items);
    }

    /// Scans one root and adds its tree to the catalogue.
    fn add_root(
        &mut self,
        root: &MediaRoot,
        index: usize,
        symlink_policy: SymlinkPolicy,
    ) -> ContentItem {
        let root_key = index.to_string();
        let policy = root.symlink_policy.unwrap_or(symlink_policy);
        let (volume, path, outcome) = self.scan_root(root, policy);
        let id = object_id(&volume, &root_key, "");
        let context = RootContext {
            entries: &outcome.entries,
            volume: &volume,
            root_key: &root_key,
        };
        let children =
            self.add_container(&id, CONTENT_DIRECTORY_OBJECT_ID, &root.name, &context, "");
        let count = children.len() as u32;
        self.catalogue
            .diagnostics
            .extend(outcome.diagnostics.clone());
        self.catalogue.roots.push(RootSummary {
            name: root.name.clone(),
            path,
            id: id.clone(),
            present: outcome.root_present,
            entries: outcome.entries.len(),
        });
        ContentItem::folder(id, CONTENT_DIRECTORY_OBJECT_ID, &root.name, count)
    }

    /// Scans one root and returns what the scan found.
    fn scan_root(
        &mut self,
        root: &MediaRoot,
        symlink_policy: SymlinkPolicy,
    ) -> (VolumeId, PathBuf, ScanOutcome) {
        let path = PathBuf::from(&root.path);
        let volume = self
            .filesystem
            .volume_of(&path)
            .unwrap_or_else(|_| VolumeId::new("local"));
        let spec = RootSpec {
            path: path.clone(),
            name: root.name.clone(),
            volume: volume.clone(),
            case_sensitivity: self.filesystem.case_sensitivity(&volume),
        };
        let options = ScanOptions {
            symlink_policy,
            excludes: root.exclude.clone(),
            ..ScanOptions::default()
        };
        let outcome = scan(&spec, &options, self.filesystem);
        (volume, path, outcome)
    }

    /// Adds one container and everything below it to the catalogue.
    ///
    /// The recursion walks the entries of the scan, not the disk: the scan has
    /// already read the disk once.
    fn add_container(
        &mut self,
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
                let below =
                    self.add_container(&child_id, id, &entry.title, context, &entry.identity);
                let count = below.len() as u32;
                children.push(ContentItem::folder(child_id, id, &entry.title, count));
            } else {
                children.push(self.file_item(entry, &child_id, id));
            }
        }
        sort_children(&mut children);
        let update_id = fingerprint.finish();
        self.catalogue.containers.insert(
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
    fn file_item(&mut self, entry: &ScanEntry, id: &str, parent_id: &str) -> ContentItem {
        let kind = entry
            .mime_type
            .as_deref()
            .map(super::media::kind_of)
            .unwrap_or(MediaKind::Other);
        let mut item = ContentItem::new(id, parent_id, &entry.title, kind);
        item.metadata.mime_type = entry.mime_type.clone();
        item.metadata.size = entry.size;
        self.catalogue.items.insert(id.to_owned(), item.clone());
        self.catalogue
            .files
            .insert(id.to_owned(), entry.path.clone());
        item
    }

    /// Adds the two standard containers every client starts at.
    fn add_standard_containers(&mut self, root_items: Vec<ContentItem>) {
        let content_directory = ContentItem::folder(
            CONTENT_DIRECTORY_OBJECT_ID.to_owned(),
            ROOT_OBJECT_ID.to_owned(),
            "Content Directory",
            root_items.len() as u32,
        );
        let content_update_id = update_id_of(&root_items);
        self.catalogue.containers.insert(
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
        self.catalogue.containers.insert(
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

    /// Finishes the catalogue and computes its update identifier.
    fn finish(mut self) -> Catalogue {
        let containers: Vec<(String, u32)> = self
            .catalogue
            .containers
            .values()
            .map(|node| (node.id.clone(), node.update_id))
            .collect();
        let fingerprint = CatalogueFingerprint::of(&containers);
        let library = self.library_key();
        self.catalogue.system_update_id = update_id_for(&library, fingerprint);
        self.catalogue
    }

    /// The key of the library: the roots with their names and paths.
    fn library_key(&self) -> String {
        self.catalogue
            .roots
            .iter()
            .map(|root| format!("{}@{}", root.name, root.path.display()))
            .collect::<Vec<_>>()
            .join("|")
    }
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
