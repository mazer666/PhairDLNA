//! The SOAP envelope of a UPnP control request and of its answer.
//!
//! What: the reading of a SOAP request into a service, an action and its
//!       arguments, and the writing of the answer and of the fault.
//! Why: a control point sends a SOAP envelope to the control URL of a
//!       service. The envelope decides which action runs, so it is read
//!       strictly: an envelope that does not have the expected shape becomes a
//!       fault, never a panic (`INT-025`, `UX-005`).
//! Example:
//!
//! ```
//! use phairdlna_core::upnp::parse_request;
//!
//! let request = parse_request(
//!     r#"<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
//!        <s:Body>
//!          <u:GetSystemUpdateID xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1"/>
//!        </s:Body>
//!      </s:Envelope>"#,
//! )
//! .expect("a well-formed request");
//! assert_eq!(request.action, "GetSystemUpdateID");
//! assert_eq!(request.service_type, "urn:schemas-upnp-org:service:ContentDirectory:1");
//! ```

use crate::upnp::error::{fault_description, UpnpError};
use crate::upnp::xml_reader::{parse_with_limits, Element, XmlLimits};
use crate::xml::XmlWriter;

/// The namespace of the SOAP envelope.
pub const SOAP_ENVELOPE_NAMESPACE: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// The encoding style of the SOAP envelope.
pub const SOAP_ENCODING_STYLE: &str = "http://schemas.xmlsoap.org/soap/encoding/";

/// The namespace of the UPnP error detail.
pub const UPNP_CONTROL_NAMESPACE: &str = "urn:schemas-upnp-org:control-1-0";

/// The content type of every document the control path answers with.
pub const XML_CONTENT_TYPE: &str = "text/xml; charset=\"utf-8\"";

/// A control request, read out of a SOAP envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoapRequest {
    /// The service the request addresses, as the namespace of the action.
    pub service_type: String,
    /// The action the request names.
    pub action: String,
    /// The arguments of the action, in the order they were sent.
    pub arguments: Vec<(String, String)>,
}

impl SoapRequest {
    /// The value of an argument, if the request carries it.
    pub fn argument(&self, name: &str) -> Option<&str> {
        self.arguments
            .iter()
            .find(|(argument, _)| argument == name)
            .map(|(_, value)| value.as_str())
    }

    /// The value of an argument that the action needs.
    pub fn required_argument(&self, name: &str) -> Result<&str, UpnpError> {
        self.argument(name)
            .ok_or_else(|| UpnpError::InvalidArgument {
                argument: name.to_owned(),
            })
    }
}

/// The answer of the control path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoapResponse {
    /// The HTTP status the answer carries.
    pub status: u16,
    /// The body of the answer.
    pub body: String,
}

impl SoapResponse {
    /// A successful answer.
    pub fn ok(body: String) -> Self {
        Self { status: 200, body }
    }

    /// A fault: the UPnP errors are carried in the body, and the status is
    /// the one the specification asks for (`INT-025`).
    pub fn fault(code: u16) -> Self {
        Self {
            status: 500,
            body: fault_body(code),
        }
    }

    /// The fault that belongs to an error.
    pub fn from_error(error: &UpnpError) -> Self {
        Self::fault(error.fault_code().unwrap_or(501))
    }
}

/// Reads a control request out of a SOAP envelope.
pub fn parse_request(body: &str) -> Result<SoapRequest, UpnpError> {
    parse_request_with_limits(body, &XmlLimits::default())
}

/// Reads a control request out of a SOAP envelope, with the given bounds.
pub fn parse_request_with_limits(body: &str, limits: &XmlLimits) -> Result<SoapRequest, UpnpError> {
    let root = parse_with_limits(body, limits)?;
    let soap_body = find_body(&root)?;
    let action = soap_body
        .children
        .first()
        .ok_or(UpnpError::MalformedRequest {
            reason: "the body of the envelope carries no action",
        })?;
    let arguments = action
        .children
        .iter()
        .map(|argument| (argument.name.clone(), argument.text.clone()))
        .collect();
    Ok(SoapRequest {
        service_type: action.namespace.clone(),
        action: action.name.clone(),
        arguments,
    })
}

/// The `Body` of the envelope.
fn find_body(root: &Element) -> Result<&Element, UpnpError> {
    if root.name != "Envelope" || root.namespace != SOAP_ENVELOPE_NAMESPACE {
        return Err(UpnpError::MalformedRequest {
            reason: "the request is not a SOAP envelope",
        });
    }
    root.children
        .iter()
        .find(|child| child.name == "Body" && child.namespace == SOAP_ENVELOPE_NAMESPACE)
        .ok_or(UpnpError::MalformedRequest {
            reason: "the envelope carries no body",
        })
}

/// Checks the `SOAPAction` header against the envelope.
///
/// A client sends the header, and a request whose header names another action
/// than its body is answered with `401 Invalid Action` rather than with an
/// answer to the wrong action.
pub fn check_soap_action(request: &SoapRequest, header: Option<&str>) -> Result<(), UpnpError> {
    let Some(header) = header else {
        return Ok(());
    };
    let action = header.trim().trim_matches('"');
    let Some((service_type, action)) = action_of_header(action) else {
        return Ok(());
    };
    if service_type == request.service_type && action == request.action {
        return Ok(());
    }
    Err(UpnpError::UnknownAction {
        action: action.to_owned(),
    })
}

/// The service and the action of a `SOAPAction` header.
fn action_of_header(header: &str) -> Option<(&str, &str)> {
    let (service_type, action) = header.split_once('#')?;
    if service_type.is_empty() || action.is_empty() {
        return None;
    }
    Some((service_type, action))
}

/// The answer of an action: the envelope with the arguments of the action.
pub fn response_body(service_type: &str, action: &str, arguments: &[(&str, &str)]) -> String {
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.open(
        "s:Envelope",
        &[
            ("xmlns:s", SOAP_ENVELOPE_NAMESPACE),
            ("s:encodingStyle", SOAP_ENCODING_STYLE),
        ],
    );
    writer.open("s:Body", &[]);
    writer.open(&format!("u:{action}Response"), &[("xmlns:u", service_type)]);
    for (name, value) in arguments {
        writer.text(name, value);
    }
    writer.close(&format!("u:{action}Response"));
    writer.close("s:Body");
    writer.close("s:Envelope");
    writer.finish()
}

/// The answer of an action whose `Result` argument carries a document.
///
/// The document is written verbatim, because the caller has already escaped
/// it for the text of an element: the `didl` module owns that second
/// escaping (`INT-073`), and escaping it again here would corrupt it
/// (`INT-065`). The document comes first, because the specification names
/// `Result` before the other arguments of a browse (`INT-031`).
pub fn response_body_with_result(
    service_type: &str,
    action: &str,
    result: &str,
    arguments: &[(&str, &str)],
) -> String {
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.open(
        "s:Envelope",
        &[
            ("xmlns:s", SOAP_ENVELOPE_NAMESPACE),
            ("s:encodingStyle", SOAP_ENCODING_STYLE),
        ],
    );
    writer.open("s:Body", &[]);
    writer.open(&format!("u:{action}Response"), &[("xmlns:u", service_type)]);
    writer.escaped_text("Result", result);
    for (name, value) in arguments {
        writer.text(name, value);
    }
    writer.close(&format!("u:{action}Response"));
    writer.close("s:Body");
    writer.close("s:Envelope");
    writer.finish()
}

/// The body of a fault.
///
/// The shape is the one of the specification: a SOAP fault whose detail
/// carries the UPnP error code and its description.
pub fn fault_body(code: u16) -> String {
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.open(
        "s:Envelope",
        &[
            ("xmlns:s", SOAP_ENVELOPE_NAMESPACE),
            ("s:encodingStyle", SOAP_ENCODING_STYLE),
        ],
    );
    writer.open("s:Body", &[]);
    writer.open("s:Fault", &[]);
    writer.text("faultcode", "s:Client");
    writer.text("faultstring", "UPnPError");
    writer.open("detail", &[]);
    writer.open("UPnPError", &[("xmlns", UPNP_CONTROL_NAMESPACE)]);
    writer.text("errorCode", &code.to_string());
    writer.text("errorDescription", fault_description(code));
    writer.close("UPnPError");
    writer.close("detail");
    writer.close("s:Fault");
    writer.close("s:Body");
    writer.close("s:Envelope");
    writer.finish()
}
