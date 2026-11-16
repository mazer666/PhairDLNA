//! The acceptance criteria of F-08 for more than one root.
//!
//! What: the tests that read two or more roots and check that the catalogue
//!       keeps them apart, that one root that is gone does not take the others
//!       with it, and that a file is served under the identifier of its own
//!       root.
//! Why: a library of a person is rarely one folder. It is a disk on the desk,
//!       a share in the house and a folder on the laptop — and the server has
//!       to publish all of them without mixing them (`FUN-017`, `FUN-019`,
//!       `SEC-003`).
//! Example: two roots that hold a file of the same name produce two files
//!       with two identifiers, and neither of them is reachable through the
//!       other root.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::{tempdir, TempDir};
/// The bytes of a file the sniffer recognises as an MP4.
const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

/// A root of the library.
fn root(name: &str, path: &Path) -> MediaRoot {
    MediaRoot {
        name: name.to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    }
}

/// Builds a catalogue over the given roots.
fn catalogue_over(roots: &[MediaRoot]) -> Catalogue {
    Catalogue::build(roots, SymlinkPolicy::Skip, &LocalFileSystem)
}

/// Writes a file, creating the folders above it.
fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the folder is created");
    }
    fs::write(path, content).expect("the file is written");
}

/// The children of a container, with their titles.
fn children_of(catalogue: &Catalogue, container_id: &str) -> Vec<(String, String)> {
    catalogue
        .container(container_id)
        .expect("the container is in the catalogue")
        .children
        .iter()
        .map(|child| (child.id.clone(), child.title.clone()))
        .collect()
}

/// The identifier of the file with the given title below a container.
fn file_below(catalogue: &Catalogue, container_id: &str, title: &str) -> String {
    let container = catalogue
        .container(container_id)
        .expect("the container is in the catalogue");
    for child in &container.children {
        if child.title == title {
            return child.id.clone();
        }
        if let Some(found) = file_in_folders(catalogue, &child.id, title) {
            return found;
        }
    }
    panic!("the file \"{title}\" is not in the catalogue");
}

/// The identifier of the file with the given title, somewhere below a container.
fn file_in_folders(catalogue: &Catalogue, container_id: &str, title: &str) -> Option<String> {
    let container = catalogue.container(container_id)?;
    for child in &container.children {
        if child.title == title {
            return Some(child.id.clone());
        }
        if let Some(found) = file_in_folders(catalogue, &child.id, title) {
            return Some(found);
        }
    }
    None
}

/// Two roots are two folders of the content directory, with two identifiers.
#[test]
fn two_roots_are_browsable_and_keep_apart() {
    let first = tempdir().expect("a temporary directory");
    let second = tempdir().expect("another temporary directory");
    write_file(&first.path().join("film.mkv"), MP4_HEAD);
    write_file(&second.path().join("track.mp3"), b"ID3\0\0\0\0");

    let catalogue = catalogue_over(&[root("Films", first.path()), root("Music", second.path())]);

    let roots = children_of(&catalogue, "1");
    let titles: Vec<&str> = roots.iter().map(|(_, title)| title.as_str()).collect();
    assert_eq!(titles, vec!["Films", "Music"], "both roots are published");
    assert_ne!(roots[0].0, roots[1].0, "the roots have two identifiers");
}

/// The same file name in two roots is two files with two identifiers.
#[test]
fn the_same_name_in_two_roots_is_two_files() {
    let first = tempdir().expect("a temporary directory");
    let second = tempdir().expect("another temporary directory");
    write_file(&first.path().join("film.mkv"), MP4_HEAD);
    write_file(&second.path().join("film.mkv"), MP4_HEAD);

    let catalogue = catalogue_over(&[root("Films", first.path()), root("Archive", second.path())]);

    let roots = children_of(&catalogue, "1");
    let first_id = file_below(&catalogue, &roots[0].0, "film.mkv");
    let second_id = file_below(&catalogue, &roots[1].0, "film.mkv");
    assert_ne!(first_id, second_id, "the two files keep two identifiers");
    assert_eq!(
        catalogue.path_of(&first_id),
        Some(first.path().join("film.mkv").as_path()),
        "the first file is the file of the first root"
    );
    assert_eq!(
        catalogue.path_of(&second_id),
        Some(second.path().join("film.mkv").as_path()),
        "the second file is the file of the second root"
    );
}

/// A root that cannot be read does not take the other roots with it.
#[test]
fn a_root_that_is_gone_does_not_stop_the_others() {
    let first = tempdir().expect("a temporary directory");
    let second = tempdir().expect("another temporary directory");
    write_file(&second.path().join("film.mkv"), MP4_HEAD);
    let missing = first.path().join("not-there");

    let catalogue = catalogue_over(&[root("Films", &missing), root("Music", second.path())]);

    assert!(
        !catalogue.roots()[0].present,
        "the root that is gone is absent"
    );
    assert!(
        catalogue.roots()[1].present,
        "the root that is there is present"
    );
    let roots = children_of(&catalogue, "1");
    assert_eq!(roots.len(), 2, "both roots are published");
    let music = file_below(&catalogue, &roots[1].0, "film.mkv");
    assert!(
        catalogue.path_of(&music).is_some(),
        "the file of the root that is there is served"
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("Films")),
        "the root that is gone is reported: {:?}",
        catalogue.diagnostics()
    );
}

/// A link from one root into another is not followed.
#[cfg(unix)]
#[test]
fn a_link_from_one_root_into_another_is_not_followed() {
    let first = tempdir().expect("a temporary directory");
    let second = tempdir().expect("another temporary directory");
    write_file(&second.path().join("film.mkv"), MP4_HEAD);
    std::os::unix::fs::symlink(
        second.path().join("film.mkv"),
        first.path().join("link.mkv"),
    )
    .expect("the link is made");

    let roots = [
        MediaRoot {
            name: "Films".to_owned(),
            path: first.path().to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: Some(SymlinkPolicy::Follow),
        },
        root("Archive", second.path()),
    ];
    let catalogue = Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);

    let published = children_of(&catalogue, "1");
    assert!(
        file_in_folders(&catalogue, &published[0].0, "link.mkv").is_none(),
        "the link that points into the other root is not indexed"
    );
    assert!(
        file_below_is_absent(&catalogue, &published[0].0, "film.mkv"),
        "the file of the other root is not published under the first root"
    );
    let film = file_below(&catalogue, &published[1].0, "film.mkv");
    assert_eq!(
        catalogue.path_of(&film),
        Some(second.path().join("film.mkv").as_path()),
        "the file is published under its own root"
    );
}

/// Whether a file with the given title is below a container.
fn file_below_is_absent(catalogue: &Catalogue, container_id: &str, title: &str) -> bool {
    file_in_folders(catalogue, container_id, title).is_none()
}

/// A root inside another root is published once, under its own identifier.
#[test]
fn a_root_inside_another_root_is_published_once() {
    let outer = tempdir().expect("a temporary directory");
    let inner = outer.path().join("Inner");
    fs::create_dir_all(&inner).expect("the inner folder is created");
    write_file(&inner.join("film.mkv"), MP4_HEAD);

    let catalogue = catalogue_over(&[root("Library", outer.path()), root("Films", &inner)]);

    let published = children_of(&catalogue, "1");
    assert_eq!(published.len(), 2, "both roots are published");
    assert!(
        file_below_is_absent(&catalogue, &published[0].0, "Inner"),
        "the inner root is not published a second time below the outer root"
    );
    let film = file_below(&catalogue, &published[1].0, "film.mkv");
    assert_eq!(
        catalogue.path_of(&film),
        Some(inner.join("film.mkv").as_path()),
        "the file is published under the inner root"
    );
}

/// The path of a file is the path the operating system reported.
#[test]
fn the_path_of_a_file_is_the_path_of_the_disk() {
    let folder = tempdir().expect("a temporary directory");
    // The name is written in the decomposed form, as a file system may hand it
    // back (`FUN-019`).
    let name = "Cafe\u{301}.mkv";
    let path = folder.path().join(name);
    write_file(&path, MP4_HEAD);

    let catalogue = catalogue_over(&[root("Films", folder.path())]);
    let published = children_of(&catalogue, "1");
    let id = file_below(&catalogue, &published[0].0, name);

    assert_eq!(
        catalogue.path_of(&id),
        Some(path.as_path()),
        "the path is the path of the disk, byte for byte"
    );
}

/// The roots of the catalogue say which of them are there.
#[test]
fn the_catalogue_says_which_roots_are_there() {
    let first = tempdir().expect("a temporary directory");
    let second = TempDir::new().expect("another temporary directory");

    let catalogue = catalogue_over(&[root("Films", first.path()), root("Music", second.path())]);

    let names: Vec<&str> = catalogue
        .roots()
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(names, vec!["Films", "Music"]);
    assert!(catalogue.roots().iter().all(|entry| entry.present));
    assert!(
        catalogue.roots().iter().all(|entry| !entry.id.is_empty()),
        "every root has an identifier"
    );
}
