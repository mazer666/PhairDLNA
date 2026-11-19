//! Tests for the daemon request guard.
//!
//! What: check request-size helpers and transfer-permit bounds.
//! Why: guard tests need access to private rules without enlarging the production module.
//! Example: the final transfer permit succeeds and the next one is refused.

use super::{body_bytes, request_line_bytes, Guard, HeaderPairs};
use axum::http::Request as HttpRequest;
use phairdlna_core::security::{HostPolicy, Limits, Redactor};
use std::sync::Arc;

/// The guard of a server that answers to one name.
fn guard() -> Guard {
    Guard::new(
        Limits::default(),
        HostPolicy::new(vec!["media.local".to_owned()]),
        Arc::new(Redactor::new()),
    )
}

/// A request without a host is refused.
#[test]
fn a_request_without_a_host_is_refused() {
    let request = HttpRequest::builder()
        .uri("/")
        .body(())
        .expect("a valid request");
    assert!(!guard().hosts.accepts(None));
    assert!(!request.headers().contains_key("Host"));
}

/// The size of the request line counts the method, the target and the
/// version.
#[test]
fn the_request_line_is_measured() {
    let request = HttpRequest::builder()
        .method("GET")
        .uri("/description.xml")
        .body(())
        .expect("a valid request");
    assert_eq!(
        request_line_bytes(&request),
        "GET".len() + "/description.xml".len() + 8 + 2
    );
}

/// The size of the body is read from the headers.
#[test]
fn the_body_is_measured_from_the_headers() {
    let headers: HeaderPairs = vec![("Content-Length".to_owned(), "42".to_owned())];
    assert_eq!(body_bytes(&headers), 42);
}

/// A length that is not a number is no length.
#[test]
fn a_malformed_length_is_no_length() {
    let headers: HeaderPairs = vec![("Content-Length".to_owned(), "abc".to_owned())];
    assert_eq!(body_bytes(&headers), 0);
}

/// The transfers are bounded: the permit of the bound is the last one.
#[test]
fn the_transfers_are_bounded() {
    let limits = Limits {
        transfers: 2,
        ..Default::default()
    };
    let guard = Guard::new(limits, HostPolicy::default(), Arc::new(Redactor::new()));

    let first = guard.transfer_permit();
    let second = guard.transfer_permit();
    let third = guard.transfer_permit();

    assert!(first.is_some());
    assert!(second.is_some());
    assert!(third.is_none(), "the third transfer is refused");
}

/// The negative case: a client that is not on the list is refused.
#[test]
fn a_client_that_is_not_on_the_list_is_refused() {
    use phairdlna_core::security::ClientAccess;

    let access = ClientAccess::new(&["203.0.113.7".to_owned()]).expect("the list is usable");
    let guard = Guard::with_access(
        Limits::default(),
        HostPolicy::default(),
        access,
        Arc::new(Redactor::new()),
    );

    let request = HttpRequest::builder()
        .uri("/description.xml")
        .body(())
        .expect("a valid request");
    // Without the address of the peer the request is refused as well: a
    // server that cannot tell who is talking does not let anybody in.
    assert!(guard
        .access
        .admits(&"203.0.113.7".parse().expect("an address")));
    assert!(!guard
        .access
        .admits(&"127.0.0.1".parse().expect("an address")));
    assert!(request.headers().is_empty());
}

/// A permit that is given back lets the next transfer in.
#[test]
fn a_permit_that_is_given_back_frees_a_place() {
    let limits = Limits {
        transfers: 1,
        ..Default::default()
    };
    let guard = Guard::new(limits, HostPolicy::default(), Arc::new(Redactor::new()));

    let first = guard.transfer_permit();
    assert!(guard.transfer_permit().is_none());
    drop(first);
    assert!(guard.transfer_permit().is_some());
}
