//! The policy that decides how a `protocolInfo` is written.
//!
//! What: the two signalling modes — the plain form and the DLNA form — and
//!       the policy that chooses between them.
//! Why: the DLNA signalling is additive (`INT-078`): a plain UPnP client
//!       keeps working when the server speaks DLNA, and the server keeps
//!       working when it does not. The mode is a property of the client
//!       profile (F-07), never of the file; the flags and the byte-range
//!       support are properties of the server.
//! Example: `ProtocolPolicy::dlna(true, DlnaFlags::BYTE_BASED_SEEK)` builds
//!       the policy of a server that can seek by byte range.

use crate::catalogue::mime::HTTP_GET;
use crate::didl::protocol_info::{
    DlnaFlags, ProtocolInfo, DLNA_CONVERSION, DLNA_CONVERSION_NONE, DLNA_FLAGS, DLNA_OPERATION,
    DLNA_OPERATION_BYTE_RANGE_SEEK, DLNA_PROFILE,
};

/// The signalling mode of the fourth field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolMode {
    /// The plain form: the fourth field is `*`.
    Plain,
    /// The DLNA form: the fourth field carries the DLNA tokens (`INT-078`).
    Dlna,
}

/// The policy that decides how the `protocolInfo` of a resource is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolPolicy {
    mode: ProtocolMode,
    byte_range_seek: bool,
    flags: DlnaFlags,
    subtitles: bool,
}

impl ProtocolPolicy {
    /// The plain mode: the fourth field is `*`.
    pub fn plain() -> Self {
        Self {
            mode: ProtocolMode::Plain,
            byte_range_seek: false,
            flags: DlnaFlags::NONE,
            subtitles: true,
        }
    }

    /// The DLNA mode: the fourth field carries the DLNA tokens.
    ///
    /// The byte-range support is claimed only when the server really serves
    /// byte ranges (`INT-079`).
    pub fn dlna(byte_range_seek: bool, flags: DlnaFlags) -> Self {
        Self {
            mode: ProtocolMode::Dlna,
            byte_range_seek,
            flags,
            subtitles: true,
        }
    }

    /// Whether the server offers a subtitle as an additional resource.
    ///
    /// The element is additive: an item with a subtitle carries one more `res`,
    /// and every other attribute of the item stays the same, so a client that
    /// does not understand it still plays the film (`FUN-016`).
    ///
    /// The default is on. The profile store of the client profiles is empty
    /// today (F-07), and a feature that is off until a profile exists would
    /// never be seen (`F-11` § 12).
    pub fn with_subtitles(mut self, enabled: bool) -> Self {
        self.subtitles = enabled;
        self
    }

    /// Whether the server offers a subtitle as an additional resource.
    pub fn subtitles(&self) -> bool {
        self.subtitles
    }

    /// The mode of the policy.
    pub fn mode(&self) -> ProtocolMode {
        self.mode
    }

    /// The entry of a resource of the given format, or `None` when the entry
    /// cannot be written.
    ///
    /// The profile is the DLNA media format profile of the content; it is
    /// only written when the content really conforms to it (`INT-079`). A
    /// profile that is not a plain name is not written at all — the server
    /// then claims no profile, which is what `INT-079` asks for — and the
    /// resource stays playable.
    pub fn protocol_info(
        &self,
        content_format: &str,
        profile: Option<&str>,
    ) -> Option<ProtocolInfo> {
        match self.mode {
            ProtocolMode::Plain => ProtocolInfo::http_get(content_format),
            ProtocolMode::Dlna => {
                let additional = self.additional_info(profile);
                ProtocolInfo::new(HTTP_GET, "*", content_format, &additional)
            }
        }
    }

    /// The fourth field of the DLNA mode.
    fn additional_info(&self, profile: Option<&str>) -> String {
        let mut tokens: Vec<String> = Vec::new();
        if let Some(profile) = profile {
            if is_profile_name(profile) {
                tokens.push(format!("{DLNA_PROFILE}={profile}"));
            }
        }
        if self.byte_range_seek {
            tokens.push(format!("{DLNA_OPERATION}={DLNA_OPERATION_BYTE_RANGE_SEEK}"));
        }
        tokens.push(format!("{DLNA_CONVERSION}={DLNA_CONVERSION_NONE}"));
        tokens.push(format!("{DLNA_FLAGS}={}", self.flags.as_text()));
        tokens.join(";")
    }
}

/// Whether a profile is a plain name.
///
/// The media format profiles are names like `MP3` or `AVC_MP4_BL`; a name
/// with a colon or a semicolon would break the fourth field.
fn is_profile_name(profile: &str) -> bool {
    !profile.is_empty()
        && profile
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

impl Default for ProtocolPolicy {
    /// The plain mode is the default: the server claims nothing it cannot
    /// prove (`INT-079`, F-05 §12, question 2).
    fn default() -> Self {
        Self::plain()
    }
}

#[cfg(test)]
mod tests {
    use super::{ProtocolMode, ProtocolPolicy};
    use crate::didl::protocol_info::DlnaFlags;

    /// The plain mode never writes a DLNA token.
    #[test]
    fn the_plain_mode_writes_no_dlna_token() {
        let policy = ProtocolPolicy::plain();
        let entry = policy
            .protocol_info("video/mp4", Some("AVC_MP4_BL"))
            .expect("a valid entry");
        assert_eq!(entry.as_string(), "http-get:*:video/mp4:*");
        assert!(!entry.as_string().contains("DLNA"));
    }

    /// The DLNA mode writes the four tokens of the reference implementation.
    #[test]
    fn the_dlna_mode_writes_the_four_tokens() {
        let policy = ProtocolPolicy::dlna(true, DlnaFlags::BYTE_BASED_SEEK);
        let entry = policy
            .protocol_info("video/mp4", Some("AVC_MP4_BL"))
            .expect("a valid entry");
        let text = entry.as_string();
        assert!(text.contains("DLNA.ORG_PN=AVC_MP4_BL"));
        assert!(text.contains("DLNA.ORG_OP=01"));
        assert!(text.contains("DLNA.ORG_CI=0"));
        assert!(text.contains("DLNA.ORG_FLAGS="));
    }

    /// The fourth field never carries a colon, or the field would be read as
    /// more than four fields.
    #[test]
    fn the_fourth_field_carries_no_colon() {
        let policy = ProtocolPolicy::dlna(true, DlnaFlags::STREAMING);
        let entry = policy
            .protocol_info("video/mp4", Some("AVC_MP4_BL"))
            .expect("a valid entry");
        assert_eq!(entry.as_string().split(':').count(), 4);
    }

    /// Without a profile no `DLNA.ORG_PN` is written, because the server may
    /// not claim a profile the content does not follow (`INT-079`).
    #[test]
    fn without_a_profile_no_profile_is_claimed() {
        let policy = ProtocolPolicy::dlna(true, DlnaFlags::NONE);
        let entry = policy
            .protocol_info("video/mp4", None)
            .expect("a valid entry");
        assert!(!entry.as_string().contains("DLNA.ORG_PN"));
        assert!(entry.as_string().contains("DLNA.ORG_CI=0"));
    }

    /// Without byte-range support no `DLNA.ORG_OP=01` is written (`INT-079`).
    #[test]
    fn without_byte_range_support_no_operation_is_claimed() {
        let policy = ProtocolPolicy::dlna(false, DlnaFlags::NONE);
        let entry = policy
            .protocol_info("video/mp4", Some("AVC_MP4_BL"))
            .expect("a valid entry");
        assert!(!entry.as_string().contains("DLNA.ORG_OP=01"));
    }

    /// A format that carries a colon cannot be written, because the four
    /// fields are separated by colons.
    #[test]
    fn a_broken_format_yields_no_entry() {
        let policy = ProtocolPolicy::plain();
        assert!(policy.protocol_info("a:b", None).is_none());
        assert!(policy.protocol_info("", None).is_none());
    }

    /// A profile that is not a plain name is not claimed, but the entry is
    /// still written: the file stays playable (`INT-079`).
    #[test]
    fn a_broken_profile_is_not_claimed() {
        let policy = ProtocolPolicy::dlna(true, DlnaFlags::NONE);
        for profile in ["a:b", "a;b", ""] {
            let entry = policy
                .protocol_info("audio/mpeg", Some(profile))
                .expect("the entry is still written");
            assert!(
                !entry.as_string().contains("DLNA.ORG_PN"),
                "{profile} must not be claimed"
            );
            assert_eq!(entry.as_string().split(':').count(), 4);
        }
    }

    /// The default policy is the plain one.
    #[test]
    fn the_default_policy_is_the_plain_one() {
        assert_eq!(ProtocolPolicy::default().mode(), ProtocolMode::Plain);
    }
}
