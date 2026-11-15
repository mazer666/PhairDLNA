//! The catalogue: the tree of what the server shares.
//!
//! What: the in-memory catalogue of the configured roots and the narrow view
//!       of it that the protocol layer reads.
//! Why: the protocol layer must not touch the filesystem, so the catalogue
//!       reads the disk once, keeps the tree in memory, and hands out opaque
//!       identifiers (`F-08`, `F-09`).
//! Example: `Catalogue::build` scans the roots and answers `container("1")`
//!       with the content directory, whose children are the roots.
//!
//! **The shape of the tree.**
//!
//! ```text
//! "0"  the root container       — every client starts here
//!  └── "1" the content directory
//!       └── <root>  the folder the owner shared
//!            ├── <folder>
//!            └── <file>
//! ```
//!
//! The identifiers `0` and `1` are reserved (`INT-038`); everything else is
//! the hash of the volume, the root and the relative path (`catalogue::identifier`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{MediaRoot, SymlinkPolicy};
use crate::platform::FileSystem;

use super::{ContentContainer, ContentItem, ContentSource, MediaKind};

pub use super::builder::RootSummary;

/// The identifier of the root container.
pub const ROOT_OBJECT_ID: &str = "0";

/// The identifier of the content directory root.
pub const CONTENT_DIRECTORY_OBJECT_ID: &str = "1";

/// The catalogue of the media the server shares.
#[derive(Debug, Clone, Default)]
pub struct Catalogue {
    pub(super) containers: BTreeMap<String, ContainerNode>,
    pub(super) items: BTreeMap<String, ContentItem>,
    pub(super) files: BTreeMap<String, PathBuf>,
    pub(super) system_update_id: u32,
    pub(super) diagnostics: Vec<String>,
    pub(super) roots: Vec<RootSummary>,
}

/// One container of the catalogue.
#[derive(Debug, Clone)]
pub(super) struct ContainerNode {
    pub(super) id: String,
    pub(super) parent_id: String,
    pub(super) title: String,
    pub(super) children: Vec<ContentItem>,
    pub(super) update_id: u32,
}

impl Catalogue {
    /// Scans the roots and builds the catalogue.
    ///
    /// The `symlink_policy` is the setting of the server; a root that names a
    /// policy of its own uses that one instead (`CFG-007`).
    ///
    /// A root that cannot be read does not stop the catalogue: it is reported
    /// as absent, and the rest of the library stays browsable (`FUN-007`).
    pub fn build(
        roots: &[MediaRoot],
        symlink_policy: SymlinkPolicy,
        filesystem: &dyn FileSystem,
    ) -> Self {
        super::builder::build(roots, symlink_policy, filesystem)
    }

    /// What the scanner could not do, in words a person can read.
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    /// What the catalogue knows about the roots.
    pub fn roots(&self) -> &[RootSummary] {
        &self.roots
    }

    /// The path of the file behind an object, if the object is a file.
    ///
    /// The path never leaves the catalogue through the protocol layer: the
    /// catalogue owns the filesystem (`SEC-006`). The path of a link that is
    /// indexed as its own entry is the path of its **target**, so that a link
    /// which is repointed after the scan cannot change what is served
    /// (`SEC-001`).
    pub fn path_of(&self, object_id: &str) -> Option<&Path> {
        self.files.get(object_id).map(PathBuf::as_path)
    }

    /// The number of containers in the catalogue.
    pub fn len(&self) -> usize {
        self.containers.len()
    }

    /// Whether the catalogue holds no container at all.
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }
}

impl ContentSource for Catalogue {
    fn system_update_id(&self) -> u32 {
        self.system_update_id
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        self.containers.get(object_id).map(|node| ContentContainer {
            id: node.id.clone(),
            parent_id: node.parent_id.clone(),
            title: node.title.clone(),
            update_id: node.update_id,
            children: node.children.clone(),
            total: Some(node.children.len() as u32),
        })
    }

    fn item_of(&self, object_id: &str) -> Option<ContentItem> {
        self.items.get(object_id).cloned()
    }
}

/// Orders the children of a container: the folders first, then the files.
pub(super) fn sort_children(children: &mut [ContentItem]) {
    children.sort_by(|left, right| {
        let rank = |item: &ContentItem| match item.kind {
            MediaKind::Folder => 0,
            _ => 1,
        };
        rank(left)
            .cmp(&rank(right))
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
}
