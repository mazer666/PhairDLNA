//! The keys that say "the same file".
//!
//! What: the two keys of a file — the **identity key**, which says "the same
//!       path, seen again", and the **uniqueness key**, which says "the same
//!       file".
//! Why: a media server sees the same file twice more often than one would
//!       think. The scanner walks two roots that overlap, the watcher reports
//!       a file that the scan has just seen, and macOS hands the same file
//!       back under a different Unicode spelling (ADR-0007). Without a key
//!       that says "already known", the catalogue lists a film twice and the
//!       client plays one of the copies.
//! Example:
//!
//! ```rust
//! use std::path::Path;
//!
//! use phairdlna_core::catalogue::identity::{identity_key, uniqueness_key};
//! use phairdlna_core::platform::{CaseSensitivity, FileId, VolumeId};
//!
//! let volume = VolumeId::new("local");
//!
//! let key = identity_key(&volume, Path::new("/Films/King Kong.mkv"), CaseSensitivity::Insensitive);
//! assert!(key.starts_with("local\0"));
//! assert!(key.ends_with("/films/king kong.mkv"));
//!
//! let file = FileId { volume, device: 16777220, inode: 42 };
//! assert_eq!(uniqueness_key(&file), "local\u{0}16777220\u{0}42");
//! ```

use std::path::Path;

use crate::platform::{identity_of_with, CaseSensitivity, FileId, VolumeId};

/// The separator between the parts of a key.
///
/// A zero byte cannot occur inside a path component on the platforms the
/// server runs on, and the volume identity is opaque text the core never
/// interprets. Two different pairs of parts therefore cannot produce the same
/// key.
const SEPARATOR: &str = "\u{0}";

/// The identity key of a file: the value that says "the same path, seen again".
///
/// The key is scoped by the volume the file lives on: the same relative path
/// on two volumes is two different files.
///
/// The path part is the identity of the path — normalised to Unicode NFC and
/// case-folded when the volume is case-insensitive (`platform::identity_of_with`).
/// The volume part is the opaque identity of the volume. The two are joined by
/// a zero byte, so the key is stable for the same triple and different for
/// every other one.
///
/// The key does not say whether the file is *there*: it is computed from the
/// volume and the path alone, without touching the filesystem.
pub fn identity_key(volume: &VolumeId, path: &Path, sensitivity: CaseSensitivity) -> String {
    let identity = identity_of_with(path, sensitivity);
    join(&[volume.as_str(), identity.as_str()])
}

/// The identity of a file that shares its identity with another file.
///
/// The identity of a path is normalised to Unicode NFC, so two names that
/// differ only in the normalisation form share one identity on every volume.
/// On a volume that keeps the bytes they are **two files**, and the platform
/// tells them apart (`FUN-020`, `PLT-005`). Such a file keeps the identity
/// the platform gives it, so that the two files have two object identifiers
/// and neither of them is lost.
///
/// The answer is stable: the identity of the platform does not change while
/// the file is there.
pub fn disambiguated_identity(identity: &str, file: &FileId) -> String {
    join(&[identity, &format!("{:x}:{:x}", file.device, file.inode)])
}

/// The uniqueness key of a file: the value that says "the same file".
///
/// Uniqueness comes from the stable identity the platform gives the file, not
/// from its path: a file that is reached under two names, or under two roots,
/// is one file.
///
/// The key is the volume, the device and the inode, joined by a zero byte. Two
/// `FileId`s that describe the same file therefore give the same key, and two
/// that describe different files give different keys — even when the two live
/// on different volumes and happen to carry the same device and inode
/// numbers.
///
/// A `FileId` derives `Hash` and can be used as a map key directly. This
/// function exists for the cases where the key has to be a *string*: a log
/// line, a diagnostic, a report.
pub fn uniqueness_key(file: &FileId) -> String {
    let device = file.device.to_string();
    let inode = file.inode.to_string();
    join(&[file.volume.as_str(), device.as_str(), inode.as_str()])
}

/// Joins the parts of a key with the separator.
fn join(parts: &[&str]) -> String {
    parts.join(SEPARATOR)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{disambiguated_identity, identity_key, uniqueness_key, SEPARATOR};
    use crate::platform::{identity_of_with, CaseSensitivity, FileId, VolumeId};

    /// The same volume and the same path always give the same key.
    #[test]
    fn the_same_path_gives_the_same_key() {
        let volume = VolumeId::new("local");
        let first = identity_key(
            &volume,
            Path::new("Films/King Kong.mkv"),
            CaseSensitivity::Sensitive,
        );
        let second = identity_key(
            &volume,
            Path::new("Films/King Kong.mkv"),
            CaseSensitivity::Sensitive,
        );
        assert_eq!(first, second);
    }

    /// Two volumes keep the same relative path apart: the key is scoped by the
    /// volume, because the same path on two volumes is two different files.
    #[test]
    fn two_volumes_give_two_keys() {
        let path = Path::new("Films/King Kong.mkv");
        let first = identity_key(&VolumeId::new("local"), path, CaseSensitivity::Sensitive);
        let second = identity_key(&VolumeId::new("other"), path, CaseSensitivity::Sensitive);
        assert_ne!(first, second);
    }

    /// On a case-insensitive volume, two spellings of one name are one file.
    #[test]
    fn a_case_insensitive_volume_folds_the_case() {
        let volume = VolumeId::new("local");
        let first = identity_key(&volume, Path::new("Film.mkv"), CaseSensitivity::Insensitive);
        let second = identity_key(&volume, Path::new("film.mkv"), CaseSensitivity::Insensitive);
        assert_eq!(first, second);
    }

    /// On a case-sensitive volume, two spellings of one name are two files.
    #[test]
    fn a_case_sensitive_volume_keeps_the_case() {
        let volume = VolumeId::new("local");
        let first = identity_key(&volume, Path::new("Film.mkv"), CaseSensitivity::Sensitive);
        let second = identity_key(&volume, Path::new("film.mkv"), CaseSensitivity::Sensitive);
        assert_ne!(first, second);
    }

    /// The key is the identity of the path joined to the volume — and not the
    /// identity of the path alone: the key alone would merge the same path on
    /// two volumes.
    #[test]
    fn the_key_is_the_path_identity_joined_to_the_volume() {
        let volume = VolumeId::new("local");
        let path = Path::new("Films/King Kong.mkv");
        let identity = identity_of_with(path, CaseSensitivity::Sensitive);
        let key = identity_key(&volume, path, CaseSensitivity::Sensitive);
        assert_eq!(key, format!("local{SEPARATOR}{identity}"));
        assert_ne!(key, identity);
    }

    /// The separator keeps the parts of the key apart: moving a character from
    /// the volume into the path must change the key.
    #[test]
    fn the_parts_of_the_key_cannot_be_shifted_into_one_another() {
        let first = identity_key(
            &VolumeId::new("ab"),
            Path::new("c"),
            CaseSensitivity::Sensitive,
        );
        let second = identity_key(
            &VolumeId::new("a"),
            Path::new("b/c"),
            CaseSensitivity::Sensitive,
        );
        assert_ne!(first, second);
    }

    /// Two descriptions of one file give one uniqueness key, and two files
    /// give two keys.
    #[test]
    fn the_uniqueness_key_says_which_file_it_is() {
        let volume = VolumeId::new("local");
        let first = FileId {
            volume: volume.clone(),
            device: 16777220,
            inode: 42,
        };
        let same = FileId {
            volume,
            device: 16777220,
            inode: 42,
        };
        let other = FileId {
            volume: VolumeId::new("local"),
            device: 16777220,
            inode: 43,
        };
        assert_eq!(uniqueness_key(&first), uniqueness_key(&same));
        assert_ne!(uniqueness_key(&first), uniqueness_key(&other));
    }

    /// Two files that share their identity keep two identities of their own.
    ///
    /// The identity of a path is normalised to Unicode NFC, so two names that
    /// differ only in the normalisation form share one identity — but they are
    /// two files on a volume that keeps the bytes (`FUN-020`, `PLT-005`).
    #[test]
    fn two_files_that_share_an_identity_keep_two_identities() {
        let first = FileId {
            volume: VolumeId::new("local"),
            device: 16777220,
            inode: 42,
        };
        let second = FileId {
            volume: VolumeId::new("local"),
            device: 16777220,
            inode: 43,
        };

        assert_ne!(
            disambiguated_identity("cafe\u{301}.mkv", &first),
            disambiguated_identity("cafe\u{301}.mkv", &second),
            "two files keep two identities"
        );
        assert_eq!(
            disambiguated_identity("cafe\u{301}.mkv", &first),
            disambiguated_identity("cafe\u{301}.mkv", &first),
            "one file keeps one identity"
        );
        assert!(
            disambiguated_identity("cafe\u{301}.mkv", &first).starts_with("cafe\u{301}.mkv"),
            "the identity of the path stays the beginning of the answer"
        );
    }

    /// The device and the inode numbers of one volume say nothing about
    /// another volume: the same numbers on two volumes are two files.
    #[test]
    fn the_uniqueness_key_keeps_two_volumes_apart() {
        let first = FileId {
            volume: VolumeId::new("local"),
            device: 16777220,
            inode: 42,
        };
        let second = FileId {
            volume: VolumeId::new("other"),
            device: 16777220,
            inode: 42,
        };
        assert_ne!(uniqueness_key(&first), uniqueness_key(&second));
    }
}
