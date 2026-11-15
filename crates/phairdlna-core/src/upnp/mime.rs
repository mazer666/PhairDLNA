//! The MIME types the server can deliver.
//!
//! What: the registry of the MIME types the server serves, and the protocol
//!       list that is derived from it.
//! Why: the `Source` list of the ConnectionManager must name only the formats
//!       the server can really deliver (`INT-051`), and the same registry
//!       decides the `Content-Type` of a media answer in F-06 and the third
//!       field of a `res` element in F-05. One registry, so that the three
//!       cannot drift apart.
//! Example: `source_protocol_info()` returns the `http-get` entries of the
//!       `Source` list.

/// The transport protocol the server uses.
pub const HTTP_GET: &str = "http-get";

/// The MIME types the server can deliver.
pub const SUPPORTED_MIME_TYPES: [&str; 6] = [
    "video/mp4",
    "video/x-matroska",
    "video/mpeg",
    "audio/mpeg",
    "audio/mp4",
    "image/jpeg",
];

/// One protocolInfo entry: the four fields of the format.
///
/// The format is `<transportProtocol>:<network>:<contentFormat>:<additionalInfo>`
/// (standards §1.3, §4.4). No field may contain a colon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfo {
    /// The transport protocol, for example `http-get`.
    pub transport_protocol: String,
    /// The network the transfer runs over, `*` for any.
    pub network: String,
    /// The content format, a MIME type.
    pub content_format: String,
    /// The additional information, `*` when no header is needed.
    pub additional_info: String,
}

impl ProtocolInfo {
    /// Builds an entry.
    pub fn new(
        transport_protocol: impl Into<String>,
        network: impl Into<String>,
        content_format: impl Into<String>,
        additional_info: impl Into<String>,
    ) -> Self {
        Self {
            transport_protocol: transport_protocol.into(),
            network: network.into(),
            content_format: content_format.into(),
            additional_info: additional_info.into(),
        }
    }

    /// The entry as it appears in the `Source` list.
    pub fn as_string(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.transport_protocol, self.network, self.content_format, self.additional_info
        )
    }
}

/// The `Source` list: the formats the server can send.
///
/// The fourth field is `*`, because the delivery of a file over `http-get`
/// depends on no request-specific header (`INT-055`). The list names no file
/// and no path (`PLT-008`).
pub fn source_protocol_info() -> Vec<ProtocolInfo> {
    SUPPORTED_MIME_TYPES
        .iter()
        .map(|mime_type| ProtocolInfo::new(HTTP_GET, "*", *mime_type, "*"))
        .collect()
}

/// The `Source` list as the comma-separated string the action returns.
pub fn source_list() -> String {
    source_protocol_info()
        .iter()
        .map(ProtocolInfo::as_string)
        .collect::<Vec<String>>()
        .join(",")
}

/// The `Sink` list: the formats the server can receive.
///
/// The server only sends media, so the list is empty (`INT-051`, standards
/// §8.4 M3).
pub fn sink_list() -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::{source_list, source_protocol_info, ProtocolInfo, SUPPORTED_MIME_TYPES};

    /// Every entry of the `Source` list has exactly four fields.
    #[test]
    fn every_entry_has_four_fields() {
        for entry in source_protocol_info() {
            let fields = entry.as_string().split(':').count();
            assert_eq!(fields, 4, "{} has {fields} fields", entry.as_string());
        }
    }

    /// The list names the formats the server can deliver, and nothing else.
    #[test]
    fn the_list_names_the_supported_formats() {
        let list = source_list();
        for mime_type in SUPPORTED_MIME_TYPES {
            assert!(
                list.contains(&format!("http-get:*:{mime_type}:*")),
                "{mime_type} is missing"
            );
        }
    }

    /// The `Source` list is a comma-separated list without spaces.
    #[test]
    fn the_list_has_no_spaces() {
        let list = source_list();
        assert!(!list.contains(' '));
        assert_eq!(list.split(',').count(), SUPPORTED_MIME_TYPES.len());
    }

    /// The transport protocol is `http-get`; the push protocol is never
    /// advertised (`INT-056`).
    #[test]
    fn only_http_get_is_advertised() {
        let list = source_list();
        assert!(!list.contains("http-subscribe"));
        assert!(list.contains("http-get:"));
    }

    /// An entry is written with the four fields in the order of the format.
    #[test]
    fn an_entry_is_written_in_the_specified_order() {
        let entry = ProtocolInfo::new("http-get", "*", "video/mp4", "*");
        assert_eq!(entry.as_string(), "http-get:*:video/mp4:*");
    }
}
