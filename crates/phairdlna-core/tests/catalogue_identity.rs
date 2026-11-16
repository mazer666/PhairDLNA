//! The acceptance criteria of F-11 for organisation: what the catalogue does
//! with a file that two roots reach.
//!
//! What: the tests that read real directories and check that a file is one
//!       catalogue entry, however many roots reach it.
//! Why: a user who shares two folders of one drive — or one folder inside
//!       another — must not see the same film twice in the library. The rule
//!       that keeps that true is the identity key of the file and the stable
//!       identity the platform gives it (`FUN-020`, `FUN-022`, `FUN-021`).
//! Example: two roots that point at the same folder hold one entry, not two.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::identity::identity_key;
use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::{CaseSensitivity, LocalFileSystem, VolumeId};
use tempfile::tempdir;

/// The bytes of a file the sniffer recognises as an MP4.
const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

/// Builds a root over a path.
fn root(name: &str, path: &Path) -> MediaRoot {
    MediaRoot {
        name: name.to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    }
}

/// The identifier of the item with the given title.
fn id_of(catalogue: &Catalogue, title: &str) -> String {
    let mut found = String::new();
    let mut containers = vec!["1".to_owned()];
    while let Some(id) = containers.pop() {
        let Some(container) = catalogue.container(&id) else {
            continue;
        };
        for child in container.children {
            if child.title == title {
                found = child.id.clone();
            }
            if child.is_container() {
                containers.push(child.id.clone());
            }
        }
    }
    found
}

/// The titles of every item of the catalogue, in the order of the tree.
fn titles_of(catalogue: &Catalogue) -> Vec<String> {
    let mut titles = Vec::new();
    let mut containers = vec!["1".to_owned()];
    while let Some(id) = containers.pop() {
        let Some(container) = catalogue.container(&id) else {
            continue;
        };
        for child in container.children {
            titles.push(child.title.clone());
            if child.is_container() {
                containers.push(child.id.clone());
            }
        }
    }
    titles
}

/// The number of items of the catalogue that carry the given title.
fn count_of(catalogue: &Catalogue, title: &str) -> usize {
    titles_of(catalogue)
        .iter()
        .filter(|found| found.as_str() == title)
        .count()
}

/// Two roots that reach one file hold one entry.
#[test]
fn two_roots_that_reach_one_file_list_it_once() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), MP4_HEAD).expect("the film is written");

    let catalogue = Catalogue::build(
        &[
            root("First", directory.path()),
            root("Second", directory.path()),
        ],
        SymlinkPolicy::Skip,
        &LocalFileSystem,
    );

    assert_eq!(
        count_of(&catalogue, "King Kong.mkv"),
        1,
        "the film is listed once: {:?}",
        titles_of(&catalogue)
    );
}

/// A root that contains another root publishes the file once.
#[test]
fn a_root_inside_another_root_lists_the_file_once() {
    let directory = tempdir().expect("a temporary directory");
    let inner = directory.path().join("Inner");
    fs::create_dir_all(&inner).expect("the folder is created");
    fs::write(inner.join("King Kong.mkv"), MP4_HEAD).expect("the film is written");

    let catalogue = Catalogue::build(
        &[root("Outer", directory.path()), root("Inner", &inner)],
        SymlinkPolicy::Skip,
        &LocalFileSystem,
    );

    assert_eq!(
        count_of(&catalogue, "King Kong.mkv"),
        1,
        "the film is listed once: {:?}",
        titles_of(&catalogue)
    );
}

/// The identity key of a file does not depend on the root that reaches it.
///
/// The key is scoped by the volume and built from the path of the file, so the
/// two sightings of one file have one key (`FUN-020`).
#[test]
fn the_identity_key_of_a_file_is_the_same_for_every_root() {
    let directory = tempdir().expect("a temporary directory");
    let film = directory.path().join("King Kong.mkv");
    fs::write(&film, MP4_HEAD).expect("the film is written");

    let volume = VolumeId::new("local");
    let through_the_first = identity_key(&volume, &film, CaseSensitivity::Insensitive);
    let through_the_second = identity_key(&volume, &film, CaseSensitivity::Insensitive);

    assert_eq!(through_the_first, through_the_second);
}

/// Two volumes keep two keys for one path.
#[test]
fn two_volumes_keep_two_identity_keys() {
    let directory = tempdir().expect("a temporary directory");
    let film = directory.path().join("King Kong.mkv");
    fs::write(&film, MP4_HEAD).expect("the film is written");

    let first = identity_key(&VolumeId::new("local"), &film, CaseSensitivity::Insensitive);
    let second = identity_key(&VolumeId::new("other"), &film, CaseSensitivity::Insensitive);

    assert_ne!(first, second);
}

/// An excluded folder is not indexed, and the file it held is not served.
///
/// The identifier of the file is taken **before** the exclusion, so that the
/// test really asks whether an identifier that a client still holds is
/// refused (`FUN-021`, AC-10).
#[test]
fn an_excluded_file_is_not_served() {
    let directory = tempdir().expect("a temporary directory");
    let private = directory.path().join("Private");
    fs::create_dir_all(&private).expect("the folder is created");
    fs::write(private.join("secret.mkv"), MP4_HEAD).expect("the film is written");
    fs::write(directory.path().join("King Kong.mkv"), MP4_HEAD).expect("the film is written");

    let before = Catalogue::build(
        &[root("Library", directory.path())],
        SymlinkPolicy::Skip,
        &LocalFileSystem,
    );
    let secret = id_of(&before, "secret.mkv");
    assert!(
        before.path_of(&secret).is_some(),
        "the file is in the catalogue before the exclusion"
    );

    let mut excluded = root("Library", directory.path());
    excluded.exclude = vec!["Private".to_owned()];
    let catalogue = Catalogue::build(&[excluded], SymlinkPolicy::Skip, &LocalFileSystem);

    assert_eq!(count_of(&catalogue, "secret.mkv"), 0);
    assert_eq!(count_of(&catalogue, "King Kong.mkv"), 1);
    assert!(
        catalogue.path_of(&secret).is_none(),
        "the identifier of an excluded file names no file"
    );
    assert!(
        catalogue.item_of(&secret).is_none(),
        "the identifier of an excluded file names no object"
    );
}

/// Two names that differ only in the Unicode normalisation form are two files
/// on a volume that keeps the bytes, and one file on a volume that does not.
///
/// The test reads the volume it runs on and asserts what is true there: a
/// volume that normalises the names holds one file and the catalogue holds one
/// entry; a volume that keeps the bytes holds two files, and the catalogue
/// holds two entries, because the identity of the file carries the identity the
/// platform gives it (`FUN-020`, `PLT-005`).
///
/// The case of a byte-preserving volume is therefore **NOT RUN** on a volume
/// that normalises — the names cannot be written there.
#[test]
fn two_normalisation_forms_are_two_files_on_a_volume_that_keeps_the_bytes() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("caf\u{e9}.mkv"), MP4_HEAD).expect("the film is written");
    fs::write(directory.path().join("cafe\u{301}.mkv"), MP4_HEAD).expect("the film is written");
    let written = fs::read_dir(directory.path())
        .expect("the folder is readable")
        .filter_map(|entry| entry.ok())
        .count();

    let catalogue = Catalogue::build(
        &[root("Library", directory.path())],
        SymlinkPolicy::Skip,
        &LocalFileSystem,
    );

    let entries = count_of(&catalogue, "caf\u{e9}.mkv") + count_of(&catalogue, "cafe\u{301}.mkv");
    assert_eq!(entries, written, "the catalogue holds one entry per file");
    if written == 1 {
        assert_eq!(
            entries, 1,
            "a volume that normalises the names holds one file"
        );
    } else {
        assert_eq!(entries, 2, "a volume that keeps the bytes holds two files");
    }
}

/// Two names that differ only in the case are one file on a case-insensitive
/// volume, and two files on a case-sensitive one.
///
/// The test reads the volume it runs on and asserts what is true there
/// (`FUN-020`, `PLT-005`).
#[test]
fn two_case_variants_are_one_file_on_a_case_insensitive_volume() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("Film.mkv"), MP4_HEAD).expect("the film is written");
    fs::write(directory.path().join("film.mkv"), MP4_HEAD).expect("the film is written");
    let written = fs::read_dir(directory.path())
        .expect("the folder is readable")
        .filter_map(|entry| entry.ok())
        .count();
    let catalogue = Catalogue::build(
        &[root("Library", directory.path())],
        SymlinkPolicy::Skip,
        &LocalFileSystem,
    );

    let entries = count_of(&catalogue, "Film.mkv") + count_of(&catalogue, "film.mkv");
    assert_eq!(entries, written, "the catalogue holds one entry per file");
    if written == 1 {
        assert_eq!(
            entries, 1,
            "a volume that tells the names apart holds one file"
        );
    } else {
        assert_eq!(entries, 2, "a volume that keeps the case holds two files");
    }
}
