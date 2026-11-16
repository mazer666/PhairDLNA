//! The tests of the memory of the volumes.
//!
//! What: the rules that say what changed at a root — a root that was never
//!       there, a root that is gone, a root that comes back, another device at
//!       the path and another volume at the path.
//! Why: the memory is what tells the catalogue that a drive was unplugged; a
//!       rule that is never tested is a rule that was never right
//!       (`FUN-007`, `FUN-017`, `PLT-011`).
//! Example: a root that comes back under another identity is a replacement.

use super::{VolumeChange, VolumeRegistry, LOCAL_VOLUME};
use crate::config::MediaRoot;
use crate::platform::{FileSystem, LocalFileSystem, VolumeFingerprint, VolumeId};
use std::path::{Path, PathBuf};

/// The volume identity of the portable adapter.
fn local() -> VolumeId {
    VolumeId::new(LOCAL_VOLUME)
}

/// A registry that has seen one root.
fn registry_with_root() -> (VolumeRegistry, PathBuf) {
    let path = PathBuf::from("/library/films");
    let mut registry = VolumeRegistry::default();
    registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:1")),
        true,
    );
    (registry, path)
}

/// The first look at a root is a first look.
#[test]
fn the_first_look_at_a_root_is_the_first_look() {
    let mut registry = VolumeRegistry::default();
    let change = registry.observe("Films", Path::new("/library/films"), &local(), None, true);
    assert_eq!(change, VolumeChange::First);
}

/// Nothing changed when the root is exactly as it was.
#[test]
fn nothing_changed_is_nothing() {
    let (mut registry, path) = registry_with_root();
    let change = registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:1")),
        true,
    );
    assert_eq!(change, VolumeChange::Unchanged);
}

/// A root that cannot be read any more is lost.
#[test]
fn a_root_that_cannot_be_read_is_lost() {
    let (mut registry, path) = registry_with_root();
    let change = registry.observe("Films", &path, &local(), None, false);
    assert_eq!(change, VolumeChange::Lost);
}

/// A root that comes back has returned.
#[test]
fn a_root_that_comes_back_has_returned() {
    let (mut registry, path) = registry_with_root();
    registry.observe("Films", &path, &local(), None, false);
    let change = registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:2")),
        true,
    );
    assert_eq!(change, VolumeChange::Returned);
}

/// Another device at the path is a remount.
#[test]
fn another_device_at_the_path_is_a_remount() {
    let (mut registry, path) = registry_with_root();
    let change = registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:2")),
        true,
    );
    assert_eq!(change, VolumeChange::Remounted);
}

/// Another identity at the path is a replacement.
#[test]
fn another_identity_at_the_path_is_a_replacement() {
    let (mut registry, path) = registry_with_root();
    let change = registry.observe(
        "Films",
        &path,
        &VolumeId::new("vol-2"),
        Some(&VolumeFingerprint::new("dev:2")),
        true,
    );
    assert_eq!(change, VolumeChange::Replaced { previous: local() });
}

/// A root that cannot be read and was not there before is not lost.
#[test]
fn a_root_that_was_never_there_is_not_lost() {
    let mut registry = VolumeRegistry::default();
    let change = registry.observe("Films", Path::new("/library/films"), &local(), None, false);
    assert_eq!(change, VolumeChange::First);
}

/// A root whose volume the platform unmounted is lost, even when its path can
/// still be read.
#[test]
fn a_root_that_the_platform_unmounted_is_lost() {
    let (mut registry, path) = registry_with_root();
    registry.mark_absent(&path);
    let change = registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:2")),
        true,
    );
    assert_eq!(change, VolumeChange::Lost);
}

/// A root whose volume the platform mounted again has returned.
#[test]
fn a_root_that_the_platform_mounted_again_has_returned() {
    let (mut registry, path) = registry_with_root();
    registry.mark_absent(&path);
    registry.observe("Films", &path, &local(), None, true);
    registry.mark_mounted(&path);
    let change = registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:2")),
        true,
    );
    assert_eq!(change, VolumeChange::Returned);
}

/// A root whose volume the platform unmounted does not ask the question again.
#[test]
fn an_unmounted_root_does_not_ask_the_question_again() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().to_path_buf();
    let mut registry = VolumeRegistry::default();
    registry.observe(
        "Films",
        &path,
        &local(),
        Some(&VolumeFingerprint::new("dev:1")),
        true,
    );
    // The platform says that the volume is gone; the place of the drive on the
    // disk is still there and can still be read.
    let filesystem = LocalFileSystem;
    let fingerprint = filesystem.volume_fingerprint(&path).ok();
    registry.mark_absent(&path);
    registry.observe("Films", &path, &local(), fingerprint.as_ref(), true);

    let roots = vec![MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    }];
    assert!(
        !registry.differs(&roots, &filesystem),
        "a root that the platform unmounted does not ask the question again"
    );
}
