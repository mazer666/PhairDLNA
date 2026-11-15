#![allow(dead_code)]
//! Helpers for the tests of the DIDL-Lite document.
//!
//! What: the prefixes a document uses, the prefixes its root element
//!       declares, and a reader that parses a document with the XML reader of
//!       the project.
//! Why: the acceptance criterion AC-01 says that every prefix the document
//!       uses is declared on the root element, and a string match cannot
//!       prove that: the prefixes have to be collected from the elements of
//!       the document and compared with the declared ones.
//! Example: `used_prefixes(&document)` returns the prefixes of the elements
//!       of the document.

use phairdlna_core::upnp::xml_reader::Element;

/// Parses a document with the reader of the project.
///
/// The reader is the one the server uses for the XML a control point sends. It
/// resolves the namespaces and decodes the entities, so a test that reads the
/// document with it proves that a client can read the document too — and not
/// only that the document looks right.
pub fn parse(xml: &str) -> Element {
    phairdlna_core::upnp::xml_reader::parse(xml).expect("a well-formed document")
}

/// The value of an attribute of an element.
pub fn attribute(element: &Element, name: &str) -> Option<String> {
    element
        .attributes
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

/// The prefixes the elements of a document use.
pub fn used_prefixes(xml: &str) -> Vec<String> {
    let mut prefixes: Vec<String> = Vec::new();
    for element in xml.split('<').skip(1) {
        let name: String = element
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == ':' || *c == '_' || *c == '-')
            .collect();
        if let Some((prefix, _)) = name.split_once(':') {
            let prefix = prefix.to_owned();
            if !prefixes.contains(&prefix) {
                prefixes.push(prefix);
            }
        }
    }
    prefixes
}

/// The prefixes the root element declares.
pub fn declared_prefixes(xml: &str) -> Vec<String> {
    let root = xml.split('<').nth(1).unwrap_or_default();
    let mut prefixes: Vec<String> = Vec::new();
    for attribute in root.split_whitespace() {
        if let Some(prefix) = attribute.strip_prefix("xmlns:") {
            let name = prefix.split('=').next().unwrap_or_default();
            let name = name.split('"').next().unwrap_or_default().to_owned();
            if !name.is_empty() {
                prefixes.push(name);
            }
        }
    }
    prefixes
}
