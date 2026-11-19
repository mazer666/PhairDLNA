//! XML entity and namespace policy layered over `quick-xml` events.
//!
//! The upstream parser recognizes syntax; this module enforces which entity
//! references and namespace prefixes PhairDLNA accepts (`SEC-010`).

use crate::upnp::error::UpnpError;

/// Decodes only the five predefined entities and numeric references.
pub(super) fn decode_entities(text: &str) -> Result<String, UpnpError> {
    if !text.contains('&') {
        return Ok(text.to_owned());
    }
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find('&') {
        decoded.push_str(&rest[..position]);
        rest = &rest[position + 1..];
        let end = rest.find(';').ok_or_else(|| unknown_entity(rest))?;
        decoded.push_str(&decode_entity(&rest[..end])?);
        rest = &rest[end + 1..];
    }
    decoded.push_str(rest);
    Ok(decoded)
}

/// Decodes one predefined or numeric reference; custom entities are refused.
pub(super) fn decode_entity(entity: &str) -> Result<String, UpnpError> {
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
    match code
        .filter(|value| is_xml_character(*value))
        .and_then(char::from_u32)
    {
        Some(character) => Ok(character.to_string()),
        None => Err(unknown_entity(entity)),
    }
}

fn is_xml_character(value: u32) -> bool {
    matches!(value, 0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}

fn unknown_entity(entity: &str) -> UpnpError {
    UpnpError::UnknownEntity {
        entity: entity.chars().take(32).collect(),
    }
}

/// Splits a qualified element name into its prefix and local name.
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
    attributes
        .iter()
        .filter_map(|(name, value)| {
            if name == "xmlns" {
                Some((String::new(), value.clone()))
            } else {
                name.strip_prefix("xmlns:")
                    .map(|prefix| (prefix.to_owned(), value.clone()))
            }
        })
        .collect()
}

/// Resolves a prefix in the stack of declarations of enclosing elements.
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
