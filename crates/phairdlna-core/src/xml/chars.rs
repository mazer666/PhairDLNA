//! The characters XML reserves.
//!
//! What: the table of the characters XML 1.0 reserves, the characters it
//!       allows, and the length a character has in its escaped form.
//! Why: escaping has to happen in exactly one place, and the length of an
//!       escaped value has to agree with it — otherwise the length policy of
//!       `INT-072` would count bytes the client never receives. The table is
//!       therefore written once and read by both.
//! Example: `replacement('&')` returns `&amp;`.

/// The replacement of a character that XML reserves.
///
/// The table is the one place that knows what XML escapes; the escaping and
/// the length of an escaped value both read it, so the two cannot disagree.
pub(crate) fn replacement(character: char, in_attribute: bool) -> Option<&'static str> {
    match character {
        '&' => Some("&amp;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' if in_attribute => Some("&quot;"),
        '\'' if in_attribute => Some("&apos;"),
        _ => None,
    }
}

/// Whether a character may appear in an XML 1.0 document.
///
/// The ranges are the ones XML 1.0 names: tab, line feed and carriage return,
/// then the printable ranges. The control characters between them are
/// forbidden, and the two noncharacters `U+FFFE` and `U+FFFF` are forbidden
/// as well. A user-chosen device name may contain such a character, and a
/// document that carries one is not well-formed, so the writer drops it
/// instead of emitting broken XML.
pub(crate) fn is_allowed(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\r')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

/// The length of one character in its XML-escaped form.
///
/// The length is counted in the text context; an attribute value escapes the
/// two quote characters as well.
pub fn escaped_character_len(character: char) -> usize {
    match replacement(character, false) {
        Some(replacement) => replacement.len(),
        None if is_allowed(character) => character.len_utf8(),
        None => 0,
    }
}

/// The length of a value in its XML-escaped form.
///
/// The length is what a client has to read, so the length policy of `INT-072`
/// counts these bytes and not the bytes of the raw value: a title of five
/// ampersands is twenty-five bytes once escaped.
pub fn escaped_text_len(value: &str) -> usize {
    value.chars().map(escaped_character_len).sum()
}

#[cfg(test)]
mod tests {
    use super::{escaped_character_len, escaped_text_len, is_allowed, replacement};

    /// The reserved characters have their replacements.
    #[test]
    fn the_reserved_characters_have_replacements() {
        assert_eq!(replacement('&', false), Some("&amp;"));
        assert_eq!(replacement('<', false), Some("&lt;"));
        assert_eq!(replacement('>', false), Some("&gt;"));
        assert_eq!(replacement('"', true), Some("&quot;"));
        assert_eq!(replacement('\'', true), Some("&apos;"));
        assert_eq!(replacement('"', false), None);
        assert_eq!(replacement('a', true), None);
    }

    /// The forbidden characters are recognised.
    #[test]
    fn the_forbidden_characters_are_recognised() {
        assert!(!is_allowed('\u{0000}'));
        assert!(!is_allowed('\u{0008}'));
        assert!(!is_allowed('\u{FFFE}'));
        assert!(!is_allowed('\u{FFFF}'));
        assert!(is_allowed('\t'));
        assert!(is_allowed('a'));
        assert!(is_allowed('\u{0080}'));
    }

    /// The length of a value is the length of its escaped form.
    #[test]
    fn the_escaped_length_counts_the_escapes() {
        assert_eq!(escaped_text_len("abc"), 3);
        assert_eq!(escaped_text_len("a & b"), 9);
    }

    /// The length of a character is the length of its replacement.
    #[test]
    fn the_length_is_the_length_of_the_replacement() {
        assert_eq!(escaped_character_len('&'), 5);
        assert_eq!(escaped_character_len('<'), 4);
        assert_eq!(escaped_character_len('a'), 1);
        assert_eq!(escaped_character_len('ä'), 2);
        assert_eq!(escaped_character_len('\u{0000}'), 0);
    }
}
