//! The images that lie next to a media file.
//!
//! What: the names a sidecar image may have, and the order in which they are
//!      tried.
//! Why: a sidecar is found by name, and the name is a convention of the
//!      community rather than a rule of a standard. The list is therefore
//!      short, ordered and closed: a file that is not on the list is never
//!      used, so that a folder full of images cannot turn into a folder full
//!      of pictures (`F-10` § 3).
//! Example: `candidates` returns the paths of `song.jpg` and `cover.jpg` for
//!       `song.mp3`, in that order.
//!
//! **The order.** The name of the media file comes first, because it is the
//! most precise answer; the names of the folder come next, because they are
//! the answer of the folder (`F-10` § 3).

use std::path::{Path, PathBuf};

/// The names a sidecar image may have beside a media file.
///
/// The names are tried in this order, and each name is tried with every
/// extension of [`IMAGE_EXTENSIONS`].
pub const SIDECAR_NAMES: [&str; 5] = ["cover", "folder", "album", "front", "artwork"];

/// The extensions a sidecar image may have.
pub const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

/// The paths of the sidecar candidates of a media file, in the order they are
/// tried.
///
/// The first candidate replaces the name of the media file with the name of an
/// image (`song.mp3` becomes `song.jpg`); the other candidates carry the names
/// of [`SIDECAR_NAMES`].
pub fn candidates(media: &Path) -> Vec<PathBuf> {
    let Some(folder) = media.parent() else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    if let Some(stem) = media.file_stem().and_then(|stem| stem.to_str()) {
        for extension in IMAGE_EXTENSIONS {
            paths.push(folder.join(format!("{stem}.{extension}")));
        }
    }
    for name in SIDECAR_NAMES {
        for extension in IMAGE_EXTENSIONS {
            paths.push(folder.join(format!("{name}.{extension}")));
        }
    }
    paths
}

/// The type of an image file, judged by its name.
///
/// A name that is not one of [`IMAGE_EXTENSIONS`] has no type: the file is not
/// an image the server can serve.
pub fn mime_type_of(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{candidates, mime_type_of, IMAGE_EXTENSIONS, SIDECAR_NAMES};
    use std::path::Path;

    #[test]
    fn the_name_of_the_media_file_comes_first() {
        let paths = candidates(Path::new("/music/song.mp3"));
        assert_eq!(paths[0], Path::new("/music/song.jpg"));
        assert_eq!(paths[3], Path::new("/music/song.webp"));
    }

    #[test]
    fn the_names_of_the_folder_follow() {
        let paths = candidates(Path::new("/music/song.mp3"));
        let names: Vec<String> = paths
            .iter()
            .filter_map(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        for name in SIDECAR_NAMES {
            assert!(names.contains(&format!("{name}.jpg")), "{name} is missing");
        }
        assert_eq!(
            paths.len(),
            IMAGE_EXTENSIONS.len() * (SIDECAR_NAMES.len() + 1)
        );
    }

    #[test]
    fn a_relative_file_gets_relative_candidates() {
        let paths = candidates(Path::new("song.mp3"));
        assert_eq!(paths[0], Path::new("song.jpg"));
        assert!(
            paths
                .iter()
                .all(|path| path.parent().is_none_or(Path::is_empty)),
            "a candidate left the folder of the file: {paths:?}"
        );
    }

    #[test]
    fn the_type_follows_the_extension() {
        assert_eq!(mime_type_of(Path::new("a.jpg")), Some("image/jpeg"));
        assert_eq!(mime_type_of(Path::new("a.JPEG")), Some("image/jpeg"));
        assert_eq!(mime_type_of(Path::new("a.png")), Some("image/png"));
        assert_eq!(mime_type_of(Path::new("a.txt")), None);
        assert_eq!(mime_type_of(Path::new("a")), None);
    }
}
