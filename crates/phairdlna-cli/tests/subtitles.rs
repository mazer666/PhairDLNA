//! The subtitle of a film, seen by a client that speaks HTTP.
//!
//! What: the test that lets a client find the subtitle of a film in the
//!       document of the library and download it from the media endpoint.
//! Why: the pairing of a subtitle with its film happens while the root is
//!       read; the client finds the subtitle as a second resource of the film
//!       and fetches it by the address that the resource names. A subtitle
//!       that cannot be fetched is worse than no subtitle at all, because the
//!       client shows a film it believes to have a caption (`FUN-016`,
//!       `FUN-026`, `INT-105`).
//! Example: the document names `text/vtt`, and the answer of the media
//!       endpoint carries the same type.

mod support;

use support::soap::{browse_request, CONTENT_DIRECTORY};
use support::Server;

/// The bytes of the film of the test.
const FILM: &[u8] = b"a film of eleven bytes";

/// The bytes of the subtitle of the film.
const SUBTITLE: &[u8] = b"WEBVTT\n\n00:00.000 --> 00:02.000\nA line.\n";

/// The address of the subtitle, as the document of the library names it.
fn subtitle_uri(document: &str) -> String {
    let marker = "http-get:*:text/vtt:*";
    let start = document
        .find(marker)
        .expect("the document names the subtitle");
    // The document travels inside a SOAP envelope, so the address of the
    // resource is the text of the element, and the tags are escaped.
    let after = &document[start..];
    let open = after.find("&gt;").expect("the resource is closed") + 4;
    let rest = &after[open..];
    let end = rest.find("&lt;").expect("the resource ends");
    rest[..end].to_owned()
}

/// The path of an address, so that the test can ask the daemon for it.
fn path_of(uri: &str) -> String {
    let without_scheme = uri.split_once("://").map_or(uri, |(_, rest)| rest);
    let path = without_scheme
        .find('/')
        .map(|slash| &without_scheme[slash..])
        .unwrap_or("/");
    path.to_owned()
}

/// A client finds the subtitle of a film and downloads it.
#[test]
fn a_client_finds_the_subtitle_and_downloads_it() {
    let mut server =
        Server::start_with_files(&[("King Kong.mkv", FILM), ("King Kong.vtt", SUBTITLE)]);

    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );
    assert_eq!(answer.status, 200);
    let document = answer.text();
    assert!(
        document.contains("http-get:*:text/vtt:*"),
        "the document names the subtitle: {document}"
    );

    let path = path_of(&subtitle_uri(&document));
    let answer = server.get(&path);

    assert_eq!(answer.status, 200);
    assert_eq!(
        answer.header("Content-Type"),
        Some("text/vtt; charset=utf-8"),
        "the answer carries the type of the file"
    );
    assert_eq!(answer.body, SUBTITLE);
    server.stop();
}

/// The film itself is still served the way it was before.
#[test]
fn the_film_is_still_served() {
    let mut server =
        Server::start_with_files(&[("King Kong.mkv", FILM), ("King Kong.vtt", SUBTITLE)]);

    let answer = server.get(&format!("/media/{}", server.media_id()));

    assert_eq!(answer.status, 200);
    assert_eq!(answer.header("Content-Type"), Some("video/x-matroska"));
    assert_eq!(answer.body, FILM);
    server.stop();
}

/// A film without a subtitle is served the way it was before.
#[test]
fn a_film_without_a_subtitle_is_unchanged() {
    let mut server = Server::start_with_film(FILM);

    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );
    let document = answer.text();
    assert!(
        !document.contains("text/vtt"),
        "the document names no subtitle: {document}"
    );
    server.stop();
}

/// A subtitle that is not UTF-8 is delivered in the character set it is
/// written in, and the answer says so.
#[test]
fn a_latin_1_subtitle_states_its_character_set() {
    let mut server = Server::start_with_files(&[
        ("King Kong.mkv", FILM),
        ("King Kong.vtt", b"WEBVTT\n\ncaf\xe9\n"),
    ]);

    let answer = server.post(
        "/ctl/CD",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        &format!("\"{CONTENT_DIRECTORY}#Browse\""),
    );
    let document = answer.text();
    let path = path_of(&subtitle_uri(&document));
    let answer = server.get(&path);

    assert_eq!(answer.status, 200);
    assert_eq!(
        answer.header("Content-Type"),
        Some("text/vtt; charset=iso-8859-1"),
        "the answer names the character set of the file"
    );
    server.stop();
}
