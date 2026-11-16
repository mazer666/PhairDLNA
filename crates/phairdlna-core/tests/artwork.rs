//! The artwork of the library.
//!
//! What: the tests of the pictures of the media files — the cover that is
//!       embedded in a track, the image that lies next to it, and the
//!       thumbnail of a photo — and the tests of the files that are broken.
//! Why: the picture is the first thing a control point asks for, and a broken
//!       file must not take the library down with it. The tests therefore
//!       check both the happy path and the damage (`F-10`, `SEC-067`, `UX-004`).
//! Example: a track with a cover in its tag gets the cover, and a track whose
//!       tag is broken gets no picture and no panic.

mod support;

use std::fs;
use std::path::Path;

use phairdlna_core::config::SymlinkPolicy;
use phairdlna_core::media::{cache_key, ArtworkFinder, ArtworkSource, PictureLimits, RootBoundary};
use support::artwork::{jpeg, m4a_with_cover, mp3_with_cover, png};
use tempfile::tempdir;

/// Builds a finder for a root.
fn finder(root: &Path) -> ArtworkFinder {
    let boundary = RootBoundary {
        path: fs::canonicalize(root).expect("the root is there"),
        symlink_policy: SymlinkPolicy::Skip,
    };
    ArtworkFinder::new(vec![boundary], PictureLimits::default())
}

/// Writes a file into a folder and gives its path back.
fn write(folder: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = folder.join(name);
    fs::write(&path, bytes).expect("the file is written");
    path
}

/// The cover of a track that is embedded in its tag is found.
#[test]
fn an_embedded_cover_is_found() {
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(16, 16);
    let path = write(
        folder.path(),
        "song.mp3",
        &mp3_with_cover("image/jpeg", &picture),
    );
    let finder = finder(folder.path());

    let source = finder
        .source_of("a3f1", &path)
        .expect("the track has a picture");

    assert_eq!(
        source,
        ArtworkSource::Embedded {
            mime_type: "image/jpeg".to_owned()
        }
    );
    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");
    assert_eq!(found.mime_type, "image/jpeg");
    assert_eq!(found.bytes, picture);
}

/// The cover of an M4A file is found as well.
#[test]
fn an_embedded_cover_of_a_m4a_is_found() {
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(8, 8);
    let path = write(folder.path(), "song.m4a", &m4a_with_cover(13, &picture));
    let finder = finder(folder.path());

    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");

    assert_eq!(found.mime_type, "image/jpeg");
    assert_eq!(found.bytes, picture);
}

/// The image that lies next to the file wins over the cover in the tag.
#[test]
fn a_sidecar_wins_over_the_embedded_cover() {
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(12, 12);
    let path = write(
        folder.path(),
        "song.mp3",
        &mp3_with_cover("image/jpeg", &jpeg(4, 4)),
    );
    write(folder.path(), "cover.jpg", &picture);
    let finder = finder(folder.path());

    let source = finder
        .source_of("a3f1", &path)
        .expect("the track has a picture");

    assert!(matches!(source, ArtworkSource::Sidecar { .. }));
    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");
    assert_eq!(found.bytes, picture);
}

/// The image that carries the name of the file is a sidecar as well.
#[test]
fn a_sidecar_may_carry_the_name_of_the_file() {
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(10, 10);
    let path = write(folder.path(), "song.mp3", b"no tag at all");
    write(folder.path(), "song.jpg", &picture);
    let finder = finder(folder.path());

    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");

    assert_eq!(found.bytes, picture);
}

/// A track without a picture has no picture, and the catalogue says so.
#[test]
fn a_track_without_a_picture_has_none() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "song.mp3", b"no tag at all");
    let finder = finder(folder.path());

    assert_eq!(finder.source_of("a3f1", &path), None);
    assert!(finder.picture_of("a3f1", &path).is_none());
}

/// A file that is broken does not stop the library and gives no picture.
#[test]
fn a_broken_file_gives_no_picture_and_no_panic() {
    let folder = tempdir().expect("a temporary directory");
    let broken = write(folder.path(), "broken.mp3", &[0xff, 0xfe, 0x00, 0x01, 0x02]);
    let finder = finder(folder.path());

    assert_eq!(finder.source_of("a3f1", &broken), None);
    assert!(finder.picture_of("a3f1", &broken).is_none());
}

/// A tag that promises more than the file holds is refused.
#[test]
fn a_tag_that_lies_about_its_size_is_refused() {
    let folder = tempdir().expect("a temporary directory");
    let mut bytes = mp3_with_cover("image/jpeg", &jpeg(4, 4));
    bytes.truncate(20);
    let path = write(folder.path(), "liar.mp3", &bytes);
    let finder = finder(folder.path());

    assert_eq!(finder.source_of("a3f1", &path), None);
}

/// A photo is its own picture, and the picture is a thumbnail of it.
#[test]
fn a_photo_is_its_own_picture() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "photo.jpg", &jpeg(64, 48));
    let finder = finder(folder.path());

    let source = finder
        .source_of("a3f1", &path)
        .expect("the photo has a picture");

    assert_eq!(
        source,
        ArtworkSource::Thumbnail {
            mime_type: "image/jpeg".to_owned()
        }
    );
    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");
    assert_eq!(found.mime_type, "image/jpeg");
    assert!(!found.bytes.is_empty());
}

/// A photo that is not a picture at all gives no picture and no panic.
#[test]
fn a_broken_photo_gives_no_picture_and_no_panic() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "broken.jpg", b"this is not a picture");
    let finder = finder(folder.path());

    let source = finder.source_of("a3f1", &path);
    assert_eq!(
        source,
        Some(ArtworkSource::Thumbnail {
            mime_type: "image/jpeg".to_owned()
        })
    );
    assert!(finder.picture_of("a3f1", &path).is_none());
}

/// A sidecar that leads out of the root is not a picture of the library.
#[test]
fn a_sidecar_outside_the_root_is_not_used() {
    let outside = tempdir().expect("a temporary directory");
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(8, 8);
    let target = write(outside.path(), "secret.jpg", &picture);
    let path = write(folder.path(), "song.mp3", b"no tag at all");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, folder.path().join("cover.jpg")).expect("the link is made");
    let finder = finder(folder.path());

    assert_eq!(finder.source_of("a3f1", &path), None);
}

/// A file that lies outside the root has no picture at all.
#[test]
fn a_file_outside_the_root_has_no_picture() {
    let outside = tempdir().expect("a temporary directory");
    let folder = tempdir().expect("a temporary directory");
    let picture = jpeg(8, 8);
    let path = write(
        outside.path(),
        "song.mp3",
        &mp3_with_cover("image/jpeg", &picture),
    );
    let finder = finder(folder.path());

    assert_eq!(finder.source_of("a3f1", &path), None);
}

/// A file that changed gets a new key, so that the old picture is never used.
#[test]
fn a_file_that_changed_gets_a_new_key() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "song.mp3", b"the first version");
    let before = cache_key("a3f1", &path);

    std::thread::sleep(std::time::Duration::from_millis(10));
    write(
        folder.path(),
        "song.mp3",
        b"the second version, and it is longer",
    );
    let after = cache_key("a3f1", &path);

    assert_ne!(before, after, "the key of a changed file is the same");
}

/// The key of a picture carries no path of the disk.
#[test]
fn the_key_of_a_picture_carries_no_path() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "song.mp3", b"a track");
    let key = cache_key("a3f1", &path);

    assert!(!key.contains('/'), "the key names a folder: {key}");
    assert!(
        key.chars()
            .all(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit()),
        "the key is not a plain name: {key}"
    );
}

/// An embedded cover that is larger than the limit is not read.
#[test]
fn an_embedded_cover_that_is_too_large_is_refused() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(
        folder.path(),
        "song.mp3",
        &mp3_with_cover("image/jpeg", &jpeg(8, 8)),
    );
    let boundary = RootBoundary {
        path: fs::canonicalize(folder.path()).expect("the root is there"),
        symlink_policy: SymlinkPolicy::Skip,
    };
    let limits = PictureLimits {
        max_embedded_bytes: 8,
        ..PictureLimits::default()
    };
    let finder = ArtworkFinder::new(vec![boundary], limits);

    assert_eq!(finder.source_of("a3f1", &path), None);
    assert!(finder.picture_of("a3f1", &path).is_none());
}

/// The sidecar of a file keeps its own type.
#[test]
fn a_sidecar_keeps_its_own_type() {
    let folder = tempdir().expect("a temporary directory");
    let path = write(folder.path(), "song.mp3", b"no tag at all");
    write(folder.path(), "cover.png", &png(8, 8));
    let finder = finder(folder.path());

    let found = finder
        .picture_of("a3f1", &path)
        .expect("the picture is there");

    assert_eq!(found.mime_type, "image/png");
}
