//! The path model of the core.
//!
//! What: the four facts about a path that the core needs — the raw path, the
//!       normalised identity, the volume and the display name — and the rules
//!       for computing them.
//! Why: a media server that keys its catalogue on a raw path breaks the moment
//!       the file system hands the path back in a different Unicode form, which
//!       on macOS it does (ADR-0007).
//! Example: two spellings of the same file on an APFS volume produce one
//!      identity and therefore one catalogue entry.

use std::path::{Path, PathBuf};

use unicase::UniCase;
use unicode_normalization::UnicodeNormalization;

/// The identity of a volume: opaque, stable, and comparable across mounts.
///
/// The content is whatever the platform uses — a device id on Unix, a volume serial
/// number on Windows. The core never interprets it; it only compares it and uses it
/// to scope an identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VolumeId(String);

impl VolumeId {
    /// Wraps a platform-supplied identifier.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// The platform identifier, as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for VolumeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// The identity of a file: the thing that says "the same file".
///
/// On Unix this is the device and inode pair; on Windows the volume serial and the
/// file index. Two paths with the same identity are one file, whatever they are
/// called.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId {
    /// The volume the file lives on.
    pub volume: VolumeId,
    /// The platform's stable identifier within that volume.
    pub device: u64,
    /// The platform's stable identifier of the file within that volume.
    pub inode: u64,
}

/// Whether a volume distinguishes `Film.mkv` from `film.mkv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseSensitivity {
    /// The volume treats names that differ only in case as the same file. This is
    /// the default on macOS and on Windows.
    #[default]
    Insensitive,
    /// The volume treats them as different files. This is the usual case on Linux.
    Sensitive,
}

/// A path, with everything the core needs to know about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPath {
    /// The path exactly as the operating system reported it. Used verbatim for
    /// every file operation, and never normalised.
    raw: PathBuf,
    /// The normalised identity: the matching and de-duplication key.
    identity: String,
    /// The volume the path lives on.
    volume: VolumeId,
    /// The name shown to a person.
    display: String,
}

impl MediaPath {
    /// Builds a path from its parts.
    pub fn new(
        raw: impl Into<PathBuf>,
        identity: impl Into<String>,
        volume: VolumeId,
        display: impl Into<String>,
    ) -> Self {
        Self {
            raw: raw.into(),
            identity: identity.into(),
            volume,
            display: display.into(),
        }
    }

    /// The raw path, verbatim.
    pub fn raw(&self) -> &Path {
        &self.raw
    }

    /// The normalised identity.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The volume the path lives on.
    pub fn volume(&self) -> &VolumeId {
        &self.volume
    }

    /// The name shown to a person.
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The raw path as an operating-system string.
    pub fn raw_os(&self) -> &std::ffi::OsStr {
        self.raw.as_os_str()
    }
}

/// Computes the identity of a path relative to a root.
///
/// The identity is the path with every component normalised to Unicode NFC and then
/// case-folded, joined with `/`. The root is **not** part of it: the identity is
/// scoped by the volume instead, because the same relative path on two volumes is
/// two different files.
///
/// The case folding is the full Unicode case folding, not a lowercase mapping. That
/// matters: on a case-insensitive volume, `Straße` and `STRASSE` are the same file,
/// and a simple lowercase mapping would treat them as two.
///
/// Use [`identity_of_with`] when the case sensitivity of the volume is known: on a
/// case-sensitive volume two names that differ only in case are two different
/// files, and folding them would merge them into one (`PLT-005`). **The catalogue
/// uses `identity_of_with`; this function is the case-insensitive variant and is
/// kept for callers that do not know the volume.**
pub fn identity_of(relative: &Path) -> String {
    identity_of_with(relative, CaseSensitivity::Insensitive)
}

/// Computes the identity of a path relative to a root, for a known volume.
///
/// The identity is the path with every component normalised to Unicode NFC, then
/// case-folded **only when the volume is case-insensitive**, joined with `/`.
///
/// The case folding is the full Unicode case folding, not a lowercase mapping. That
/// matters: on a case-insensitive volume, `Straße` and `STRASSE` are the same file,
/// and a simple lowercase mapping would treat them as two.
pub fn identity_of_with(relative: &Path, sensitivity: CaseSensitivity) -> String {
    relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            std::path::Component::RootDir => Some(String::new()),
            _ => None,
        })
        .map(|part| fold_component(&part, sensitivity))
        .collect::<Vec<_>>()
        .join("/")
}

/// Normalises and case-folds one component.
fn fold_component(component: &str, sensitivity: CaseSensitivity) -> String {
    let normalised: String = component.nfc().collect();
    match sensitivity {
        CaseSensitivity::Sensitive => normalised,
        CaseSensitivity::Insensitive => {
            let folded = UniCase::new(normalised.as_str()).to_folded_case();
            folded.nfc().collect()
        }
    }
}

/// The display name of a path: its last component, normalised for reading.
pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::{identity_of, CaseSensitivity, FileId, MediaPath, VolumeId};

    /// The identity is the same for two spellings of the same name.
    #[test]
    fn the_identity_ignores_case() {
        assert_eq!(identity_of(std::path::Path::new("Film.mkv")), "film.mkv");
        assert_eq!(
            identity_of(std::path::Path::new("FILM.MKV")),
            identity_of(std::path::Path::new("film.mkv"))
        );
    }

    /// The full case folding maps the German sharp s to "ss", which a simple
    /// lowercase mapping would not do.
    #[test]
    fn the_identity_uses_full_case_folding() {
        assert_eq!(
            identity_of(std::path::Path::new("Straße.mkv")),
            "strasse.mkv"
        );
        assert_eq!(
            identity_of(std::path::Path::new("STRASSE.mkv")),
            identity_of(std::path::Path::new("Straße.mkv"))
        );
    }

    /// The identity is normalised, so the two Unicode spellings of "café" agree.
    #[test]
    fn the_identity_normalises_unicode() {
        let composed = std::path::Path::new("caf\u{e9}.mkv");
        let decomposed = std::path::Path::new("cafe\u{301}.mkv");
        assert_eq!(identity_of(composed), identity_of(decomposed));
    }

    /// Two components are folded separately, so a separator inside a name cannot
    /// imitate a directory boundary.
    #[test]
    fn components_are_folded_separately() {
        assert_eq!(
            identity_of(std::path::Path::new("a/b")),
            identity_of(std::path::Path::new("A/b"))
        );
        assert_ne!(
            identity_of(std::path::Path::new("a/b")),
            identity_of(std::path::Path::new("ab"))
        );
    }

    /// The raw path is kept verbatim; it is never replaced by the identity.
    #[test]
    fn the_raw_path_is_kept_verbatim() {
        let raw = std::path::PathBuf::from("cafe\u{301}.mkv");
        let path = MediaPath::new(raw.clone(), "café.mkv", VolumeId::new("v1"), "café.mkv");
        assert_eq!(path.raw(), raw.as_path());
        assert_eq!(path.identity(), "café.mkv");
    }

    /// The file identity, not the path, says whether two entries are one file.
    #[test]
    fn the_file_identity_is_separate_from_the_path() {
        let first = FileId {
            volume: VolumeId::new("v1"),
            device: 1,
            inode: 42,
        };
        let second = FileId {
            volume: VolumeId::new("v1"),
            device: 1,
            inode: 42,
        };
        assert_eq!(first, second);
        assert_ne!(
            first,
            FileId {
                volume: VolumeId::new("v1"),
                device: 1,
                inode: 43
            }
        );
    }

    /// The default case sensitivity is the one macOS and Windows use.
    #[test]
    fn the_default_case_sensitivity_is_insensitive() {
        assert_eq!(CaseSensitivity::default(), CaseSensitivity::Insensitive);
    }
}
