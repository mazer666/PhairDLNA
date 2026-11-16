//! The acceptance criteria of F-08 for the volumes of a root.
//!
//! What: the tests that read a root, lose it, get it back and read it again —
//!       and the tests that read it when the volume behind it is another one.
//! Why: a library lives on drives that come and go. The server has to notice
//!       what happened, tell the client, and keep the rest of the library
//!       browsable while it does (`FUN-007`, `FUN-017`, `PLT-011`).
//! Example: the drive "Films" is unplugged; its files leave the catalogue, the
//!       drive "Music" stays, and the server says that the drive is not
//!       connected.

mod support;

use std::fs;

use phairdlna_core::catalogue::{Catalogue, ContentSource, VolumeChange, VolumeRegistry};
use phairdlna_core::config::SymlinkPolicy;
use support::volumes::{
    children_of, file_below, library, roots_of, write_file, ScriptedFileSystem, MP4_HEAD,
};

/// A drive that is unplugged takes its files with it, and the rest stays.
#[test]
fn a_drive_that_is_unplugged_takes_its_files_with_it() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let film = file_below(&catalogue, &before[0].0, "film.mkv").expect("the film is there");
    let update_id_before = catalogue.system_update_id();

    // The drive is unplugged: its place on the disk is taken by nothing.
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    let report = catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    let after = children_of(&catalogue, "1");
    assert_eq!(after.len(), 2, "both roots are still published");
    assert!(
        file_below(&catalogue, &after[0].0, "film.mkv").is_none(),
        "the file of the drive that is gone is not published any more"
    );
    assert!(
        file_below(&catalogue, &after[1].0, "track.mp3").is_some(),
        "the file of the other root is still published"
    );
    assert!(
        catalogue.path_of(&film).is_none(),
        "the file of the drive that is gone cannot be served any more"
    );
    assert!(
        catalogue.system_update_id() > update_id_before,
        "the identifier of the catalogue changed, so that a client browses again"
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.change == VolumeChange::Lost),
        "the drive is reported as lost: {:?}",
        report.changes
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("Films")),
        "the person is told that the drive is not connected: {:?}",
        catalogue.diagnostics()
    );
}

/// A drive that comes back is published again, with the same identifiers.
#[test]
fn a_drive_that_comes_back_keeps_its_identifiers() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let film = file_below(&catalogue, &before[0].0, "film.mkv").expect("the film is there");

    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    // The drive is plugged in again, with the same volume.
    fs::create_dir_all(drive.path()).expect("the drive is plugged in");
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    let report = catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    let after = children_of(&catalogue, "1");
    let film_again = file_below(&catalogue, &after[0].0, "film.mkv").expect("the film is back");
    assert_eq!(
        film, film_again,
        "the identifier of the file did not change"
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.change == VolumeChange::Returned),
        "the drive is reported as back: {:?}",
        report.changes
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("connected again")),
        "the person is told that the drive is back: {:?}",
        catalogue.diagnostics()
    );
}

/// A drive that comes back under another identity is given new identifiers.
#[test]
fn a_drive_that_comes_back_under_another_identity_is_given_new_identifiers() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let film = file_below(&catalogue, &before[0].0, "film.mkv").expect("the film is there");
    let update_id_before = catalogue.system_update_id();

    // The drive is unplugged and another volume is plugged in at the same
    // place: the platform reports another identity (`PLT-011`).
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    fs::create_dir_all(drive.path()).expect("another volume is plugged in");
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    filesystem.set_volume(drive.path(), "vol-2");
    let report = catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    let after = children_of(&catalogue, "1");
    let film_again = file_below(&catalogue, &after[0].0, "film.mkv").expect("the film is there");
    assert_ne!(film, film_again, "the file was given a new identifier");
    assert!(
        catalogue.system_update_id() > update_id_before,
        "the identifier of the catalogue changed, so that a client browses again"
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| matches!(change.change, VolumeChange::Replaced { .. })),
        "the volume is reported as replaced: {:?}",
        report.changes
    );
    assert!(
        report.rekeyed().contains(&"Films"),
        "the report names the root whose files were given new identifiers"
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("different volume")),
        "the person is told that it is another volume: {:?}",
        catalogue.diagnostics()
    );
}

/// The files of the other roots are not touched by the loss of a drive.
#[test]
fn the_other_roots_are_not_touched_by_the_loss_of_a_drive() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let track = file_below(&catalogue, &before[1].0, "track.mp3").expect("the track is there");

    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    let after = children_of(&catalogue, "1");
    let track_again = file_below(&catalogue, &after[1].0, "track.mp3").expect("the track is there");
    assert_eq!(
        track, track_again,
        "the file of the other root kept its identifier"
    );
    assert!(
        catalogue.path_of(&track_again).is_some(),
        "the file of the other root is still served"
    );
}

/// The server notices that a drive came back without being asked to.
#[test]
fn the_server_notices_that_a_drive_came_back() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    fs::create_dir_all(drive.path()).expect("the drive is plugged in");
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    assert!(
        registry.differs(&roots, &filesystem),
        "the memory of the volumes knows that the drive is back"
    );
}

/// A volume that comes back under another identity is reported as a
/// replacement, even when the catalogue saw that it was gone in between.
#[test]
fn a_volume_that_comes_back_under_another_identity_is_a_replacement() {
    let (drive, disk) = library();
    let filesystem = ScriptedFileSystem::new();
    filesystem.set_volume(drive.path(), "vol-1");
    let mut registry = VolumeRegistry::default();
    let roots = roots_of(drive.path(), disk.path());
    let mut catalogue =
        Catalogue::build_with(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry).catalogue;

    let before = children_of(&catalogue, "1");
    let film = file_below(&catalogue, &before[0].0, "film.mkv").expect("the film is there");

    // The drive is unplugged and the catalogue sees that it is gone.
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    // Another volume is plugged in at the same place.
    fs::create_dir_all(drive.path()).expect("another volume is plugged in");
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    filesystem.set_volume(drive.path(), "vol-2");
    let report = catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    let after = children_of(&catalogue, "1");
    let film_again = file_below(&catalogue, &after[0].0, "film.mkv").expect("the film is there");
    assert_ne!(film, film_again, "the file was given a new identifier");
    assert!(
        report
            .changes
            .iter()
            .any(|change| matches!(change.change, VolumeChange::Replaced { .. })),
        "the volume is reported as replaced: {:?}",
        report.changes
    );
    assert!(
        report.rekeyed().contains(&"Films"),
        "the report names the root whose files were given new identifiers"
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("different volume")),
        "the person is told that it is another volume: {:?}",
        catalogue.diagnostics()
    );
}

/// A root that is gone is asked about once, not at every poll.
#[test]
fn a_root_that_is_gone_is_asked_about_once() {
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

    // The drive is unplugged.
    fs::remove_dir_all(drive.path()).expect("the drive is unplugged");
    assert!(
        registry.differs(&roots, &filesystem),
        "the memory of the volumes notices that the drive is gone"
    );

    // The catalogue is read again; the identity of the drive is remembered.
    catalogue.refresh(&roots, SymlinkPolicy::Skip, &filesystem, &mut registry);

    assert!(
        !registry.differs(&roots, &filesystem),
        "the memory of the volumes does not ask the question again"
    );
}
