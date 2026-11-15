//! What the scanner says when it cannot do what it was asked.
//!
//! What: the tests of the diagnostics of a scan — a root that is missing, a
//!       folder that is too deep, a folder that cannot be read.
//! Why: a scanner that fails silently is a scanner the owner cannot trust
//!       (`FUN-007`, `SEC-094`, `UX-011`).
//! Example: a root that is a file and not a folder is reported as absent.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// Builds a catalogue over one root.
fn catalogue_over(path: &Path, exclude: Vec<String>) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude,
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// The title of the child of a container.
fn child_titles(catalogue: &Catalogue, container_id: &str) -> Vec<String> {
    catalogue
        .container(container_id)
        .expect("the container is known")
        .children
        .into_iter()
        .map(|child| child.title)
        .collect()
}

/// The identifier of the root container.
fn root_container_id(catalogue: &Catalogue) -> String {
    catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone()
}

/// Writes a file, creating the folders above it.
fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the folder is created");
    }
    fs::write(path, content).expect("the file is written");
}

/// The diagnostics of the server name the path below the root and never the
/// path of the disk (`SEC-094`, `UX-011`).
#[test]
fn the_diagnostics_name_no_absolute_path() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Films/King Kong.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), vec!["Nowhere".to_owned()]);

    for diagnostic in catalogue.diagnostics() {
        assert!(
            !diagnostic.contains(directory.path().to_string_lossy().as_ref()),
            "the diagnostic names the path of the disk: {diagnostic}"
        );
    }
}

/// A root that is a file and not a folder is not a root.
#[test]
fn a_root_that_is_not_a_folder_is_reported_as_absent() {
    let directory = tempdir().expect("a temporary directory");
    let file = directory.path().join("King Kong.mkv");
    write_file(&file, b"a film");

    let catalogue = catalogue_over(&file, Vec::new());

    assert!(!catalogue.roots()[0].present);
    assert!(!catalogue.diagnostics().is_empty());
}

/// An excluded folder is excluded whatever the case of its name.
#[test]
fn an_excluded_folder_is_excluded_whatever_its_case() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Private/secret.mkv"), b"a film");
    write_file(&directory.path().join("keep.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), vec!["private".to_owned()]);
    let root = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &root), vec!["keep.mkv"]);
}
