//! A bounded reader for the XML a control point sends.
//!
//! What: a small parser that turns an XML document into a tree of elements,
//!       with the namespace of every element resolved.
//! Why: the server reads a SOAP request that arrives from the network, so the
//!       parser is hostile by default. It refuses a document type declaration,
//!       it understands only the five predefined entities and numeric character
//!       references, and it bounds the size, the depth, the number of nodes and
//!       the text of the document (`SEC-010`, `SEC-011`, `SEC-060`).
//! Example:
//!
//! ```
//! use phairdlna_core::upnp::xml_reader::parse;
//!
//! let root = parse(r#"<s:Envelope xmlns:s="http://example.org/soap"/>"#)
//!     .expect("a well-formed document");
//! assert_eq!(root.name, "Envelope");
//! assert_eq!(root.namespace, "http://example.org/soap");
//! ```

use crate::upnp::error::UpnpError;

use super::xml_syntax::{
    declarations, expect, find, is_declaration, read_attributes, read_cdata, read_name, read_text,
    resolve, skip_prolog, split_name, Cursor,
};

/// The bounds a parsed document must stay inside.
#[derive(Debug, Clone, Copy)]
pub struct XmlLimits {
    /// The largest document that is read, in bytes.
    pub max_bytes: usize,
    /// The deepest nesting that is read.
    pub max_depth: usize,
    /// The largest number of elements in the document.
    pub max_nodes: usize,
    /// The largest text of a single element, in bytes.
    pub max_text_bytes: usize,
}

impl Default for XmlLimits {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024,
            max_depth: 32,
            max_nodes: 1_000,
            max_text_bytes: 32 * 1024,
        }
    }
}

/// One element of a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// The local name of the element, without its prefix.
    pub name: String,
    /// The namespace the element belongs to, or an empty string.
    pub namespace: String,
    /// The attributes of the element, without the namespace declarations.
    pub attributes: Vec<(String, String)>,
    /// The child elements.
    pub children: Vec<Element>,
    /// The text directly inside the element.
    pub text: String,
}

impl Element {
    /// The child element with the given name, if the element has one.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    /// The text of the child element with the given name.
    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|child| child.text.as_str())
    }
}

/// Reads a document with the default bounds.
pub fn parse(input: &str) -> Result<Element, UpnpError> {
    parse_with_limits(input, &XmlLimits::default())
}

/// Reads a document with the given bounds.
pub fn parse_with_limits(input: &str, limits: &XmlLimits) -> Result<Element, UpnpError> {
    if input.len() > limits.max_bytes {
        return Err(UpnpError::RequestTooLarge {
            limit: limits.max_bytes,
        });
    }
    let mut cursor = Cursor {
        input: input.as_bytes(),
        position: 0,
        limits,
        nodes: 0,
    };
    // XML 1.0 binds the prefix `xml` to its own namespace; a document may
    // use it without declaring it (`xml:lang`, for example).
    let mut scope: Vec<(String, String)> = vec![(
        "xml".to_owned(),
        "http://www.w3.org/XML/1998/namespace".to_owned(),
    )];
    skip_prolog(&mut cursor)?;
    let root = parse_element(&mut cursor, &mut scope, 0)?;
    skip_prolog(&mut cursor)?;
    if cursor.position < cursor.input.len() {
        return Err(UpnpError::MalformedRequest {
            reason: "the document carries more than one root element",
        });
    }
    Ok(root)
}

/// Reads one element and everything inside it.
fn parse_element(
    cursor: &mut Cursor,
    scope: &mut Vec<(String, String)>,
    depth: usize,
) -> Result<Element, UpnpError> {
    if depth > cursor.limits.max_depth {
        return Err(UpnpError::RequestTooDeep {
            limit: cursor.limits.max_depth,
        });
    }
    cursor.nodes += 1;
    if cursor.nodes > cursor.limits.max_nodes {
        return Err(UpnpError::TooManyNodes {
            limit: cursor.limits.max_nodes,
        });
    }
    expect(cursor, "<")?;
    let raw_name = read_name(cursor)?;
    let attributes = read_attributes(cursor)?;
    let scope_length = scope.len();
    scope.extend(declarations(&attributes));
    let (prefix, name) = split_name(&raw_name);
    let namespace = resolve(scope, prefix)?;
    let mut element = Element {
        name,
        namespace,
        attributes: attributes
            .iter()
            .filter(|(attribute, _)| !is_declaration(attribute))
            .cloned()
            .collect(),
        children: Vec::new(),
        text: String::new(),
    };
    if cursor.starts_with("/>") {
        cursor.skip(2);
        scope.truncate(scope_length);
        return Ok(element);
    }
    expect(cursor, ">")?;
    read_content(cursor, scope, depth, &raw_name, &mut element)?;
    scope.truncate(scope_length);
    Ok(element)
}

/// Reads the content of an element up to its closing tag.
fn read_content(
    cursor: &mut Cursor,
    scope: &mut Vec<(String, String)>,
    depth: usize,
    raw_name: &str,
    element: &mut Element,
) -> Result<(), UpnpError> {
    loop {
        if cursor.starts_with("</") {
            cursor.skip(2);
            let closing = read_name(cursor)?;
            cursor.skip_whitespace();
            expect(cursor, ">")?;
            if closing != raw_name {
                return Err(UpnpError::MalformedRequest {
                    reason: "an element is closed with a different name",
                });
            }
            return Ok(());
        }
        if cursor.starts_with("<!--") {
            skip_comment(cursor)?;
            continue;
        }
        if cursor.starts_with("<?") {
            skip_instruction(cursor)?;
            continue;
        }
        if cursor.starts_with("<![CDATA[") {
            element.text.push_str(&read_cdata(cursor)?);
            if element.text.len() > cursor.limits.max_text_bytes {
                return Err(UpnpError::TextTooLong {
                    limit: cursor.limits.max_text_bytes,
                });
            }
            continue;
        }
        if cursor.starts_with("<") {
            let child = parse_element(cursor, scope, depth + 1)?;
            element.children.push(child);
            continue;
        }
        element.text.push_str(&read_text(cursor)?);
        if element.text.len() > cursor.limits.max_text_bytes {
            return Err(UpnpError::TextTooLong {
                limit: cursor.limits.max_text_bytes,
            });
        }
    }
}

/// Skips a comment.
fn skip_comment(cursor: &mut Cursor) -> Result<(), UpnpError> {
    skip_to(cursor, "-->")
}

/// Skips a processing instruction.
fn skip_instruction(cursor: &mut Cursor) -> Result<(), UpnpError> {
    skip_to(cursor, "?>")
}

/// Skips everything up to the given closing text.
fn skip_to(cursor: &mut Cursor, closing: &str) -> Result<(), UpnpError> {
    match find(cursor.rest(), closing.as_bytes()) {
        Some(position) => {
            cursor.skip(position + closing.len());
            Ok(())
        }
        None => Err(UpnpError::MalformedRequest {
            reason: "the document ends inside a declaration",
        }),
    }
}
