//! The one XML writer of the project.
//!
//! What: a small writer that builds a document element by element and escapes
//!       every value that goes into it.
//! Why: escaping has to happen in exactly one place, otherwise a value that
//!       reaches the XML unescaped can inject an element (`INT-027`). The same
//!       writer never emits a DOCTYPE, because a document type declaration is
//!       the entry point of an entity-expansion attack (`INT-024`).
//! Example:
//!
//! ```
//! use phairdlna_core::xml::XmlWriter;
//!
//! let mut writer = XmlWriter::new();
//! writer.declaration();
//! writer.open("root", &[("xmlns", "urn:schemas-upnp-org:device-1-0")]);
//! writer.text("friendlyName", "Living & room");
//! writer.close("root");
//!
//! let xml = writer.finish();
//! assert!(xml.contains("<friendlyName>Living &amp; room</friendlyName>"));
//! assert!(!xml.contains("<!DOCTYPE"));
//! ```

mod chars;

pub use chars::{escaped_character_len, escaped_text_len};

use chars::{is_allowed, replacement};

/// The XML declaration every document starts with.
pub const XML_DECLARATION: &str = r#"<?xml version="1.0" encoding="utf-8"?>"#;

/// Escapes a value for use as element text.
///
/// XML 1.0 requires `&`, `<` and `>` to be escaped in element text.
pub fn escape_text(value: &str) -> String {
    escape(value, false)
}

/// Escapes a value for use as an attribute value.
///
/// XML 1.0 requires `&`, `<`, `>`, `"` and `'` to be escaped in attribute
/// values.
pub fn escape_attribute(value: &str) -> String {
    escape(value, true)
}

/// Escapes a value, dropping the characters XML 1.0 does not allow.
fn escape(value: &str, in_attribute: bool) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match replacement(character, in_attribute) {
            Some(replacement) => escaped.push_str(replacement),
            None if is_allowed(character) => escaped.push(character),
            None => {}
        }
    }
    escaped
}

/// A writer that builds a document element by element.
///
/// The writer keeps the indentation of the document, so that the description
/// stays readable in a log or in the diagnostic command (`OPS-005`).
#[derive(Debug, Default)]
pub struct XmlWriter {
    output: String,
    open_elements: Vec<String>,
}

impl XmlWriter {
    /// Starts an empty document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes the XML declaration.
    pub fn declaration(&mut self) {
        self.output.push_str(XML_DECLARATION);
    }

    /// Opens an element with optional attributes.
    pub fn open(&mut self, name: &str, attributes: &[(&str, &str)]) {
        self.indent();
        self.write_tag(name, attributes);
        self.open_elements.push(name.to_owned());
    }

    /// Writes the opening tag of an element.
    fn write_tag(&mut self, name: &str, attributes: &[(&str, &str)]) {
        self.output.push('<');
        self.output.push_str(name);
        for (attribute, value) in attributes {
            self.output.push(' ');
            self.output.push_str(attribute);
            self.output.push_str("=\"");
            self.output.push_str(&escape_attribute(value));
            self.output.push('"');
        }
        self.output.push('>');
    }

    /// Closes the element that was opened last.
    ///
    /// A name that does not match the open element is a mistake in the code,
    /// not a mistake of the user, so it is caught by a debug assertion.
    pub fn close(&mut self, name: &str) {
        let open = self.open_elements.pop();
        debug_assert_eq!(
            open.as_deref(),
            Some(name),
            "the element `{name}` is closed, but `{open:?}` is open"
        );
        self.indent();
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
    }

    /// Writes an element that carries text.
    pub fn text(&mut self, name: &str, value: &str) {
        self.indent();
        self.output.push('<');
        self.output.push_str(name);
        self.output.push('>');
        self.output.push_str(&escape_text(value));
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
    }

    /// Writes an element that carries attributes and text.
    ///
    /// The element is written on one line, because its text is a value the
    /// client reads — the URI of a `res` element, for example — and a line
    /// break inside the element would become part of that value.
    pub fn element_with_text(&mut self, name: &str, attributes: &[(&str, &str)], value: &str) {
        self.indent();
        self.write_tag(name, attributes);
        self.output.push_str(&escape_text(value));
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
    }

    /// Writes an element that carries text that is already escaped.
    ///
    /// The value is written verbatim: the caller owns the escaping. The method
    /// exists for the one place where a document is carried as the text of
    /// another document (`INT-073`), where escaping it twice would corrupt it.
    ///
    /// The method is not public: a value that is written verbatim and was not
    /// escaped is an injection, and only the crate can hold that responsibility.
    pub(crate) fn escaped_text(&mut self, name: &str, value: &str) {
        self.indent();
        self.output.push('<');
        self.output.push_str(name);
        self.output.push('>');
        self.output.push_str(value);
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
    }

    /// Writes a comment.
    ///
    /// XML forbids the sequence `--` inside a comment, so it is replaced.
    pub fn comment(&mut self, text: &str) {
        self.indent();
        self.output.push_str("<!-- ");
        self.output.push_str(&text.replace("--", "- -"));
        self.output.push_str(" -->");
    }

    /// Finishes the document and returns it.
    pub fn finish(mut self) -> String {
        debug_assert!(
            self.open_elements.is_empty(),
            "the document ends with an open element: {:?}",
            self.open_elements
        );
        self.output.push('\n');
        self.output
    }

    /// Starts a new line at the current depth.
    fn indent(&mut self) {
        self.output.push('\n');
        for _ in 0..self.open_elements.len() {
            self.output.push_str("  ");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_attribute, escape_text, escaped_text_len, XmlWriter, XML_DECLARATION};

    /// The five characters XML 1.0 reserves are escaped in element text.
    #[test]
    fn element_text_is_escaped() {
        assert_eq!(escape_text("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    }

    /// The length of a value is the length of its escaped form.
    #[test]
    fn the_escaped_length_counts_the_escapes() {
        assert_eq!(escaped_text_len("abc"), 3);
        assert_eq!(escaped_text_len("a & b"), 9);
        assert_eq!(escaped_text_len("<"), 4);
        assert_eq!(escaped_text_len("ä"), 2);
    }

    /// The quotes are escaped in an attribute value as well.
    #[test]
    fn attribute_values_are_escaped() {
        assert_eq!(
            escape_attribute("say \"hi\" & 'bye'"),
            "say &quot;hi&quot; &amp; &apos;bye&apos;"
        );
    }

    /// The ampersand is escaped once, not twice.
    #[test]
    fn the_ampersand_is_escaped_once() {
        assert_eq!(escape_text("&amp;"), "&amp;amp;");
    }

    /// The control characters XML 1.0 forbids are dropped, so that the
    /// document stays well-formed.
    #[test]
    fn forbidden_control_characters_are_dropped() {
        assert_eq!(escape_text("a\u{0000}b\u{0008}c"), "abc");
        assert_eq!(escape_text("keep\tthe\ntabs"), "keep\tthe\ntabs");
    }

    /// The writer never emits a DOCTYPE, because the writer has no way to.
    #[test]
    fn the_writer_never_emits_a_doctype() {
        let mut writer = XmlWriter::new();
        writer.declaration();
        writer.open("root", &[]);
        writer.text("name", "value");
        writer.close("root");
        let xml = writer.finish();
        assert!(!xml.contains("<!DOCTYPE"));
        assert!(!xml.to_lowercase().contains("doctype"));
        assert!(xml.starts_with(XML_DECLARATION));
    }

    /// An attribute value cannot break out of its attribute.
    #[test]
    fn an_attribute_value_cannot_escape_its_attribute() {
        let mut writer = XmlWriter::new();
        writer.open("root", &[("name", "\"><injected/>")]);
        writer.close("root");
        let xml = writer.finish();
        assert!(!xml.contains("<injected/>"));
        assert!(xml.contains("&quot;&gt;&lt;injected/&gt;"));
    }

    /// An element with attributes and text is written on one line, so that
    /// the text is the value and not the value and a line break.
    #[test]
    fn an_element_with_text_is_written_on_one_line() {
        let mut writer = XmlWriter::new();
        writer.element_with_text(
            "res",
            &[("protocolInfo", "http-get:*:audio/mpeg:*")],
            "http://host/a",
        );
        let xml = writer.finish();
        assert!(xml.contains("<res protocolInfo=\"http-get:*:audio/mpeg:*\">http://host/a</res>"));
    }

    /// The text of an element with text is escaped.
    #[test]
    fn the_text_of_an_element_with_text_is_escaped() {
        let mut writer = XmlWriter::new();
        writer.element_with_text("res", &[], "a & b");
        let xml = writer.finish();
        assert!(xml.contains("<res>a &amp; b</res>"));
    }

    /// Already escaped text is written as it is, and only once.
    #[test]
    fn already_escaped_text_is_written_verbatim() {
        let mut writer = XmlWriter::new();
        writer.escaped_text("Result", "&lt;DIDL-Lite&gt;");
        let xml = writer.finish();
        assert!(xml.contains("<Result>&lt;DIDL-Lite&gt;</Result>"));
        assert!(!xml.contains("&amp;lt;"));
    }
}
