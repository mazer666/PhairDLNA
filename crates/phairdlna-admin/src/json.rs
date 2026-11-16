//! The JSON the administration surface answers with.
//!
//! What: a very small writer for the JSON the surface produces.
//! Why: the answers of the surface are built from a handful of known shapes.
//!      A dependency that brings a parser and a data format into the build
//!      would be more than the surface needs, and the surface is the part of
//!      the server that a browser talks to.
//! Example: `Json::map(vec![("uptime".to_owned(), Json::uint(42))])` writes
//!          `{"uptime":42}`.
//!
//! **The rules.** A text is escaped, so that a value that carries a quote
//! cannot break the answer. A number is written as a number. The order of the
//! fields is the order they were given in, so that two answers of the same
//! state look the same.

use std::fmt;

/// A value of the answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// Nothing.
    Null,
    /// A yes or a no.
    Bool(bool),
    /// A number that may be negative.
    Int(i64),
    /// A number that is not negative.
    Uint(u64),
    /// A text.
    Text(String),
    /// A list of values.
    List(Vec<Json>),
    /// A set of fields, in the order they were given.
    Map(Vec<(String, Json)>),
}

impl Json {
    /// A text value.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// A number that is not negative.
    pub fn uint(value: u64) -> Self {
        Self::Uint(value)
    }

    /// A number that may be negative.
    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }

    /// A yes or a no.
    pub fn flag(value: bool) -> Self {
        Self::Bool(value)
    }

    /// A list of values.
    pub fn list(values: Vec<Json>) -> Self {
        Self::List(values)
    }

    /// A set of fields, in the order they were given.
    pub fn map(entries: Vec<(String, Json)>) -> Self {
        Self::Map(entries)
    }

    /// A number that may be missing.
    ///
    /// A number the platform does not know is written as `null`: the answer
    /// says that the figure is not known, and not that it is zero.
    pub fn maybe_uint(value: Option<u64>) -> Self {
        match value {
            Some(value) => Self::Uint(value),
            None => Self::Null,
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("null"),
            Self::Bool(true) => formatter.write_str("true"),
            Self::Bool(false) => formatter.write_str("false"),
            Self::Int(value) => write!(formatter, "{value}"),
            Self::Uint(value) => write!(formatter, "{value}"),
            Self::Text(value) => write_text(formatter, value),
            Self::List(values) => write_list(formatter, values),
            Self::Map(entries) => write_map(formatter, entries),
        }
    }
}

/// Writes a list of values.
fn write_list(formatter: &mut fmt::Formatter<'_>, values: &[Json]) -> fmt::Result {
    formatter.write_str("[")?;
    for (index, value) in values.iter().enumerate() {
        write_separator(formatter, index)?;
        write!(formatter, "{value}")?;
    }
    formatter.write_str("]")
}

/// Writes a set of fields.
fn write_map(formatter: &mut fmt::Formatter<'_>, entries: &[(String, Json)]) -> fmt::Result {
    formatter.write_str("{")?;
    for (index, (name, value)) in entries.iter().enumerate() {
        write_separator(formatter, index)?;
        write_text(formatter, name)?;
        formatter.write_str(":")?;
        write!(formatter, "{value}")?;
    }
    formatter.write_str("}")
}

/// Writes the comma between two values.
fn write_separator(formatter: &mut fmt::Formatter<'_>, index: usize) -> fmt::Result {
    if index > 0 {
        formatter.write_str(",")
    } else {
        Ok(())
    }
}

/// Writes a text as a JSON string, with the quotes and the escapes.
fn write_text(formatter: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    formatter.write_str("\"")?;
    for character in value.chars() {
        match character {
            '"' => formatter.write_str("\\\"")?,
            '\\' => formatter.write_str("\\\\")?,
            '\n' => formatter.write_str("\\n")?,
            '\r' => formatter.write_str("\\r")?,
            '\t' => formatter.write_str("\\t")?,
            character if character < ' ' => write!(formatter, "\\u{:04x}", character as u32)?,
            character => formatter.write_fmt(format_args!("{character}"))?,
        }
    }
    formatter.write_str("\"")
}

#[cfg(test)]
mod tests {
    use super::Json;

    /// The fields of an object keep their order.
    #[test]
    fn the_fields_keep_their_order() {
        let value = Json::map(vec![
            ("b".to_owned(), Json::uint(1)),
            ("a".to_owned(), Json::uint(2)),
        ]);

        assert_eq!(value.to_string(), r#"{"b":1,"a":2}"#);
    }

    /// A text with a quote in it cannot break the answer.
    #[test]
    fn a_quote_in_a_text_is_escaped() {
        let value = Json::text("a \"quote\" and a \\ backslash");

        assert_eq!(value.to_string(), r#""a \"quote\" and a \\ backslash""#);
    }

    /// A line break in a text becomes an escape.
    #[test]
    fn a_line_break_in_a_text_is_escaped() {
        assert_eq!(Json::text("a\nb").to_string(), r#""a\nb""#);
    }

    /// A number that is not known is written as `null`.
    #[test]
    fn a_missing_number_is_null() {
        assert_eq!(Json::maybe_uint(None).to_string(), "null");
        assert_eq!(Json::maybe_uint(Some(3)).to_string(), "3");
    }

    /// A list of values is written in brackets.
    #[test]
    fn a_list_is_written_in_brackets() {
        let value = Json::list(vec![Json::uint(1), Json::flag(true)]);

        assert_eq!(value.to_string(), "[1,true]");
    }
}
