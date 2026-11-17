//! The reader of the JSON documents.
//!
//! What: a small reader of JSON documents and the reasons it refuses one.
//! Why: the command line and the menu bar application are clients of the
//!      administration surface and must read what the surface answers. A
//!      reader of its own keeps the project free of a serialization
//!      framework, and a reader with bounds keeps a broken or hostile answer
//!      from taking the process down (`SEC-063`). The reader lives in the core
//!      so that the two clients share one implementation and cannot disagree
//!      about what an answer means.
//! Example: the surface answers `"active_streams": 2`, and the client reads
//!          the number with `value.as_u64()`.
//!
//! **The numbers keep their text.** A number is kept as it was written and
//! not as a floating-point number, because the numbers of the surface are
//! counters and byte counts, and a counter of more than fifty-three bits must
//! not lose a digit.
//!
//! **The reader is bounded.** A document of more than a megabyte is refused,
//! and a document that nests deeper than thirty-two levels is refused: a
//! document that nests deeply would otherwise fill the stack of the reader.

use std::collections::BTreeMap;
use std::fmt;

mod reader;

use reader::Parser;

/// The largest document the reader accepts, in bytes.
const MAX_BYTES: usize = 1024 * 1024;

/// The deepest nesting the reader accepts.
const MAX_DEPTH: usize = 32;

/// A value of a JSON document.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    /// The value `null`.
    Null,
    /// The value `true` or `false`.
    Bool(bool),
    /// A number, kept as it was written.
    Number(String),
    /// A text.
    Text(String),
    /// A list of values.
    List(Vec<Json>),
    /// An object, by key.
    Map(BTreeMap<String, Json>),
}

impl Json {
    /// A text value.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// A number value.
    pub fn number(value: impl Into<String>) -> Self {
        Self::Number(value.into())
    }

    /// A list of values.
    pub fn list(values: Vec<Json>) -> Self {
        Self::List(values)
    }

    /// An object of values, by key.
    ///
    /// The keys are sorted, so that the same facts always have the same
    /// document.
    pub fn map(entries: impl IntoIterator<Item = (String, Json)>) -> Self {
        Self::Map(entries.into_iter().collect())
    }

    /// A number that may be missing.
    ///
    /// A number the platform does not know is written as `null`: the answer
    /// says that the figure is not known, and not that it is zero.
    pub fn maybe_number(value: Option<u64>) -> Self {
        match value {
            Some(value) => Self::Number(value.to_string()),
            None => Self::Null,
        }
    }

    /// Reads a document.
    ///
    /// The answer is an error when the text is not a document, when it holds
    /// more than a megabyte or when it nests too deeply.
    pub fn parse(text: &str) -> Result<Json, JsonError> {
        if text.len() > MAX_BYTES {
            return Err(JsonError::TooLong { length: text.len() });
        }
        Parser::new(text.as_bytes()).parse_document()
    }

    /// The value of a key of an object.
    ///
    /// The answer is `None` when the value is not an object or when the object
    /// does not carry the key: a missing value is a missing value and not a
    /// crash.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Map(entries) => entries.get(key),
            _ => None,
        }
    }

    /// The value as a text.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The value as a flag.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(flag) => Some(*flag),
            _ => None,
        }
    }

    /// The value as a whole number.
    ///
    /// A number with a fraction or an exponent is not a whole number and is
    /// refused; the counters of the surface are whole numbers.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Number(raw) => raw.parse::<u64>().ok(),
            _ => None,
        }
    }

    /// The value as a list.
    pub fn as_list(&self) -> Option<&[Json]> {
        match self {
            Json::List(values) => Some(values),
            _ => None,
        }
    }

    /// The value as an object.
    pub fn as_map(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Map(entries) => Some(entries),
            _ => None,
        }
    }

    /// Whether the value is the empty one.
    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }
}

/// Why a document was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonError {
    /// The document is larger than the reader accepts.
    TooLong {
        /// The size of the document, in bytes.
        length: usize,
    },
    /// The document ends in the middle of a value.
    UnexpectedEnd,
    /// The document holds something that is not JSON.
    BadSyntax {
        /// Where the reader stopped.
        position: usize,
    },
    /// The document holds a number that cannot be read.
    BadNumber {
        /// Where the reader stopped.
        position: usize,
    },
    /// The document holds a text that cannot be read.
    BadString {
        /// Where the reader stopped.
        position: usize,
    },
    /// The document nests deeper than the reader accepts.
    TooDeep {
        /// The depth the reader reached.
        depth: usize,
    },
    /// The document holds something after its last value.
    Trailing {
        /// Where the reader stopped.
        position: usize,
    },
}

/// The document of a value, written as JSON.
///
/// The writer is the counterpart of the reader: the same value that the reader
/// hands to the code is written back as the document it came from. A text is
/// escaped, so that a value that carries a quote cannot break the document.
impl fmt::Display for Json {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("null"),
            Self::Bool(true) => formatter.write_str("true"),
            Self::Bool(false) => formatter.write_str("false"),
            Self::Number(raw) => formatter.write_str(raw),
            Self::Text(text) => write_text(formatter, text),
            Self::List(values) => write_list(formatter, values),
            Self::Map(entries) => write_map(formatter, entries),
        }
    }
}

/// Writes a list of JSON values.
fn write_list(formatter: &mut fmt::Formatter<'_>, values: &[Json]) -> fmt::Result {
    formatter.write_str("[")?;
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatter.write_str(",")?;
        }
        write!(formatter, "{value}")?;
    }
    formatter.write_str("]")
}

/// Writes a map of JSON key-value pairs.
fn write_map(
    formatter: &mut fmt::Formatter<'_>,
    entries: &std::collections::BTreeMap<String, Json>,
) -> fmt::Result {
    formatter.write_str("{")?;
    for (index, (name, value)) in entries.iter().enumerate() {
        if index > 0 {
            formatter.write_str(",")?;
        }
        write_text(formatter, name)?;
        formatter.write_str(":")?;
        write!(formatter, "{value}")?;
    }
    formatter.write_str("}")
}

/// Writes a text as a value of a JSON document.
fn write_text(formatter: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    formatter.write_str("\"")?;
    for character in text.chars() {
        match character {
            '"' => formatter.write_str("\\\"")?,
            '\\' => formatter.write_str("\\\\")?,
            '\n' => formatter.write_str("\\n")?,
            '\r' => formatter.write_str("\\r")?,
            '\t' => formatter.write_str("\\t")?,
            other => write!(formatter, "{other}")?,
        }
    }
    formatter.write_str("\"")
}

impl fmt::Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { length } => {
                write!(formatter, "the document of {length} bytes is too large")
            }
            Self::UnexpectedEnd => {
                formatter.write_str("the document ends in the middle of a value")
            }
            Self::BadSyntax { position } => {
                write!(
                    formatter,
                    "the document is not valid json at byte {position}"
                )
            }
            Self::BadNumber { position } => {
                write!(formatter, "the number at byte {position} cannot be read")
            }
            Self::BadString { position } => {
                write!(formatter, "the text at byte {position} cannot be read")
            }
            Self::TooDeep { depth } => {
                write!(formatter, "the document nests deeper than {depth} levels")
            }
            Self::Trailing { position } => {
                write!(
                    formatter,
                    "the document holds something after byte {position}"
                )
            }
        }
    }
}

impl std::error::Error for JsonError {}

/// The number of bytes the character that starts with a byte has.
///
/// A byte that starts a character of the basic plane is followed by no other
/// byte; a byte that starts a character outside it is followed by two.
pub(super) fn utf8_length(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::Json;

    /// A document with an object, a list and the three kinds of value.
    #[test]
    fn a_document_is_read() {
        let value =
            Json::parse(r#"{"a": 1, "b": [true, null], "c": "x"}"#).expect("the document is read");
        assert_eq!(value.get("a").and_then(Json::as_u64), Some(1));
        assert_eq!(value.get("c").and_then(Json::as_str), Some("x"));
        assert_eq!(
            value.get("b").and_then(Json::as_list).map(<[Json]>::len),
            Some(2)
        );
    }

    /// The negative case: a document that is not a document is refused.
    #[test]
    fn a_broken_document_is_refused() {
        for text in ["", "{", "{\"a\" 1}", "[1,]", "tru", "{} {}"] {
            assert!(
                Json::parse(text).is_err(),
                "the document {text:?} is refused"
            );
        }
    }

    /// A document that nests too deeply is refused.
    #[test]
    fn a_deep_document_is_refused() {
        let deep = "[".repeat(64) + &"]".repeat(64);
        assert!(Json::parse(&deep).is_err(), "the deep document is refused");
    }

    /// A key that the document does not carry is nothing and not a crash.
    #[test]
    fn a_missing_key_is_nothing() {
        let value = Json::parse("{}").expect("the document is read");
        assert!(value.get("a").is_none());
        assert!(value.as_str().is_none());
        assert!(value.as_u64().is_none());
    }
}
