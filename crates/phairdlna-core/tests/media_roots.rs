//! The acceptance criteria of F-08: what the scanner does with a root.
//!
//! What: the tests that read a real directory and check what the catalogue
//!       makes of it.
//! Why: the scanner touches the filesystem of the owner. The rules it follows —
//!       no hidden files, no excluded folders, no writes, no guessing — are
//!       the rules that keep the server from publishing something it was not
//!       asked to publish (`FUN-003`, `FUN-004`, `FUN-005`, `FUN-007`,
//!       `FUN-013`, `FUN-018`).
//! Example: a root that holds one film produces one folder and one file.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource, MediaKind};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// The bytes of a file the sniffer recognises as an MP4.
const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

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

/// Writes a file, creating the folders above it.
fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the folder is created");
    }
    fs::write(path, content).expect("the file is written");
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

/// A root with one film produces one folder and one file.
#[test]
fn a_root_is_scanned_into_a_tree() {
    let directory = tempdir().expect("a temporary directory");
    let films = directory.path().join("Films");
    write_file(&films.join("King Kong.mkv"), b"not really a film");

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &root), vec!["Films"]);
    let folder = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a folder");
    let file = catalogue
        .container(&folder.id)
        .expect("the folder is known")
        .children
        .first()
        .cloned()
        .expect("the folder has a file");
    assert_eq!(file.title, "King Kong.mkv");
    assert_eq!(file.kind, MediaKind::Movie);
    assert_eq!(file.metadata.mime_type.as_deref(), Some("video/x-matroska"));
    assert_eq!(file.metadata.size, Some(17));
}

/// A folder of the library is a container with the number of its children.
#[test]
fn a_folder_is_a_container() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Films/King Kong.mkv"), b"a film");
    write_file(&directory.path().join("Films/Godzilla.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);
    let folder = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a folder");

    assert_eq!(folder.kind, MediaKind::Folder);
    assert_eq!(folder.child_count, Some(2));
    assert_eq!(
        child_titles(&catalogue, &folder.id),
        vec!["Godzilla.mkv", "King Kong.mkv"]
    );
}

/// A root that cannot be read is reported, and the library stays browsable.
#[test]
fn a_missing_root_is_reported_and_the_library_stays_browsable() {
    let directory = tempdir().expect("a temporary directory");
    let missing = directory.path().join("not there");

    let catalogue = catalogue_over(&missing, Vec::new());

    assert!(!catalogue.roots()[0].present);
    assert!(
        !catalogue.diagnostics().is_empty(),
        "the missing root must be reported"
    );
    assert!(catalogue.container("1").is_some());
    let root = root_container_id(&catalogue);
    assert!(child_titles(&catalogue, &root).is_empty());
}

/// An excluded folder is not indexed.
#[test]
fn an_excluded_folder_is_not_indexed() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Films/King Kong.mkv"), b"a film");
    write_file(&directory.path().join("Private/notes.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), vec!["Private".to_owned()]);
    let root = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &root), vec!["Films"]);
}

/// A hidden file is not indexed.
#[test]
fn a_hidden_file_is_not_indexed() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join(".hidden.mkv"), b"a film");
    write_file(
        &directory.path().join("._King Kong.mkv"),
        b"a resource fork",
    );
    write_file(&directory.path().join("King Kong.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &root), vec!["King Kong.mkv"]);
}

/// A file the catalogue cannot place stays out of the tree.
#[test]
fn a_file_of_an_unknown_kind_stays_out_of_the_tree() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("notes.txt"), b"just text");
    write_file(&directory.path().join("King Kong.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &root), vec!["King Kong.mkv"]);
}
/// A file without a name is placed by its content.
#[test]
fn a_file_without_a_name_is_placed_by_its_content() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("holiday"), MP4_HEAD);

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);
    let file = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a child");

    assert_eq!(file.metadata.mime_type.as_deref(), Some("video/mp4"));
    assert_eq!(file.kind, MediaKind::Movie);
}

/// The scanner never writes to the library.
#[test]
fn the_scanner_never_writes_to_the_library() {
    let directory = tempdir().expect("a temporary directory");
    let film = directory.path().join("Films/King Kong.mkv");
    write_file(&film, b"a film");
    let before = fs::metadata(&film).expect("the film is there");

    let catalogue = catalogue_over(directory.path(), Vec::new());

    let after = fs::metadata(&film).expect("the film is still there");
    assert_eq!(before.len(), after.len());
    assert_eq!(before.modified().ok(), after.modified().ok());
    assert_eq!(
        fs::read_dir(directory.path())
            .expect("the root is readable")
            .count(),
        1
    );
    assert_eq!(catalogue.diagnostics().len(), 0);
}

/// A playlist is a container of the catalogue, not a file it delivers.
///
/// The M2.2 baseline indexed a playlist as an ordinary file, because the
/// server could not read one. Since M2.3 the server reads the playlist and
/// shows it as the container of the tracks it names (`FUN-015`).
#[test]
fn a_playlist_is_indexed() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("favourites.m3u"), b"#EXTM3U\n");

    let catalogue = catalogue_over(directory.path(), Vec::new());
    let root = root_container_id(&catalogue);
    let file = catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .first()
        .cloned()
        .expect("the root has a child");

    assert_eq!(file.kind, MediaKind::Playlist);
    assert!(file.metadata.mime_type.is_none());
    assert!(
        catalogue.path_of(&file.id).is_none(),
        "a playlist is not a file the server delivers"
    );
}

/// A tree that is deeper than the limit is cut off, and the cut is reported.
#[test]
fn a_tree_deeper_than_the_limit_is_reported() {
    let directory = tempdir().expect("a temporary directory");
    let mut path = directory.path().to_path_buf();
    for level in 0..40 {
        path = path.join(format!("level{level}"));
    }
    write_file(&path.join("deep.mkv"), b"a film");

    let catalogue = catalogue_over(directory.path(), Vec::new());

    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("deeper than the limit")),
        "the cut must be reported: {:?}",
        catalogue.diagnostics()
    );
}

/// A root whose path holds a space, a non-ASCII character and an emoji is
/// scanned like any other root.
#[test]
fn a_root_with_an_awkward_name_is_scanned() {
    let directory = tempdir().expect("a temporary directory");
    let root = directory.path().join("Mes films — 日本 🎬");
    write_file(&root.join("King Kong.mkv"), b"a film");

    let catalogue = catalogue_over(&root, Vec::new());
    let container = root_container_id(&catalogue);

    assert_eq!(child_titles(&catalogue, &container), vec!["King Kong.mkv"]);
}
