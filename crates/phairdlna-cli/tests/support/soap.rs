//! The SOAP requests of the tests.
//!
//! What: the envelopes a control point sends.
//! Why: a client sends a SOAP request, so the tests send one too.
//! Example: `browse_request("1", "BrowseDirectChildren")` is the request a
//!       television sends to see the content of a folder.

use phairdlna_core::upnp::{SOAP_ENCODING_STYLE, SOAP_ENVELOPE_NAMESPACE};

/// The service type of the ContentDirectory service.
pub const CONTENT_DIRECTORY: &str = "urn:schemas-upnp-org:service:ContentDirectory:1";

/// The service type of the ConnectionManager service.
pub const CONNECTION_MANAGER: &str = "urn:schemas-upnp-org:service:ConnectionManager:1";

/// Builds a SOAP envelope.
pub fn envelope(service_type: &str, action: &str, arguments: &[(&str, &str)]) -> String {
    let mut body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <s:Envelope xmlns:s=\"{SOAP_ENVELOPE_NAMESPACE}\" s:encodingStyle=\"{SOAP_ENCODING_STYLE}\">\n\
         \x20 <s:Body>\n\
         \x20   <u:{action} xmlns:u=\"{service_type}\">"
    );
    for (name, value) in arguments {
        body.push_str(&format!("\n\x20     <{name}>{value}</{name}>"));
    }
    body.push_str(&format!(
        "\n\x20   </u:{action}>\n\x20 </s:Body>\n</s:Envelope>\n"
    ));
    body
}

/// Builds a browse request for the ContentDirectory service.
pub fn browse_request(object_id: &str, flag: &str) -> String {
    envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", object_id),
            ("BrowseFlag", flag),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "0"),
            ("SortCriteria", ""),
        ],
    )
}
