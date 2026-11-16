//! The pseudonyms that replace the private values of a log record.
//!
//! What: the mapping from a path, a client address or a device identifier to
//!       a stable pseudonym, and the function that applies it to a record.
//! Why: the log is the one place where the server says what it does, and it
//!       must not become a file that lists the user's home directory and the
//!       addresses of their neighbours (`SEC-094`, `SEC-093`). A pseudonym
//!       keeps the record useful — the same path is still the same path — and
//!       keeps it private.
//! Example: the path `/Users/me/Movies` becomes `<path:1>` in every record
//!       that names it, and the file names its owner no longer.
//!
//! **How a value is recognised.** A value is a path when it starts with a
//! separator, an address when it is an IPv4 or IPv6 literal, and a device
//! identifier when it is a UUID. Everything else is left alone: a log that
//! redacts everything is a log nobody reads.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Mutex;

/// What a redacted value was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    /// A path on the file system.
    Path,
    /// The address of a client.
    Address,
    /// The identifier of the device.
    Device,
}

impl Kind {
    /// The name the pseudonym carries.
    fn name(self) -> &'static str {
        match self {
            Kind::Path => "path",
            Kind::Address => "address",
            Kind::Device => "device",
        }
    }
}

/// The pseudonyms of the values that were redacted.
///
/// The map grows as new values are seen, and every value keeps its pseudonym
/// for the life of the process: the same path is the same `<path:1>` in every
/// record.
#[derive(Debug, Default)]
pub struct Redactor {
    /// The pseudonyms, one map per kind.
    names: Mutex<HashMap<(Kind, String), String>>,
    /// How many pseudonyms of each kind were handed out.
    counters: Mutex<HashMap<Kind, usize>>,
}

impl Redactor {
    /// Builds a redactor that has handed out no pseudonym yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The pseudonym of a value, and the pseudonym it keeps from now on.
    ///
    /// A value that was never seen gets the next free pseudonym of its kind.
    fn pseudonym(&self, kind: Kind, value: &str) -> String {
        let key = (kind, value.to_owned());
        let mut names = self.names.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(existing) = names.get(&key) {
            return existing.clone();
        }
        let mut counters = self
            .counters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let counter = counters.entry(kind).or_insert(0);
        *counter += 1;
        let name = format!("<{}:{}>", kind.name(), counter);
        names.insert(key, name.clone());
        name
    }

    /// The pseudonym of a path.
    pub fn path(&self, path: &Path) -> String {
        self.pseudonym(Kind::Path, &path.to_string_lossy())
    }

    /// The pseudonym of a client address.
    pub fn address(&self, address: &IpAddr) -> String {
        self.pseudonym(Kind::Address, &address.to_string())
    }

    /// The pseudonym of a device identifier.
    pub fn device(&self, identifier: &str) -> String {
        self.pseudonym(Kind::Device, identifier)
    }

    /// Redacts a value, whatever it is.
    ///
    /// A value that is not a path, an address or a device identifier is left
    /// as it is: the number of a port and the name of a service are not
    /// private.
    pub fn value(&self, value: &str) -> String {
        match classify(value) {
            Some((Kind::Path, _)) => self.pseudonym(Kind::Path, value),
            Some((Kind::Address, _)) => self.pseudonym(Kind::Address, value),
            Some((Kind::Device, _)) => self.pseudonym(Kind::Device, value),
            None => value.to_owned(),
        }
    }

    /// Redacts every field of a record.
    ///
    /// The pairs are the fields of the record; the result is the same pairs
    /// with the private values replaced. The message itself is not redacted:
    /// it is written by the code and names no path.
    pub fn fields(&self, fields: &[(&str, &str)]) -> Vec<(String, String)> {
        fields
            .iter()
            .map(|(name, value)| ((*name).to_owned(), self.value(value)))
            .collect()
    }
}

/// What a value is, if it is anything the redactor knows.
///
/// The order matters: a path is checked first, because a path may contain
/// something that looks like an address.
fn classify(value: &str) -> Option<(Kind, &str)> {
    // A URL names a place on the network, not a place on the disk of the
    // user: the base URL of the server belongs in the log, so that the user
    // can see where the server is.
    if is_url(value) {
        return None;
    }
    if looks_like_path(value) {
        return Some((Kind::Path, value));
    }
    if is_address(value) {
        return Some((Kind::Address, value));
    }
    if looks_like_uuid(value) {
        return Some((Kind::Device, value));
    }
    None
}

/// Whether a value is a URL.
fn is_url(value: &str) -> bool {
    value.contains("://")
}

/// Whether a value is an address of a client, with or without a port.
///
/// The port is the part a client adds to an address it puts into a header or
/// a log record, and it must not keep the address from being recognised.
fn is_address(value: &str) -> bool {
    if value.parse::<IpAddr>().is_ok() {
        return true;
    }
    let host = match value.rfind(':') {
        Some(position) if !value[position..].contains(']') => &value[..position],
        _ => return false,
    };
    host.trim_matches(|c| c == '[' || c == ']')
        .parse::<IpAddr>()
        .is_ok()
}

/// Whether a value is a path on the file system.
///
/// A path is absolute, or it is relative and carries a separator, so that
/// `film.mkv` is a file name and `Movies/film.mkv` is a path.
fn looks_like_path(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value.starts_with('/') || value.starts_with('\\') {
        return true;
    }
    value.contains('/') || value.contains('\\')
}

/// Whether a value is a UUID.
///
/// The shape is eight hexadecimal digits, four dashes and three groups of
/// hexadecimal digits — the shape of the unique device name (`SEC-096`).
fn looks_like_uuid(value: &str) -> bool {
    let groups: Vec<&str> = value.split('-').collect();
    if groups.len() != 5 {
        return false;
    }
    let widths = [8, 4, 4, 4, 12];
    groups
        .iter()
        .zip(widths.iter())
        .all(|(group, width)| group.len() == *width && group.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::Redactor;
    use std::net::IpAddr;
    use std::path::Path;

    /// A path becomes a pseudonym and keeps it.
    #[test]
    fn a_path_becomes_a_pseudonym() {
        let redactor = Redactor::new();
        let first = redactor.path(Path::new("/Users/me/Movies/film.mkv"));
        let second = redactor.path(Path::new("/Users/me/Movies/film.mkv"));

        assert_eq!(first, "<path:1>");
        assert_eq!(first, second, "the same path keeps the same pseudonym");
        assert!(!first.contains("Users"));
    }

    /// Two different paths get two different pseudonyms.
    #[test]
    fn two_paths_get_two_pseudonyms() {
        let redactor = Redactor::new();
        let first = redactor.path(Path::new("/Users/me/Movies/a.mkv"));
        let second = redactor.path(Path::new("/Users/me/Movies/b.mkv"));

        assert_ne!(first, second);
        assert_eq!(second, "<path:2>");
    }

    /// An address becomes a pseudonym.
    #[test]
    fn an_address_becomes_a_pseudonym() {
        let redactor = Redactor::new();
        let address: IpAddr = "192.168.1.20".parse().expect("a valid address");

        assert_eq!(redactor.address(&address), "<address:1>");
    }

    /// A device identifier becomes a pseudonym.
    #[test]
    fn a_device_identifier_becomes_a_pseudonym() {
        let redactor = Redactor::new();
        let identifier = "11111111-2222-3333-4444-555555555555";

        assert_eq!(redactor.device(identifier), "<device:1>");
    }

    /// A value that is nothing private is left alone.
    #[test]
    fn a_public_value_is_left_alone() {
        let redactor = Redactor::new();

        assert_eq!(redactor.value("8200"), "8200");
        assert_eq!(redactor.value("info"), "info");
        assert_eq!(redactor.value("film.mkv"), "film.mkv");
    }

    /// A path inside a field is redacted.
    #[test]
    fn a_path_inside_a_field_is_redacted() {
        let redactor = Redactor::new();
        let fields = redactor.fields(&[("path", "/Users/me/Movies/film.mkv")]);

        assert_eq!(fields[0].1, "<path:1>");
    }

    /// The address of a client is redacted in a field.
    #[test]
    fn an_address_inside_a_field_is_redacted() {
        let redactor = Redactor::new();
        let fields = redactor.fields(&[("client", "192.168.1.20")]);

        assert_eq!(fields[0].1, "<address:1>");
    }

    /// The identifier of the device is redacted in a field.
    #[test]
    fn a_device_identifier_inside_a_field_is_redacted() {
        let redactor = Redactor::new();
        let fields = redactor.fields(&[("udn", "11111111-2222-3333-4444-555555555555")]);

        assert_eq!(fields[0].1, "<device:1>");
    }

    /// Every occurrence of a value carries the same pseudonym.
    #[test]
    fn every_occurrence_carries_the_same_pseudonym() {
        let redactor = Redactor::new();
        let first = redactor.fields(&[("path", "/Users/me/Movies")]);
        let second = redactor.fields(&[("path", "/Users/me/Movies")]);

        assert_eq!(first[0].1, second[0].1);
    }

    /// An address with a port is an address.
    #[test]
    fn an_address_with_a_port_is_an_address() {
        let redactor = Redactor::new();

        assert_eq!(redactor.value("192.168.1.20:8200"), "<address:1>");
        assert_eq!(redactor.value("[::1]:8200"), "<address:2>");
    }

    /// A URL is not a path: the user must see where the server is.
    #[test]
    fn a_url_is_left_alone() {
        let redactor = Redactor::new();

        assert_eq!(
            redactor.value("http://127.0.0.1:8200"),
            "http://127.0.0.1:8200"
        );
        assert_eq!(
            redactor.value("http://192.168.1.20:8200/description.xml"),
            "http://192.168.1.20:8200/description.xml"
        );
    }
}
