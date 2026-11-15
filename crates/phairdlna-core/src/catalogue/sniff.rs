//! What a file really is, judged by its first bytes.
//!
//! What: a small sniffer that names the format of a file from its head.
//! Why: a file whose name lies — or a file with no name at all — is still a file
//!       the server can serve. The catalogue asks the name first and the content
//!       second (`FUN-005`).
//! Example: a file called `holiday` that begins with `ftyp` is an MP4.
//!
//! The sniffer is deliberately small. It answers only the formats the server
//! serves or indexes, and it says nothing when it is not sure: a wrong guess
//! would put a file into the tree that the client cannot play.

/// The number of bytes the sniffer needs.
pub const HEAD_LENGTH: usize = 16;

/// The format of a file, judged by its first bytes.
pub fn sniff(head: &[u8]) -> Option<&'static str> {
    if starts_with(head, b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if starts_with(head, b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if starts_with(head, b"\x1a\x45\xdf\xa3") {
        return Some("video/x-matroska");
    }
    if starts_with(head, b"OggS") {
        return Some("audio/ogg");
    }
    if starts_with(head, b"ID3") {
        return Some("audio/mpeg");
    }
    if is_mpeg_audio_frame(head) {
        return Some("audio/mpeg");
    }
    if is_iso_base_media(head) {
        return Some(iso_brand_mime(head));
    }
    if is_mpeg_transport_stream(head) {
        return Some("video/mpeg");
    }
    None
}

/// Whether the head begins with the given bytes.
fn starts_with(head: &[u8], prefix: &[u8]) -> bool {
    head.len() >= prefix.len() && &head[..prefix.len()] == prefix
}

/// Whether the head is the beginning of an MPEG audio frame.
///
/// Every MPEG audio frame begins with eleven set bits, which is what separates
/// the frame from the noise around it.
fn is_mpeg_audio_frame(head: &[u8]) -> bool {
    head.len() >= 2 && head[0] == 0xff && (head[1] & 0xe0) == 0xe0
}

/// Whether the head is the beginning of an ISO base media file.
///
/// The `ftyp` box is the fourth box of the file and carries the brand.
fn is_iso_base_media(head: &[u8]) -> bool {
    head.len() >= 12 && &head[4..8] == b"ftyp"
}

/// The format an ISO base media file announces.
///
/// An audio brand gives audio, every other brand gives video: the catalogue
/// shows both, and the client decides what it can play.
fn iso_brand_mime(head: &[u8]) -> &'static str {
    match &head[8..12] {
        b"M4A " | b"M4B " => "audio/mp4",
        _ => "video/mp4",
    }
}

/// Whether the head is the beginning of an MPEG transport stream.
///
/// The packets of a transport stream are 188 bytes long and each one begins
/// with the same synchronisation byte, so the second packet proves the first
/// was not a coincidence.
fn is_mpeg_transport_stream(head: &[u8]) -> bool {
    head.len() > 188 && head[0] == 0x47 && head[188] == 0x47
}

#[cfg(test)]
mod tests {
    use super::sniff;

    /// A JPEG is recognised by its marker.
    #[test]
    fn a_jpeg_is_recognised() {
        assert_eq!(sniff(b"\xff\xd8\xff\xe0\x00\x10JFIF"), Some("image/jpeg"));
    }

    /// A Matroska file is recognised by its magic number.
    #[test]
    fn a_matroska_file_is_recognised() {
        assert_eq!(
            sniff(b"\x1a\x45\xdf\xa3\x01\x00\x00\x00"),
            Some("video/x-matroska")
        );
    }

    /// An MP4 is recognised by its `ftyp` box.
    #[test]
    fn an_mp4_is_recognised() {
        assert_eq!(sniff(b"\x00\x00\x00\x18ftypmp42"), Some("video/mp4"));
    }

    /// An M4A is an MP4 with an audio brand.
    #[test]
    fn an_m4a_is_recognised() {
        assert_eq!(sniff(b"\x00\x00\x00\x18ftypM4A "), Some("audio/mp4"));
    }

    /// An MP3 with a tag is recognised by its tag.
    #[test]
    fn a_tagged_mp3_is_recognised() {
        assert_eq!(sniff(b"ID3\x03\x00\x00\x00"), Some("audio/mpeg"));
    }

    /// An MP3 without a tag is recognised by its frame header.
    #[test]
    fn an_untagged_mp3_is_recognised() {
        assert_eq!(sniff(b"\xff\xfb\x90\x00"), Some("audio/mpeg"));
    }

    /// A transport stream is recognised by two synchronisation bytes.
    #[test]
    fn a_transport_stream_is_recognised() {
        let mut head = vec![0x47; 189];
        head[1] = 0x11;
        assert_eq!(sniff(&head), Some("video/mpeg"));
    }

    /// A file the sniffer does not know is not guessed.
    #[test]
    fn an_unknown_file_is_not_guessed() {
        assert_eq!(sniff(b"just some text"), None);
        assert_eq!(sniff(b""), None);
    }
}
