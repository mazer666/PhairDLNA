//! Tests of the daemon.
//!
//! What: proves that a running daemon answers the requests a client makes —
//!       the device description, the service descriptions, a SOAP browse, a
//!       media answer with a byte range and the answers a client expects to
//!       be refused.
//! Why: the daemon is the product. A server that starts but does not answer is
//!       worse than a server that does not start, and the only way to know is
//!       to speak to it over a socket.
//! Example: `the_description_is_answered` opens a connection to the daemon and
//!       reads the document a television would read.

mod support;

use std::net::TcpStream;
use std::time::Duration;

use support::soap::{browse_request, envelope, CONNECTION_MANAGER, CONTENT_DIRECTORY};
use support::{Server, FILM};

/// The positive case: the device description is served.
#[test]
fn the_description_is_answered() {
    let server = Server::start();

    let answer = server.get("/description.xml");

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("<root"),
        "the document is a UPnP document"
    );
    assert!(
        answer
            .text()
            .contains("urn:schemas-upnp-org:device:MediaServer:1"),
        "the device type is the one of a media server"
    );
    assert!(
        answer.text().contains("<UDN>uuid:"),
        "the device names itself: {}",
        answer.text()
    );
    assert_eq!(
        answer.header("content-type"),
        Some("text/xml; charset=\"utf-8\""),
        "the answer names its type"
    );
    assert!(
        answer.text().contains("/ContentDirectory.xml"),
        "the description names the service document: {}",
        answer.text()
    );
    assert!(
        answer.text().contains("/ctl/CD"),
        "the description names the control endpoint: {}",
        answer.text()
    );
    assert!(
        answer.text().contains("http://127.0.0.1:"),
        "the description carries the absolute address of the server: {}",
        answer.text()
    );
}

/// The positive case: the service descriptions are served at the paths the
/// device description advertises.
#[test]
fn the_service_descriptions_are_answered() {
    let server = Server::start();

    let content = server.get("/ContentDirectory.xml");
    let manager = server.get("/ConnectionManager.xml");

    assert_eq!(content.status, 200, "{}", content.text());
    assert!(
        content.text().contains("<scpd"),
        "the ContentDirectory service"
    );
    assert!(content.text().contains("<name>Browse</name>"));
    assert_eq!(manager.status, 200, "{}", manager.text());
    assert!(
        manager.text().contains("<scpd"),
        "the ConnectionManager service"
    );
}

/// The positive case: a SOAP browse is answered with the real catalogue.
#[test]
fn a_browse_is_answered() {
    let server = Server::start();

    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("DIDL-Lite"),
        "the result is a DIDL-Lite"
    );
    assert!(
        answer.text().contains("King Kong.mkv"),
        "the file of the library is listed: {}",
        answer.text()
    );
}

/// The positive case: the other control endpoint answers as well.
#[test]
fn the_connection_manager_is_answered() {
    let server = Server::start();

    let answer = server.post(
        "/ctl/CM",
        &envelope(CONNECTION_MANAGER, "GetProtocolInfo", &[]),
        &format!("\"{CONNECTION_MANAGER}#GetProtocolInfo\""),
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains("Source"),
        "the answer names the protocols: {}",
        answer.text()
    );
}

/// The positive case: a media file is sent, whole and in pieces.
#[test]
fn a_media_file_is_served() {
    let server = Server::start();
    let identifier = server.media_id();

    let whole = server.get(&format!("/media/{identifier}"));

    assert_eq!(whole.status, 200, "{}", whole.text());
    assert_eq!(
        whole.body, FILM,
        "the client receives the bytes of the file"
    );
    assert_eq!(
        whole.header("content-length"),
        Some(FILM.len().to_string().as_str())
    );
}

/// The positive case: a byte range is answered with `206` and the slice.
#[test]
fn a_range_is_served() {
    let server = Server::start();
    let identifier = server.media_id();

    let answer = server.get_with(&format!("/media/{identifier}"), &[("Range", "bytes=2-5")]);

    assert_eq!(answer.status, 206, "{}", answer.text());
    assert_eq!(answer.body, b"film", "the client receives the slice");
    assert_eq!(
        answer.header("content-range"),
        Some(format!("bytes 2-5/{}", FILM.len()).as_str())
    );
    assert_eq!(answer.header("content-length"), Some("4"));
}

/// The positive case: a `HEAD` answers the headers without a body.
#[test]
fn a_head_answers_without_a_body() {
    let server = Server::start();
    let identifier = server.media_id();

    let answer = server.head(&format!("/media/{identifier}"));

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(answer.body.is_empty(), "the body of a HEAD is empty");
    assert_eq!(
        answer.header("content-length"),
        Some(FILM.len().to_string().as_str())
    );
}

/// The negative case: a range behind the file is refused with `416`.
#[test]
fn a_range_behind_the_file_is_refused() {
    let server = Server::start();
    let identifier = server.media_id();

    let answer = server.get_with(
        &format!("/media/{identifier}"),
        &[("Range", "bytes=9999-10000")],
    );

    assert_eq!(answer.status, 416, "{}", answer.text());
}

/// The negative case: an unknown object is answered with `404`.
#[test]
fn an_unknown_object_is_refused() {
    let server = Server::start();

    let answer = server.get("/media/00000000000000000000000000000000");

    assert_eq!(answer.status, 404, "{}", answer.text());
}

/// The negative case: there is no artwork yet, and the server says so.
#[test]
fn the_artwork_is_not_there_yet() {
    let server = Server::start();
    let identifier = server.media_id();

    let answer = server.get(&format!("/art/{identifier}"));

    assert_eq!(answer.status, 404, "{}", answer.text());
    assert!(
        answer.body.is_empty(),
        "the answer carries no placeholder image"
    );
}

/// The positive case: the daemon stops when it is told to.
#[test]
fn the_daemon_stops_when_it_is_told_to() {
    let mut server = Server::start();
    let address = server.address();

    server.stop();

    // The port is free again: the listener is closed.
    let refused = TcpStream::connect_timeout(&address, Duration::from_secs(2));
    assert!(refused.is_err(), "the port is closed after the shutdown");
}

/// The negative case: a file that left the root is not served.
///
/// The catalogue is built at the start of the daemon. A file that is replaced
/// by a link to somewhere else afterwards must not be read (`SEC-001`,
/// `INT-098`).
#[cfg(unix)]
#[test]
fn a_file_that_left_the_root_is_not_served() {
    use std::os::unix::fs::symlink;

    let server = Server::start();
    let identifier = server.media_id();
    let outside = tempfile::tempdir().expect("another temporary directory");
    let secret = outside.path().join("Secret.mkv");
    std::fs::write(&secret, b"a secret").expect("the secret is written");
    let film = server.library().join("King Kong.mkv");
    std::fs::remove_file(&film).expect("the film is removed");
    symlink(&secret, &film).expect("the link is made");

    let answer = server.get(&format!("/media/{identifier}"));

    assert_eq!(answer.status, 404, "{}", answer.text());
    assert!(
        !answer.text().contains("a secret"),
        "the bytes outside the root are not sent"
    );
}

/// The positive case: a transfer that is running when the daemon stops is
/// allowed to finish.
///
/// The daemon that stops must not cut a film in the middle: the open transfers
/// are drained before the process ends (`OPS-008`).
#[test]
fn an_open_transfer_ends_with_the_daemon() {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let size = 512 * 1024;
    let mut server = Server::start_with_film(&vec![b'x'; size]);
    let identifier = server.media_id();
    let address = server.address();

    let mut stream = TcpStream::connect(address).expect("the daemon accepts connections");
    let request =
        format!("GET /media/{identifier} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("the request is sent");
    // The client reads slowly, so that the transfer is still running when the
    // daemon is told to stop.
    let reader = std::thread::spawn(move || {
        // The head of the answer is read first; the body is what counts. The
        // head and the first bytes of the body may arrive in one piece, and
        // the bytes behind the separator belong to the body: they are counted
        // from the start, or the body looks shorter than it is.
        let mut head = Vec::new();
        let mut chunk = [0u8; 4096];
        let mut received = 0usize;
        loop {
            let read = stream.read(&mut chunk).expect("the head is read");
            if read == 0 {
                break;
            }
            head.extend_from_slice(&chunk[..read]);
            if let Some(end) = head.windows(4).position(|window| window == b"\r\n\r\n") {
                received = head.len() - (end + 4);
                break;
            }
        }
        loop {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    received += read;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
        received
    });

    std::thread::sleep(std::time::Duration::from_millis(200));
    let began = std::time::Instant::now();
    server.stop();
    let waited = began.elapsed();

    let received = reader.join().expect("the reader ends");
    assert_eq!(
        received, size,
        "the transfer was cut off instead of being drained"
    );
    // The drain ends when the transfer ends. A daemon that waits for its
    // budget instead would hold the shutdown for ten seconds.
    assert!(
        waited < std::time::Duration::from_secs(5),
        "the shutdown waited {waited:?} for the transfer to end"
    );
}

/// The negative case: a file that shrank after the scan is answered with the
/// length it really has.
///
/// The catalogue remembers the length of the file at the moment of the scan. A
/// file that has changed since then must not make the server promise bytes it
/// does not have (`INT-081`, `INT-088`).
#[test]
fn a_file_that_shrank_is_answered_with_its_real_length() {
    let server = Server::start();
    let identifier = server.media_id();
    std::fs::write(server.library().join("King Kong.mkv"), b"12345").expect("the film is written");

    let answer = server.get_with(&format!("/media/{identifier}"), &[("Range", "bytes=0-9")]);

    assert_eq!(answer.status, 206, "{}", answer.text());
    assert_eq!(
        answer.body, b"12345",
        "the client receives the bytes there are"
    );
    assert_eq!(answer.header("content-length"), Some("5"));
    assert_eq!(
        answer.header("content-range"),
        Some("bytes 0-4/5"),
        "the range describes the bytes that are really sent"
    );
}

/// The negative case: a range behind the end of a file that shrank cannot be
/// satisfied.
///
/// The catalogue knows a longer file; the file on the disk is shorter. The
/// answer must be `416`, not a `206` with an empty body (`INT-082`).
#[test]
fn a_range_behind_a_shrunk_file_is_refused() {
    let server = Server::start();
    let identifier = server.media_id();
    std::fs::write(server.library().join("King Kong.mkv"), b"12345").expect("the film is written");

    let answer = server.get_with(&format!("/media/{identifier}"), &[("Range", "bytes=15-21")]);

    assert_eq!(answer.status, 416, "{}", answer.text());
    assert_eq!(answer.header("content-range"), Some("bytes */5"));
    assert!(answer.body.is_empty(), "the answer carries no bytes");
}

/// The negative case: a `HEAD` of a file that left the root is refused, like
/// the `GET` of it.
///
/// The two methods must not disagree about whether the file may be read
/// (`INT-098`).
#[cfg(unix)]
#[test]
fn a_head_of_a_file_that_left_the_root_is_refused() {
    use std::os::unix::fs::symlink;

    let server = Server::start();
    let identifier = server.media_id();
    let outside = tempfile::tempdir().expect("another temporary directory");
    let secret = outside.path().join("Secret.mkv");
    std::fs::write(&secret, b"a secret").expect("the secret is written");
    let film = server.library().join("King Kong.mkv");
    std::fs::remove_file(&film).expect("the film is removed");
    symlink(&secret, &film).expect("the link is made");

    let answer = server.head(&format!("/media/{identifier}"));

    assert_eq!(answer.status, 404, "{}", answer.text());
}

/// The positive case: the address of an event subscription is answered.
///
/// The device description advertises the address, so the address must not be
/// a `404`; the answer says that the feature is not there yet.
#[test]
fn an_event_subscription_is_answered() {
    let server = Server::start();

    let answer = server.post(
        "/evt/CD",
        "<subscription/>",
        "\"urn:schemas-upnp-org:service:ContentDirectory:1#Browse\"",
    );

    assert_eq!(answer.status, 501, "{}", answer.text());
    assert!(
        answer.text().contains("not implemented"),
        "the answer says what is missing: {}",
        answer.text()
    );
}

/// The negative case: a service never answers on the address of another one.
///
/// A `Browse` of the ContentDirectory sent to the ConnectionManager is refused.
#[test]
fn a_request_for_another_service_is_refused() {
    let server = Server::start();

    let answer = server.post(
        "/ctl/CM",
        &browse_request("1", "BrowseDirectChildren"),
        "\"urn:schemas-upnp-org:service:ContentDirectory:1#Browse\"",
    );

    assert_eq!(answer.status, 400, "{}", answer.text());
    assert!(
        !answer.text().contains("BrowseResponse"),
        "the service did not answer: {}",
        answer.text()
    );
}

/// The negative case: a client that is not on the access-control list is
/// refused.
///
/// The list of the test names an address that is not the address of the test,
/// so the daemon refuses the client of the test itself (`CFG-009`).
#[test]
fn a_client_that_is_not_on_the_list_is_refused() {
    let server = Server::start_with_clients(&["203.0.113.7"]);

    let answer = server.get("/description.xml");

    assert_eq!(answer.status, 403, "{}", answer.text());
}

/// The positive case: a client that is on the access-control list is answered.
#[test]
fn a_client_that_is_on_the_list_is_answered() {
    let server = Server::start_with_clients(&["127.0.0.1", "::1"]);

    let answer = server.get("/description.xml");

    assert_eq!(answer.status, 200, "{}", answer.text());
}
