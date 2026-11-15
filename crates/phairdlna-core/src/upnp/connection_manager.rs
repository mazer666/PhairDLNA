//! The ConnectionManager service: the actions a control point calls.
//!
//! What: the handlers of `GetProtocolInfo`, `GetCurrentConnectionIDs`,
//!       `GetCurrentConnectionInfo`, `PrepareForConnection` and
//!       `ConnectionComplete`.
//! Why: the service tells a client which formats the server can send and
//!       which transfers are running. This server only pushes media over
//!       `http-get`, so the `Sink` list is empty and no peer connection is
//!       ever negotiated (`INT-051`, `INT-056`, standards §8.4 M3). The
//!       answers never name a file, a folder or a path (`PLT-008`).
//! Example: `get_protocol_info` answers with the `Source` list of the MIME
//!       registry and an empty `Sink`.

use crate::upnp::error::UpnpError;
use crate::upnp::mime::{sink_list, source_list};
use crate::upnp::soap::{response_body, SoapRequest, SoapResponse};

/// The answer of `GetProtocolInfo`.
///
/// The `Source` list names the formats the server can deliver, and the `Sink`
/// list is empty because the server never receives media (`INT-050`,
/// `INT-051`). The list is a comma-separated list of protocolInfo strings
/// without surrounding whitespace (standards §1.3).
pub fn get_protocol_info() -> SoapResponse {
    answer(
        "GetProtocolInfo",
        &[("Source", &source_list()), ("Sink", &sink_list())],
    )
}

/// The answer of `GetCurrentConnectionIDs`.
///
/// The list is empty when no transfer is running. The literal `0` that some
/// servers answer would make a client count a connection that does not exist
/// (`INT-052`).
pub fn get_current_connection_ids() -> SoapResponse {
    answer("GetCurrentConnectionIDs", &[("ConnectionIDs", "")])
}

/// The answer of `GetCurrentConnectionInfo`.
///
/// This server negotiates no peer connection: it delivers media over
/// `http-get`, which the client fetches itself. A connection identifier that
/// was never issued is therefore the fault `701 No Such Object`
/// (`INT-053`).
pub fn get_current_connection_info(request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
    let connection_id = request.required_argument("ConnectionID")?.trim().to_owned();
    Err(UpnpError::NoSuchObject {
        object_id: connection_id,
    })
}

/// The answer of `PrepareForConnection`.
///
/// The server pushes media over `http-get` and negotiates no peer connection,
/// so there is nothing to prepare. The action is marked optional in the
/// service description (standards §8.4 M5), and the server answers it with
/// the fault `600 Optional Action Not Implemented`. No connection is created,
/// which keeps the connection table free of a connection that would never
/// leave the pending state (`INT-053`).
///
/// A direction outside the two of the specification is refused with `402
/// Invalid Args` before that, because a request the server does not
/// understand is a bad argument and not a missing feature.
pub fn prepare_for_connection(request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
    let direction = request.required_argument("Direction")?.trim().to_owned();
    if direction != "Input" && direction != "Output" {
        return Err(UpnpError::InvalidArgument {
            argument: "Direction".to_owned(),
        });
    }
    Err(UpnpError::OptionalActionNotImplemented {
        action: "PrepareForConnection".to_owned(),
    })
}

/// The answer of `ConnectionComplete`.
///
/// No connection is ever opened, so there is none to complete. The action is
/// marked optional in the service description, and the server answers it with
/// the fault `600 Optional Action Not Implemented` (`INT-053`).
pub fn connection_complete(_request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
    Err(UpnpError::OptionalActionNotImplemented {
        action: "ConnectionComplete".to_owned(),
    })
}

/// The answer of an action of the ConnectionManager service.
fn answer(action: &str, arguments: &[(&str, &str)]) -> SoapResponse {
    SoapResponse::ok(response_body(
        crate::upnp::services::CONNECTION_MANAGER.service_type,
        action,
        arguments,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        connection_complete, get_current_connection_ids, get_current_connection_info,
        get_protocol_info, prepare_for_connection,
    };
    use crate::upnp::soap::{parse_request, SoapRequest};

    /// A request with the given body.
    fn request(body: &str) -> SoapRequest {
        parse_request(body).expect("a well-formed request")
    }

    /// The `Source` list names the formats and the `Sink` list is empty.
    #[test]
    fn the_protocol_info_answers_with_the_source_and_an_empty_sink() {
        let body = get_protocol_info().body;
        assert!(body.contains("<Source>http-get:*:video/mp4:*"));
        assert!(body.contains("<Sink></Sink>"));
    }

    /// The list of the running connections is empty.
    #[test]
    fn the_connection_list_is_empty() {
        let body = get_current_connection_ids().body;
        assert!(body.contains("<ConnectionIDs></ConnectionIDs>"));
    }

    /// A connection that was never issued is the fault `701`.
    #[test]
    fn an_unknown_connection_is_the_fault_701() {
        let request = request(
            r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                 <s:Body>
                   <u:GetCurrentConnectionInfo
                     xmlns:u="urn:schemas-upnp-org:service:ConnectionManager:1">
                     <ConnectionID>7</ConnectionID>
                   </u:GetCurrentConnectionInfo>
                 </s:Body>
               </s:Envelope>"#,
        );
        let error = get_current_connection_info(&request).expect_err("the connection is unknown");
        assert_eq!(error.fault_code(), Some(701));
    }

    /// A direction outside the two of the specification is the fault `402`.
    #[test]
    fn a_wrong_direction_is_the_fault_402() {
        let request = request(
            r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                 <s:Body>
                   <u:PrepareForConnection
                     xmlns:u="urn:schemas-upnp-org:service:ConnectionManager:1">
                     <Direction>Sideways</Direction>
                   </u:PrepareForConnection>
                 </s:Body>
               </s:Envelope>"#,
        );
        let error = prepare_for_connection(&request).expect_err("the direction is wrong");
        assert_eq!(error.fault_code(), Some(402));
    }

    /// A connection that was never issued cannot be completed.
    #[test]
    fn a_connection_cannot_be_completed() {
        let request = request(
            r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                 <s:Body>
                   <u:ConnectionComplete
                     xmlns:u="urn:schemas-upnp-org:service:ConnectionManager:1">
                     <ConnectionID>0</ConnectionID>
                   </u:ConnectionComplete>
                 </s:Body>
               </s:Envelope>"#,
        );
        let error = connection_complete(&request).expect_err("the action is not implemented");
        assert_eq!(error.fault_code(), Some(600));
    }
}
