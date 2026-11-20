//! Helper methods for scanning roots and determining volume identities.

use std::path::{Path, PathBuf};

use crate::config::{MediaRoot, SymlinkPolicy};
use crate::platform::{FileSystem, VolumeId};

use super::super::roots::diagnostic_for;
use super::super::scanner::{scan, RootSpec, ScanOptions, ScanOutcome};
use super::super::volumes::{RootChange, VolumeChange, VolumeRegistry, VolumeReport};
use super::super::Catalogue;

/// Scans a root with configured symlink policies and excludes.
pub fn scan_root(
    filesystem: &dyn FileSystem,
    registry: &VolumeRegistry,
    root: &MediaRoot,
    symlink_policy: SymlinkPolicy,
    nested: Vec<String>,
) -> (VolumeId, PathBuf, ScanOutcome) {
    let path = PathBuf::from(&root.path);
    let volume = volume_of(filesystem, registry, &path);
    let spec = RootSpec {
        path: path.clone(),
        name: root.name.clone(),
        volume: volume.clone(),
        case_sensitivity: filesystem.case_sensitivity(&volume),
    };
    let mut excludes = root.exclude.clone();
    excludes.extend(nested);
    let options = ScanOptions {
        symlink_policy,
        excludes,
        ..ScanOptions::default()
    };
    let outcome = scan(&spec, &options, filesystem);
    (volume, path, outcome)
}

/// The identity of the volume a path lives on.
pub fn volume_of(filesystem: &dyn FileSystem, registry: &VolumeRegistry, path: &Path) -> VolumeId {
    if filesystem.is_readable_folder(path) {
        return filesystem
            .volume_of(path)
            .unwrap_or_else(|_| VolumeId::new("local"));
    }
    registry
        .remembered_identity(path)
        .cloned()
        .unwrap_or_else(|| VolumeId::new("local"))
}

/// Records a change in the build report and catalogue diagnostics.
pub fn report_change(
    catalogue: &mut Catalogue,
    report: &mut VolumeReport,
    name: &str,
    change: VolumeChange,
) {
    if let Some(sentence) = diagnostic_for(name, &change) {
        catalogue.diagnostics.push(sentence);
    }
    report.changes.push(RootChange {
        name: name.to_owned(),
        change,
    });
}
