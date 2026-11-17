//! The acceptance criteria of F-09 on the index of the catalogue.
//!
//! What: the tests that write a catalogue into the index on disk, that read it
//!       back after the server stopped, that browse while the catalogue is
//!       written, and that recover from a damaged index.
//! Why: the index is a cache of the catalogue; it must survive a restart, it
//!       must never show half a catalogue, and a damaged file must never stop
//!       the server (`FUN-008`, `FUN-009`, `FUN-010`).
//! Example: a catalogue that was written into the index is read back after the
//!       index was closed and opened again.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use phairdlna_core::catalogue::index::CatalogueIndex;
use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use phairdlna_core::upnp::Dispatcher;
use support::browse_request;
use tempfile::{tempdir, TempDir};

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

/// A library with the given films in it.
fn library_with(films: &[&str]) -> TempDir {
    let directory = tempdir().expect("a temporary directory");
    for film in films {
        fs::write(directory.path().join(film), b"a film").expect("the file is written");
    }
    directory
}

/// The file of the index in a temporary folder.
///
/// The folder is returned as well: it lives as long as the test does, and the
/// file of the index must not vanish under it.
fn index_path() -> (TempDir, PathBuf) {
    let directory = tempdir().expect("a temporary directory");
    let path = directory.path().join("catalogue.db");
    (directory, path)
}

/// The number of the entries of a catalogue: the folders and the files.
fn entry_count(catalogue: &Catalogue) -> usize {
    let mut total = 0;
    let mut open = vec!["1".to_owned()];
    while let Some(id) = open.pop() {
        let Some(container) = catalogue.container(&id) else {
            continue;
        };
        for child in &container.children {
            total += 1;
            open.push(child.id.clone());
        }
    }
    total
}

/// The index is written and read back.
#[test]
fn the_index_is_written_and_read_back() {
    let library = library_with(&["King Kong.mkv", "Godzilla.mkv"]);
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    assert!(opened.recovery.is_none(), "a new index is not a repair");
    let mut catalogue = catalogue_over(library.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");

    let loaded = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    assert_eq!(entry_count(&loaded), entry_count(&catalogue));
    assert_eq!(loaded.roots().len(), catalogue.roots().len());
    assert_eq!(
        browse_of(&loaded),
        browse_of(&catalogue),
        "the catalogue of the index must show the same library to a client"
    );
}

/// The answer of a browse of the whole tree, as a client sees it.
fn browse_of(catalogue: &Catalogue) -> String {
    let dispatcher = Dispatcher::new(catalogue, BASE_URL);
    let mut answer = String::new();
    for id in ["0", "1"] {
        let body = dispatcher
            .dispatch(&browse_request(id, "BrowseDirectChildren", 0, 0), None)
            .body;
        answer.push_str(&body);
    }
    answer
}

/// The catalogue of the last run is read back after the server stopped.
#[test]
fn the_catalogue_survives_a_restart_of_the_server() {
    let library = library_with(&["King Kong.mkv"]);
    let (_folder, path) = index_path();

    {
        let opened = CatalogueIndex::open(&path).expect("the index opens");
        let mut catalogue = catalogue_over(library.path());
        opened
            .index
            .store(&mut catalogue)
            .expect("the catalogue is written");
    }

    let opened = CatalogueIndex::open(&path).expect("the index opens again");
    assert!(
        opened.recovery.is_none(),
        "a sound index must not be reported as a repair"
    );
    let loaded = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    assert!(
        entry_count(&loaded) >= 1,
        "the file of the library must still be known"
    );
}

/// The identifier of the catalogue is the same after a restart.
#[test]
fn the_identifier_of_the_catalogue_survives_a_restart() {
    let library = library_with(&["King Kong.mkv"]);
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let mut catalogue = catalogue_over(library.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");
    let before = catalogue.system_update_id();

    let loaded = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    assert_eq!(
        loaded.system_update_id(),
        before,
        "the identifier of an unchanged catalogue must not change"
    );
}

/// A browse during a rescan sees the catalogue of before, never half of one.
#[test]
fn a_browse_during_a_rescan_sees_the_previous_catalogue() {
    let library = library_with(&["King Kong.mkv"]);
    let bigger = library_with(&["King Kong.mkv", "Godzilla.mkv", "Mothra.mkv"]);
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let index = Arc::new(opened.index);
    let mut before = catalogue_over(library.path());
    index.store(&mut before).expect("the catalogue is written");

    let before_count = entry_count(&before);
    let after_count = entry_count(&catalogue_over(bigger.path()));
    let writer = Arc::clone(&index);
    let handle = thread::spawn(move || {
        let mut after = catalogue_over(bigger.path());
        writer.store(&mut after).expect("the catalogue is written");
    });

    for _ in 0..200 {
        if let Some(seen) = index.load().expect("the catalogue is read") {
            let count = entry_count(&seen);
            assert!(
                count == before_count || count == after_count,
                "a browse must see the whole catalogue of before or the whole catalogue of now, \
                 but it saw {count} entries"
            );
        }
    }
    handle.join().expect("the writer finishes");

    let seen = index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    assert_eq!(
        entry_count(&seen),
        after_count,
        "the new catalogue is written"
    );
}

/// A damaged index is moved aside, and the server goes on.
#[test]
fn a_damaged_index_is_moved_aside_and_rebuilt() {
    let library = library_with(&["King Kong.mkv"]);
    let (_folder, path) = index_path();
    fs::write(&path, b"this is not a database at all").expect("the file is damaged");

    let opened = CatalogueIndex::open(&path).expect("the index opens anyway");
    let repair = opened.recovery.expect("the damage is reported");
    assert!(
        repair.moved_to.exists(),
        "the damaged file must be kept: {}",
        repair.moved_to.display()
    );
    assert!(
        repair
            .moved_to
            .file_name()
            .expect("the file has a name")
            .to_string_lossy()
            .contains(".corrupt."),
        "the damaged file must carry the mark of the damage: {}",
        repair.moved_to.display()
    );
    assert!(
        opened.index.load().expect("the index is read").is_none(),
        "the new index must be empty"
    );

    let mut catalogue = catalogue_over(library.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written into the new index");
    let rebuilt = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    assert!(entry_count(&rebuilt) >= 1, "the library is read again");
}

/// An index of another schema is moved aside as well.
#[test]
fn an_index_of_another_schema_is_moved_aside() {
    let (_folder, path) = index_path();
    {
        let connection = rusqlite::Connection::open(&path).expect("the database is created");
        connection
            .execute_batch("CREATE TABLE something (id TEXT);")
            .expect("the table is created");
        connection
            .pragma_update(None, "user_version", 99)
            .expect("the version is written");
    }

    let opened = CatalogueIndex::open(&path).expect("the index opens anyway");
    let repair = opened.recovery.expect("the damage is reported");
    assert!(
        repair.moved_to.exists(),
        "the file of the other schema must be kept: {}",
        repair.moved_to.display()
    );
}

/// The file of the index belongs to the user alone.
#[test]
#[cfg(unix)]
fn the_file_of_the_index_belongs_to_the_user_alone() {
    use std::os::unix::fs::PermissionsExt;

    let (_folder, path) = index_path();
    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let mut catalogue = catalogue_over(library_with(&["King Kong.mkv"]).path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");

    let mode = fs::metadata(&path)
        .expect("the file is there")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "only the user may read the index");

    // The write-ahead log belongs to the index as well, and it holds the same
    // paths: it may not be read by anybody else.
    for entry in fs::read_dir(path.parent().expect("the folder is there"))
        .expect("the folder is read")
        .filter_map(|entry| entry.ok())
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("catalogue.db") {
            continue;
        }
        let mode = entry
            .metadata()
            .expect("the file is there")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "{name} may be read by the user alone");
    }
}

/// The media of the library is never written to.
#[test]
fn the_media_of_the_library_is_never_written_to() {
    let library = library_with(&["King Kong.mkv"]);
    let film = library.path().join("King Kong.mkv");
    let before = fs::read(&film).expect("the file is read");
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let mut catalogue = catalogue_over(library.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");

    assert_eq!(
        fs::read(&film).expect("the file is read"),
        before,
        "the media of the user must not be touched"
    );
}

/// A deleted index is built again from the library.
#[test]
fn a_deleted_index_is_built_again() {
    let library = library_with(&["King Kong.mkv"]);
    let (_folder, path) = index_path();

    {
        let opened = CatalogueIndex::open(&path).expect("the index opens");
        let mut catalogue = catalogue_over(library.path());
        opened
            .index
            .store(&mut catalogue)
            .expect("the catalogue is written");
    }
    fs::remove_file(&path).expect("the index is deleted");

    let opened = CatalogueIndex::open(&path).expect("the index opens again");
    assert!(
        opened.index.load().expect("the index is read").is_none(),
        "a deleted index holds nothing"
    );
    let mut catalogue = catalogue_over(library.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written again");
    assert!(
        opened.index.load().expect("the index is read").is_some(),
        "the index holds the catalogue again"
    );
}
