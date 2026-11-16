//! The volume state of the roots.
//!
//! What: what the catalogue remembers about the volume of every root — the
//!       identity the platform reports, what it observes at the path, and
//!       whether the root could be read.
//! Why: a media library lives on drives that come and go. A server that does
//!       not remember what it saw last time cannot tell a drive that was
//!       unplugged from a drive that was replaced, and it cannot tell a client
//!       that the library changed (`FUN-007`, `FUN-017`, `PLT-011`).
//! Example: the catalogue remembers that the root "Films" lived on the volume
//!       `vol-2`. When the drive comes back with another identity, the entries
//!       are given new identifiers and the change is reported.
//!
//! **Two identities, two purposes.**
//!
//! - The **identity** of a volume (`platform::VolumeId`) is what the object
//!   identifiers are built on. It must survive a remount, so that a client
//!   that cached an identifier keeps working (`INT-062`). A volume that comes
//!   back under another identity is a different volume as far as the catalogue
//!   is concerned: its entries are given new identifiers (`PLT-011`).
//! - The **fingerprint** of a path (`platform::VolumeFingerprint`) is what the
//!   platform observes *right now*. It says that something at the path is the
//!   same as it was, or that it is not — and nothing more. The portable
//!   adapter reports the device of the path, which changes when a volume is
//!   unmounted and its place on the disk is taken by a folder of another
//!   device (`PLT-005`).

use std::path::{Path, PathBuf};

use crate::config::MediaRoot;
use crate::platform::{FileSystem, VolumeFingerprint, VolumeId};

#[cfg(test)]
mod tests;

/// The volume identity the portable adapter reports.
const LOCAL_VOLUME: &str = "local";

/// What the catalogue knows about the volume of one root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootVolume {
    /// The name the client shows for the root.
    pub name: String,
    /// The path of the root, as it was configured.
    pub path: PathBuf,
    /// The identity of the volume, as the platform reports it.
    pub identity: VolumeId,
    /// What the platform observed at the path, when it can tell.
    pub fingerprint: Option<VolumeFingerprint>,
    /// Whether the root could be read the last time the catalogue looked.
    pub present: bool,
    /// Whether the platform said that the volume of the root is gone.
    ///
    /// The mount point of a volume that was unmounted often stays behind as an
    /// empty folder, so that the path still reads while the volume is gone.
    /// The platform knows better than the path does, and the catalogue believes
    /// the platform (`PLT-011`).
    pub unmounted: bool,
}

/// What changed about the volume of a root since the last look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeChange {
    /// The root is seen for the first time.
    First,
    /// Nothing changed.
    Unchanged,
    /// The root was there and cannot be read any more.
    Lost,
    /// The root could not be read and can be read again.
    Returned,
    /// Another device is mounted at the path than the one that was there.
    Remounted,
    /// The volume at the path has another identity: the entries were re-keyed.
    Replaced {
        /// The identity the volume had before.
        previous: VolumeId,
    },
}

/// What changed for one root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootChange {
    /// The name the client shows for the root.
    pub name: String,
    /// What changed about the volume of the root.
    pub change: VolumeChange,
}

/// What the catalogue did about the volumes of the roots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumeReport {
    /// What changed for every root.
    pub changes: Vec<RootChange>,
}

impl VolumeReport {
    /// Whether something changed at all.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// The roots whose entries were given new identifiers.
    pub fn rekeyed(&self) -> Vec<&str> {
        self.changes
            .iter()
            .filter(|change| matches!(change.change, VolumeChange::Replaced { .. }))
            .map(|change| change.name.as_str())
            .collect()
    }
}

/// What the catalogue remembers about the volumes of the roots.
///
/// The memory is kept between two builds of the catalogue, so that the second
/// build can tell what changed. A catalogue that is built once and thrown away
/// cannot tell a drive that was unplugged from one that was never there.
#[derive(Debug, Clone, Default)]
pub struct VolumeRegistry {
    roots: Vec<RootVolume>,
}

impl VolumeRegistry {
    /// The state of every root, in the order the roots were seen.
    pub fn roots(&self) -> &[RootVolume] {
        &self.roots
    }

    /// The state of the root at a path.
    pub fn root_at(&self, path: &Path) -> Option<&RootVolume> {
        self.roots.iter().find(|root| root.path == path)
    }

    /// The identity the catalogue remembers for a root.
    ///
    /// A path that cannot be read has no volume that the platform could report
    /// for it: the place of an unplugged drive is a folder of another volume.
    /// The remembered identity is the one that keeps the identifiers of the
    /// root stable while the drive is away (`INT-062`) and the one that lets
    /// the event of the platform find the root again (`PLT-011`).
    pub fn remembered_identity(&self, path: &Path) -> Option<&VolumeId> {
        self.root_at(path).map(|root| &root.identity)
    }

    /// Records what the platform sees at a root and says what changed.
    ///
    /// The order of the rules matters: a root that cannot be read is lost,
    /// whether it was there before or not; a root that comes back has returned;
    /// only a root that stayed there can be replaced by another volume.
    pub fn observe(
        &mut self,
        name: &str,
        path: &Path,
        identity: &VolumeId,
        fingerprint: Option<&VolumeFingerprint>,
        present: bool,
    ) -> VolumeChange {
        match self.position_of(path) {
            Some(index) => {
                let change = classify(&self.roots[index], identity, fingerprint, present);
                self.roots[index] = RootVolume {
                    name: name.to_owned(),
                    path: path.to_path_buf(),
                    identity: identity.clone(),
                    fingerprint: fingerprint.cloned(),
                    present: present && !self.roots[index].unmounted,
                    unmounted: self.roots[index].unmounted,
                };
                change
            }
            None => {
                self.roots.push(RootVolume {
                    name: name.to_owned(),
                    path: path.to_path_buf(),
                    identity: identity.clone(),
                    fingerprint: fingerprint.cloned(),
                    present,
                    unmounted: false,
                });
                VolumeChange::First
            }
        }
    }

    /// Marks a root as gone, because the platform said that its volume is gone.
    ///
    /// The platform knows better than the path does: the mount point of a
    /// volume that was unmounted often stays behind as an empty folder, and a
    /// catalogue that believes the path would publish an empty root where the
    /// drive used to be (`PLT-011`).
    pub fn mark_absent(&mut self, path: &Path) {
        if let Some(index) = self.position_of(path) {
            self.roots[index].unmounted = true;
        }
    }

    /// Says that the volume of a root is there again.
    ///
    /// The root is published again as soon as the next read finds it.
    pub fn mark_mounted(&mut self, path: &Path) {
        if let Some(index) = self.position_of(path) {
            self.roots[index].unmounted = false;
        }
    }

    /// Whether anything changed at the roots since the last look.
    ///
    /// The check is cheap: it looks at the roots only, and it reads no file of
    /// the library. A daemon uses it to decide whether a rescan is worth it.
    pub fn differs(&self, roots: &[MediaRoot], filesystem: &dyn FileSystem) -> bool {
        roots.iter().any(|root| self.differs_at(root, filesystem))
    }

    /// Whether the root at a path changed since the last look.
    fn differs_at(&self, root: &MediaRoot, filesystem: &dyn FileSystem) -> bool {
        let path = Path::new(&root.path);
        let readable = filesystem.is_readable_folder(path);
        let fingerprint = filesystem.volume_fingerprint(path).ok();
        match self.root_at(path) {
            None => true,
            Some(known) => {
                // A root whose volume the platform unmounted is absent, even
                // when its path can still be read; the question is therefore
                // not asked twice.
                let present = readable && !known.unmounted;
                // A path that cannot be read has no volume that the platform
                // could report for it: the identity the catalogue remembers
                // is the one that counts, so that the question is not asked
                // twice either.
                let identity = if readable {
                    identity_of(filesystem, path)
                } else {
                    known.identity.clone()
                };
                known.present != present
                    || known.identity != identity
                    || known.fingerprint.as_ref() != fingerprint.as_ref()
            }
        }
    }

    /// The place of a root in the memory.
    fn position_of(&self, path: &Path) -> Option<usize> {
        self.roots.iter().position(|root| root.path == path)
    }
}

/// What the platform says about the volume of a path.
fn classify(
    previous: &RootVolume,
    identity: &VolumeId,
    fingerprint: Option<&VolumeFingerprint>,
    present: bool,
) -> VolumeChange {
    // The root was there when the platform said that its volume was gone, and
    // it is not there now: that is a loss, not a change of the device.
    let was_present = previous.present;
    let present = present && !previous.unmounted;
    if !present {
        return if was_present {
            VolumeChange::Lost
        } else {
            VolumeChange::Unchanged
        };
    }
    // Another volume at the path is a replacement, whether the root stayed
    // there or came back: the identifiers of its files change either way.
    if &previous.identity != identity {
        return VolumeChange::Replaced {
            previous: previous.identity.clone(),
        };
    }
    if !was_present {
        return VolumeChange::Returned;
    }
    if previous.fingerprint.as_ref() != fingerprint {
        return VolumeChange::Remounted;
    }
    VolumeChange::Unchanged
}

/// The identity the platform reports for a path.
fn identity_of(filesystem: &dyn FileSystem, path: &Path) -> VolumeId {
    filesystem
        .volume_of(path)
        .unwrap_or_else(|_| VolumeId::new(LOCAL_VOLUME))
}
