//! The object identifiers of the catalogue.
//!
//! What: the rule that turns a volume, a root and a relative identity into the
//!       opaque identifier a client sees.
//! Why: the identifier is the only thing a client keeps. It must not change
//!       when the library is rescanned, and it must not leak the path of a file
//!       into a URL a third party can read (ADR-0007, `SEC-005`, `INT-062`).
//! Example: `object_id` of the file `Films/King Kong.mkv` is 32 hexadecimal
//!       characters, and the same file always gets the same one.
//!
//! **The rule.** The identifier is the blake3 hash of
//!
//! ```text
//! volume identity ‖ 0x00 ‖ root key ‖ 0x00 ‖ relative identity
//! ```
//!
//! rendered as hexadecimal and cut to 128 bits. The three parts are separated by
//! a zero byte, which cannot occur inside a path component on the platforms the
//! server runs on, so two different triples can never produce the same input.
//!
//! - The **volume identity** keeps two volumes apart. The portable adapter
//!   reports the constant `local` (see `platform::local`), so the identifier of
//!   a file does not change when the volume is remounted.
//! - The **root key** keeps two roots apart. It is the position of the root in
//!   the configuration, and it is what makes the identifier of a file in the
//!   first root differ from the identifier of the same relative path in the
//!   second root.
//! - The **relative identity** is the path below the root, normalised and
//!   case-folded according to the volume (`platform::identity_of_with`).

use blake3::Hasher;

use crate::platform::VolumeId;

/// The number of hexadecimal characters of an object identifier.
///
/// 128 bits of a cryptographic hash: more than enough to keep two objects apart,
/// short enough to keep a URL readable.
pub const IDENTIFIER_LENGTH: usize = 32;

/// The separator between the parts of the hashed input.
const SEPARATOR: u8 = 0x00;

/// Computes the identifier of an object.
///
/// The `root_key` distinguishes the roots of the configuration from one another;
/// it is the position of the root in the configuration, as text.
pub fn object_id(volume: &VolumeId, root_key: &str, identity: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(volume.as_str().as_bytes());
    hasher.update(&[SEPARATOR]);
    hasher.update(root_key.as_bytes());
    hasher.update(&[SEPARATOR]);
    hasher.update(identity.as_bytes());
    let hex = hasher.finalize().to_hex().to_string();
    hex[..IDENTIFIER_LENGTH].to_owned()
}

/// Whether the text has the shape of an object identifier.
///
/// The check is on the shape only: it says nothing about whether the object
/// exists, and it is never a substitute for looking the object up.
pub fn is_object_id(text: &str) -> bool {
    text.len() == IDENTIFIER_LENGTH && text.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{is_object_id, object_id, IDENTIFIER_LENGTH};
    use crate::platform::VolumeId;

    /// The identifier has the shape a client can rely on.
    #[test]
    fn the_identifier_has_the_expected_shape() {
        let identifier = object_id(&VolumeId::new("local"), "0", "Films/King Kong.mkv");
        assert_eq!(identifier.len(), IDENTIFIER_LENGTH);
        assert!(is_object_id(&identifier));
    }

    /// The same triple always gives the same identifier.
    #[test]
    fn the_identifier_is_stable() {
        let first = object_id(&VolumeId::new("local"), "0", "Films/King Kong.mkv");
        let second = object_id(&VolumeId::new("local"), "0", "Films/King Kong.mkv");
        assert_eq!(first, second);
    }

    /// Two different files never share an identifier.
    #[test]
    fn different_files_get_different_identifiers() {
        let first = object_id(&VolumeId::new("local"), "0", "Films/King Kong.mkv");
        let second = object_id(&VolumeId::new("local"), "0", "Films/King Kong (1999).mkv");
        assert_ne!(first, second);
    }

    /// Two roots on the same volume keep their files apart.
    #[test]
    fn different_roots_get_different_identifiers() {
        let first = object_id(&VolumeId::new("local"), "0", "King Kong.mkv");
        let second = object_id(&VolumeId::new("local"), "1", "King Kong.mkv");
        assert_ne!(first, second);
    }

    /// Two volumes keep the same relative path apart.
    #[test]
    fn different_volumes_get_different_identifiers() {
        let first = object_id(&VolumeId::new("local"), "0", "King Kong.mkv");
        let second = object_id(&VolumeId::new("other"), "0", "King Kong.mkv");
        assert_ne!(first, second);
    }

    /// The separator keeps the parts of the input apart: moving a character
    /// from one part to another must change the result.
    #[test]
    fn the_parts_of_the_input_cannot_be_shifted_into_one_another() {
        let first = object_id(&VolumeId::new("ab"), "0", "c");
        let second = object_id(&VolumeId::new("a"), "b", "c");
        assert_ne!(first, second);
    }

    /// The identifier leaks no part of the path it was made from.
    #[test]
    fn the_identifier_does_not_leak_the_path() {
        let identifier = object_id(&VolumeId::new("local"), "0", "Secret Folder/Film.mkv");
        assert!(!identifier.contains("Secret"));
        assert!(!identifier.contains("Film"));
    }

    /// The shape check rejects what is not an identifier.
    #[test]
    fn the_shape_check_rejects_other_text() {
        assert!(!is_object_id("0"));
        assert!(!is_object_id(""));
        assert!(!is_object_id(&"z".repeat(IDENTIFIER_LENGTH)));
    }
}
