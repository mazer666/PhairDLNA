//! The acceptance criterion of F-08 on symbolic links.
//!
//! What: the tests that put a symbolic link into a root and check what the
//!       catalogue makes of it.
//! Why: a symbolic link is the one way out of a root. The policy decides
//!       whether it is followed, and the rule that decides whether the target
//!       lies inside the root is not negotiable (`FUN-006`, `SEC-003`,
//!       `CFG-007`).
//! Example: a link that points outside the root is never indexed, whatever the
//!       policy says.
//!
//! The tests run on Unix only: on Windows a symbolic link needs a privilege
//! that a test must not assume.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// Builds a catalogue over one root with the policy of the server.
fn catalogue_over(path: &Path, policy: SymlinkPolicy) -> Catalogue {
    catalogue_with(path, policy, None)
}

/// Builds a catalogue over one root with a policy of its own.
fn catalogue_with(path: &Path, server: SymlinkPolicy, own: Option<SymlinkPolicy>) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: own,
    };
    Catalogue::build(&[root], server, &LocalFileSystem)
}

/// The titles of the children of the root container.
fn root_titles(catalogue: &Catalogue) -> Vec<String> {
    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .into_iter()
        .map(|child| child.title)
        .collect()
}

/// Writes a film and a link to it.
fn film_with_link(directory: &Path) {
    let film = directory.join("King Kong.mkv");
    fs::write(&film, b"a film").expect("the file is written");
    symlink(&film, directory.join("Linked.mkv")).expect("the link is made");
}

/// A link is not followed when the policy says so.
#[test]
fn a_link_is_not_followed_by_default() {
    let directory = tempdir().expect("a temporary directory");
    film_with_link(directory.path());

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::Skip);

    assert_eq!(root_titles(&catalogue), vec!["King Kong.mkv"]);
}

/// A link is transparent when the policy says so: the target is indexed where
/// it lives, and the link itself is not an entry.
#[test]
fn a_link_is_transparent_when_the_policy_says_so() {
    let directory = tempdir().expect("a temporary directory");
    film_with_link(directory.path());

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::Follow);

    assert_eq!(root_titles(&catalogue), vec!["King Kong.mkv"]);
}

/// A link is an entry of its own when the policy says so.
#[test]
fn a_link_is_indexed_as_its_own_entry() {
    let directory = tempdir().expect("a temporary directory");
    film_with_link(directory.path());

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::IndexAsLink);

    assert_eq!(root_titles(&catalogue), vec!["King Kong.mkv", "Linked.mkv"]);
}

/// A root may follow links that the server does not follow (`CFG-007`).
#[test]
fn a_root_may_have_a_policy_of_its_own() {
    let directory = tempdir().expect("a temporary directory");
    film_with_link(directory.path());

    let catalogue = catalogue_with(
        directory.path(),
        SymlinkPolicy::Skip,
        Some(SymlinkPolicy::IndexAsLink),
    );

    assert_eq!(root_titles(&catalogue), vec!["King Kong.mkv", "Linked.mkv"]);
}

/// A link that points outside the root is never indexed.
#[test]
fn a_link_that_points_outside_the_root_is_never_indexed() {
    let directory = tempdir().expect("a temporary directory");
    let outside = tempdir().expect("a temporary directory");
    let film = outside.path().join("Secret.mkv");
    fs::write(&film, b"a film").expect("the file is written");
    symlink(&film, directory.path().join("Linked.mkv")).expect("the link is made");

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::Follow);

    assert!(root_titles(&catalogue).is_empty());
    assert!(
        !catalogue.diagnostics().is_empty(),
        "the link that leaves the root must be reported"
    );
}

/// A link to a folder outside the root is never walked.
#[test]
fn a_link_to_a_folder_outside_the_root_is_never_walked() {
    let directory = tempdir().expect("a temporary directory");
    let outside = tempdir().expect("a temporary directory");
    fs::write(outside.path().join("Secret.mkv"), b"a film").expect("the file is written");
    symlink(outside.path(), directory.path().join("Elsewhere")).expect("the link is made");

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::Follow);

    assert!(root_titles(&catalogue).is_empty());
}

/// A link that is repointed after the scan does not change what the catalogue
/// serves: the catalogue holds the target, not the link (`SEC-001`).
#[test]
fn a_link_that_is_repointed_after_the_scan_cannot_escape_the_root() {
    let directory = tempdir().expect("a temporary directory");
    let outside = tempdir().expect("a temporary directory");
    let real = directory.path().join("real");
    fs::create_dir_all(&real).expect("the folder is created");
    let film = real.join("King Kong.mkv");
    fs::write(&film, b"a film").expect("the file is written");
    let link = directory.path().join("Linked.mkv");
    symlink(&film, &link).expect("the link is made");

    let catalogue = catalogue_over(directory.path(), SymlinkPolicy::IndexAsLink);
    let entry = catalogue
        .container(
            &catalogue
                .container("1")
                .expect("the content directory is known")
                .children
                .first()
                .expect("the content directory has a root")
                .id,
        )
        .expect("the root is known")
        .children
        .into_iter()
        .find(|child| child.title == "Linked.mkv")
        .expect("the link is indexed");

    let secret = outside.path().join("Secret.mkv");
    fs::write(&secret, b"a secret").expect("the file is written");
    fs::remove_file(&link).expect("the link is removed");
    symlink(&secret, &link).expect("the link is repointed");

    let path = catalogue
        .path_of(&entry.id)
        .expect("the entry has a path")
        .to_path_buf();
    assert_eq!(path, fs::canonicalize(&film).expect("the film is there"));
}
