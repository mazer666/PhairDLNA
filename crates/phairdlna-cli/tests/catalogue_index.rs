//! The index of the catalogue in the daemon.
//!
//! What: the tests that show that the daemon writes its catalogue into the
//!       index on disk, that it serves the library after a restart, and that a
//!       damaged index does not stop it.
//! Why: the index is the part of the server that survives a restart; a server
//!       that cannot open it must still serve the library (`FUN-009`,
//!       `FUN-010`).
//! Example: a daemon that was stopped and started again serves the same
//!       library, and the index of the run before is still there.

mod support;

use support::soap::{browse_request, CONTENT_DIRECTORY};
use support::{Server, FILM};

/// The file of the index of a daemon that ran in the folder.
fn index_of(directory: &std::path::Path) -> std::path::PathBuf {
    directory.join("catalogue.db")
}

/// The daemon writes the catalogue of the library into its index.
#[test]
fn the_daemon_writes_the_catalogue_into_its_index() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);

    let index = index_of(directory.path());
    assert!(index.exists(), "the index of the catalogue is on disk");
    assert!(
        std::fs::metadata(&index).expect("the index is there").len() > 0,
        "the index holds the catalogue"
    );
    drop(server);
}

/// The daemon serves the library again after it was stopped and started.
#[test]
fn the_daemon_serves_the_library_after_a_restart() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let mut server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);
    server.stop();

    let server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);
    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("King Kong.mkv"),
        "the library is served after the restart: {}",
        answer.text()
    );
}

/// A damaged index does not stop the daemon.
///
/// The damaged file is kept beside the new one, and the library is read again
/// (`FUN-010`).
#[test]
fn a_damaged_index_does_not_stop_the_daemon() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let mut server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);
    server.stop();

    std::fs::write(index_of(directory.path()), b"this is not a database")
        .expect("the index is damaged");

    let server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);
    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("King Kong.mkv"),
        "the library is served although the index was damaged: {}",
        answer.text()
    );
    let kept = std::fs::read_dir(directory.path())
        .expect("the folder is read")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".corrupt."))
        .collect::<Vec<_>>();
    assert_eq!(kept.len(), 1, "the damaged file is kept: {kept:?}");
}

/// The daemon never writes to the media of the user.
#[test]
fn the_daemon_never_writes_to_the_media() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let server = Server::start_in(directory.path(), &[("King Kong.mkv", FILM)]);
    let film = directory.path().join("Library").join("King Kong.mkv");
    let before = std::fs::read(&film).expect("the film is read");

    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );
    assert_eq!(answer.status, 200, "{}", answer.text());

    assert_eq!(
        std::fs::read(&film).expect("the film is read"),
        before,
        "the media of the user must not be touched"
    );
}
