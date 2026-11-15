//! What kind of media a file is.
//!
//! What: the map from a file name to a MIME type, the map from a MIME type to
//!       a kind of media, and the rule that decides whether a file belongs in
//!       the catalogue at all.
//! Why: the catalogue shows movies, music and photos. A file it cannot describe
//!       is a file a client would fetch and fail to play, so the catalogue keeps
//!       it out of the tree and reports it as an excluded file instead
//!       (`FUN-005`, `FUN-019`).
//! Example: `kind_of("audio/mpeg")` answers `MediaKind::MusicTrack`, and
//!       `mime_of_extension("mkv")` answers `video/x-matroska`.

use super::MediaKind;

/// The MIME type of a file, judged by its extension.
///
/// The extension is the first answer because it is cheap and because it is what
/// the person who named the file meant. A file whose extension says nothing is
/// sniffed (`catalogue::sniff`).
pub fn mime_of_extension(extension: &str) -> Option<&'static str> {
    match extension.to_ascii_lowercase().as_str() {
        "mp4" | "m4v" => Some("video/mp4"),
        "mkv" | "mk3d" => Some("video/x-matroska"),
        "webm" => Some("video/x-matroska"),
        "mpeg" | "mpg" | "m2v" | "ts" => Some("video/mpeg"),
        "mp3" => Some("audio/mpeg"),
        "m4a" | "m4b" => Some("audio/mp4"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "m3u" | "m3u8" => Some("audio/x-mpegurl"),
        "pls" => Some("audio/x-scpls"),
        _ => None,
    }
}

/// The kind of media a MIME type describes.
///
/// A playlist is not a track: it is a list of tracks, and the schema has no
/// class for it, so it keeps the generic class of the schema.
pub fn kind_of(mime_type: &str) -> MediaKind {
    if matches!(mime_type, "audio/x-mpegurl" | "audio/x-scpls") {
        return MediaKind::Other;
    }
    let family = mime_type.split('/').next().unwrap_or_default();
    match family {
        "audio" => MediaKind::MusicTrack,
        "video" => MediaKind::Movie,
        "image" => MediaKind::Photo,
        _ => MediaKind::Other,
    }
}

/// Whether a file of this MIME type belongs in the catalogue.
///
/// A playlist is indexed, although the server cannot deliver it: a client
/// shows it, and a client that asks for it gets a truthful answer
/// (`INT-064`). Everything the catalogue cannot place at all — a text file, a
/// subtitle, a disc image — stays out of the tree.
pub fn is_indexable(mime_type: &str) -> bool {
    matches!(
        kind_of(mime_type),
        MediaKind::MusicTrack | MediaKind::Movie | MediaKind::Photo
    ) || matches!(mime_type, "audio/x-mpegurl" | "audio/x-scpls")
}

#[cfg(test)]
mod tests {
    use super::{is_indexable, kind_of, mime_of_extension};
    use crate::catalogue::MediaKind;

    /// The extension of a film gives the format the server serves.
    #[test]
    fn the_extension_gives_the_format() {
        assert_eq!(mime_of_extension("mp4"), Some("video/mp4"));
        assert_eq!(mime_of_extension("mkv"), Some("video/x-matroska"));
        assert_eq!(mime_of_extension("MP3"), Some("audio/mpeg"));
        assert_eq!(mime_of_extension("jpg"), Some("image/jpeg"));
    }

    /// An extension nobody knows gives no answer.
    #[test]
    fn an_unknown_extension_gives_no_answer() {
        assert_eq!(mime_of_extension("txt"), None);
        assert_eq!(mime_of_extension(""), None);
    }

    /// The family of the format gives the kind of media.
    #[test]
    fn the_family_gives_the_kind() {
        assert_eq!(kind_of("audio/mpeg"), MediaKind::MusicTrack);
        assert_eq!(kind_of("video/mp4"), MediaKind::Movie);
        assert_eq!(kind_of("image/jpeg"), MediaKind::Photo);
        assert_eq!(kind_of("audio/x-mpegurl"), MediaKind::Other);
    }

    /// A file the catalogue cannot place stays out of the tree.
    #[test]
    fn a_file_the_catalogue_cannot_place_stays_out() {
        assert!(!is_indexable("text/plain"));
        assert!(!is_indexable("application/octet-stream"));
    }

    /// A playlist is indexed, although it cannot be delivered.
    #[test]
    fn a_playlist_is_indexed() {
        assert!(is_indexable("audio/x-mpegurl"));
        assert!(is_indexable("audio/x-scpls"));
    }
}
