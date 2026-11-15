//! The headers of a media request and of a media response.
//!
//! What: the header set, the names of the headers the server reads and writes,
//!       and the transfer mode of the DLNA layer.
//! Why: the headers are how the client learns what it received. A renderer
//!       that does not see `Content-Length` does not know when the body ends
//!       (INT-088), and a renderer that does not see the DLNA headers may
//!       refuse the content (standards §6.4). The names are compared without
//!       regard to case, as HTTP requires (RFC 9110 §5.1).
//! Example: `HeaderMap::get("range")` finds the header a client wrote as
//!       `Range`.

/// The header that asks for a byte range.
pub const RANGE: &str = "Range";

/// The header that makes a range conditional.
pub const IF_RANGE: &str = "If-Range";

/// The header that announces the ranges the server supports.
pub const ACCEPT_RANGES: &str = "Accept-Ranges";

/// The header that carries the type of the body.
pub const CONTENT_TYPE: &str = "Content-Type";

/// The header that carries the length of the body.
pub const CONTENT_LENGTH: &str = "Content-Length";

/// The header that carries the range of the body.
pub const CONTENT_RANGE: &str = "Content-Range";

/// The header that carries the validator of the representation.
pub const ETAG: &str = "ETag";

/// The header that carries the connection handling.
pub const CONNECTION: &str = "Connection";

/// The header that carries the methods a resource allows.
pub const ALLOW: &str = "Allow";

/// The DLNA header that carries the transfer mode.
pub const TRANSFER_MODE: &str = "transferMode.dlna.org";

/// The DLNA header that carries the features of the content.
pub const CONTENT_FEATURES: &str = "contentFeatures.dlna.org";

/// The DLNA header that asks for the features of the content.
pub const GET_CONTENT_FEATURES: &str = "getcontentFeatures.dlna.org";

/// The DLNA header that carries the real time information.
pub const REAL_TIME_INFO: &str = "realTimeInfo.dlna.org";

/// The only value of `getcontentFeatures.dlna.org` the server accepts.
///
/// Any other value is a bad request (INT-097, standards §8.8 N6).
pub const GET_CONTENT_FEATURES_VALUE: &str = "1";

/// The value of `realTimeInfo.dlna.org` on a media response.
///
/// The server sends media as fast as the client reads it, so the time lag is
/// unknown and the header says so (INT-099, standards §6.4).
pub const REAL_TIME_INFO_VALUE: &str = "DLNA.ORG_TLAG=*";

/// The value of `Accept-Ranges` on a media response.
pub const ACCEPT_RANGES_VALUE: &str = "bytes";

/// A set of HTTP headers.
///
/// The names are compared without regard to case, as HTTP requires
/// (RFC 9110 §5.1). A name may appear once; `set` replaces the value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderMap {
    entries: Vec<(String, String)>,
}

impl HeaderMap {
    /// Builds an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The value of a header, if the set has one.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Whether the set has a header.
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Sets a header, replacing an existing value of the same name.
    pub fn set(&mut self, name: &str, value: impl Into<String>) {
        let value = value.into();
        match self
            .entries
            .iter_mut()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
        {
            Some(entry) => entry.1 = value,
            None => self.entries.push((name.to_owned(), value)),
        }
    }

    /// The headers, in the order they were set.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    /// The number of headers.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The transfer mode of the DLNA layer.
///
/// The client asks for a mode with the `transferMode.dlna.org` header; the
/// server names the mode it uses in its answer (INT-096, standards §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    /// The server sends the media as fast as the client reads it.
    Streaming,
    /// The client controls the transfer, as it does for a still image.
    Interactive,
    /// The transfer runs in the background.
    Background,
}

impl TransferMode {
    /// The mode as the header spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferMode::Streaming => "Streaming",
            TransferMode::Interactive => "Interactive",
            TransferMode::Background => "Background",
        }
    }

    /// The mode a header value names, if it names one.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "Streaming" => Some(TransferMode::Streaming),
            "Interactive" => Some(TransferMode::Interactive),
            "Background" => Some(TransferMode::Background),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HeaderMap, TransferMode, ACCEPT_RANGES_VALUE, GET_CONTENT_FEATURES_VALUE,
        REAL_TIME_INFO_VALUE,
    };

    /// A header is found whatever the case of its name.
    #[test]
    fn a_header_is_found_in_any_case() {
        let mut headers = HeaderMap::new();
        headers.set("Content-Type", "audio/mpeg");
        assert_eq!(headers.get("content-type"), Some("audio/mpeg"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("audio/mpeg"));
        assert!(headers.contains("Content-Type"));
        assert_eq!(headers.get("Range"), None);
    }

    /// Setting a header twice replaces the value, and the name keeps the case
    /// of the first setting.
    #[test]
    fn setting_a_header_twice_replaces_it() {
        let mut headers = HeaderMap::new();
        headers.set("Accept-Ranges", "bytes");
        headers.set("accept-ranges", "none");
        assert_eq!(headers.len(), 1);
        assert_eq!(headers.get("ACCEPT-RANGES"), Some("none"));
    }

    /// The set keeps the order of the headers.
    #[test]
    fn the_set_keeps_the_order() {
        let mut headers = HeaderMap::new();
        headers.set("A", "1");
        headers.set("B", "2");
        let names: Vec<&str> = headers.iter().map(|(name, _)| name).collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    /// The three transfer modes are the ones of the DLNA layer.
    #[test]
    fn the_transfer_modes_are_the_three_of_the_layer() {
        assert_eq!(TransferMode::Streaming.as_str(), "Streaming");
        assert_eq!(TransferMode::Interactive.as_str(), "Interactive");
        assert_eq!(TransferMode::Background.as_str(), "Background");
        assert_eq!(
            TransferMode::parse("Streaming"),
            Some(TransferMode::Streaming)
        );
        assert_eq!(
            TransferMode::parse(" Interactive "),
            Some(TransferMode::Interactive)
        );
        assert_eq!(TransferMode::parse("Unknown"), None);
        assert_eq!(TransferMode::parse(""), None);
    }

    /// The DLNA constants are the ones of the reference implementation.
    #[test]
    fn the_dlna_constants_are_the_ones_of_the_layer() {
        assert_eq!(GET_CONTENT_FEATURES_VALUE, "1");
        assert_eq!(REAL_TIME_INFO_VALUE, "DLNA.ORG_TLAG=*");
        assert_eq!(ACCEPT_RANGES_VALUE, "bytes");
    }
}
