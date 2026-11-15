//! The answers the server gives to a request it cannot carry out.
//!
//! What: the acceptance criteria of F-06 that concern the negative requests:
//!       an unknown object, an unsupported method and a malformed target.
//! Why: a control point that gets a fault learns what went wrong; a control
//!       point that gets a dropped connection does not. The answer never
//!       carries a path, because the path tells a stranger what the disk of
//!       the user looks like (SEC-013).
//! Example: `cargo test --package phairdlna-core --test http_negative`.

mod support;

use phairdlna_core::http::{evaluate, ContentKind, Method, ALLOW, CONTENT_LENGTH};
use support::media::{request, request_with, TestMedia};

/// A source with one audio file of a thousand bytes.
fn source() -> TestMedia {
    let mut source = TestMedia::new();
    source.with_file("a3f1", "audio/mpeg", ContentKind::Audio, 1000);
    source
}

/// AC-08 — an object the server does not have is answered with `404`.
#[test]
fn an_unknown_object_is_answered_with_404() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/does-not-exist"), &media);

    assert_eq!(response.status.code(), 404);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("9"));
}

/// AC-08 — a method the media layer does not serve is answered with `405` and
/// the methods that are served.
#[test]
fn an_unsupported_method_is_answered_with_405() {
    let media = source();
    let response = evaluate(
        &request(Method::Other("POST".to_owned()), "/media/a3f1"),
        &media,
    );

    assert_eq!(response.status.code(), 405);
    assert_eq!(response.headers.get(ALLOW), Some("GET, HEAD"));
}

/// AC-08 — a target that cannot be read is answered with `400`.
#[test]
fn a_malformed_target_is_answered_with_400() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a%zz"), &media);
    assert_eq!(response.status.code(), 400);
}

/// AC-08 — no answer carries a path.
#[test]
fn no_answer_carries_a_path() {
    let media = source();
    let answers = vec![
        evaluate(&request(Method::Get, "/media/does-not-exist"), &media),
        evaluate(
            &request(Method::Other("DELETE".to_owned()), "/media/a3f1"),
            &media,
        ),
        evaluate(&request(Method::Get, "/media/a%zz"), &media),
        evaluate(&request(Method::Get, "/etc/passwd"), &media),
        evaluate(&request(Method::Get, "/media/../etc/passwd"), &media),
    ];

    for response in &answers {
        let body = match response.body {
            phairdlna_core::http::Body::Text(text) => text.to_owned(),
            other => format!("{other:?}"),
        };
        for secret in ["/etc/passwd", "a3f1", "..", "/media/"] {
            assert!(!body.contains(secret), "the answer names {secret}: {body}");
        }
    }
}

/// A target that is not a media URL is answered with `404`.
#[test]
fn another_url_is_answered_with_404() {
    let media = source();
    for target in ["/", "/description.xml", "/art/a3f1", "/media/"] {
        let response = evaluate(&request(Method::Get, target), &media);
        assert_eq!(response.status.code(), 404, "{target} is not a media URL");
    }
}

/// The answer of a `HEAD` carries the headers of the `GET` and no body.
#[test]
fn the_answer_of_a_head_has_no_body() {
    let media = source();
    let response = evaluate(&request(Method::Head, "/media/does-not-exist"), &media);
    assert_eq!(response.status.code(), 404);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("9"));
    assert_eq!(response.body, phairdlna_core::http::Body::None);
}

/// A request that asks for the DLNA features with a broken value is answered
/// with `400`.
#[test]
fn a_broken_dlna_request_is_answered_with_400() {
    let media = source();
    let response = evaluate(
        &request_with(
            Method::Get,
            "/media/a3f1",
            "getcontentFeatures.dlna.org",
            "2",
        ),
        &media,
    );
    assert_eq!(response.status.code(), 400);
}

/// A request that is broken is refused before the catalogue is touched, so a
/// broken header is answered with `400` even for an object that does not
/// exist.
#[test]
fn a_broken_request_is_refused_before_the_catalogue() {
    let media = source();
    let response = evaluate(
        &request_with(
            Method::Get,
            "/media/does-not-exist",
            "getcontentFeatures.dlna.org",
            "2",
        ),
        &media,
    );
    assert_eq!(response.status.code(), 400);
}
