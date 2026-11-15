//! The addresses the document points at.
//!
//! What: the address of the bytes of a file, the address of its artwork and
//!       the encoding of an identifier that reaches a URL.
//! Why: the document must never carry a path (`INT-062`, `PLT-008`), and the
//!       HTTP layer serves exactly the address the document names (`INT-090`).
//!       The identifier is opaque, but it still reaches a URL, so it is
//!       encoded: a hostile identifier cannot change the path.
//! Example: `media_uri` builds the address of the file with the identifier
//!       `a3f1`.

/// The path of the endpoint that serves the media.
///
/// The HTTP layer serves exactly the address the document names (`INT-090`).
pub const MEDIA_PATH: &str = "/media";

/// The path of the endpoint that serves the artwork.
pub const ARTWORK_PATH: &str = "/art";

/// The address of the bytes of an object.
pub fn media_uri(base_url: &str, id: &str) -> String {
    format!(
        "{}{MEDIA_PATH}/{}",
        base_url.trim_end_matches('/'),
        encode_segment(id)
    )
}

/// The address of the artwork of an object.
pub fn artwork_uri(base_url: &str, id: &str) -> String {
    format!(
        "{}{ARTWORK_PATH}/{}",
        base_url.trim_end_matches('/'),
        encode_segment(id)
    )
}

/// Percent-encodes one segment of a path.
///
/// The identifier is opaque, but it reaches a URL, so every character that is
/// not unreserved is encoded. A hostile identifier cannot change the path.
fn encode_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{artwork_uri, media_uri};

    /// The address of the media ends in the identifier of the object.
    #[test]
    fn the_media_address_names_the_object() {
        assert_eq!(
            media_uri("http://192.168.1.20:8200", "a3f1"),
            "http://192.168.1.20:8200/media/a3f1"
        );
    }

    /// A trailing slash of the base address does not change the address.
    #[test]
    fn a_trailing_slash_is_ignored() {
        assert_eq!(
            media_uri("http://192.168.1.20:8200/", "a3f1"),
            "http://192.168.1.20:8200/media/a3f1"
        );
    }

    /// An identifier cannot change the path, because it is encoded.
    #[test]
    fn a_hostile_identifier_cannot_change_the_path() {
        let address = media_uri("http://192.168.1.20:8200", "../../etc/passwd");
        assert_eq!(
            address,
            "http://192.168.1.20:8200/media/..%2F..%2Fetc%2Fpasswd"
        );
        assert!(!address.contains("/../"));
    }

    /// The address of the artwork is an address of this server.
    #[test]
    fn the_artwork_address_is_an_address_of_this_server() {
        assert_eq!(
            artwork_uri("http://192.168.1.20:8200", "art-1"),
            "http://192.168.1.20:8200/art/art-1"
        );
    }
}
