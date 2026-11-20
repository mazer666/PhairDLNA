//! The builder that turns the roots into the tree of the catalogue.
//!
//! What: the scan of every root and the assembly of the containers, the items
//!       and the identifiers of the catalogue.
//! Why: the catalogue is read-only for the protocol layer, so everything that
//!       writes into it happens here, in one place (`F-08`, `F-09`).
//! Example: `build` scans the roots and returns the catalogue the ContentDirectory
//!       service reads.
//!
//! **The roots are kept apart.**
//!
//! Every root has its own key in the identifiers, its own place in the tree and
//! its own boundary: a link that points from one root into another is not
//! followed, and a root that lies inside another root is published once, under
//! its own key (`FUN-017`, `SEC-003`).
//!
//! **The volumes are watched.**
//!
//! Every root is observed while it is scanned, and the observation is compared
//! with what the catalogue saw the last time: a drive that was unplugged, a
//! drive that was plugged in again and a drive that was replaced by another
//! volume are three different things, and each of them is reported
//! (`FUN-007`, `FUN-017`, `PLT-011`).

use crate::config::{MediaRoot, SymlinkPolicy};
use crate::platform::{FileId, FileSystem};
use std::collections::HashMap;

use super::identifier::object_id;
use super::library::{Catalogue, CONTENT_DIRECTORY_OBJECT_ID};
use super::roots::{excluded_roots, RootSummary};
use super::scanner::ScanEntry;
use super::tree::{add_container, add_standard_containers, fill_playlists, RootContext};
use super::update_id::{update_id_for, CatalogueFingerprint};
use super::volumes::{VolumeRegistry, VolumeReport};
use super::ContentItem;

mod scan;

/// A catalogue and what happened to the volumes of the roots while it was built.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    /// The catalogue that was built.
    pub catalogue: Catalogue,
    /// What happened to the volumes of the roots.
    pub report: VolumeReport,
}

/// Scans the roots and builds the catalogue.
pub fn build(
    roots: &[MediaRoot],
    symlink_policy: SymlinkPolicy,
    filesystem: &dyn FileSystem,
) -> Catalogue {
    build_with(
        roots,
        symlink_policy,
        filesystem,
        &mut VolumeRegistry::default(),
    )
    .catalogue
}

/// Scans the roots and builds the catalogue, and remembers the volumes.
///
/// The memory of the volumes is what makes the second build different from the
/// first: the catalogue can then tell a drive that was unplugged from a drive
/// that was never there (`FUN-007`, `PLT-011`).
pub fn build_with(
    roots: &[MediaRoot],
    symlink_policy: SymlinkPolicy,
    filesystem: &dyn FileSystem,
    registry: &mut VolumeRegistry,
) -> BuildOutcome {
    let mut builder = Builder {
        catalogue: Catalogue::default(),
        filesystem,
        registry,
        report: VolumeReport::default(),
        seen_files: HashMap::new(),
        seen_keys: HashMap::new(),
    };
    builder.add_roots(roots, symlink_policy);
    builder.finish()
}

/// The builder that turns the roots into the tree.
struct Builder<'a> {
    catalogue: Catalogue,
    filesystem: &'a dyn FileSystem,
    registry: &'a mut VolumeRegistry,
    report: VolumeReport,
    /// The files that are already indexed, by the identity the platform gives
    /// them.
    ///
    /// Two roots that reach one file must not list it twice: the second
    /// sighting resolves to the entry that is already there (`FUN-020`,
    /// `FUN-022`).
    seen_files: HashMap<FileId, String>,
    /// The files that are already indexed, by their identity key.
    ///
    /// The key is scoped by the volume, so the same file reached through two
    /// roots of one volume has one key (`FUN-020`).
    seen_keys: HashMap<String, String>,
}

impl Builder<'_> {
    /// Scans every root and adds it to the tree.
    fn add_roots(&mut self, roots: &[MediaRoot], symlink_policy: SymlinkPolicy) {
        let mut root_items = Vec::new();
        for (index, root) in roots.iter().enumerate() {
            root_items.push(self.add_root(root, index, symlink_policy, roots));
        }
        add_standard_containers(&mut self.catalogue, root_items);
    }

    /// Scans one root and adds its tree to the catalogue.
    fn add_root(
        &mut self,
        root: &MediaRoot,
        index: usize,
        symlink_policy: SymlinkPolicy,
        roots: &[MediaRoot],
    ) -> ContentItem {
        let root_key = index.to_string();
        let policy = root.symlink_policy.unwrap_or(symlink_policy);
        let excludes = excluded_roots(roots, index);
        let (volume, path, outcome) =
            scan::scan_root(self.filesystem, self.registry, root, policy, excludes);
        let fingerprint = self.filesystem.volume_fingerprint(&path).ok();
        let change = self.registry.observe(
            &root.name,
            &path,
            &volume,
            fingerprint.as_ref(),
            outcome.root_present,
        );
        scan::report_change(&mut self.catalogue, &mut self.report, &root.name, change);
        let id = object_id(&volume, &root_key, "");
        let entries = self.without_duplicates(outcome.entries);
        let context = RootContext {
            entries: &entries,
            volume: &volume,
            root_key: &root_key,
        };
        let mut pending = Vec::new();
        let children = add_container(
            &mut self.catalogue,
            &id,
            CONTENT_DIRECTORY_OBJECT_ID,
            &root.name,
            &context,
            "",
            &mut pending,
        );
        fill_playlists(&mut self.catalogue, &context, pending);
        let count = children.len() as u32;
        self.catalogue
            .diagnostics
            .extend(outcome.diagnostics.clone());
        self.catalogue.roots.push(RootSummary {
            name: root.name.clone(),
            path,
            id: id.clone(),
            present: outcome.root_present,
            entries: entries.len(),
        });
        ContentItem::folder(id, CONTENT_DIRECTORY_OBJECT_ID, &root.name, count)
    }

    /// Removes the files that are already indexed under another root.
    ///
    /// A file is one file, whatever the roots are that reach it: the second
    /// sighting resolves to the entry that is already there, and the catalogue
    /// lists the file once (`FUN-020`, `FUN-022`).
    ///
    /// Two keys say that a file was seen: the identity the platform gives it —
    /// the device and the inode — and the identity key, which is scoped by the
    /// volume. The first says "the same file", the second says "the same path
    /// on the same volume".
    fn without_duplicates(&mut self, entries: Vec<ScanEntry>) -> Vec<ScanEntry> {
        let mut kept = Vec::with_capacity(entries.len());
        for entry in entries {
            if entry.is_folder {
                kept.push(entry);
                continue;
            }
            let known = entry
                .file_identity
                .as_ref()
                .filter(|file| is_telling(file))
                .is_some_and(|file| self.seen_files.contains_key(file));
            if known || self.seen_keys.contains_key(&entry.identity_key) {
                continue;
            }
            if let Some(file) = entry.file_identity.as_ref().filter(|file| is_telling(file)) {
                self.seen_files.insert(file.clone(), entry.identity.clone());
            }
            self.seen_keys
                .insert(entry.identity_key.clone(), entry.identity.clone());
            kept.push(entry);
        }
        kept
    }

    /// Finishes the catalogue and computes its update identifier.
    fn finish(mut self) -> BuildOutcome {
        let containers: Vec<(String, u32)> = self
            .catalogue
            .containers
            .values()
            .map(|node| (node.id.clone(), node.update_id))
            .collect();
        let fingerprint = CatalogueFingerprint::of(&containers);
        let library = self.library_key();
        self.catalogue.fingerprint = fingerprint.value();
        self.catalogue.system_update_id = update_id_for(&library, fingerprint);
        BuildOutcome {
            catalogue: self.catalogue,
            report: self.report,
        }
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

/// Whether the identity of a file says something about it.
///
/// A platform that gives no device and no inode gives every file the same
/// identity. Such an identity cannot tell two files apart, and using it would
/// merge the whole library into one entry (`PLT-003`).
fn is_telling(file: &FileId) -> bool {
    file.device != 0 || file.inode != 0
}
