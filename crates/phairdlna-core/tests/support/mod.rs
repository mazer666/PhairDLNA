//! A small XML checker for the protocol tests.
//!
//! What: a checker that verifies that a generated document is well-formed —
//!       that the elements nest, that every element is closed in order, and
//!       that no reserved character is left unescaped.
//! Why: the acceptance criteria of F-02 say "*when it is parsed*". A string
//!       match on the document cannot tell a well-formed document from a
//!       broken one, and the project has no XML parser as a dependency yet.
//!       The checker reads the documents the project itself produces, which
//!       escape every reserved character, so a small checker is enough.
//! Example: `assert_well_formed(&description.to_xml())` panics with the
//!       offending fragment if the document is malformed.

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
        if tag.starts_with('?') {
            continue;
        }
        if tag.starts_with('!') {
            panic!("the document carries a forbidden declaration: <!{tag}>");
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
        let end = match rest.find(';') {
            Some(end) => end,
            None => panic!("the text carries an unescaped ampersand: {text}"),
        };
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
