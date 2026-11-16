//! The artwork of the daemon.
//!
//! What: the tests of the route that answers a request for a picture — the
//!       cover of a track, the thumbnail of a photo, and the files that carry
//!       no picture at all.
//! Why: a control point asks for the cover of an album before it plays
//!       anything, and a broken file must not take the daemon down with it
//!       (`F-10`, `INT-065`, `INT-067`, `UX-004`).
//! Example: a track with a cover in its tag gets the cover, byte for byte.

mod support;

use support::artwork::{jpeg, m4a_with_cover, mp3_with_cover, png};
use support::soap::{browse_request, CONTENT_DIRECTORY};
use support::Server;

/// The one element of a document, with its attributes and its text.
///
/// The element is found by its name; the answer of a browse is a small
/// document, and the test only needs the one element it is about. The document
/// of a browse is carried inside the answer of the SOAP call, so its marks are
/// escaped; they are read back before the element is looked for.
fn element_of(document: &str, name: &str) -> String {
    let document = document
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    let open = format!("<{name}");
    let start = document
        .find(&open)
        .unwrap_or_else(|| panic!("the document holds no {name}: {document}"));
    let rest = &document[start..];
    let end = rest
        .find("</")
        .unwrap_or_else(|| panic!("the element {name} is not closed: {document}"));
    rest[..end].to_owned()
}

/// The cover of a track is answered, byte for byte.
#[test]
fn the_cover_of_a_track_is_answered() {
    let cover = jpeg(16, 16);
    let server = Server::start_with_files(&[("song.mp3", &mp3_with_cover("image/jpeg", &cover))]);
    let id = server.item_id("song.mp3");

    let answer = server.get(&format!("/art/{id}"));

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert_eq!(answer.header("content-type"), Some("image/jpeg"));
    assert_eq!(answer.body, cover);
}

/// The cover that an M4A file carries is answered as well.
#[test]
fn the_cover_of_a_m4a_is_answered() {
    let cover = jpeg(16, 16);
    let server = Server::start_with_files(&[("song.m4a", &m4a_with_cover(13, &cover))]);
    let id = server.item_id("song.m4a");

    let answer = server.get(&format!("/art/{id}"));

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert_eq!(answer.header("content-type"), Some("image/jpeg"));
    assert_eq!(answer.body, cover);
}

/// The image that lies next to a track is answered with its own type.
#[test]
fn the_sidecar_of_a_track_is_answered() {
    let server = Server::start_with_files(&[("song.mp3", b"no tag at all")]);
    let id = server.item_id("song.mp3");
    let sidecar = png(8, 8);
    std::fs::write(server.library_folder().join("cover.png"), &sidecar)
        .expect("the cover is written");

    let answer = server.get(&format!("/art/{id}"));

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert_eq!(answer.header("content-type"), Some("image/png"));
    assert_eq!(answer.body, sidecar);
}

/// The browse names the address of the cover of the track.
#[test]
fn the_browse_carries_the_address_of_the_cover() {
    let server =
        Server::start_with_files(&[("song.mp3", &mp3_with_cover("image/jpeg", &jpeg(8, 8)))]);
    let id = server.item_id("song.mp3");

    let answer = server.post(
        "/ContentDirectory/control",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        CONTENT_DIRECTORY,
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        answer.text().contains(&format!("/art/{id}")),
        "the browse names the cover: {}",
        answer.text()
    );
}

/// The address of the cover is an absolute address of this server, and the
/// element that carries it names no profile (`INT-067`, `PLT-008`).
#[test]
fn the_address_of_the_cover_is_absolute_and_names_no_profile() {
    let server =
        Server::start_with_files(&[("song.mp3", &mp3_with_cover("image/jpeg", &jpeg(8, 8)))]);
    let id = server.item_id("song.mp3");

    let answer = server.post(
        "/ContentDirectory/control",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        CONTENT_DIRECTORY,
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    let element = element_of(&answer.text(), "upnp:albumArtURI");
    assert!(
        element.contains("http://"),
        "the address is absolute: {element}"
    );
    assert!(element.contains(&format!("/art/{id}")), "{element}");
    assert!(
        !element.contains("dlna:profileID"),
        "the element names no profile: {element}"
    );
}

/// A track without a picture gets a truthful answer and no broken image.
#[test]
fn a_track_without_a_picture_is_refused() {
    let server = Server::start_with_files(&[("song.mp3", b"no tag at all")]);
    let id = server.item_id("song.mp3");

    let answer = server.get(&format!("/art/{id}"));

    assert_eq!(answer.status, 404);
    assert!(answer.body.is_empty());
}

/// An identifier that tries to leave the library reaches no file at all.
#[test]
fn a_hostile_identifier_reaches_no_file() {
    let server = Server::start_with_files(&[("song.mp3", b"no tag at all")]);

    for id in [
        "..%2F..%2Fetc%2Fpasswd",
        "%2Fetc%2Fpasswd",
        "a3f1/../../secret",
        "not-an-identifier",
    ] {
        let answer = server.get(&format!("/art/{id}"));
        assert_eq!(answer.status, 404, "the answer for {id}");
        assert!(answer.body.is_empty(), "the answer for {id} carries bytes");
    }
}

/// The browse of a track without a picture names no address at all.
#[test]
fn the_browse_of_a_track_without_a_picture_names_no_address() {
    let server = Server::start_with_files(&[("song.mp3", b"no tag at all")]);

    let answer = server.post(
        "/ContentDirectory/control",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        CONTENT_DIRECTORY,
    );

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        !answer.text().contains("/art/"),
        "the browse names no cover: {}",
        answer.text()
    );
}

/// A track whose tag is broken is still browsable, and it has no cover.
#[test]
fn a_broken_track_does_not_stop_the_browse() {
    let server = Server::start_with_files(&[("song.mp3", &[0xff, 0xfe, 0x00, 0x01, 0x02])]);
    let id = server.item_id("song.mp3");

    let answer = server.post(
        "/ContentDirectory/control",
        &browse_request(&server.library_id(), "BrowseDirectChildren"),
        CONTENT_DIRECTORY,
    );
    assert_eq!(answer.status, 200, "{}", answer.text());
    assert!(
        !answer.text().contains("/art/"),
        "the browse names no cover: {}",
        answer.text()
    );

    let picture = server.get(&format!("/art/{id}"));
    assert_eq!(picture.status, 404);
    let description = server.get("/description.xml");
    assert_eq!(description.status, 200, "the daemon keeps serving");
}

/// The thumbnail of a photo is answered.
#[test]
fn the_thumbnail_of_a_photo_is_answered() {
    let server = Server::start_with_files(&[("photo.jpg", &jpeg(64, 64))]);
    let id = server.item_id("photo.jpg");

    let answer = server.get(&format!("/art/{id}"));

    assert_eq!(answer.status, 200, "{}", answer.text());
    assert_eq!(answer.header("content-type"), Some("image/jpeg"));
    assert!(!answer.body.is_empty());
}

/// A photo that is broken is refused, and the daemon keeps serving.
#[test]
fn a_broken_photo_is_refused_and_the_daemon_keeps_serving() {
    let server = Server::start_with_files(&[("broken.jpg", b"this is not a picture")]);
    let id = server.item_id("broken.jpg");

    let answer = server.get(&format!("/art/{id}"));
    assert_eq!(answer.status, 404);

    let description = server.get("/description.xml");
    assert_eq!(description.status, 200);
}

/// The second request for a picture is answered from the cache.
#[test]
fn the_second_request_is_answered_from_the_cache() {
    let cover = jpeg(16, 16);
    let server = Server::start_with_files(&[("song.mp3", &mp3_with_cover("image/jpeg", &cover))]);
    let id = server.item_id("song.mp3");

    let first = server.get(&format!("/art/{id}"));
    let second = server.get(&format!("/art/{id}"));

    assert_eq!(first.status, 200);
    assert_eq!(second.status, 200);
    assert_eq!(first.body, second.body);
    assert_eq!(second.body, cover);
}
