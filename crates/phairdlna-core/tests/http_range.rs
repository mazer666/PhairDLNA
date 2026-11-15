//! The byte ranges of the media answers.
//!
//! What: the acceptance criteria of F-06 that concern the `Range` header: the
//!       three forms of a range, a range the file cannot satisfy, a set of
//!       ranges, a range on a method that does not take one, and `If-Range`.
//! Why: the range is what makes a seek work. A range that is answered with
//!       the wrong bytes does not break the film visibly — it breaks it at the
//!       moment the user drags the timeline (INT-081, INT-082, INT-085).
//! Example: `cargo test --package phairdlna-core --test http_range`.

mod support;

use phairdlna_core::http::{
    evaluate, Body, ContentKind, Method, CONTENT_LENGTH, CONTENT_RANGE, ETAG, IF_RANGE, RANGE,
};
use support::media::{request_with, TestMedia};

/// A source with one audio file of a thousand bytes.
fn source() -> TestMedia {
    let mut source = TestMedia::new();
    source.with_file("a3f1", "audio/mpeg", ContentKind::Audio, 1000);
    source
}

/// AC-02 — a closed range is answered with `206` and the exact bytes.
#[test]
fn a_closed_range_is_answered_with_206() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=100-199"),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.headers.get(CONTENT_RANGE),
        Some("bytes 100-199/1000")
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("100"));
    assert_eq!(response.content_length, 100);

    let body = media.body_of("a3f1", &response);
    assert_eq!(body.len(), 100);
    assert_eq!(body, &media.bytes_of("a3f1").unwrap_or_default()[100..200]);
}

/// AC-02 — an open range reaches the end of the file.
#[test]
fn an_open_range_is_answered_with_206() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=900-"),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.headers.get(CONTENT_RANGE),
        Some("bytes 900-999/1000")
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("100"));
    assert_eq!(media.body_of("a3f1", &response).len(), 100);
}

/// AC-02 — a suffix range reaches the end of the file.
#[test]
fn a_suffix_range_is_answered_with_206() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=-100"),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.headers.get(CONTENT_RANGE),
        Some("bytes 900-999/1000")
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("100"));
}

/// AC-03 — a range that starts behind the file is answered with `416` and the
/// size of the file.
#[test]
fn a_range_behind_the_file_is_answered_with_416() {
    let media = source();
    for header in ["bytes=1000-", "bytes=5000-6000"] {
        let response = evaluate(
            &request_with(Method::Get, "/media/a3f1", RANGE, header),
            &media,
        );
        assert_eq!(response.status.code(), 416, "{header} is not satisfiable");
        assert_eq!(
            response.headers.get(CONTENT_RANGE),
            Some("bytes */1000"),
            "{header} names the size of the file"
        );
        assert_eq!(response.headers.get(CONTENT_LENGTH), Some("0"));
    }
}

/// AC-03 — a set of ranges is answered as one range and never as a multipart
/// body.
#[test]
fn a_set_of_ranges_is_answered_as_one_range() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=0-1,5-6"),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(response.headers.get(CONTENT_RANGE), Some("bytes 0-1/1000"));
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("2"));
    assert!(
        response.headers.get("Content-Type").is_some(),
        "the answer is not a multipart body"
    );
    assert_eq!(media.body_of("a3f1", &response).len(), 2);
}

/// AC-04 — a range on a method that does not take one is ignored.
#[test]
fn a_range_on_a_head_is_ignored() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Head, "/media/a3f1", RANGE, "bytes=100-199"),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1000"));
    assert!(response.headers.get(CONTENT_RANGE).is_none());
}

/// AC-04 — a range unit the server does not know is ignored.
#[test]
fn an_unknown_range_unit_is_ignored() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "items=0-10"),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1000"));
}

/// AC-04 — a range whose value cannot be read is answered with `416` and
/// never with a broken body.
#[test]
fn a_broken_range_is_answered_with_416() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=abc-def"),
        &media,
    );

    assert_eq!(response.status.code(), 416);
    assert_eq!(response.headers.get(CONTENT_RANGE), Some("bytes */1000"));
    assert_eq!(response.content_length, 0);
}

/// AC-04 — an `If-Range` whose condition does not hold makes the server ignore
/// the range.
#[test]
fn an_if_range_that_does_not_hold_ignores_the_range() {
    let media = source();
    let response = evaluate(
        &request_with(
            Method::Get,
            "/media/a3f1",
            IF_RANGE,
            "\"a-different-validator\"",
        ),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1000"));
    assert!(response.headers.get(CONTENT_RANGE).is_none());
}

/// AC-04 — an `If-Range` whose condition holds lets the range through.
#[test]
fn an_if_range_that_holds_honours_the_range() {
    let mut media = TestMedia::new();
    media.with_features(
        "a3f1",
        "audio/mpeg",
        ContentKind::Audio,
        1000,
        "DLNA.ORG_PN=MP3",
        Some("\"v1\""),
    );
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", IF_RANGE, "\"v1\""),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1000"));
}

/// AC-04 — the `ETag` of the object and the condition of an `If-Range` are
/// compared when both are there.
#[test]
fn an_if_range_is_compared_with_the_validator_of_the_object() {
    let mut media = TestMedia::new();
    media.with_features(
        "a3f1",
        "audio/mpeg",
        ContentKind::Audio,
        1000,
        "",
        Some("\"v1\""),
    );
    let mut headers = phairdlna_core::http::HeaderMap::new();
    headers.set(RANGE, "bytes=0-9");
    headers.set(IF_RANGE, "\"v1\"");
    headers.set(ETAG, "\"v1\"");
    let request = phairdlna_core::http::MediaRequest::new(Method::Get, "/media/a3f1", headers);
    let response = evaluate(&request, &media);

    assert_eq!(response.status.code(), 206);
    assert_eq!(response.headers.get(CONTENT_RANGE), Some("bytes 0-9/1000"));
}

/// A range that ends behind the file is cut at the end.
#[test]
fn a_range_behind_the_end_is_cut() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", RANGE, "bytes=900-5000"),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.headers.get(CONTENT_RANGE),
        Some("bytes 900-999/1000")
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("100"));
}

/// The offset of a file beyond four gigabytes is not truncated (FUN-028).
#[test]
fn an_offset_beyond_four_gigabytes_is_kept() {
    let start = 5_000_000_000u64;
    let mut media = TestMedia::new();
    media.with_sparse_file("big", "video/mp4", ContentKind::Video, start + 10);
    let response = evaluate(
        &request_with(
            Method::Get,
            "/media/big",
            RANGE,
            &format!("bytes={start}-{start}"),
        ),
        &media,
    );

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.headers.get(CONTENT_RANGE),
        Some(format!("bytes {start}-{start}/{}", start + 10).as_str())
    );
    assert_eq!(response.headers.get(CONTENT_LENGTH), Some("1"));
    assert_eq!(
        response.body,
        Body::Slice {
            offset: start,
            length: 1
        }
    );
}
