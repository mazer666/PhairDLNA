//! The media answers of the server.
//!
//! What: the acceptance criteria of F-06 that concern the answer of a `GET`
//!       and of a `HEAD`: the status, the headers, the length and the bytes.
//! Why: the answer is what the television sees. An answer without a
//!       `Content-Length` leaves the client guessing where the body ends
//!       (INT-088), and an answer whose length does not match its bytes
//!       truncates the film (FUN-028).
//! Example: `cargo test --package phairdlna-core --test http_media`.

mod support;

use phairdlna_core::http::{
    evaluate, ContentKind, HeaderMap, Method, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE,
};
use support::media::{request, request_with, TestMedia};

/// A source with one audio file of a thousand bytes.
fn source() -> TestMedia {
    let mut source = TestMedia::new();
    source.with_file("a3f1", "audio/mpeg", ContentKind::Audio, 1000);
    source
}

/// AC-01 — a `GET` answers the whole file with the type, the length and the
/// range support.
#[test]
fn a_get_answers_the_whole_file() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);

    assert_eq!(response.status.code(), 200);
    assert_eq!(
        response.headers.get(CONTENT_TYPE),
        Some("audio/mpeg"),
        "the type is the one of the res element"
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1000"));
    assert_eq!(response.headers.get(ACCEPT_RANGES), Some("bytes"));
    assert_eq!(response.content_length, 1000);
}

/// AC-01 — the body of a `GET` is the whole file.
#[test]
fn the_body_of_a_get_is_the_file() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let body = media.body_of("a3f1", &response);
    assert_eq!(body.len(), 1000);
    assert_eq!(body, media.bytes_of("a3f1").unwrap_or_default());
}

/// AC-01 — a `HEAD` answers with the same headers and without a body.
#[test]
fn a_head_answers_the_headers_without_a_body() {
    let media = source();
    let get = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let head = evaluate(&request(Method::Head, "/media/a3f1"), &media);

    assert_eq!(head.status.code(), 200);
    assert_eq!(head.headers.get(CONTENT_LENGTH), Some("1000"));
    assert_eq!(head.headers.get(ACCEPT_RANGES), Some("bytes"));
    assert_eq!(
        head.headers.get(CONTENT_TYPE),
        get.headers.get(CONTENT_TYPE)
    );
    assert!(media.body_of("a3f1", &head).is_empty(), "the body is empty");
}

/// AC-05 — the answer carries a `Content-Length` and never a chunked body.
#[test]
fn the_answer_carries_a_length_and_no_chunked_encoding() {
    let media = source();
    for method in [Method::Get, Method::Head] {
        let response = evaluate(&request(method.clone(), "/media/a3f1"), &media);
        assert!(response.headers.get(CONTENT_LENGTH).is_some());
        assert!(
            response.headers.get("Transfer-Encoding").is_none(),
            "a media body is never chunked"
        );
    }
}

/// AC-05 — a connection that asks to be closed is told to close, and a
/// connection that does not is kept alive.
#[test]
fn a_connection_is_kept_alive_unless_it_asks_to_close() {
    let media = source();
    let kept = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    assert!(kept.headers.get("Connection").is_none());

    let closed = evaluate(
        &request_with(Method::Get, "/media/a3f1", "Connection", "close"),
        &media,
    );
    assert_eq!(closed.headers.get("Connection"), Some("close"));
}

/// AC-05 — the type of the answer is the type of the file.
#[test]
fn the_type_of_the_answer_is_the_type_of_the_file() {
    let mut media = TestMedia::new();
    media.with_file("b7c2", "video/mp4", ContentKind::Video, 100);
    media.with_file("c9d4", "image/jpeg", ContentKind::Image, 100);

    let video = evaluate(&request(Method::Get, "/media/b7c2"), &media);
    assert_eq!(video.headers.get(CONTENT_TYPE), Some("video/mp4"));
    let image = evaluate(&request(Method::Get, "/media/c9d4"), &media);
    assert_eq!(image.headers.get(CONTENT_TYPE), Some("image/jpeg"));
}

/// AC-05 — a file whose type is unknown is sent as `application/octet-stream`.
#[test]
fn a_file_without_a_type_is_sent_as_octet_stream() {
    let mut media = TestMedia::new();
    media.with_file("d1e2", "", ContentKind::Other, 100);
    let response = evaluate(&request(Method::Get, "/media/d1e2"), &media);

    assert_eq!(response.status.code(), 200);
    assert_eq!(
        response.headers.get(CONTENT_TYPE),
        Some("application/octet-stream")
    );
}

/// AC-07 — the URL of an object keeps its meaning: two requests return the
/// same bytes.
#[test]
fn the_url_keeps_its_meaning() {
    let media = source();
    let first = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let second = evaluate(&request(Method::Get, "/media/a3f1"), &media);

    assert_eq!(first.status.code(), 200);
    assert_eq!(second.status.code(), 200);
    assert_eq!(
        media.body_of("a3f1", &first),
        media.body_of("a3f1", &second)
    );
}

/// AC-07 — the bytes the client receives are the bytes of the file.
#[test]
fn the_client_receives_the_bytes_of_the_file() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let body = media.body_of("a3f1", &response);
    let expected = media.bytes_of("a3f1").unwrap_or_default();
    assert_eq!(body.len(), expected.len());
    assert!(body == expected);
}

/// The answer of a `GET` names the file the client has to read.
#[test]
fn the_answer_names_the_file_to_read() {
    use phairdlna_core::http::Body;
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    assert_eq!(response.body, Body::Whole);
}

/// The header set of the answer keeps the order of the headers.
#[test]
fn the_headers_keep_their_order() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let names: Vec<&str> = response.headers.iter().map(|(name, _)| name).collect();
    assert_eq!(names[0], CONTENT_TYPE);
    assert_eq!(names[1], CONTENT_LENGTH);
    assert!(names.contains(&ACCEPT_RANGES));
}

/// An answer never carries a header the client did not ask for and the server
/// does not need.
#[test]
fn the_answer_carries_no_unexpected_header() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);
    let headers: HeaderMap = response.headers.clone();
    assert!(headers.get("ETag").is_none());
}
