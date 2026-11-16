//! The reader of a `launchd` job file.
//!
//! What: a small reader that turns a property list back into the values it
//!       carries.
//! Why: the command must be able to check what it wrote — that the file it
//!      installed is the file it meant to install — and a check that cannot
//!      read the file cannot check anything (`AC-11`).
//! Example: `parse` of a job file gives the label, the program and the logs
//!      of the job.
//!
//! The reader knows the shape of a property list and nothing else: it reads
//! strings, the two flags, arrays of strings and dictionaries of strings. A
//! property list with a value it does not know is refused rather than
//! half-read.

use std::collections::BTreeMap;

/// One value of a property list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlistValue {
    /// A text.
    Text(String),
    /// A flag that is on or off.
    Flag(bool),
    /// A list of texts.
    List(Vec<String>),
    /// A dictionary of texts.
    Dictionary(BTreeMap<String, String>),
}

/// A property list: the entries of its dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlistDocument {
    /// The entries, by key.
    pub entries: BTreeMap<String, PlistValue>,
}

impl PlistDocument {
    /// The text of an entry.
    pub fn text(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(PlistValue::Text(value)) => Some(value),
            _ => None,
        }
    }

    /// The texts of an entry.
    pub fn list(&self, key: &str) -> Option<&[String]> {
        match self.entries.get(key) {
            Some(PlistValue::List(values)) => Some(values),
            _ => None,
        }
    }

    /// The dictionary of an entry.
    pub fn dictionary(&self, key: &str) -> Option<&BTreeMap<String, String>> {
        match self.entries.get(key) {
            Some(PlistValue::Dictionary(values)) => Some(values),
            _ => None,
        }
    }

    /// Whether an entry is a flag that is on.
    pub fn flag(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(PlistValue::Flag(value)) => Some(*value),
            _ => None,
        }
    }
}

/// One tag of a property list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tag {
    /// The name of the tag.
    name: String,
    /// Whether the tag closes an element.
    closing: bool,
    /// Whether the tag closes itself.
    self_closing: bool,
}

/// Reads a property list.
///
/// The function refuses a file it cannot understand: a broken job file that is
/// installed is a job that never starts, and the user would find out much
/// later.
pub fn parse(text: &str) -> Result<PlistDocument, String> {
    let mut scanner = Scanner::new(text);
    let mut document = PlistDocument {
        entries: BTreeMap::new(),
    };
    while let Some(tag) = scanner.next_tag() {
        if tag.name == "dict" && !tag.closing && !tag.self_closing {
            document.entries = read_dictionary(&mut scanner)?;
            break;
        }
    }
    if document.entries.is_empty() {
        return Err("the property list has no dictionary".to_owned());
    }
    Ok(document)
}

/// Reads the entries of a dictionary.
fn read_dictionary(scanner: &mut Scanner<'_>) -> Result<BTreeMap<String, PlistValue>, String> {
    let mut entries = BTreeMap::new();
    loop {
        let Some(tag) = scanner.next_tag() else {
            return Err("the dictionary of the property list is not closed".to_owned());
        };
        if tag.name == "dict" && tag.closing {
            return Ok(entries);
        }
        if tag.name != "key" || tag.closing {
            continue;
        }
        let key = unescape(scanner.text());
        close_the_key(scanner, &key)?;
        let Some(value_tag) = scanner.next_tag() else {
            return Err(format!("the key \"{key}\" has no value"));
        };
        let value = read_value(scanner, &value_tag)?;
        entries.insert(key, value);
    }
}

/// Reads the tag that closes a key.
///
/// A key is closed by a tag of its own: `<key>Label</key>`. The tag that
/// closes it says nothing about the value, and a reader that took it for the
/// value would read the wrong entry (`F-13` § 11).
fn close_the_key(scanner: &mut Scanner<'_>, key: &str) -> Result<(), String> {
    let Some(tag) = scanner.next_tag() else {
        return Err(format!("the key \"{key}\" is not closed"));
    };
    if tag.closing && tag.name == "key" {
        return Ok(());
    }
    Err(format!("the key \"{key}\" is not closed"))
}

/// Reads the value of an entry.
fn read_value(scanner: &mut Scanner<'_>, tag: &Tag) -> Result<PlistValue, String> {
    match tag.name.as_str() {
        "string" => Ok(PlistValue::Text(unescape(scanner.text()))),
        "true" => Ok(PlistValue::Flag(true)),
        "false" => Ok(PlistValue::Flag(false)),
        "array" => Ok(PlistValue::List(read_array(scanner)?)),
        "dict" => {
            let entries = read_dictionary(scanner)?;
            let mut texts = BTreeMap::new();
            for (name, value) in entries {
                texts.insert(name, text_of(&value));
            }
            Ok(PlistValue::Dictionary(texts))
        }
        other => Err(format!("the value \"{other}\" is not known")),
    }
}

/// Reads a text with the escapes of the property list undone.
///
/// A property list escapes the characters that XML does not allow: a reader
/// that does not undo the escapes compares a path with a path that is not the
/// same, and reports a difference that is not there (`F-13` § 11).
pub fn unescape(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        text.push_str(&rest[..start]);
        let tail = &rest[start + 1..];
        let Some(end) = tail.find(';') else {
            text.push('&');
            text.push_str(tail);
            return text;
        };
        let entity = &tail[..end];
        match entity {
            "amp" => text.push('&'),
            "lt" => text.push('<'),
            "gt" => text.push('>'),
            "quot" => text.push('"'),
            "apos" => text.push('\''),
            other => {
                text.push('&');
                text.push_str(other);
                text.push(';');
            }
        }
        rest = &tail[end + 1..];
    }
    text.push_str(rest);
    text
}

/// The text of a value, whatever it is.
fn text_of(value: &PlistValue) -> String {
    match value {
        PlistValue::Text(text) => text.clone(),
        PlistValue::Flag(flag) => flag.to_string(),
        PlistValue::List(values) => values.join(" "),
        PlistValue::Dictionary(values) => values
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<String>>()
            .join(" "),
    }
}

/// Reads the items of an array.
fn read_array(scanner: &mut Scanner<'_>) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    loop {
        let Some(tag) = scanner.next_tag() else {
            return Err("the array of the property list is not closed".to_owned());
        };
        if tag.name == "array" && tag.closing {
            return Ok(values);
        }
        if tag.name == "string" && !tag.closing {
            values.push(unescape(scanner.text()));
        }
    }
}

/// The reader of the tags of a property list.
struct Scanner<'a> {
    /// The text that is read.
    text: &'a str,
    /// The place the reader is at.
    position: usize,
}

impl<'a> Scanner<'a> {
    /// Builds a reader for a text.
    fn new(text: &'a str) -> Self {
        Self { text, position: 0 }
    }

    /// The next tag of the text.
    ///
    /// A declaration (`<?xml … ?>`) and a doctype (`<!DOCTYPE …>`) are skipped:
    /// they say nothing about the entries of the list.
    fn next_tag(&mut self) -> Option<Tag> {
        while let Some(tag) = self.read_tag() {
            if tag.name.starts_with('?') || tag.name.starts_with('!') {
                continue;
            }
            return Some(tag);
        }
        None
    }

    /// Reads the next tag, whatever it is.
    fn read_tag(&mut self) -> Option<Tag> {
        let start = self.text[self.position..].find('<')? + self.position;
        let end = self.text[start..].find('>')? + start;
        let inner = self.text[start + 1..end].trim();
        self.position = end + 1;
        let closing = inner.starts_with('/');
        let self_closing = inner.ends_with('/');
        let name = inner
            .trim_start_matches('/')
            .trim_end_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_owned();
        Some(Tag {
            name,
            closing,
            self_closing,
        })
    }

    /// The text between the last tag and the next one.
    fn text(&mut self) -> &'a str {
        let start = self.position;
        let end = self.text[start..].find('<').map(|end| start + end);
        self.position = end.unwrap_or(self.text.len());
        &self.text[start..self.position]
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, PlistValue};
    use crate::service::layout::{LayoutRequest, Platform, Scope, ServiceLayout};
    use crate::service::plist::render;
    use std::path::PathBuf;

    /// The program of the service, for the tests.
    fn a_program() -> PathBuf {
        std::env::current_exe().expect("the running program")
    }

    /// A layout on macOS, for the tests.
    fn a_layout() -> ServiceLayout {
        let request = LayoutRequest {
            platform: Platform::MacOs,
            scope: Scope::User,
            home: PathBuf::from("/Users/tester"),
            program: a_program(),
            config: PathBuf::from("/Users/tester/.config/phairdlna/config.toml"),
        };
        ServiceLayout::build(&request).expect("a layout")
    }

    /// The positive case: what was written can be read back.
    #[test]
    fn what_was_written_can_be_read_back() {
        let file = render(&a_layout());
        let document = parse(&file).expect("the file is read");

        assert_eq!(document.text("Label"), Some("org.phairdlna.server"));
        assert_eq!(document.text("WorkingDirectory"), Some("/Users/tester"));
        assert_eq!(document.flag("KeepAlive"), Some(true));
        assert_eq!(document.flag("RunAtLoad"), Some(true));
    }

    /// The positive case: the arguments are read back in their order.
    #[test]
    fn the_arguments_are_read_back() {
        let file = render(&a_layout());
        let document = parse(&file).expect("the file is read");

        let arguments = document.list("ProgramArguments").expect("the arguments");
        assert_eq!(arguments[0], a_program().to_string_lossy());
        assert_eq!(arguments[1], "serve");
        assert_eq!(arguments[2], "--config");
    }

    /// The positive case: the environment is read back.
    #[test]
    fn the_environment_is_read_back() {
        let file = render(&a_layout());
        let document = parse(&file).expect("the file is read");

        let environment = document
            .dictionary("EnvironmentVariables")
            .expect("the environment");
        assert_eq!(
            environment.get("HOME").map(String::as_str),
            Some("/Users/tester")
        );
    }

    /// The negative case: a file that is not a property list is refused.
    #[test]
    fn a_file_that_is_not_a_property_list_is_refused() {
        let error = parse("this is not a property list").expect_err("the file is refused");

        assert!(error.contains("dictionary"), "{error}");
    }

    /// The negative case: a dictionary that is not closed is refused.
    #[test]
    fn a_dictionary_that_is_not_closed_is_refused() {
        let error = parse("<plist><dict><key>Label</key><string>x</string>")
            .expect_err("the file is refused");

        assert!(error.contains("not closed"), "{error}");
    }

    /// The negative case: a value that is not known is refused.
    #[test]
    fn a_value_that_is_not_known_is_refused() {
        let error = parse("<plist><dict><key>Label</key><date>2026-11-16</date></dict></plist>")
            .expect_err("the file is refused");

        assert!(error.contains("not known"), "{error}");
    }

    /// The negative case: a text with an escaped ampersand is read back with
    /// the ampersand.
    #[test]
    fn an_escaped_value_is_read_back() {
        let file = "<plist><dict><key>Label</key><string>a &amp; b</string></dict></plist>";
        let document = parse(file).expect("the file is read");

        assert_eq!(
            document.entries.get("Label"),
            Some(&PlistValue::Text("a & b".to_owned()))
        );
    }
}
