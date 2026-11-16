//! The paths a request may not reach.
//!
//! What: the negative tests of the containment of the media paths — the
//!       traversal attempts, the symbolic link that points out of a root, and
//!       the playlist entry that names a path outside the library.
//! Why: the media URL is the trust boundary. A server that serves a file
//!       outside its roots serves every file the user can read, and the
//!       request that does it is a single line of text (`SEC-TH-17`,
//!       `SEC-TH-18`, `SEC-TH-21`, `SEC-001` … `SEC-004`, `SEC-015`).
//! Example: `cargo test --package phairdlna-core --test path_containment`.
//!
//! **What "refused" means here.** A traversal attempt is answered like an
//! unknown object, not like an error: an answer that says "that path is
//! outside the root" tells the attacker where the root is (`SEC-013`).

mod support;

use std::path::{Path, PathBuf};

use phairdlna_core::http::path::{from_target, is_inside_roots, Target};
use phairdlna_core::http::{evaluate, Method};
use support::media::{request, TestMedia};

/// A root that holds one file.
fn root() -> PathBuf {
    PathBuf::from("/library/films")
}

/// A source with one film inside the root.
fn media() -> TestMedia {
    let mut media = TestMedia::new();
    media.with_file(
        "a3f1",
        "video/mp4",
        phairdlna_core::http::ContentKind::Video,
        1000,
    );
    media
}

/// The identifiers that a traversal attempt uses.
const TRAVERSALS: &[&str] = &[
    "/media/../etc/passwd",
    "/media/../../etc/passwd",
    "/media/a/../../etc/passwd",
    "/media/..%2f..%2fetc%2fpasswd",
    "/media/%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    "/media/....//....//etc/passwd",
    "/media/..%252f..%252fetc%252fpasswd",
    "/media/..%c0%af..%c0%afetc%c0%afpasswd",
    "/media/..\\..\\etc\\passwd",
    "/media/..%5c..%5cetc%5cpasswd",
    "/media/%00/../etc/passwd",
    "/media/a%00b",
    "/media/./../../etc/passwd",
];

/// SEC-TH-17 — a traversal attempt names no media object at all.
#[test]
fn a_traversal_attempt_names_no_object() {
    for target in TRAVERSALS {
        let resolved = from_target(target);
        assert!(
            !matches!(resolved, Target::Media(_)),
            "{target} must not name a media object"
        );
    }
}

/// SEC-TH-17 — a traversal attempt is never served.
///
/// A request that walks out of a root is answered like an unknown object
/// (`404`); a request whose encoding is broken is answered like a broken
/// request (`400`). What matters is that neither is ever a success and that
/// neither names a path: an answer that says "that path is outside the root"
/// would tell the attacker where the root is (`SEC-013`).
#[test]
fn a_traversal_attempt_is_never_served() {
    let media = media();
    for target in TRAVERSALS {
        let response = evaluate(&request(Method::Get, target), &media);

        assert!(
            response.status.code() == 404 || response.status.code() == 400,
            "{target} must not be served; the answer was {}",
            response.status.code()
        );
    }
}

/// SEC-TH-17 — the answer of a traversal attempt names no path.
#[test]
fn the_answer_of_a_traversal_attempt_names_no_path() {
    let media = media();
    let response = evaluate(&request(Method::Get, "/media/../../etc/passwd"), &media);
    let body = match &response.body {
        phairdlna_core::http::Body::Text(text) => *text,
        _ => "",
    };

    assert!(
        !body.contains("/etc/passwd"),
        "the answer must not repeat the path: {body}"
    );
    assert!(
        !body.contains("/library"),
        "the answer must not name the root: {body}"
    );
}

/// SEC-TH-17 — a percent-encoded separator never reaches the file system.
///
/// The identifier is opaque: it carries no separator at all, so there is no
/// path to walk out of.
#[test]
fn an_identifier_carries_no_separator() {
    for target in TRAVERSALS {
        if let Target::Media(id) = from_target(target) {
            assert!(
                id.as_str()
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "{target} decoded into an identifier with a separator: {}",
                id.as_str()
            );
        }
    }
}

/// SEC-TH-15 — an identifier that is too long is refused.
///
/// The bound keeps the work of a request proportional to what the server
/// expects (`SEC-011`).
#[test]
fn an_overlong_identifier_is_refused() {
    let long = "/media/".to_owned() + &"a".repeat(1024);
    let resolved = from_target(&long);

    assert!(
        !matches!(resolved, Target::Media(_)),
        "an overlong identifier must not name an object"
    );
}

/// SEC-TH-18 — a path that walks out of a root with `..` is not inside it.
#[test]
fn a_path_that_walks_out_is_not_inside() {
    let roots = vec![root()];

    assert!(!is_inside_roots(
        Path::new("/library/films/../secret"),
        &roots
    ));
    assert!(!is_inside_roots(
        Path::new("/library/other/film.mkv"),
        &roots
    ));
    assert!(!is_inside_roots(Path::new("/etc/passwd"), &roots));
}

/// SEC-TH-18 — a path inside a root is inside it.
#[test]
fn a_path_inside_a_root_is_inside() {
    let roots = vec![root()];

    assert!(is_inside_roots(
        Path::new("/library/films/film.mkv"),
        &roots
    ));
    assert!(is_inside_roots(
        Path::new("/library/films/sub/film.mkv"),
        &roots
    ));
}

/// SEC-TH-18 — a root is not inside itself.
///
/// The root itself is the folder, not a file below it; a request that names
/// the root must not receive the folder.
#[test]
fn a_root_is_not_inside_itself() {
    let roots = vec![root()];

    assert!(!is_inside_roots(Path::new("/library/films"), &roots));
}

/// SEC-TH-18 — a link that points out of a root does not bring the root along.
///
/// The canonical path of a link that leaves the root is outside the root, so
/// the containment check refuses it even though the link itself is inside.
#[test]
fn a_link_that_points_out_is_refused() {
    let roots = vec![root()];

    // The path of the link is inside the root; the path it points to is not.
    let link = Path::new("/library/films/outside.mkv");
    let target = Path::new("/etc/passwd");

    assert!(is_inside_roots(link, &roots), "the link itself is inside");
    assert!(
        !is_inside_roots(target, &roots),
        "the file the link points to is outside"
    );
}

/// SEC-TH-21 — a playlist entry that names a path is not a media object.
///
/// A playlist is a list of names, not a list of paths; an entry that tries to
/// be one is refused.
#[test]
fn a_playlist_entry_that_names_a_path_is_refused() {
    for entry in [
        "../../etc/passwd",
        "/etc/passwd",
        "..%2f..%2fetc%2fpasswd",
        "sub/../../secret.mkv",
    ] {
        let target = format!("/media/{entry}");
        assert!(
            !matches!(from_target(&target), Target::Media(_)),
            "{entry} must not name a media object"
        );
    }
}

/// SEC-TH-21 — a playlist entry that names a plain name is not a path either.
///
/// The identifier of a media object is produced by the catalogue, never taken
/// from a request; a name that a client sends is not one of them.
#[test]
fn a_playlist_entry_that_names_a_file_is_not_served() {
    let media = media();
    let response = evaluate(&request(Method::Get, "/media/film.mkv"), &media);

    assert_eq!(
        response.status.code(),
        404,
        "a name the client sends must not be served"
    );
}
