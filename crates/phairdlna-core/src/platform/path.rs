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

#[cfg(test)]
mod tests;

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

/// What the platform observes at a path right now.
///
/// The fingerprint is **not** the identity of the volume. The identity must
/// survive a remount, so that an object identifier stays the same
/// (`INT-062`); the fingerprint describes what is mounted at the path at this
/// moment, so that the server can notice that a drive was unplugged, was
/// plugged in again or was replaced by another one (`FUN-007`, `FUN-017`).
///
/// The portable adapter reports the device the path lives on. A platform that
/// can tell volumes apart reports a stronger fingerprint — the serial number of
/// the volume, for instance (`PLT-005`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VolumeFingerprint(String);

impl VolumeFingerprint {
    /// Wraps a platform-supplied fingerprint.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// The platform fingerprint, as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for VolumeFingerprint {
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

/// Whether a path is a folder that the catalogue can read.
///
/// The path is resolved first: a path that cannot be resolved is not a folder,
/// and a folder that cannot be resolved is not there (`FUN-007`).
///
/// The rule is the rule of the scanner: a folder that exists but that the
/// operating system refuses to open is a folder that is there — the scanner
/// reports it and asks for consent (`FUN-004`, `PLT-003`).
pub fn is_readable_folder(path: &Path) -> bool {
    std::fs::canonicalize(path).is_ok_and(|resolved| resolved.is_dir())
}

/// The display name of a path: its last component, normalised for reading.
pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}
