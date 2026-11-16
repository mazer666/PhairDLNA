//! The framing of a request that a client writes.
//!
//! What: the negative tests of the framing — the request that carries two
//!       lengths, the header block that is too large, the range that names more
//!       bytes than the file has, and the concurrency that is bounded.
//! Why: the framing is where a client and a server stop agreeing about where
//!       a request ends. A disagreement is a request smuggling, and a smuggling
//!       is a request the server reads differently from the proxy in front of
//!       it (`SEC-TH-12`, `SEC-TH-13`, `SEC-TH-16`, `SEC-011`, `SEC-012`,
//!       `SEC-065`).
//! Example: `cargo test --package phairdlna-core --test http_framing`.
//!
//! **What is checked here.** The core owns the rules; the socket code owns the
//! sockets. A rule that can be tested without a network is a rule that is
//! actually tested, and the negative tests of the daemon live in
//! `crates/phairdlna-cli/tests/`.

mod support;

use phairdlna_core::http::{evaluate, Method, RANGE};
use phairdlna_core::security::{check, header_block_size, FramingError, HeaderPairs, Limits};
use support::media::{request_with, TestMedia};

/// A source with one film of a thousand bytes.
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

/// SEC-TH-13 — a header block that is too large is refused.
#[test]
fn a_header_block_that_is_too_large_is_refused() {
    let limits = Limits::default();
    let headers = vec![("X-Filler".to_owned(), "a".repeat(limits.header_block_bytes))];

    assert!(
        !limits.accepts_header_block(header_block_size(&headers)),
        "a header block that is too large must be refused"
    );
}

/// SEC-TH-13 — a header block inside the bound is read.
#[test]
fn a_header_block_inside_the_bound_is_read() {
    let limits = Limits::default();
    let headers = vec![("Range".to_owned(), "bytes=0-99".to_owned())];

    assert!(
        limits.accepts_header_block(header_block_size(&headers)),
        "a header block inside the bound must be read"
    );
}

/// SEC-TH-13 — too many headers are refused.
#[test]
fn too_many_headers_are_refused() {
    let limits = Limits::default();
    let headers: Vec<(String, String)> = (0..limits.header_count + 1)
        .map(|index| (format!("X-{index}"), "1".to_owned()))
        .collect();

    assert!(
        !limits.accepts_header_count(headers.len()),
        "too many headers must be refused"
    );
    assert!(
        limits.accepts_header_count(limits.header_count),
        "the bound itself must be accepted"
    );
}

/// SEC-TH-13 — a request line that is too long is refused.
#[test]
fn a_request_line_that_is_too_long_is_refused() {
    let limits = Limits::default();
    let line = format!("/media/{}", "a".repeat(limits.request_line_bytes));

    assert!(
        !limits.accepts_request_line(line.len()),
        "a request line that is too long must be refused"
    );
}

/// SEC-TH-12 — two lengths in one request are refused.
///
/// A request that carries both `Content-Length` and `Transfer-Encoding` says
/// two different things about where it ends. The server reads one of them and
/// a proxy in front of it may read the other; that is the smuggling
/// (`SEC-TH-12`, RFC 9112 § 6.1).
#[test]
fn two_lengths_in_one_request_are_refused() {
    let headers: HeaderPairs = vec![
        ("Content-Length".to_owned(), "5".to_owned()),
        ("Transfer-Encoding".to_owned(), "chunked".to_owned()),
    ];

    assert_eq!(
        check(100, &headers, 0, &Limits::default()),
        Err(FramingError::ConflictingLength),
        "a request with two lengths must be refused"
    );
}

/// SEC-TH-12 — a length that is not a number is refused.
#[test]
fn a_length_that_is_not_a_number_is_refused() {
    let headers: HeaderPairs = vec![("Content-Length".to_owned(), "5, 5".to_owned())];

    assert_eq!(
        check(100, &headers, 0, &Limits::default()),
        Err(FramingError::MalformedLength),
        "a request that names two lengths in one header must be refused"
    );
}

/// SEC-TH-12 — the names of the headers are read without regard to case.
#[test]
fn the_length_headers_are_read_without_regard_to_case() {
    let headers: HeaderPairs = vec![
        ("content-length".to_owned(), "5".to_owned()),
        ("Transfer-Encoding".to_owned(), "chunked".to_owned()),
    ];

    assert_eq!(
        check(100, &headers, 0, &Limits::default()),
        Err(FramingError::ConflictingLength),
        "the names of the headers are not case sensitive"
    );
}

/// SEC-TH-12 — a request that names one length is accepted.
#[test]
fn a_request_that_names_one_length_is_accepted() {
    let headers: HeaderPairs = vec![("Content-Length".to_owned(), "5".to_owned())];

    assert!(
        check(100, &headers, 5, &Limits::default()).is_ok(),
        "a request that names one length must be accepted"
    );
}

/// SEC-TH-16 — a range that starts behind the end of the file is refused.
///
/// A range that *starts* behind the end cannot be satisfied, and the answer
/// is `416`. A range that *ends* behind the end is cut at the end, as
/// RFC 9110 § 14.2 asks.
#[test]
fn a_range_that_starts_behind_the_end_is_refused() {
    let media = media();
    let request = request_with(Method::Get, "/media/a3f1", RANGE, "bytes=5000-6000");

    let response = evaluate(&request, &media);

    assert_eq!(
        response.status.code(),
        416,
        "a range that starts behind the end of the file must be refused"
    );
    assert!(
        response.content_length == 0,
        "the answer of a refused range must carry no body"
    );
}

/// SEC-TH-16 — a range that ends behind the file is cut at the end.
///
/// The server answers with the bytes that exist and promises no more than the
/// file holds, so that a client that asks for a petabyte gets the film and
/// not a promise it cannot keep (`SEC-065`).
#[test]
fn a_range_that_ends_behind_the_file_is_cut_at_the_end() {
    let media = media();
    let request = request_with(Method::Get, "/media/a3f1", RANGE, "bytes=0-999999");

    let response = evaluate(&request, &media);

    assert_eq!(response.status.code(), 206);
    assert_eq!(
        response.content_length, 1000,
        "the answer must hold the bytes of the file and no more"
    );
}

/// SEC-TH-16 — a range of the file itself is answered.
#[test]
fn a_range_of_the_file_is_answered() {
    let media = media();
    let request = request_with(Method::Get, "/media/a3f1", RANGE, "bytes=0-99");

    let response = evaluate(&request, &media);

    assert_eq!(response.status.code(), 206);
    assert_eq!(response.content_length, 100);
}

/// SEC-TH-16 — the answer of a range is never larger than the file.
///
/// The promise of the answer and the bytes of the answer are the same, so a
/// client cannot be told that there are more bytes than there are.
#[test]
fn the_answer_of_a_range_is_never_larger_than_the_file() {
    let media = media();
    let request = request_with(Method::Get, "/media/a3f1", RANGE, "bytes=0-999999");

    let response = evaluate(&request, &media);

    assert!(
        response.content_length <= 1000,
        "the answer must not promise more bytes than the file has"
    );
}

/// SEC-TH-16 — a body that is too large is refused.
#[test]
fn a_body_that_is_too_large_is_refused() {
    let limits = Limits::default();

    assert!(
        !limits.accepts_body(limits.body_bytes + 1),
        "a body that is too large must be refused"
    );
    assert!(
        limits.accepts_body(limits.body_bytes),
        "the bound itself must be accepted"
    );
}

/// SEC-TH-15 — the number of connections is bounded.
#[test]
fn the_number_of_connections_is_bounded() {
    let limits = Limits::default();

    assert!(limits.connections > 0, "the bound must be a real number");
    assert!(
        limits.transfers > 0,
        "the number of transfers must be bounded as well"
    );
}

/// SEC-TH-15 — the number of transfers is bounded.
#[test]
fn the_number_of_transfers_is_bounded() {
    let limits = Limits::default();
    let section = phairdlna_core::config::LimitsSection::default();

    assert_eq!(
        limits.transfers, section.max_concurrent_streams,
        "the bound of the transfers is the one the configuration names"
    );
}

/// SEC-TH-14 — a connection that stays idle is closed.
///
/// The read timeout is the bound on a connection that says nothing; without
/// it, a client that opens a socket and never speaks holds one of the
/// connections of the server for ever (`SEC-TH-14`, `SEC-016`).
#[test]
fn a_connection_that_stays_idle_is_closed() {
    let limits = Limits::default();

    assert!(
        limits.read_timeout.as_secs() > 0,
        "an idle connection must be closed after a bounded time"
    );
    assert!(
        limits.write_timeout.as_secs() > 0,
        "a write must be bounded as well"
    );
}
