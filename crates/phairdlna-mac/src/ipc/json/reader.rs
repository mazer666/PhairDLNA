//! The reader of a document.
//!
//! What: the parser that walks the bytes of a JSON document.
//! Why: the reader of the answers has a part that knows the grammar of JSON
//!      and a part that knows the values of the surface; the two are apart so
//!      that each of them stays small (`SEC-063`).
//! Example: the reader walks `{"a": 1}` and hands the values to the reader of
//!          the answers.

use std::collections::BTreeMap;

use super::{utf8_length, Json, JsonError, MAX_DEPTH};

/// The reader of a document.
pub(super) struct Parser<'a> {
    /// The bytes of the document.
    bytes: &'a [u8],
    /// Where the reader stands.
    position: usize,
}

impl<'a> Parser<'a> {
    /// Creates the reader of a document.
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    /// Reads the whole document.
    pub(super) fn parse_document(&mut self) -> Result<Json, JsonError> {
        let value = self.parse_value(0)?;
        self.skip_whitespace();
        if self.position != self.bytes.len() {
            return Err(JsonError::Trailing {
                position: self.position,
            });
        }
        Ok(value)
    }

    /// Reads one value.
    fn parse_value(&mut self, depth: usize) -> Result<Json, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::TooDeep { depth });
        }
        self.skip_whitespace();
        let Some(byte) = self.peek() else {
            return Err(JsonError::UnexpectedEnd);
        };
        match byte {
            b'{' => self.parse_object(depth),
            b'[' => self.parse_array(depth),
            b'"' => Ok(Json::Text(self.parse_string()?)),
            b't' => self.parse_literal("true", Json::Bool(true)),
            b'f' => self.parse_literal("false", Json::Bool(false)),
            b'n' => self.parse_literal("null", Json::Null),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(JsonError::BadSyntax {
                position: self.position,
            }),
        }
    }

    /// Reads an object.
    fn parse_object(&mut self, depth: usize) -> Result<Json, JsonError> {
        self.position += 1;
        let mut entries = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Map(entries));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(JsonError::BadSyntax {
                    position: self.position,
                });
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(JsonError::BadSyntax {
                    position: self.position,
                });
            }
            self.position += 1;
            let value = self.parse_value(depth + 1)?;
            entries.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Map(entries));
                }
                _ => {
                    return Err(JsonError::BadSyntax {
                        position: self.position,
                    })
                }
            }
        }
    }

    /// Reads a list.
    fn parse_array(&mut self, depth: usize) -> Result<Json, JsonError> {
        self.position += 1;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Json::List(values));
        }
        loop {
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::List(values));
                }
                _ => {
                    return Err(JsonError::BadSyntax {
                        position: self.position,
                    })
                }
            }
        }
    }

    /// Reads a text.
    fn parse_string(&mut self) -> Result<String, JsonError> {
        self.position += 1;
        let mut text = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.bad_string());
            };
            self.position += 1;
            match byte {
                b'"' => return Ok(text),
                b'\\' => self.parse_escape(&mut text)?,
                control if control < 0x20 => return Err(self.bad_string()),
                _ => text.push_str(self.take_character()?),
            }
        }
    }

    /// The character that begins with the byte the reader stands in front of.
    ///
    /// A character of the language of the user may be several bytes long; the
    /// reader collects them and checks that they are a character.
    fn take_character(&mut self) -> Result<&'a str, JsonError> {
        let start = self.position - 1;
        let end = start + utf8_length(self.bytes[start]);
        if end > self.bytes.len() {
            return Err(self.bad_string());
        }
        match std::str::from_utf8(&self.bytes[start..end]) {
            Ok(text) => {
                self.position = end;
                Ok(text)
            }
            Err(_) => Err(self.bad_string()),
        }
    }

    /// The reason why a text cannot be read.
    fn bad_string(&self) -> JsonError {
        JsonError::BadString {
            position: self.position,
        }
    }

    /// Reads the escape of a text.
    fn parse_escape(&mut self, text: &mut String) -> Result<(), JsonError> {
        let Some(byte) = self.peek() else {
            return Err(JsonError::BadString {
                position: self.position,
            });
        };
        self.position += 1;
        match byte {
            b'"' => text.push('"'),
            b'\\' => text.push('\\'),
            b'/' => text.push('/'),
            b'b' => text.push('\u{8}'),
            b'f' => text.push('\u{c}'),
            b'n' => text.push('\n'),
            b'r' => text.push('\r'),
            b't' => text.push('\t'),
            b'u' => self.parse_escape_of_unicode(text)?,
            _ => {
                return Err(JsonError::BadString {
                    position: self.position,
                })
            }
        }
        Ok(())
    }

    /// Reads the escape of a character of sixteen bits.
    fn parse_escape_of_unicode(&mut self, text: &mut String) -> Result<(), JsonError> {
        let first = self.read_hex_quad()?;
        let code = if (0xd800..0xdc00).contains(&first) {
            // A character outside the basic plane stands for two escapes; the
            // second one must follow at once.
            if self.peek() != Some(b'\\') {
                return Err(JsonError::BadString {
                    position: self.position,
                });
            }
            self.position += 1;
            if self.peek() != Some(b'u') {
                return Err(JsonError::BadString {
                    position: self.position,
                });
            }
            self.position += 1;
            let second = self.read_hex_quad()?;
            if !(0xdc00..0xe000).contains(&second) {
                return Err(JsonError::BadString {
                    position: self.position,
                });
            }
            0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else {
            u32::from(first)
        };
        match char::from_u32(code) {
            Some(character) => {
                text.push(character);
                Ok(())
            }
            None => Err(JsonError::BadString {
                position: self.position,
            }),
        }
    }

    /// Reads the four hexadecimal digits of an escape.
    fn read_hex_quad(&mut self) -> Result<u16, JsonError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let Some(byte) = self.peek() else {
                return Err(JsonError::BadString {
                    position: self.position,
                });
            };
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a') + 10,
                b'A'..=b'F' => u16::from(byte - b'A') + 10,
                _ => {
                    return Err(JsonError::BadString {
                        position: self.position,
                    })
                }
            };
            value = value * 16 + digit;
            self.position += 1;
        }
        Ok(value)
    }

    /// Reads a number and keeps its text.
    fn parse_number(&mut self) -> Result<Json, JsonError> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        self.read_digits()?;
        if self.peek() == Some(b'.') {
            self.position += 1;
            self.read_digits()?;
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.position += 1;
            }
            self.read_digits()?;
        }
        let text = String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned();
        Ok(Json::Number(text))
    }

    /// Reads the digits of a number.
    fn read_digits(&mut self) -> Result<(), JsonError> {
        let start = self.position;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        if self.position == start {
            return Err(JsonError::BadNumber {
                position: self.position,
            });
        }
        Ok(())
    }

    /// Reads one of the words of the language.
    fn parse_literal(&mut self, word: &str, value: Json) -> Result<Json, JsonError> {
        if self.bytes[self.position..].starts_with(word.as_bytes()) {
            self.position += word.len();
            return Ok(value);
        }
        Err(JsonError::BadSyntax {
            position: self.position,
        })
    }

    /// The byte the reader stands in front of.
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    /// Skips the space of the document.
    fn skip_whitespace(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
        ) {
            self.position += 1;
        }
    }
}
