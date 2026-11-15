//! The MIME types the server can deliver.
//!
//! What: the protocol list of the ConnectionManager: the `Source` list the
//!       server advertises and the `Sink` list it offers.
//! Why: the `Source` list must name only the formats the server can really
//!       deliver (`INT-051`), and the same registry decides the `Content-Type`
//!       of a media answer in F-06 and the third field of a `res` element in
//!       F-05. One registry, so that the three cannot drift apart.
//! Example: `source_protocol_info()` returns the `http-get` entries of the
//!       `Source` list.

use crate::catalogue::mime::SUPPORTED_MIME_TYPES;
use crate::didl::ProtocolInfo;

/// The `Source` list: the formats the server can send.
///
/// The fourth field is `*`, because the delivery of a file over `http-get`
/// depends on no request-specific header (`INT-055`). The list names no file
/// and no path (`PLT-008`). Every format of the registry yields an entry — the
/// registry holds MIME types, and a MIME type can always be written.
pub fn source_protocol_info() -> Vec<ProtocolInfo> {
    SUPPORTED_MIME_TYPES
        .iter()
        .filter_map(|mime_type| ProtocolInfo::http_get(mime_type))
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
        let entry = ProtocolInfo::new("http-get", "*", "video/mp4", "*").expect("a valid entry");
        assert_eq!(entry.as_string(), "http-get:*:video/mp4:*");
    }
}
