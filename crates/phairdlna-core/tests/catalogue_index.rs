//! The acceptance criteria of F-09 on the catalogue as a client sees it.
//!
//! What: the tests that browse the catalogue of a real directory, and the
//!       tests that check that a browse reads nothing from the disk.
//! Why: a browse that touches the filesystem is a browse that breaks when the
//!       disk is slow, and a catalogue whose identifier changes when the
//!       library changes is a catalogue every client has to rebuild
//!       (`PERF-008`, `FUN-011`, `INT-037`).
//! Example: a catalogue that was built once answers a browse even after the
//!       library it came from is gone.

mod support;

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use phairdlna_core::upnp::Dispatcher;
use support::{browse_request, element_text_unescaped};
use tempfile::tempdir;

/// The base URL the dispatcher builds the addresses with.
const BASE_URL: &str = "http://192.168.1.20:8200";

/// Builds a catalogue over one root.
fn catalogue_over(path: &Path) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// The identifier of the container of the root.
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

/// The identifier of the folder of the library.
fn folder_container_id(catalogue: &Catalogue) -> String {
    let root = root_container_id(catalogue);
    catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a folder")
        .id
}

/// A browse of a real folder shows the files of that folder.
#[test]
fn a_browse_of_a_real_folder_shows_its_files() {
    let directory = tempdir().expect("a temporary directory");
    fs::create_dir(directory.path().join("Films")).expect("the folder is created");
    fs::write(directory.path().join("Films/King Kong.mkv"), b"a film")
        .expect("the file is written");

    let catalogue = catalogue_over(directory.path());
    let folder = folder_container_id(&catalogue);
    let dispatcher = Dispatcher::new(&catalogue, BASE_URL);
    let answer = dispatcher
        .dispatch(&browse_request(&folder, "BrowseDirectChildren", 0, 0), None)
        .body;

    let result = element_text_unescaped(&answer, "Result").expect("the result is present");
    assert!(
        result.contains("King Kong.mkv"),
        "the browse must show the file: {result}"
    );
    assert!(
        result.contains("video/x-matroska"),
        "the browse must name the format: {result}"
    );
}

/// A folder of the library is listed as a container and not as an item.
#[test]
fn a_folder_is_listed_as_a_container() {
    let directory = tempdir().expect("a temporary directory");
    fs::create_dir(directory.path().join("Films")).expect("the folder is created");
    fs::write(directory.path().join("Films/King Kong.mkv"), b"a film")
        .expect("the file is written");

    let catalogue = catalogue_over(directory.path());
    let root = root_container_id(&catalogue);
    let dispatcher = Dispatcher::new(&catalogue, BASE_URL);
    let answer = dispatcher
        .dispatch(&browse_request(&root, "BrowseDirectChildren", 0, 0), None)
        .body;

    let result = element_text_unescaped(&answer, "Result").expect("the result is present");
    assert!(
        result.contains("<container"),
        "the folder must be a container: {result}"
    );
    assert!(
        result.contains("childCount=\"1\""),
        "the folder must carry the number of its children: {result}"
    );
}

/// A browse reads nothing from the disk: it answers from the catalogue alone.
#[test]
fn a_browse_reads_nothing_from_the_disk() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let catalogue = catalogue_over(directory.path());
    fs::remove_dir_all(directory.path()).expect("the library is gone");

    let root = root_container_id(&catalogue);
    let container = catalogue.container(&root).expect("the root is known");
    assert_eq!(container.children.len(), 1);
    assert_eq!(container.children[0].title, "King Kong.mkv");
}

/// The identifier of the catalogue changes when the library changes.
#[test]
fn the_update_id_changes_when_the_library_changes() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let first = catalogue_over(directory.path()).system_update_id();
    fs::write(directory.path().join("Godzilla.mkv"), b"a film").expect("the file is written");
    let second = catalogue_over(directory.path()).system_update_id();

    assert_ne!(first, second);
}

/// The identifier of the catalogue is the same for two scans of the same library.
#[test]
fn the_update_id_is_the_same_for_two_scans_of_the_same_library() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let first = catalogue_over(directory.path()).system_update_id();
    let second = catalogue_over(directory.path()).system_update_id();

    assert_eq!(first, second);
}

/// The identifier of the catalogue never goes backwards (`INT-037`).
#[test]
fn the_update_id_never_goes_backwards() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let first = catalogue_over(directory.path()).system_update_id();
    fs::write(directory.path().join("Godzilla.mkv"), b"a film").expect("the file is written");
    let second = catalogue_over(directory.path()).system_update_id();
    fs::remove_file(directory.path().join("Godzilla.mkv")).expect("the file is removed");
    let third = catalogue_over(directory.path()).system_update_id();

    assert!(second > first, "{second} must be higher than {first}");
    assert!(third > second, "{third} must be higher than {second}");
}

/// The identifier of the catalogue changes when a file changes (`INT-037`).
#[test]
fn the_update_id_changes_when_a_file_changes() {
    let directory = tempdir().expect("a temporary directory");
    let film = directory.path().join("King Kong.mkv");
    fs::write(&film, b"a film").expect("the file is written");

    let first = catalogue_over(directory.path()).system_update_id();
    fs::write(&film, b"a different film, with other bytes").expect("the file is written");
    let second = catalogue_over(directory.path()).system_update_id();

    assert_ne!(first, second, "the change of the file must be seen");
    assert!(second > first, "{second} must be higher than {first}");
}

/// The metadata of an item can be asked for by the identifier of the item.
#[test]
fn the_metadata_of_an_item_can_be_asked_for() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let catalogue = catalogue_over(directory.path());
    let root = root_container_id(&catalogue);
    let file = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a file");
    let dispatcher = Dispatcher::new(&catalogue, BASE_URL);

    let answer = dispatcher
        .dispatch(&browse_request(&file.id, "BrowseMetadata", 0, 0), None)
        .body;

    let result = element_text_unescaped(&answer, "Result").expect("the result is present");
    assert!(
        result.contains("King Kong.mkv"),
        "the answer must carry the item: {result}"
    );
    assert!(
        !answer.contains("701"),
        "the item must not be refused: {answer}"
    );
}

/// The root container has no parent but itself, and says so with `-1`.
#[test]
fn the_root_container_has_the_parent_id_minus_one() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");

    let catalogue = catalogue_over(directory.path());
    let dispatcher = Dispatcher::new(&catalogue, BASE_URL);

    let answer = dispatcher
        .dispatch(&browse_request("0", "BrowseMetadata", 0, 0), None)
        .body;

    let result = element_text_unescaped(&answer, "Result").expect("the result is present");
    assert!(
        result.contains("parentID=\"-1\""),
        "the root container must have the parent id -1: {result}"
    );
}
