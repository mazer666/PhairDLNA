//! The XML reader that reads what a control point sends.
//!
//! What: the acceptance criteria of the reader itself — the documents it
//!       accepts and the documents it refuses.
//! Why: the reader is the boundary between the network and the server, so it
//!       is the place where a hostile document has to fail. A reader that
//!       accepts a document type declaration, or an entity it does not
//!       define, is an entity-expansion attack waiting to happen (`SEC-010`).
//! Example: `cargo test --package phairdlna-core --test xml_reader`.

use phairdlna_core::upnp::xml_reader::{parse, parse_with_limits, XmlLimits};
use phairdlna_core::upnp::UpnpError;

#[path = "xml_reader/truncation.rs"]
mod truncation;

/// A namespace is resolved.
#[test]
fn a_namespace_is_resolved() {
    let root = parse(r#"<s:Envelope xmlns:s="http://example.org/soap"/>"#)
        .expect("a well-formed document");
    assert_eq!(root.name, "Envelope");
    assert_eq!(root.namespace, "http://example.org/soap");
}

/// The text of an element is read, and its entities are decoded.
#[test]
fn the_text_is_decoded() {
    let root = parse("<a>one &amp; two &lt;three&gt;</a>").expect("a well-formed document");
    assert_eq!(root.text, "one & two <three>");
}

/// A numeric character reference is decoded.
#[test]
fn a_numeric_character_reference_is_decoded() {
    let root = parse("<a>&#65;&#x42;</a>").expect("a well-formed document");
    assert_eq!(root.text, "AB");
}

/// Numeric character references in attributes keep the existing semantics.
#[test]
fn a_numeric_character_reference_in_an_attribute_is_decoded() {
    let root = parse("<a name=\"&#65;&#x42;\"/>").expect("a well-formed document");
    assert_eq!(root.attributes[0].1, "AB");
}

/// Numeric references must name characters allowed by XML 1.0.
#[test]
fn forbidden_xml_characters_are_refused_in_text_and_attributes() {
    for document in ["<a>&#x1;</a>", "<a name=\"&#x1;\"/>", "<a>&#xFFFE;</a>"] {
        assert!(matches!(
            parse(document),
            Err(UpnpError::UnknownEntity { .. })
        ));
    }
}

/// A custom entity is refused in an attribute just as it is in text.
#[test]
fn an_undefined_entity_in_an_attribute_is_refused() {
    assert!(matches!(
        parse("<a name=\"&boom;\"/>"),
        Err(UpnpError::UnknownEntity { .. })
    ));
}

/// A document type declaration is refused, because it is the entry point of
/// an entity-expansion attack (`SEC-010`).
#[test]
fn a_document_type_declaration_is_refused() {
    let document = "<!DOCTYPE root [<!ENTITY x \"y\">]><root>&x;</root>";
    assert!(matches!(
        parse(document),
        Err(UpnpError::MalformedRequest { .. })
    ));
}

/// An entity the document does not define is refused.
#[test]
fn an_undefined_entity_is_refused() {
    assert!(matches!(
        parse("<a>&boom;</a>"),
        Err(UpnpError::UnknownEntity { .. })
    ));
}

/// A prefix that was never declared is refused, because a strict control
/// point rejects such a document anyway.
#[test]
fn an_undeclared_prefix_is_refused() {
    assert!(matches!(
        parse("<s:Envelope/>"),
        Err(UpnpError::MalformedRequest { .. })
    ));
}

/// A document that is larger than the bound is refused.
#[test]
fn a_document_that_is_too_large_is_refused() {
    let limits = XmlLimits {
        max_bytes: 8,
        ..XmlLimits::default()
    };
    assert!(matches!(
        parse_with_limits("<a>0123456789</a>", &limits),
        Err(UpnpError::RequestTooLarge { .. })
    ));
}

/// A document that is nested too deeply is refused.
#[test]
fn a_document_that_is_too_deep_is_refused() {
    let limits = XmlLimits {
        max_depth: 2,
        ..XmlLimits::default()
    };
    assert!(matches!(
        parse_with_limits("<a><b><c><d/></c></b></a>", &limits),
        Err(UpnpError::RequestTooDeep { .. })
    ));
}

/// The configured maximum depth includes the root at depth zero.
#[test]
fn the_document_at_the_depth_boundary_is_accepted() {
    let limits = XmlLimits {
        max_depth: 2,
        ..XmlLimits::default()
    };
    assert!(parse_with_limits("<a><b><c/></b></a>", &limits).is_ok());
}

/// A document with too many elements is refused.
#[test]
fn a_document_with_too_many_nodes_is_refused() {
    let limits = XmlLimits {
        max_nodes: 2,
        ..XmlLimits::default()
    };
    assert!(matches!(
        parse_with_limits("<a><b/><c/></a>", &limits),
        Err(UpnpError::TooManyNodes { .. })
    ));
}

/// An element that is closed with a different name is refused.
#[test]
fn a_mismatched_closing_tag_is_refused() {
    assert!(matches!(
        parse("<a><b></c></a>"),
        Err(UpnpError::MalformedRequest { .. })
    ));
}

/// The children of an element are read, and a child is found by name.
#[test]
fn the_children_are_read() {
    let root = parse("<a><b>one</b><b>two</b></a>").expect("a well-formed document");
    assert_eq!(root.children.len(), 2);
    assert_eq!(root.child_text("b"), Some("one"));
}

/// A CDATA section is read as text.
#[test]
fn a_cdata_section_is_read() {
    let root = parse("<a><![CDATA[<not a tag>]]></a>").expect("a well-formed document");
    assert_eq!(root.text, "<not a tag>");
}

/// An attribute value cannot break out of its attribute.
#[test]
fn an_attribute_value_cannot_escape_its_attribute() {
    let root = parse("<a name=\"&quot;&gt;&lt;injected/&gt;\"/>").expect("a well-formed document");
    let value = &root.attributes[0].1;
    assert_eq!(value, "\"><injected/>");
    assert_eq!(root.children.len(), 0);
}

/// The text of an element that is too long is refused.
#[test]
fn a_text_that_is_too_long_is_refused() {
    let limits = XmlLimits {
        max_text_bytes: 4,
        ..XmlLimits::default()
    };
    assert!(matches!(
        parse_with_limits("<a>0123456789</a>", &limits),
        Err(UpnpError::TextTooLong { .. })
    ));
}

/// A document with two root elements is refused.
#[test]
fn a_document_with_two_roots_is_refused() {
    assert!(matches!(
        parse("<a/><b/>"),
        Err(UpnpError::MalformedRequest { .. })
    ));
}

/// The prefix `xml` is bound by XML 1.0 itself and needs no declaration.
///
/// A control point that adds `xml:lang` to its action element must not be
/// refused.
#[test]
fn the_xml_prefix_needs_no_declaration() {
    let root = parse(r#"<a xml:lang="en">text</a>"#).expect("a well-formed document");
    assert_eq!(root.text, "text");
    assert_eq!(root.attributes[0].1, "en", "the attribute is read");
}

/// A CDATA section counts towards the bound of the text of an element.
#[test]
fn a_cdata_section_that_is_too_long_is_refused() {
    let limits = XmlLimits {
        max_text_bytes: 4,
        ..XmlLimits::default()
    };
    assert!(matches!(
        parse_with_limits("<a><![CDATA[0123456789]]></a>", &limits),
        Err(UpnpError::TextTooLong { .. })
    ));
}
