//! The low-level scanning of an XML document.
//!
//! What: the cursor over the bytes of a document, and the reading of names,
//!       attributes, text, CDATA sections, entity references and namespace
//!       declarations.
//! Why: the parsing of a document that arrives from the network must never
//!       read past its end, guess a boundary, or read without a bound. The
//!       cursor makes every step explicit and every failure an error instead
//!       of a guess, and the entity decoding refuses everything that XML 1.0
//!       does not define (`SEC-010`).
//! Example: `read_name` reads the name that follows a `<`.

use crate::upnp::error::UpnpError;

use super::xml_reader::XmlLimits;

/// The position inside a document.
pub(super) struct Cursor<'a> {
    /// The bytes of the document.
    pub(super) input: &'a [u8],
    /// Where the cursor is.
    pub(super) position: usize,
    /// The bounds the document must stay inside.
    pub(super) limits: &'a XmlLimits,
    /// How many elements have been read.
    pub(super) nodes: usize,
}

impl Cursor<'_> {
    /// The bytes that are left.
    pub(super) fn rest(&self) -> &[u8] {
        &self.input[self.position..]
    }

    /// Whether the document continues with the given text.
    pub(super) fn starts_with(&self, text: &str) -> bool {
        self.rest().starts_with(text.as_bytes())
    }

    /// Moves past the given number of bytes.
    pub(super) fn skip(&mut self, count: usize) {
        self.position += count;
    }

    /// Skips the spaces and the line breaks.
    pub(super) fn skip_whitespace(&mut self) {
        while let Some(byte) = self.rest().first() {
            if byte.is_ascii_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }
}

/// Skips the declaration, the comments and the whitespace around the root.
pub(super) fn skip_prolog(cursor: &mut Cursor) -> Result<(), UpnpError> {
    loop {
        cursor.skip_whitespace();
        if cursor.starts_with("<?") {
            skip_to(cursor, "?>")?;
        } else if cursor.starts_with("<!--") {
            skip_to(cursor, "-->")?;
        } else {
            return Ok(());
        }
    }
}

/// Skips everything up to the given closing text.
pub(super) fn skip_to(cursor: &mut Cursor, closing: &str) -> Result<(), UpnpError> {
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

/// Reads the attributes of an element.
pub(super) fn read_attributes(cursor: &mut Cursor) -> Result<Vec<(String, String)>, UpnpError> {
    let mut attributes = Vec::new();
    loop {
        cursor.skip_whitespace();
        if cursor.starts_with("/>") || cursor.starts_with(">") {
            return Ok(attributes);
        }
        let name = read_name(cursor)?;
        cursor.skip_whitespace();
        expect(cursor, "=")?;
        cursor.skip_whitespace();
        let value = read_attribute_value(cursor)?;
        attributes.push((name, value));
    }
}

/// Reads a quoted attribute value and decodes its entities.
fn read_attribute_value(cursor: &mut Cursor) -> Result<String, UpnpError> {
    let quote = cursor
        .rest()
        .first()
        .copied()
        .ok_or(UpnpError::MalformedRequest {
            reason: "an attribute has no value",
        })?;
    if quote != b'"' && quote != b'\'' {
        return Err(UpnpError::MalformedRequest {
            reason: "an attribute value is not quoted",
        });
    }
    cursor.skip(1);
    let start = cursor.position;
    let end = match find(cursor.rest(), &[quote]) {
        Some(position) => position,
        None => {
            return Err(UpnpError::MalformedRequest {
                reason: "an attribute value is not closed",
            })
        }
    };
    let raw = String::from_utf8_lossy(&cursor.input[start..start + end]).into_owned();
    cursor.skip(end + 1);
    decode_entities(&raw)
}

/// Reads the text up to the next tag and decodes its entities.
pub(super) fn read_text(cursor: &mut Cursor) -> Result<String, UpnpError> {
    let start = cursor.position;
    let end = find(cursor.rest(), b"<").unwrap_or(cursor.rest().len());
    let raw = String::from_utf8_lossy(&cursor.input[start..start + end]).into_owned();
    cursor.skip(end);
    decode_entities(&raw)
}

/// Reads a CDATA section.
pub(super) fn read_cdata(cursor: &mut Cursor) -> Result<String, UpnpError> {
    cursor.skip("<![CDATA[".len());
    let start = cursor.position;
    let end = match find(cursor.rest(), b"]]>") {
        Some(position) => position,
        None => {
            return Err(UpnpError::MalformedRequest {
                reason: "a CDATA section is not closed",
            })
        }
    };
    let text = String::from_utf8_lossy(&cursor.input[start..start + end]).into_owned();
    cursor.skip(end + "]]>".len());
    Ok(text)
}

/// Reads a name: a letter, a digit, a colon, a dash or a dot.
pub(super) fn read_name(cursor: &mut Cursor) -> Result<String, UpnpError> {
    let start = cursor.position;
    while let Some(byte) = cursor.rest().first() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':') {
            cursor.position += 1;
        } else {
            break;
        }
    }
    if cursor.position == start {
        return Err(UpnpError::MalformedRequest {
            reason: "a name is expected",
        });
    }
    Ok(String::from_utf8_lossy(&cursor.input[start..cursor.position]).into_owned())
}

/// Consumes the given text.
pub(super) fn expect(cursor: &mut Cursor, text: &str) -> Result<(), UpnpError> {
    if cursor.starts_with(text) {
        cursor.skip(text.len());
        return Ok(());
    }
    Err(UpnpError::MalformedRequest {
        reason: "the document does not have the expected shape",
    })
}

/// Finds a byte sequence in a slice.
pub(super) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Decodes the entity references of a text.
///
/// Only the five predefined entities and the numeric character references are
/// understood. An unknown entity is an error, because a document that names
/// an entity it does not define is exactly the shape of an entity-expansion
/// attack (`SEC-010`).
pub(super) fn decode_entities(text: &str) -> Result<String, UpnpError> {
    if !text.contains('&') {
        return Ok(text.to_owned());
    }
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find('&') {
        decoded.push_str(&rest[..position]);
        rest = &rest[position + 1..];
        let end = rest.find(';').ok_or(UpnpError::UnknownEntity {
            entity: rest.chars().take(32).collect(),
        })?;
        let entity = &rest[..end];
        decoded.push_str(&decode_entity(entity)?);
        rest = &rest[end + 1..];
    }
    decoded.push_str(rest);
    Ok(decoded)
}

/// Decodes one entity reference.
fn decode_entity(entity: &str) -> Result<String, UpnpError> {
    match entity {
        "amp" => return Ok("&".to_owned()),
        "lt" => return Ok("<".to_owned()),
        "gt" => return Ok(">".to_owned()),
        "quot" => return Ok("\"".to_owned()),
        "apos" => return Ok("'".to_owned()),
        _ => {}
    }
    let code = if let Some(hexadecimal) = entity.strip_prefix("#x") {
        u32::from_str_radix(hexadecimal, 16).ok()
    } else if let Some(decimal) = entity.strip_prefix('#') {
        decimal.parse::<u32>().ok()
    } else {
        None
    };
    match code.and_then(char::from_u32) {
        Some(character) => Ok(character.to_string()),
        None => Err(UpnpError::UnknownEntity {
            entity: entity.chars().take(32).collect(),
        }),
    }
}

/// Splits a qualified name into its prefix and its local name.
pub(super) fn split_name(raw: &str) -> (&str, String) {
    match raw.split_once(':') {
        Some((prefix, local)) => (prefix, local.to_owned()),
        None => ("", raw.to_owned()),
    }
}

/// Whether an attribute is a namespace declaration.
pub(super) fn is_declaration(attribute: &str) -> bool {
    attribute == "xmlns" || attribute.starts_with("xmlns:")
}

/// The namespace declarations of an element.
pub(super) fn declarations(attributes: &[(String, String)]) -> Vec<(String, String)> {
    let mut declared = Vec::new();
    for (name, value) in attributes {
        if name == "xmlns" {
            declared.push((String::new(), value.clone()));
        } else if let Some(prefix) = name.strip_prefix("xmlns:") {
            declared.push((prefix.to_owned(), value.clone()));
        }
    }
    declared
}

/// Resolves the namespace of a prefix in the current scope.
///
/// The scope is a stack of the declarations of the enclosing elements, so the
/// innermost declaration of a prefix wins. An element without a prefix belongs
/// to the default namespace, and to no namespace at all when no default
/// namespace is declared.
pub(super) fn resolve(scope: &[(String, String)], prefix: &str) -> Result<String, UpnpError> {
    for (declared, uri) in scope.iter().rev() {
        if declared == prefix {
            return Ok(uri.clone());
        }
    }
    if prefix.is_empty() {
        return Ok(String::new());
    }
    Err(UpnpError::MalformedRequest {
        reason: "a namespace prefix is used but never declared",
    })
}
