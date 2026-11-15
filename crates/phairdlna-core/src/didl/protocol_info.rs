//! The `protocolInfo` of a resource.
//!
//! What: the four fields that say how the bytes of a resource are delivered,
//!       the plain form and the DLNA form, and the flags of the DLNA form.
//! Why: the third field is the MIME type the server really sends (`INT-064`),
//!       and the fourth field is where the DLNA signalling lives (`INT-078`).
//!       The DLNA part is additive: a plain UPnP client keeps working when
//!       the server speaks DLNA, and the server keeps working when it does
//!       not.
//! Example: `ProtocolPolicy::plain().protocol_info("audio/mpeg", None)`
//!       returns `http-get:*:audio/mpeg:*`.

use crate::catalogue::mime::HTTP_GET;

/// The token of the DLNA media format profile.
pub const DLNA_PROFILE: &str = "DLNA.ORG_PN";

/// The token of the supported operation.
pub const DLNA_OPERATION: &str = "DLNA.ORG_OP";

/// The token of the conversion.
pub const DLNA_CONVERSION: &str = "DLNA.ORG_CI";

/// The token of the capability flags.
pub const DLNA_FLAGS: &str = "DLNA.ORG_FLAGS";

/// The operation that says that the server can seek by byte range.
pub const DLNA_OPERATION_BYTE_RANGE_SEEK: &str = "01";

/// The conversion that says that the content is not transcoded.
pub const DLNA_CONVERSION_NONE: &str = "0";

/// One protocolInfo entry: the four fields of the format.
///
/// The format is `<transportProtocol>:<network>:<contentFormat>:<additionalInfo>`
/// (standards §4.4). No field may contain a colon.
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
    /// Builds an entry, or `None` when a field breaks the format.
    ///
    /// The four fields are separated by colons, so no field may be empty and
    /// no field may contain a colon: an entry that breaks the rule cannot be
    /// read by a client (standards §4.4).
    pub fn new(
        transport_protocol: impl Into<String>,
        network: impl Into<String>,
        content_format: impl Into<String>,
        additional_info: impl Into<String>,
    ) -> Option<Self> {
        let entry = Self {
            transport_protocol: transport_protocol.into(),
            network: network.into(),
            content_format: content_format.into(),
            additional_info: additional_info.into(),
        };
        if entry.is_well_formed() {
            Some(entry)
        } else {
            None
        }
    }

    /// The plain entry of a format: `http-get:*:<mime>:*`, or `None` when the
    /// format cannot be written.
    ///
    /// The fourth field is `*`, because the delivery of a file over `http-get`
    /// depends on no request-specific header (`INT-055`).
    pub fn http_get(content_format: &str) -> Option<Self> {
        Self::new(HTTP_GET, "*", content_format, "*")
    }

    /// Whether the entry can be written as it is.
    pub fn is_well_formed(&self) -> bool {
        [
            &self.transport_protocol,
            &self.network,
            &self.content_format,
            &self.additional_info,
        ]
        .iter()
        .all(|field| !field.is_empty() && !field.contains(':'))
    }

    /// The entry as it appears in the `protocolInfo` attribute.
    pub fn as_string(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.transport_protocol, self.network, self.content_format, self.additional_info
        )
    }
}

/// The capability flags of the DLNA form.
///
/// The bits are the ones of the reference implementation (standards §6.4):
/// bit 30 time-based seek, bit 29 byte-based seek, bit 28 playcontainer, bit 24
/// streaming transfer mode, bit 23 interactive, bit 22 background, bit 21
/// connection stalling, bit 20 DLNA v1.5. Which of them the server sets is
/// still an open question (F-05 §12, question 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DlnaFlags(u32);

impl DlnaFlags {
    /// No flag at all.
    pub const NONE: DlnaFlags = DlnaFlags(0);
    /// Bit 30: the client may seek by time.
    pub const TIME_BASED_SEEK: DlnaFlags = DlnaFlags(1 << 30);
    /// Bit 29: the client may seek by byte range.
    pub const BYTE_BASED_SEEK: DlnaFlags = DlnaFlags(1 << 29);
    /// Bit 28: the object is a playlist that can be played as a whole.
    pub const PLAY_CONTAINER: DlnaFlags = DlnaFlags(1 << 28);
    /// Bit 24: the transfer runs in the streaming mode.
    pub const STREAMING: DlnaFlags = DlnaFlags(1 << 24);
    /// Bit 23: the transfer runs in the interactive mode.
    pub const INTERACTIVE: DlnaFlags = DlnaFlags(1 << 23);
    /// Bit 22: the transfer runs in the background mode.
    pub const BACKGROUND: DlnaFlags = DlnaFlags(1 << 22);
    /// Bit 21: the server may stall the connection.
    pub const CONNECTION_STALLING: DlnaFlags = DlnaFlags(1 << 21);
    /// Bit 20: the content follows DLNA v1.5.
    pub const DLNA_V1_5: DlnaFlags = DlnaFlags(1 << 20);

    /// The flags with the bits of both sets.
    pub fn union(self, other: DlnaFlags) -> DlnaFlags {
        DlnaFlags(self.0 | other.0)
    }

    /// The bits of the flags.
    pub fn bits(&self) -> u32 {
        self.0
    }

    /// The flags as the DLNA token carries them.
    ///
    /// The reference implementation writes the significant digits first and
    /// pads the rest with zeros, so the token is always 32 characters long.
    pub fn as_text(&self) -> String {
        let digits = format!("{:X}", self.0);
        let mut text = digits;
        while text.len() < 32 {
            text.push('0');
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::{DlnaFlags, ProtocolInfo};

    /// The plain entry has four fields and the wildcard in the fourth.
    #[test]
    fn the_plain_entry_has_four_fields() {
        let entry = ProtocolInfo::http_get("video/mp4").expect("a valid entry");
        assert_eq!(entry.as_string(), "http-get:*:video/mp4:*");
        assert_eq!(entry.as_string().split(':').count(), 4);
    }

    /// An entry whose field carries a colon cannot be built.
    #[test]
    fn an_entry_with_a_colon_is_refused() {
        assert!(ProtocolInfo::new("http-get", "*", "a:b", "*").is_none());
        assert!(ProtocolInfo::new("", "*", "video/mp4", "*").is_none());
        assert!(ProtocolInfo::http_get("audio/mpeg").is_some());
    }

    /// The flags are the documented bits.
    #[test]
    fn the_flags_are_the_documented_bits() {
        assert_eq!(DlnaFlags::TIME_BASED_SEEK.bits(), 1 << 30);
        assert_eq!(DlnaFlags::BYTE_BASED_SEEK.bits(), 1 << 29);
        assert_eq!(DlnaFlags::PLAY_CONTAINER.bits(), 1 << 28);
        assert_eq!(DlnaFlags::STREAMING.bits(), 1 << 24);
        assert_eq!(DlnaFlags::INTERACTIVE.bits(), 1 << 23);
        assert_eq!(DlnaFlags::BACKGROUND.bits(), 1 << 22);
        assert_eq!(DlnaFlags::CONNECTION_STALLING.bits(), 1 << 21);
        assert_eq!(DlnaFlags::DLNA_V1_5.bits(), 1 << 20);
    }

    /// The flags are written as 32 characters, the significant digits first.
    #[test]
    fn the_flags_are_written_as_thirty_two_characters() {
        let text = DlnaFlags::BYTE_BASED_SEEK
            .union(DlnaFlags::STREAMING)
            .as_text();
        assert_eq!(text.len(), 32);
        assert!(text.starts_with("21000000"));
    }
}
