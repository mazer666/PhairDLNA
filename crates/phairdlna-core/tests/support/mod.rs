#![allow(dead_code)]
//! Fixtures for the protocol tests.
//!
//! What: a checker that verifies that a generated document is well-formed —
//!       that the elements nest, that every element is closed in order, and
//!       that no reserved character is left unescaped — and a builder for
//!       SOAP control requests.
//! Why: the acceptance criteria of F-02 say "*when it is parsed*". A string
//!       match on the document cannot tell a well-formed document from a
//!       broken one, and the project has no XML parser as a dependency yet.
//!       The checker reads the documents the project itself produces, which
//!       escape every reserved character, so a small checker is enough.
//! Example: `assert_well_formed(&description.to_xml())` panics with the
//!       offending fragment if the document is malformed.

pub mod catalogue;

/// The service type of the ContentDirectory service.
pub const CONTENT_DIRECTORY: &str = "urn:schemas-upnp-org:service:ContentDirectory:1";

/// The service type of the ConnectionManager service.
pub const CONNECTION_MANAGER: &str = "urn:schemas-upnp-org:service:ConnectionManager:1";

/// The namespace of the SOAP envelope.
const SOAP_ENVELOPE: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// The encoding style of the SOAP envelope.
const SOAP_ENCODING: &str = "http://schemas.xmlsoap.org/soap/encoding/";

/// Builds a SOAP control request.
///
/// The envelope is the one of standards §3.3: the action element carries the
/// namespace of its service, and every argument is an element of its own.
pub fn envelope(service_type: &str, action: &str, arguments: &[(&str, &str)]) -> String {
    let mut body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <s:Envelope xmlns:s=\"{SOAP_ENVELOPE}\" s:encodingStyle=\"{SOAP_ENCODING}\">\n\
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
pub fn browse_request(
    object_id: &str,
    flag: &str,
    starting_index: u32,
    requested_count: u32,
) -> String {
    envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", object_id),
            ("BrowseFlag", flag),
            ("Filter", "*"),
            ("StartingIndex", &starting_index.to_string()),
            ("RequestedCount", &requested_count.to_string()),
            ("SortCriteria", ""),
        ],
    )
}

/// Checks that a document is well-formed.
pub fn assert_well_formed(xml: &str) {
    let mut open: Vec<String> = Vec::new();
    let mut rest = xml;
    while let Some(position) = rest.find('<') {
        let (text, after) = rest.split_at(position);
        assert_text(text);
        rest = after;
        let end = match rest.find('>') {
            Some(end) => end,
            None => panic!("the document ends inside a tag: {rest}"),
        };
        let tag = &rest[1..end];
        rest = &rest[end + 1..];
        if tag.starts_with('?') || tag.starts_with('!') {
            continue;
        }
        if let Some(name) = tag.strip_prefix('/') {
            let name = name.trim();
            let expected = open.pop().unwrap_or_else(|| {
                panic!("the element `{name}` is closed, but no element is open")
            });
            assert_eq!(expected, name, "the elements are closed in the wrong order");
            continue;
        }
        let name = tag.split_whitespace().next().unwrap_or_default();
        if !tag.ends_with('/') {
            open.push(name.to_owned());
        }
    }
    assert_text(rest);
    assert!(
        open.is_empty(),
        "the document ends with an open element: {open:?}"
    );
}

/// Checks that the text between two tags carries no unescaped ampersand.
fn assert_text(text: &str) {
    let mut rest = text;
    while let Some(position) = rest.find('&') {
        rest = &rest[position + 1..];
        let end = rest
            .find(';')
            .unwrap_or_else(|| panic!("the text carries an unescaped ampersand: {text}"));
        let entity = &rest[..end];
        assert!(
            is_known_entity(entity),
            "the text carries an unknown entity: &{entity};"
        );
        rest = &rest[end + 1..];
    }
}

/// Whether an entity reference is one that XML 1.0 defines.
fn is_known_entity(entity: &str) -> bool {
    if matches!(entity, "amp" | "lt" | "gt" | "quot" | "apos") {
        return true;
    }
    let number = entity.strip_prefix('#').unwrap_or_default();
    if let Some(hexadecimal) = number.strip_prefix('x') {
        return !hexadecimal.is_empty() && hexadecimal.chars().all(|c| c.is_ascii_hexdigit());
    }
    !number.is_empty() && number.chars().all(|c| c.is_ascii_digit())
}

/// The text of an element of an XML document.
///
/// The element is found by its name, so that a test can read an argument of
/// an answer without pulling in a parser.
pub fn element_text(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_owned())
}

/// The text of an element, with the entities of XML decoded once.
pub fn element_text_unescaped(xml: &str, name: &str) -> Option<String> {
    element_text(xml, name).map(|text| unescape_once(&text))
}

/// Decodes the entity references of a text once.
pub fn unescape_once(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find('&') {
        decoded.push_str(&rest[..position]);
        rest = &rest[position + 1..];
        let Some(end) = rest.find(';') else {
            decoded.push('&');
            decoded.push_str(rest);
            return decoded;
        };
        let entity = &rest[..end];
        let replacement = match entity {
            "amp" => "&".to_owned(),
            "lt" => "<".to_owned(),
            "gt" => ">".to_owned(),
            "quot" => "\"".to_owned(),
            "apos" => "'".to_owned(),
            other => format!("&{other};"),
        };
        decoded.push_str(&replacement);
        rest = &rest[end + 1..];
    }
    decoded.push_str(rest);
    decoded
}
