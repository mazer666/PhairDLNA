//! The DLNA headers of a media answer.
//!
//! What: the acceptance criteria of F-06 that concern the DLNA layer: the
//!       three headers a media answer carries, the transfer mode the server
//!       offers for a content, the request for the content features and the
//!       refusal of a mode the server does not offer.
//! Why: a DLNA renderer expects these headers on a media answer, and the
//!       reference implementation answers `406` when the client asks for a
//!       transfer mode the content does not take (INT-096, INT-097,
//!       INT-099, standards §5.2, §8.8 N4–N6).
//! Example: `cargo test --package phairdlna-core --test http_dlna_headers`.

mod support;

use phairdlna_core::http::{
    evaluate, ContentKind, Method, CONTENT_FEATURES, GET_CONTENT_FEATURES,
    GET_CONTENT_FEATURES_VALUE, REAL_TIME_INFO, REAL_TIME_INFO_VALUE, TRANSFER_MODE,
};
use support::media::{request, request_with, TestMedia};

/// A source with one audio file and one image.
fn source() -> TestMedia {
    let mut source = TestMedia::new();
    source.with_features(
        "a3f1",
        "audio/mpeg",
        ContentKind::Audio,
        1000,
        "DLNA.ORG_PN=MP3;DLNA.ORG_OP=01;DLNA.ORG_CI=0",
        None,
    );
    source.with_features(
        "c9d4",
        "image/jpeg",
        ContentKind::Image,
        1000,
        "DLNA.ORG_PN=JPEG_TN",
        None,
    );
    source
}

/// AC-09 — a media answer carries the three DLNA headers.
#[test]
fn a_media_answer_carries_the_dlna_headers() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/a3f1"), &media);

    assert_eq!(response.headers.get(TRANSFER_MODE), Some("Streaming"));
    assert_eq!(
        response.headers.get(CONTENT_FEATURES),
        Some("DLNA.ORG_PN=MP3;DLNA.ORG_OP=01;DLNA.ORG_CI=0")
    );
    assert_eq!(
        response.headers.get(REAL_TIME_INFO),
        Some(REAL_TIME_INFO_VALUE)
    );
}

/// AC-09 — an image is delivered interactively.
#[test]
fn an_image_is_delivered_interactively() {
    let media = source();
    let response = evaluate(&request(Method::Get, "/media/c9d4"), &media);

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(TRANSFER_MODE), Some("Interactive"));
    assert_eq!(
        response.headers.get(CONTENT_FEATURES),
        Some("DLNA.ORG_PN=JPEG_TN")
    );
}

/// AC-09 — a transfer mode the server does not offer for the content is
/// answered with `406`.
#[test]
fn a_transfer_mode_the_server_does_not_offer_is_answered_with_406() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/c9d4", TRANSFER_MODE, "Streaming"),
        &media,
    );

    assert_eq!(response.status.code(), 406);
    assert!(
        response.headers.get(CONTENT_FEATURES).is_none(),
        "the answer describes no content"
    );
}

/// AC-09 — a transfer mode the server offers for the content is answered with
/// the media.
#[test]
fn a_transfer_mode_the_server_offers_is_answered_with_the_media() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", TRANSFER_MODE, "Streaming"),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.get(TRANSFER_MODE), Some("Streaming"));
}

/// AC-09 — a request for the content features is answered with them.
#[test]
fn a_request_for_the_content_features_is_answered_with_them() {
    let media = source();
    let response = evaluate(
        &request_with(
            Method::Get,
            "/media/a3f1",
            GET_CONTENT_FEATURES,
            GET_CONTENT_FEATURES_VALUE,
        ),
        &media,
    );

    assert_eq!(response.status.code(), 200);
    assert_eq!(
        response.headers.get(CONTENT_FEATURES),
        Some("DLNA.ORG_PN=MP3;DLNA.ORG_OP=01;DLNA.ORG_CI=0")
    );
}

/// AC-09 — a request for the content features with another value is answered
/// with `400`.
#[test]
fn another_value_for_the_content_features_is_answered_with_400() {
    let media = source();
    for value in ["2", "0", "yes", ""] {
        let response = evaluate(
            &request_with(Method::Get, "/media/a3f1", GET_CONTENT_FEATURES, value),
            &media,
        );
        assert_eq!(response.status.code(), 400, "the value {value} is refused");
    }
}

/// A transfer mode the client names wrongly is refused with `406`, because the
/// server does not know what the client wants.
#[test]
fn an_unknown_transfer_mode_is_answered_with_406() {
    let media = source();
    let response = evaluate(
        &request_with(Method::Get, "/media/a3f1", TRANSFER_MODE, "Turbo"),
        &media,
    );

    assert_eq!(response.status.code(), 406);
}
