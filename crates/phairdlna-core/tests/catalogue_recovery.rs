//! The recovery of the index from a damaged file.
//!
//! What: the tests that damage the file of the index and check that the server
//!       moves it aside, makes a new one and reads the library again.
//! Why: the index is a cache, and a cache that cannot be read must cost a
//!       re-scan and nothing else — never a server that does not start
//!       (`FUN-010`).
//! Example: a file of bytes that are not a database is moved aside under the
//!       name `.corrupt.<moment>`, and the library is served again.

mod support;

use std::fs;

use phairdlna_core::catalogue::index::CatalogueIndex;
use support::index::{catalogue_over, entry_count, index_path, library_with};

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
