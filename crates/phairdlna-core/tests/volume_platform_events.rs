//! The acceptance criteria of F-08 for the events of the platform.
//!
//! What: the tests that drive the catalogue with the events the platform
//!       reports — a volume that was mounted, a volume that was unmounted and a
//!       volume that was mounted again.
//! Why: the mount point of a volume that was unmounted often stays behind as an
//!       empty folder, so that the path still reads while the volume is gone.
//!       The platform knows better than the path does, and the catalogue has
//!       to believe the platform (`PLT-011`).
//! Example: the platform says that a volume was unmounted, and the files of
//!       that volume leave the catalogue although their folder is still there.

mod support;

use std::fs;

use phairdlna_core::catalogue::{Catalogue, VolumeChange, VolumeRegistry};
use phairdlna_core::config::SymlinkPolicy;
use phairdlna_core::platform::{VolumeEvent, VolumeId};
use support::volumes::{
    children_of, file_below, library, roots_of, write_file, ScriptedFileSystem, MP4_HEAD,
};

/// A volume that the platform unmounted takes its files with it, even when its
/// place on the disk is still there.
#[test]
fn a_volume_that_was_unmounted_takes_its_files_with_it() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    // The platform reports the identity of the volume of the drive.
    filesystem.set_volume(drive.path(), "vol-1");
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    assert!(
        file_below(&catalogue, &before[0].0, "film.mkv").is_some(),
        "the film of the drive is published"
    );

    // The volume is unmounted. The place of the drive on the disk is still
    // there, and it is empty — the platform is the only one that knows.
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    fs::create_dir_all(drive.path()).expect("the empty place is left behind");
    let report = catalogue.apply_volume_event(
        &VolumeEvent::Unmounted(VolumeId::new("vol-1")),
        &roots,
        SymlinkPolicy::Skip,
        &filesystem,
        &mut registry,
    );

    let after = children_of(&catalogue, "1");
    assert!(
        file_below(&catalogue, &after[0].0, "film.mkv").is_none(),
        "the file of the drive that is gone is not published"
    );
    assert!(
        file_below(&catalogue, &after[1].0, "track.mp3").is_some(),
        "the file of the other root is still published"
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.change == VolumeChange::Lost),
        "the drive is reported as lost: {:?}",
        report.changes
    );
}

/// A volume that the platform mounted again is published again.
#[test]
fn a_volume_that_was_mounted_again_is_published_again() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    filesystem.set_volume(drive.path(), "vol-1");
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    fs::create_dir_all(drive.path()).expect("the empty place is left behind");
    catalogue.apply_volume_event(
        &VolumeEvent::Unmounted(VolumeId::new("vol-1")),
        &roots,
        SymlinkPolicy::Skip,
        &filesystem,
        &mut registry,
    );

    // The drive is plugged in again and the platform says so.
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    let report = catalogue.apply_volume_event(
        &VolumeEvent::Mounted(VolumeId::new("vol-1")),
        &roots,
        SymlinkPolicy::Skip,
        &filesystem,
        &mut registry,
    );

    let after = children_of(&catalogue, "1");
    assert!(
        file_below(&catalogue, &after[0].0, "film.mkv").is_some(),
        "the file of the drive is published again"
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.change == VolumeChange::Returned),
        "the drive is reported as back: {:?}",
        report.changes
    );
}

/// A volume that comes back is found again, even when the platform could not
/// report it while it was away.
#[test]
fn a_volume_that_comes_back_is_found_again() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    // The platform reports the identity of the volume of the drive — as long
    // as the drive is there. A path that is not there has no volume.
    filesystem.set_volume(drive.path(), "vol-1");
    filesystem.blind_to_missing_paths();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let root_id = before[0].0.clone();

    // The drive is unplugged: the platform cannot say which volume it was.
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    catalogue.apply_volume_event(
        &VolumeEvent::Unmounted(VolumeId::new("vol-1")),
        &roots,
        SymlinkPolicy::Skip,
        &filesystem,
        &mut registry,
    );

    // The identifier of the root did not change while the drive was away.
    let away = children_of(&catalogue, "1");
    assert_eq!(
        away[0].0, root_id,
        "the identifier of the root did not change while the drive was away"
    );

    // The drive is plugged in again, and the platform says so.
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    catalogue.apply_volume_event(
        &VolumeEvent::Mounted(VolumeId::new("vol-1")),
        &roots,
        SymlinkPolicy::Skip,
        &filesystem,
        &mut registry,
    );

    let after = children_of(&catalogue, "1");
    assert!(
        file_below(&catalogue, &after[0].0, "film.mkv").is_some(),
        "the file of the drive is published again"
    );
    assert_eq!(
        after[0].0, root_id,
        "the identifier of the root is the one it had before"
    );
}
