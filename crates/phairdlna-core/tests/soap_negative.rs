//! The SOAP faults of the control path.
//!
//! What: the answers the server gives to a request it cannot carry out —
//!       an unknown action, an unknown service, a broken envelope, a bad
//!       argument.
//! Why: a control point that gets a fault learns what went wrong; a control
//!       point that gets a dropped connection does not. Every failure of the
//!       control path is therefore an answer with a fault, and never a panic
//!       (`INT-025`, `UX-005`).
//! Example: `cargo test --package phairdlna-core --test soap_negative`.

mod support;

use phairdlna_core::upnp::Dispatcher;
use support::{catalogue::TestCatalogue, envelope, CONNECTION_MANAGER, CONTENT_DIRECTORY};

/// The dispatcher of the test catalogue.
fn dispatcher() -> Dispatcher<'static> {
    Dispatcher::new(&TestCatalogue)
}

/// The answer of a request, with its status.
fn answer(request: &str) -> (u16, String) {
    let response = dispatcher().dispatch(request, None);
    (response.status, response.body)
}

/// An action the service does not have is the fault `401`.
#[test]
fn an_unknown_action_is_the_fault_401() {
    let (status, body) = answer(&envelope(CONTENT_DIRECTORY, "NoSuchAction", &[]));
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>401</errorCode>"));
    assert!(body.contains("<errorDescription>Invalid Action</errorDescription>"));
    assert!(body.contains("<faultcode>s:Client</faultcode>"));
    assert!(body.contains("<faultstring>UPnPError</faultstring>"));
}

/// A service the server does not have is the fault `401`.
#[test]
fn an_unknown_service_is_the_fault_401() {
    let (status, body) = answer(&envelope(
        "urn:schemas-upnp-org:service:RenderingControl:1",
        "GetProtocolInfo",
        &[],
    ));
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>401</errorCode>"));
}

/// A missing argument is the fault `402`.
#[test]
fn a_missing_argument_is_the_fault_402() {
    let (_, body) = answer(&envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[("ObjectID", "0"), ("BrowseFlag", "BrowseMetadata")],
    ));
    assert!(body.contains("<errorCode>402</errorCode>"));
    assert!(body.contains("<errorDescription>Invalid Args</errorDescription>"));
}

/// An argument that is not a number is the fault `402`.
#[test]
fn an_argument_that_is_not_a_number_is_the_fault_402() {
    let (_, body) = answer(&envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", "0"),
            ("BrowseFlag", "BrowseMetadata"),
            ("Filter", "*"),
            ("StartingIndex", "not-a-number"),
            ("RequestedCount", "0"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A browse flag the server does not know is the fault `402`.
#[test]
fn an_unknown_browse_flag_is_the_fault_402() {
    let (_, body) = answer(&envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", "0"),
            ("BrowseFlag", "BrowseSideways"),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "0"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request that is not an envelope is the fault `402`, and not a crash.
#[test]
fn a_broken_envelope_is_the_fault_402() {
    let (status, body) = answer("<html><body>not a soap request</body></html>");
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request that is not XML at all is the fault `402`.
#[test]
fn a_request_that_is_not_xml_is_the_fault_402() {
    let (status, body) = answer("this is not xml");
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request with a document type declaration is refused: the reader never
/// processes a DTD (`SEC-010`).
#[test]
fn a_request_with_a_document_type_declaration_is_refused() {
    let request = "<!DOCTYPE Envelope [<!ENTITY boom \"x\">]><s:Envelope \
                   xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body/></s:Envelope>";
    let (status, body) = answer(request);
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request that names an entity XML does not define is refused.
#[test]
fn a_request_with_an_unknown_entity_is_refused() {
    let request = "<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body>\
                   <u:Browse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">\
                   <ObjectID>&boom;</ObjectID></u:Browse></s:Body></s:Envelope>";
    let (status, body) = answer(request);
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request that is larger than the server accepts is refused.
#[test]
fn a_request_that_is_too_large_is_refused() {
    let huge = "x".repeat(128 * 1024);
    let request = envelope(CONTENT_DIRECTORY, "Browse", &[("ObjectID", &huge)]);
    let (status, body) = answer(&request);
    assert_eq!(status, 500);
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// The fault names the action and the code, and nothing else of the request.
#[test]
fn a_fault_reveals_nothing_of_the_request() {
    let secret = "secret-path-and-name";
    let (_, body) = answer(&envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[("ObjectID", secret)],
    ));
    assert!(
        !body.contains(secret),
        "the fault must not echo the request"
    );
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// A request whose `SOAPAction` header names another action is refused.
#[test]
fn a_mismatching_soap_action_header_is_refused() {
    let request = envelope(CONTENT_DIRECTORY, "Browse", &[]);
    let response = dispatcher().dispatch(
        &request,
        Some("\"urn:schemas-upnp-org:service:ContentDirectory:1#Search\""),
    );
    assert!(response.body.contains("<errorCode>401</errorCode>"));
}

/// The answer of a valid request carries no fault.
#[test]
fn a_valid_request_is_answered_without_a_fault() {
    let (status, body) = answer(&envelope(CONNECTION_MANAGER, "GetProtocolInfo", &[]));
    assert_eq!(status, 200);
    assert!(!body.contains("<errorCode>"));
    assert!(body.contains("<Source>"));
}
