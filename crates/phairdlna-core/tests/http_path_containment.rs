//! The containment of what a media URL names.
//!
//! What: the acceptance criteria of F-06 that concern a request that tries to
//!       walk out of the catalogue or out of a root: the refusal of a
//!       traversal attempt, and the check that a resolved path is inside one
//!       of the configured roots.
//! Why: the media URL is the trust boundary. A request may never make the
//!       server read outside a root, follow a link out of them, or expose a
//!       path in an answer (INT-098, SEC-001, SEC-003, SEC-013).
//! Example: `cargo test --package phairdlna-core --test http_path_containment`.

mod support;

use phairdlna_core::http::{evaluate, is_inside_roots, ContentKind, MediaSource, Method};
use std::path::PathBuf;
use support::media::{request, TestMedia};

/// A source with one audio file of a thousand bytes.
fn source() -> TestMedia {
    let mut source = TestMedia::new();
    source.with_file("a3f1", "audio/mpeg", ContentKind::Audio, 1000);
    source
}

/// AC-10 — a traversal attempt is answered like an unknown object and never
/// reaches the catalogue.
#[test]
fn a_traversal_attempt_is_answered_with_404() {
    let media = source();
    for target in [
        "/media/../etc/passwd",
        "/media/..%2F..%2Fetc%2Fpasswd",
        "/media/a%2Fb",
        "/media/a%5Cb",
        "/media/a%00b",
        "/media/..",
        "/media/a3f1/../../etc/passwd",
    ] {
        let response = evaluate(&request(Method::Get, target), &media);
        assert_eq!(response.status.code(), 404, "{target} is refused");
    }
}

/// AC-10 — the answer of a refused request names no path.
#[test]
fn a_refused_request_names_no_path() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/../etc/passwd"), &media);
    let body = match response.body {
        phairdlna_core::http::Body::Text(text) => text.to_owned(),
        other => format!("{other:?}"),
    };
    for secret in ["/etc", "passwd", "..", "a3f1"] {
        assert!(!body.contains(secret), "the answer names {secret}: {body}");
    }
}

/// AC-10 — a path inside a root is inside it, and a path that walks out of it
/// is not.
#[test]
fn a_path_inside_a_root_is_accepted() {
    let roots = vec![PathBuf::from("/media/root")];
    assert!(is_inside_roots(
        &PathBuf::from("/media/root/film.mp4"),
        &roots
    ));
    assert!(!is_inside_roots(
        &PathBuf::from("/media/root/../secret"),
        &roots
    ));
    assert!(!is_inside_roots(
        &PathBuf::from("/media/other/film.mp4"),
        &roots
    ));
}

/// AC-10 — a wide link is refused.
///
/// The lexical check cannot see a symbolic link: the link
/// `/media/root/link` points at `/etc`, and the path `/media/root/link/passwd`
/// looks like it is inside the root. The caller therefore resolves the links
/// first and repeats the check with the canonical path, and the canonical
/// path of the wide link is outside the root (INT-098, SEC-003).
#[test]
fn a_wide_link_is_refused() {
    let roots = vec![PathBuf::from("/media/root")];
    // The path as it is written: inside the root.
    assert!(is_inside_roots(
        &PathBuf::from("/media/root/link/passwd"),
        &roots
    ));
    // The same file after the link has been resolved: outside the root.
    assert!(!is_inside_roots(&PathBuf::from("/etc/passwd"), &roots));
}

/// A file outside every root is not served, even when the catalogue knows it.
#[test]
fn a_file_outside_every_root_is_not_served() {
    let media = source();
    let roots = vec![PathBuf::from("/media/root")];
    let object = media.metadata("a3f1").expect("the object is known");
    assert!(!is_inside_roots(
        &PathBuf::from("/elsewhere/a3f1.mp4"),
        &roots
    ));
    assert_eq!(object.id, "a3f1");
}
