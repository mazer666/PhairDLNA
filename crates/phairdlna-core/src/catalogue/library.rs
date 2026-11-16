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
use crate::media::{Artwork, ArtworkFinder, ArtworkSource};
use crate::platform::{FileSystem, VolumeEvent, VolumeId};

use super::volumes::{VolumeRegistry, VolumeReport};
use super::{ContentContainer, ContentItem, ContentSource, MediaKind};

pub use super::roots::RootSummary;

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
    /// The finder of the pictures of the library, when the server looks for them.
    ///
    /// The finder is `None` when the pictures are switched off: the catalogue
    /// then reads no file of the library to look for a cover (`PERF-004`).
    pub(super) pictures: Option<ArtworkFinder>,
}

/// One container of the catalogue.
#[derive(Debug, Clone)]
pub(super) struct ContainerNode {
    pub(super) id: String,
    pub(super) parent_id: String,
    pub(super) title: String,
    /// What kind of container this is.
    pub(super) kind: MediaKind,
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

    /// Scans the roots again and replaces the catalogue.
    ///
    /// The memory of the volumes is kept, so that the catalogue can tell what
    /// changed at the roots: a drive that was unplugged, a drive that was
    /// plugged in again, and a drive that was replaced by another volume. The
    /// entries of a replaced volume are given new identifiers, because the
    /// identifiers carry the identity of the volume (`PLT-011`, `INT-062`).
    ///
    /// The catalogue that was there is replaced as a whole: a root that is not
    /// there any more takes its entries with it, and the rest of the library
    /// stays browsable (`FUN-007`).
    pub fn refresh(
        &mut self,
        roots: &[MediaRoot],
        symlink_policy: SymlinkPolicy,
        filesystem: &dyn FileSystem,
        registry: &mut VolumeRegistry,
    ) -> VolumeReport {
        let outcome = super::builder::build_with(roots, symlink_policy, filesystem, registry);
        // The pictures belong to the library and not to the scan: a catalogue
        // that is read again keeps looking for them.
        let pictures = self.pictures.take();
        *self = outcome.catalogue;
        self.pictures = pictures;
        outcome.report
    }

    /// Scans the roots and builds the catalogue, and remembers the volumes.
    pub fn build_with(
        roots: &[MediaRoot],
        symlink_policy: SymlinkPolicy,
        filesystem: &dyn FileSystem,
        registry: &mut VolumeRegistry,
    ) -> super::builder::BuildOutcome {
        super::builder::build_with(roots, symlink_policy, filesystem, registry)
    }

    /// Applies an event of the platform and reads the catalogue again.
    ///
    /// The event is a first-class thing (`PLT-011`): the platform says that a
    /// volume was mounted, was unmounted or was mounted again, and the
    /// catalogue believes the platform, reads the roots again and says what
    /// changed.
    ///
    /// A volume that was unmounted takes its files with it, even when its mount
    /// point is still there as an empty folder. A volume that comes back is
    /// published again, and a volume that comes back under another identity is
    /// given new identifiers, so that a client browses again (`INT-062`).
    pub fn apply_volume_event(
        &mut self,
        event: &VolumeEvent,
        roots: &[MediaRoot],
        symlink_policy: SymlinkPolicy,
        filesystem: &dyn FileSystem,
        registry: &mut VolumeRegistry,
    ) -> VolumeReport {
        match event {
            VolumeEvent::Unmounted(volume) => {
                for path in roots_of_volume(registry, volume) {
                    registry.mark_absent(&path);
                }
            }
            VolumeEvent::Mounted(volume) | VolumeEvent::Remounted(volume) => {
                for path in roots_of_volume(registry, volume) {
                    registry.mark_mounted(&path);
                }
            }
            VolumeEvent::RescanRequired => {}
        }
        self.refresh(roots, symlink_policy, filesystem, registry)
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

    /// Lets the catalogue look for the pictures of the library.
    ///
    /// The finder is given to the catalogue and not built by it, because the
    /// roots and the bounds of the pictures are the settings of the server
    /// (`CFG-008`, `PERF-004`).
    pub fn with_pictures(&mut self, finder: ArtworkFinder) {
        self.pictures = Some(finder);
    }

    /// The picture of an object, when the object has one.
    ///
    /// The bytes are read only when a client asks for them, and the answer is
    /// remembered, so that a browse that asks again reads no file again
    /// (`F-10` § 5).
    pub fn picture_of(&self, object_id: &str) -> Option<Artwork> {
        let finder = self.pictures.as_ref()?;
        let path = self.files.get(object_id)?;
        finder.picture_of(object_id, path)
    }

    /// Where the picture of an object comes from, when the object has one.
    pub fn picture_source_of(&self, object_id: &str) -> Option<ArtworkSource> {
        let finder = self.pictures.as_ref()?;
        let path = self.files.get(object_id)?;
        finder.source_of(object_id, path)
    }

    /// The identifier of the artwork of an object, when the object has one.
    ///
    /// The identifier is the one of the object itself: the address of the
    /// artwork is built from it, and the address never carries a path of the
    /// disk (`INT-067`, `PLT-008`).
    fn artwork_id_of(&self, object_id: &str) -> Option<String> {
        self.picture_source_of(object_id)
            .map(|_| object_id.to_owned())
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
        let node = self.containers.get(object_id)?;
        let mut container = ContentContainer {
            id: node.id.clone(),
            parent_id: node.parent_id.clone(),
            title: node.title.clone(),
            kind: node.kind,
            update_id: node.update_id,
            children: node.children.clone(),
            total: Some(node.children.len() as u32),
        };
        // The artwork of the children is looked for here, and not while the
        // catalogue is built: the scan of the library reads no file to look
        // for a cover (`PERF-004`).
        for child in &mut container.children {
            if let Some(artwork) = self.artwork_id_of(&child.id) {
                child.metadata.album_art_id = Some(artwork);
            }
        }
        Some(container)
    }

    fn item_of(&self, object_id: &str) -> Option<ContentItem> {
        let mut item = self.items.get(object_id).cloned()?;
        if let Some(artwork) = self.artwork_id_of(object_id) {
            item.metadata.album_art_id = Some(artwork);
        }
        Some(item)
    }
}

/// Orders the children of a container: the folders first, then the files.
///
/// A playlist is a container like a folder and is listed with them. The
/// children of a playlist are not sorted: they keep the order of the playlist
/// (`FUN-015`).
pub(super) fn sort_children(children: &mut [ContentItem]) {
    children.sort_by(|left, right| {
        let rank = |item: &ContentItem| match item.kind {
            MediaKind::Folder | MediaKind::Playlist => 0,
            _ => 1,
        };
        rank(left)
            .cmp(&rank(right))
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
}

/// The paths of the roots that live on a volume.
///
/// A platform that reports volumes knows which root is on which volume; the
/// portable adapter does not, and then every root shares the one identity
/// (`PLT-005`, `PLT-011`).
fn roots_of_volume(registry: &VolumeRegistry, volume: &VolumeId) -> Vec<PathBuf> {
    registry
        .roots()
        .iter()
        .filter(|root| &root.identity == volume)
        .map(|root| root.path.clone())
        .collect()
}
