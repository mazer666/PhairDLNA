//! The acceptance criteria of F-09 on object identifiers.
//!
//! What: the tests that check the identifiers of the catalogue.
//! Why: the identifier is the only thing a client keeps. It must not change
//!       when the library is rescanned, it must not change when the library
//!       moves, and it must not carry the name of a file into a URL that a
//!       third party can read (`FUN-004`, `INT-062`, `SEC-005`).
//! Example: the same file gets the same identifier after a second scan.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{is_object_id, Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// Builds a catalogue over one root.
fn catalogue_over(path: &Path, name: &str) -> Catalogue {
    let root = MediaRoot {
        name: name.to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// The identifiers of every container of the catalogue, with their children.
fn identifiers(catalogue: &Catalogue) -> Vec<(String, Vec<String>)> {
    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    let mut found = Vec::new();
    let mut queue = vec![root];
    while let Some(id) = queue.pop() {
        let Some(container) = catalogue.container(&id) else {
            continue;
        };
        let children: Vec<String> = container
            .children
            .iter()
            .map(|child| format!("{}|{}", child.id, child.title))
            .collect();
        found.push((id.clone(), children));
        queue.extend(container.children.iter().map(|child| child.id.clone()));
    }
    found.sort();
    found
}

/// The identifier of a file does not change when the library is rescanned.
#[test]
fn the_identifier_of_a_file_is_the_same_after_a_second_scan() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let first = identifiers(&catalogue_over(directory.path(), "Films"));
    let second = identifiers(&catalogue_over(directory.path(), "Films"));

    assert_eq!(first, second);
    assert!(!first.is_empty());
}

/// The identifier of a file does not change when the library moves.
#[test]
fn the_identifier_of_a_file_does_not_change_when_the_library_moves() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let first = identifiers(&catalogue_over(directory.path(), "Films"));
    let moved = tempdir().expect("another temporary directory");
    fs::copy(
        directory.path().join("King Kong.mkv"),
        moved.path().join("King Kong.mkv"),
    )
    .expect("the file is copied");
    let second = identifiers(&catalogue_over(moved.path(), "Films"));

    assert_eq!(first, second);
}

/// The identifier of a file carries no part of the path.
#[test]
fn the_identifier_carries_no_part_of_the_path() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("Secret Documentary.mkv"), b"a film")
        .expect("the file is written");

    let catalogue = catalogue_over(directory.path(), "Films");
    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    let file = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a file");

    assert!(is_object_id(&file.id));
    assert!(!file.id.contains("Secret"));
    assert!(!file.id.contains("Documentary"));
    assert!(!file.id.contains("mkv"));
}

/// Two roots keep the same relative path apart.
#[test]
fn two_roots_keep_their_files_apart() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");
    let roots = vec![
        MediaRoot {
            name: "Films".to_owned(),
            path: directory.path().to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        },
        MediaRoot {
            name: "Archive".to_owned(),
            path: directory.path().to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        },
    ];

    let catalogue = Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
    let children = catalogue
        .container("1")
        .expect("the content directory is known")
        .children;

    assert_eq!(children.len(), 2);
    assert_ne!(children[0].id, children[1].id);
}

/// Two names of one file are one object, not two (`FUN-011`).
///
/// A hard link is a second name of a file that is already there. The library
/// must not count it twice.
#[cfg(unix)]
#[test]
fn a_hard_link_is_not_a_second_object() {
    let directory = tempdir().expect("a temporary directory");
    let film = directory.path().join("King Kong.mkv");
    fs::write(&film, b"a film").expect("the file is written");
    std::fs::hard_link(&film, directory.path().join("King Kong (copy).mkv"))
        .expect("the link is made");

    let catalogue = catalogue_over(directory.path(), "Films");
    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    let children = catalogue
        .container(&root)
        .expect("the root is known")
        .children;

    assert_eq!(children.len(), 1, "one file, one object: {children:?}");
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("another name")),
        "the second name must be reported: {:?}",
        catalogue.diagnostics()
    );
}
