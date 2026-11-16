//! The tests of the path model.
//!
//! What: the rules of the identity — the folding of the case, the
//!       normalisation of the Unicode form, and the fact that the raw path is
//!       never replaced by the identity.
//! Why: a media server that keys its catalogue on a raw path breaks the moment
//!       the file system hands the path back in another Unicode form, which on
//!       macOS it does (ADR-0007).
//! Example: two spellings of one name produce one identity.

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
