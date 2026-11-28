//! Acceptance tests for AGPL Corresponding Source Offer (SEC-070, OPS-016).

mod support;

use support::Server;

#[test]
fn unauthenticated_source_offer_route_returns_ok_and_provenance() {
    let server = Server::start();

    let answer = server.get("/source");

    assert_eq!(answer.status, 200, "{}", answer.text());
    let content_type = answer.header("content-type").expect("content type header");
    assert!(content_type.contains("text/plain"));

    let body = answer.text();
    assert!(body.contains("PhairDLNA Corresponding Source Offer"));
    assert!(body.contains("AGPL-3.0-or-later"));
    assert!(body.contains("https://github.com/mazer666/PhairDLNA"));
    assert!(body.contains("cargo build --release"));
    assert!(body.contains("Dependency Tree"));
    assert!(body.contains("phairdlna-core"));
}

#[test]
fn head_source_offer_route_returns_ok_without_body() {
    let server = Server::start();

    let answer = server.head("/source");

    assert_eq!(answer.status, 200);
    assert!(answer.body.is_empty(), "HEAD request must carry no body");
}
