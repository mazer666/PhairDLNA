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
use std::sync::Arc;
use std::thread;

use phairdlna_core::catalogue::index::CatalogueIndex;
use phairdlna_core::catalogue::ContentSource;
use support::index::{
    browse_of, catalogue_over, container_titled, entry_count, index_path, item_titled, library_with,
};
use tempfile::tempdir;

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

/// A playlist survives the round trip through the index.
///
/// The playlist is the hard case: a file lives in a folder *and* in a playlist,
/// and the playlist gives its entries a title of its own (`FUN-015`).
#[test]
fn a_playlist_survives_the_round_trip() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");
    fs::write(directory.path().join("Nightfall.mkv"), b"a film").expect("the file is written");
    fs::write(
        directory.path().join("Best of.m3u"),
        b"King Kong.mkv\nNightfall.mkv\n",
    )
    .expect("the playlist is written");
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let mut catalogue = catalogue_over(directory.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");

    let loaded = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    let playlist = container_titled(&loaded, "Best of.m3u");
    let children: Vec<String> = loaded
        .container(&playlist)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect();
    assert_eq!(
        children,
        vec!["King Kong.mkv".to_owned(), "Nightfall.mkv".to_owned()],
        "the playlist keeps its entries and their order"
    );
    assert_eq!(
        browse_of(&loaded),
        browse_of(&catalogue),
        "the catalogue of the index must show the same library to a client"
    );
}

/// A subtitle survives the round trip through the index.
///
/// The sidecar is an additional resource of the film, and the client asks for
/// it by its identifier; the index must keep both (`FUN-016`).
#[test]
fn a_subtitle_survives_the_round_trip() {
    let directory = tempdir().expect("a temporary directory");
    fs::write(directory.path().join("Nightfall.mkv"), b"a film").expect("the file is written");
    fs::write(
        directory.path().join("Nightfall.vtt"),
        b"WEBVTT\n\n00:00.000 --> 00:02.000\nA line.\n",
    )
    .expect("the subtitle is written");
    let (_folder, path) = index_path();

    let opened = CatalogueIndex::open(&path).expect("the index opens");
    let mut catalogue = catalogue_over(directory.path());
    opened
        .index
        .store(&mut catalogue)
        .expect("the catalogue is written");

    let loaded = opened
        .index
        .load()
        .expect("the catalogue is read")
        .expect("the index holds a catalogue");
    let film = item_titled(&loaded, "Nightfall.mkv");
    let sidecar = film
        .metadata
        .subtitle
        .as_ref()
        .expect("the film carries its subtitle");
    assert_eq!(sidecar.mime_type, "text/vtt");
    assert!(
        loaded.item_of(&sidecar.id).is_some(),
        "the sidecar is known"
    );
}
