//! The XML and the SOAP that the server reads from a client.
//!
//! What: the negative tests of the parser — the document type declaration, the
//!       entity that expands, the document that nests too deep, and the action
//!       that is not one of the three the server knows.
//! Why: the SOAP envelope of a control request is the one document a client
//!       writes freely. A parser that reads a document type declaration
//!       resolves entities, and an entity that expands is a denial of service
//!       in three lines of text (`SEC-TH-06`, `SEC-TH-07`, `SEC-TH-08`,
//!       `SEC-010`, `SEC-060`).
//! Example: `cargo test --package phairdlna-core --test xml_hardening`.
//!
//! **What "refused" means here.** A document the server cannot read is
//! answered with a fault, never with a dropped connection: a client that sent
//! a broken envelope deserves to learn that (`INT-094`).

mod support;

use phairdlna_core::upnp::xml_reader::{parse_with_limits, XmlLimits};
use phairdlna_core::upnp::{parse_request_with_limits, Dispatcher, UpnpError};
use support::catalogue::TestCatalogue;
use support::envelope;

/// A document type declaration, which the server must refuse.
const WITH_DOCTYPE: &str = r#"<?xml version="1.0"?>
<!DOCTYPE foo [ <!ENTITY xxe SYSTEM "file:///etc/passwd"> ]>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"><s:Body/></s:Envelope>"#;

/// An entity that expands into other entities.
const WITH_BILLION_LAUGHS: &str = r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
 <!ENTITY lol "lol">
 <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
 <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
]>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"><s:Body/></s:Envelope>"#;

/// An envelope that nests deeper than the limit.
fn deeply_nested(depth: usize) -> String {
    let opening = "<a>".repeat(depth);
    let closing = "</a>".repeat(depth);
    format!("{opening}{closing}")
}

/// SEC-TH-06 — a document type declaration is refused.
#[test]
fn a_document_type_declaration_is_refused() {
    let result = parse_with_limits(WITH_DOCTYPE, &XmlLimits::default());

    assert!(
        result.is_err(),
        "a document with a document type declaration must be refused"
    );
}

/// SEC-TH-06 — the file the declaration names is never read.
///
/// The parser has no file access at all: the declaration is refused before it
/// is read, so the file is never opened.
#[test]
fn the_file_of_the_declaration_is_never_read() {
    let result = parse_with_limits(WITH_DOCTYPE, &XmlLimits::default());

    match result {
        Err(UpnpError::MalformedRequest { reason }) => {
            assert!(
                !reason.contains("/etc/passwd"),
                "the reason must not repeat the path of the file: {reason}"
            );
        }
        other => panic!("the document must be refused; the result was {other:?}"),
    }
}

/// SEC-TH-07 — an entity that expands is refused.
#[test]
fn an_entity_that_expands_is_refused() {
    let result = parse_with_limits(WITH_BILLION_LAUGHS, &XmlLimits::default());

    assert!(
        result.is_err(),
        "a document with an expanding entity must be refused"
    );
}

/// SEC-TH-07 — the expansion never happens.
///
/// The document is small and the limit on the size of a document is far below
/// the size the expansion would produce, so the bound catches it even if the
/// declaration were read.
#[test]
fn the_expansion_never_happens() {
    let limits = XmlLimits {
        max_bytes: 1024,
        ..XmlLimits::default()
    };
    let result = parse_with_limits(WITH_BILLION_LAUGHS, &limits);

    assert!(result.is_err(), "the document must be refused");
}

/// SEC-TH-08 — a document that nests too deep is refused.
#[test]
fn a_deeply_nested_document_is_refused() {
    let limits = XmlLimits {
        max_depth: 16,
        ..XmlLimits::default()
    };
    let result = parse_with_limits(&deeply_nested(64), &limits);

    assert!(
        result.is_err(),
        "a document that nests deeper than the limit must be refused"
    );
}

/// SEC-TH-08 — a document that stays inside the depth is read.
#[test]
fn a_document_inside_the_depth_is_read() {
    let limits = XmlLimits {
        max_depth: 16,
        ..XmlLimits::default()
    };
    let result = parse_with_limits(&deeply_nested(8), &limits);

    assert!(result.is_ok(), "a document inside the limit must be read");
}

/// SEC-TH-08 — a document that is too large is refused.
#[test]
fn a_document_that_is_too_large_is_refused() {
    let limits = XmlLimits {
        max_bytes: 128,
        ..XmlLimits::default()
    };
    let huge = format!("<a>{}</a>", "x".repeat(1024));
    let result = parse_with_limits(&huge, &limits);

    assert!(
        result.is_err(),
        "a document that is larger than the limit must be refused"
    );
}

/// SEC-TH-09 — an action the server does not know is refused.
///
/// The UPnP error code is in the body of the fault, not in the status of the
/// answer: the specification puts the code in the body and the status is the
/// status of the fault (`INT-025`).
#[test]
fn an_unknown_action_is_refused() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "SetValue",
        &[("InstanceID", "0")],
    );

    let response = dispatcher.dispatch(&body, None);

    assert!(
        response.body.contains("<errorCode>401</errorCode>"),
        "an unknown action must be refused with the code 401: {}",
        response.body
    );
}

/// SEC-TH-09 — the answer of an unknown action names no internal detail.
#[test]
fn the_answer_of_an_unknown_action_names_no_detail() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "SetValue",
        &[("InstanceID", "0")],
    );

    let response = dispatcher.dispatch(&body, None);
    let text = response.body;

    assert!(
        !text.contains("192.168.1.20"),
        "the answer must not name the address of the server: {text}"
    );
    assert!(
        !text.contains("panasonic"),
        "the answer must not name a vendor: {text}"
    );
}

/// SEC-TH-09 — an action of another protocol is refused.
///
/// The server answers the three actions it knows and nothing else; an action
/// of the Internet Gateway Device protocol is one of them that it does not
/// know (`SEC-TH-10`).
#[test]
fn an_action_of_another_protocol_is_refused() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:WANIPConnection:1",
        "AddPortMapping",
        &[
            ("NewRemoteHost", "0.0.0.0"),
            ("NewExternalPort", "1234"),
            ("NewProtocol", "TCP"),
            ("NewInternalPort", "80"),
            ("NewInternalClient", "192.168.1.20"),
            ("NewEnabled", "1"),
            ("NewPortMappingDescription", "x"),
            ("NewLeaseDuration", "0"),
        ],
    );

    let response = dispatcher.dispatch(&body, None);

    assert!(
        response.body.contains("<errorCode>401</errorCode>"),
        "an action of another protocol must be refused with the code 401: {}",
        response.body
    );
}

/// SEC-TH-09 — a broken envelope is refused with a fault.
#[test]
fn a_broken_envelope_is_refused() {
    let result = parse_request_with_limits("<s:Envelope", &XmlLimits::default());

    assert!(result.is_err(), "a broken envelope must be refused");
}

/// SEC-TH-09 — a document type declaration inside a SOAP request is refused.
#[test]
fn a_document_type_declaration_in_a_request_is_refused() {
    let result = parse_request_with_limits(WITH_DOCTYPE, &XmlLimits::default());

    assert!(
        result.is_err(),
        "a request with a document type declaration must be refused"
    );
}
