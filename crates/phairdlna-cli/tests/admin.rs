//! Tests of the administration surface of the daemon.
//!
//! What: proves that a running daemon with the administration surface answers
//!       the status, the diagnostics and the scan of the library, and that it
//!       refuses every request that does not come from the owner of the
//!       machine.
//! Why: the surface is the door into the server. A door that opens for a page
//!       of the internet is a door into the files of the user (`SEC-071`,
//!       `SEC-TH-23`).
//! Example: `the_status_of_the_server_is_answered` asks the surface of a
//!          running daemon and expects the answer `200`.

#![cfg(feature = "admin")]

mod support;

use support::Server;

/// The positive case: the status of the server is answered.
#[test]
fn the_status_of_the_server_is_answered() {
    let server = Server::start_with_admin();

    let answer = server.admin_get("/api/status");

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("\"uptime_seconds\""),
        "{}",
        answer.text()
    );
    assert!(
        answer.text().contains("\"active_streams\":0"),
        "no stream runs: {}",
        answer.text()
    );
    assert!(
        answer.text().contains("\"source_revision\""),
        "the status names the revision: {}",
        answer.text()
    );
    assert!(
        answer.text().contains("\"Films\""),
        "the status names the root: {}",
        answer.text()
    );
}

/// The positive case: the diagnostics of the server are answered.
#[test]
fn the_diagnostics_of_the_server_are_answered() {
    let server = Server::start_with_admin();

    let answer = server.admin_get("/api/diagnostics");

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(answer.text().contains("\"records\""), "{}", answer.text());
    assert!(answer.text().contains("\"warnings\""), "{}", answer.text());
}

/// The positive case: the library is read again.
#[test]
fn the_library_is_read_again() {
    let server = Server::start_with_admin();

    let answer = server.admin_post("/api/catalogue/rescan");

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("\"rescanned\":true"),
        "{}",
        answer.text()
    );
    assert!(answer.text().contains("\"objects\""), "{}", answer.text());
}

/// The positive case: the diagnostics do not name the files of the user.
#[test]
fn the_diagnostics_do_not_name_the_files_of_the_user() {
    let server = Server::start_with_admin();

    let answer = server.admin_get("/api/diagnostics");

    assert!(
        !answer.text().contains("King Kong.mkv"),
        "the file of the user is not named: {}",
        answer.text()
    );
}

/// The negative case: the media server does not know the routes of the
/// administration surface.
///
/// The surface has its own listener; a request for its path on the port of
/// the media server finds nothing (`OPS-024`).
#[test]
fn the_media_server_does_not_know_the_routes_of_the_surface() {
    let server = Server::start_with_admin();

    let answer = server.get("/api/status");

    assert_eq!(answer.status, 404, "{}", answer.text());
}

/// The negative case: a request without the token is refused.
#[test]
fn a_request_without_the_token_is_refused() {
    let server = Server::start_with_admin();
    let address = server.admin_address();

    let answer = server.admin_request(&format!(
        "GET /api/status HTTP/1.1\r\nHost: {address}\r\nX-Phair-Admin: 1\r\n\
         Connection: close\r\nContent-Length: 0\r\n\r\n"
    ));

    assert_eq!(answer.status, 401, "{}", answer.text());
}

/// The negative case: a request that names another host is refused.
///
/// The name is what an attacker controls; the surface answers only to the
/// loopback (`SEC-TH-23`).
#[test]
fn a_request_that_names_another_host_is_refused() {
    let server = Server::start_with_admin();

    let answer = server.admin_request(&format!(
        "GET /api/status HTTP/1.1\r\nHost: attacker.example\r\n\
         Authorization: Bearer {}\r\nX-Phair-Admin: 1\r\n\
         Connection: close\r\nContent-Length: 0\r\n\r\n",
        server.admin_token()
    ));

    assert_eq!(answer.status, 421, "{}", answer.text());
}

/// The negative case: a request without the custom header is refused.
#[test]
fn a_request_without_the_custom_header_is_refused() {
    let server = Server::start_with_admin();
    let address = server.admin_address();

    let answer = server.admin_request(&format!(
        "GET /api/status HTTP/1.1\r\nHost: {address}\r\n\
         Authorization: Bearer {}\r\n\
         Connection: close\r\nContent-Length: 0\r\n\r\n",
        server.admin_token()
    ));

    assert_eq!(answer.status, 403, "{}", answer.text());
}

/// The negative case: the token file belongs to its owner alone.
#[cfg(unix)]
#[test]
fn the_token_file_belongs_to_its_owner_alone() {
    use std::os::unix::fs::PermissionsExt;

    let server = Server::start_with_admin();
    let path = server
        .library_folder()
        .parent()
        .expect("the folder is there");

    let mode = std::fs::metadata(path.join("admin.token"))
        .expect("the token file is there")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "the file belongs to its owner alone");
}
