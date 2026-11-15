//! The SOAP envelope of the control path.
//!
//! What: the reading of a control request out of its envelope, and the
//!       writing of the answer and of the fault.
//! Why: the envelope decides which action runs, so a broken envelope has to
//!       become a fault and never a panic (`INT-025`, `UX-005`).
//! Example: `cargo test --package phairdlna-core --test soap_envelope`.

use phairdlna_core::upnp::{
    check_soap_action, fault_body, parse_request, response_body, SoapResponse,
};

/// A request is read out of its envelope.
#[test]
fn a_request_is_read() {
    let request = parse_request(
        r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                 <s:Body>
                   <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
                     <ObjectID>0</ObjectID>
                     <BrowseFlag>BrowseDirectChildren</BrowseFlag>
                   </u:Browse>
                 </s:Body>
               </s:Envelope>"#,
    )
    .expect("a well-formed request");
    assert_eq!(request.action, "Browse");
    assert_eq!(
        request.service_type,
        "urn:schemas-upnp-org:service:ContentDirectory:1"
    );
    assert_eq!(request.argument("ObjectID"), Some("0"));
    assert_eq!(request.argument("BrowseFlag"), Some("BrowseDirectChildren"));
}

/// The negative case: a document that is not an envelope is refused.
#[test]
fn a_document_that_is_not_an_envelope_is_refused() {
    assert!(parse_request("<html><body/></html>").is_err());
}

/// The negative case: an envelope without a body is refused.
#[test]
fn an_envelope_without_a_body_is_refused() {
    let request = r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"/>"#;
    assert!(parse_request(request).is_err());
}

/// The negative case: a body without an action is refused.
#[test]
fn a_body_without_an_action_is_refused() {
    let request = r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                           <s:Body/>
                         </s:Envelope>"#;
    assert!(parse_request(request).is_err());
}

/// The answer carries the arguments in the order they were given.
#[test]
fn the_answer_carries_the_arguments_in_order() {
    let body = response_body(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "Browse",
        &[
            ("Result", "<DIDL-Lite/>"),
            ("NumberReturned", "3"),
            ("TotalMatches", "17"),
            ("UpdateID", "42"),
        ],
    );
    let result = body.find("<Result>").expect("the result is present");
    let number = body.find("<NumberReturned>").expect("the count is present");
    assert!(result < number, "the arguments keep their order");
    assert!(body.contains("<u:BrowseResponse xmlns:u="));
}

/// A fault carries the code and the description of the error.
#[test]
fn a_fault_carries_the_code() {
    let body = fault_body(401);
    assert!(body.contains("<errorCode>401</errorCode>"));
    assert!(body.contains("<errorDescription>Invalid Action</errorDescription>"));
    assert!(body.contains("<faultcode>s:Client</faultcode>"));
    assert!(body.contains("<faultstring>UPnPError</faultstring>"));
    assert!(body.contains("urn:schemas-upnp-org:control-1-0"));
}

/// A fault is carried with the status the specification asks for.
#[test]
fn a_fault_is_carried_with_status_500() {
    assert_eq!(SoapResponse::fault(402).status, 500);
    assert_eq!(SoapResponse::ok(String::new()).status, 200);
}

/// A `SOAPAction` header that names another action is refused.
#[test]
fn a_mismatching_soap_action_header_is_refused() {
    let request = parse_request(
        r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                 <s:Body>
                   <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1"/>
                 </s:Body>
               </s:Envelope>"#,
    )
    .expect("a well-formed request");
    assert!(check_soap_action(&request, None).is_ok());
    assert!(check_soap_action(
        &request,
        Some("\"urn:schemas-upnp-org:service:ContentDirectory:1#Browse\"")
    )
    .is_ok());
    assert!(check_soap_action(
        &request,
        Some("\"urn:schemas-upnp-org:service:ContentDirectory:1#Search\"")
    )
    .is_err());
}
