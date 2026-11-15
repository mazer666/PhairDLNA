//! The formats the server can deliver.
//!
//! What: the registry of the MIME types the server serves and the transport
//!       protocol it delivers them with.
//! Why: the ConnectionManager advertises the formats in its `Source` list
//!       (`INT-051`), the DIDL-Lite document names the same formats in the
//!       `protocolInfo` of a resource (`INT-064`), and the HTTP layer sends
//!       the same formats as the `Content-Type` of a media answer (`INT-087`).
//!       One registry, so that the three cannot drift apart. The registry
//!       lives in the catalogue, because the catalogue is the module that
//!       knows what the server shares — and the module that both the protocol
//!       and the DIDL-Lite writer can depend on without a cycle.
//! Example: `is_served("audio/mpeg")` answers whether the server can deliver
//!       that format at all.

/// The transport protocol the server delivers media with.
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

/// Whether the server can deliver the given format.
///
/// A format that is not in the registry must not appear in the `protocolInfo`
/// of a resource, because the client would then ask for bytes the server
/// cannot deliver (`INT-064`).
pub fn is_served(mime_type: &str) -> bool {
    SUPPORTED_MIME_TYPES.contains(&mime_type)
}

#[cfg(test)]
mod tests {
    use super::{is_served, SUPPORTED_MIME_TYPES};

    /// Every format of the registry is a format the server serves.
    #[test]
    fn every_registered_format_is_served() {
        for mime_type in SUPPORTED_MIME_TYPES {
            assert!(is_served(mime_type), "{mime_type} must be served");
        }
    }

    /// A format the server does not know is refused.
    #[test]
    fn an_unknown_format_is_not_served() {
        assert!(!is_served("video/x-msvideo"));
        assert!(!is_served(""));
        assert!(!is_served("audio/mpeg; charset=utf-8"));
    }
}
