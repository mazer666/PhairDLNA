//! The subtitle sidecars that lie next to a media file.
//!
//! What: the subtitle formats the server knows, the MIME type it sends for
//!       each of them, and the rule that finds the sidecar of a media file in
//!       its folder.
//! Why: a client cannot see a subtitle that merely lies next to the film —
//!       it has to be served as an item of its own. One place must therefore
//!       say which files are subtitles, how they are named and which one
//!       belongs to which film (`F-11`).
//! Example:
//!
//! ```
//! use phairdlna_core::catalogue::subtitle::{sidecar_of, SubtitleFormat};
//! use std::collections::HashSet;
//!
//! let folder: HashSet<String> = ["Nightfall.mkv", "Nightfall.vtt"]
//!     .into_iter()
//!     .map(str::to_string)
//!     .collect();
//! let sidecar = sidecar_of("Nightfall", &folder);
//!
//! assert_eq!(sidecar.as_ref().map(|s| s.format), Some(SubtitleFormat::Vtt));
//! assert_eq!(sidecar.as_ref().map(|s| s.mime_type.as_str()), Some("text/vtt"));
//! ```

use std::collections::HashSet;
use std::path::Path;

/// The extensions of the subtitle formats, in the order of priority.
///
/// WebVTT comes first because it is the format a browser-based client reads
/// without a conversion; SubRip comes next because it is the format films are
/// usually shipped with.
pub const SUBTITLE_EXTENSIONS: [&str; 5] = ["vtt", "srt", "ass", "sub", "smi"];

/// The subtitle formats the server knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubtitleFormat {
    /// WebVTT — the format the web uses, the one a DLNA client is most likely
    /// to read.
    Vtt,
    /// SubRip — the format most films come with.
    Srt,
    /// Advanced SubStation Alpha.
    Ass,
    /// MicroDVD — a subtitle that carries its own timings.
    Sub,
    /// SMIL — the subtitle markup of the web.
    Smil,
}

impl SubtitleFormat {
    /// The format of a file, judged by its extension (case-insensitive).
    ///
    /// The extension is what the person who named the file meant, and it is
    /// the only thing this module is allowed to look at — it never reads the
    /// file.
    pub fn of_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "vtt" => Some(Self::Vtt),
            "srt" => Some(Self::Srt),
            "ass" => Some(Self::Ass),
            "sub" => Some(Self::Sub),
            "smi" => Some(Self::Smil),
            _ => None,
        }
    }

    /// The MIME type the server sends for the format.
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Vtt => "text/vtt",
            Self::Srt => "application/x-subrip",
            Self::Ass => "text/x-ssa",
            Self::Sub => "text/x-microdvd",
            Self::Smil => "application/smil+xml",
        }
    }

    /// The extension the format is written with, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Vtt => "vtt",
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Sub => "sub",
            Self::Smil => "smi",
        }
    }
}

/// One subtitle sidecar that lies next to a media file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleSidecar {
    /// The name of the file, as the owner wrote it.
    pub name: String,
    /// The format of the file.
    pub format: SubtitleFormat,
    /// The MIME type the server sends for the file.
    pub mime_type: String,
}

/// The sidecar of a media file, when the folder holds one.
///
/// `stem` is the name of the media file **without** its extension (e.g.
/// `Nightfall` for `Nightfall.mkv`); `siblings` is the set of the names of the
/// files that lie in the same folder.
///
/// When the folder holds more than one sidecar for the media file, the format
/// decides: the first format of [`SUBTITLE_EXTENSIONS`] that is there wins.
/// The name that is answered is the one on disk, never a rewritten spelling,
/// because the catalogue has to be able to find the file again.
pub fn sidecar_of(stem: &str, siblings: &HashSet<String>) -> Option<SubtitleSidecar> {
    SUBTITLE_EXTENSIONS.iter().find_map(|extension| {
        let format = SubtitleFormat::of_extension(extension)?;
        let name = pick_sibling(stem, extension, siblings)?;
        Some(SubtitleSidecar {
            name,
            format,
            mime_type: format.mime_type().to_string(),
        })
    })
}

/// The sibling that is the sidecar of `stem` in the given extension.
///
/// The set of names has no order of its own, so the smallest name wins: the
/// answer must not depend on the order the filesystem handed the names over.
fn pick_sibling(stem: &str, extension: &str, siblings: &HashSet<String>) -> Option<String> {
    siblings
        .iter()
        .filter(|name| is_sidecar_name(name, stem, extension))
        .min()
        .cloned()
}

/// Whether the name is the sidecar of `stem` in the given extension.
///
/// The name is split at its last dot: the part before it must be the stem of
/// the media file, the part after it the extension. A name that merely
/// contains the stem — `Nightfall.forced.srt` — is therefore not a sidecar of
/// `Nightfall`.
fn is_sidecar_name(name: &str, stem: &str, extension: &str) -> bool {
    let Some((name_stem, name_extension)) = name.rsplit_once('.') else {
        return false;
    };
    name_extension.eq_ignore_ascii_case(extension) && name_stem.eq_ignore_ascii_case(stem)
}

/// The format of a subtitle file, judged by its name.
pub fn format_of(path: &Path) -> Option<SubtitleFormat> {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(SubtitleFormat::of_extension)
}

/// The MIME type the server sends for a subtitle file.
pub fn mime_type_of(path: &Path) -> Option<&'static str> {
    format_of(path).map(SubtitleFormat::mime_type)
}

#[cfg(test)]
mod tests {
    use super::{
        format_of, mime_type_of, sidecar_of, SubtitleFormat, SubtitleSidecar, SUBTITLE_EXTENSIONS,
    };
    use std::collections::HashSet;
    use std::path::Path;

    /// The names of the files of a folder, as the scanner hands them over.
    fn folder(names: &[&str]) -> HashSet<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    /// The format of the file at the given path.
    fn format_at(path: &str) -> Option<SubtitleFormat> {
        format_of(Path::new(path))
    }

    /// The MIME type of the file at the given path.
    fn mime_at(path: &str) -> Option<&'static str> {
        mime_type_of(Path::new(path))
    }

    /// The extension of a file gives its format.
    #[test]
    fn the_extension_gives_the_format() {
        let srt = SubtitleFormat::Srt;
        let vtt = SubtitleFormat::Vtt;
        let ass = SubtitleFormat::Ass;
        let sub = SubtitleFormat::Sub;

        assert_eq!(SubtitleFormat::of_extension("srt"), Some(srt));
        assert_eq!(SubtitleFormat::of_extension("vtt"), Some(vtt));
        assert_eq!(SubtitleFormat::of_extension("ass"), Some(ass));
        assert_eq!(SubtitleFormat::of_extension("sub"), Some(sub));
        assert_eq!(SubtitleFormat::of_extension("SRT"), Some(srt));
    }

    /// An extension nobody knows gives no format.
    #[test]
    fn an_unknown_extension_gives_no_format() {
        assert_eq!(SubtitleFormat::of_extension("txt"), None);
        assert_eq!(SubtitleFormat::of_extension(""), None);
    }

    /// The format gives the MIME type the server sends and the extension it
    /// is written with.
    #[test]
    fn the_format_gives_the_mime_type_and_the_extension() {
        assert_eq!(SubtitleFormat::Vtt.mime_type(), "text/vtt");
        assert_eq!(SubtitleFormat::Srt.mime_type(), "application/x-subrip");
        assert_eq!(SubtitleFormat::Ass.mime_type(), "text/x-ssa");
        assert_eq!(SubtitleFormat::Sub.mime_type(), "text/x-microdvd");
        assert_eq!(SubtitleFormat::Vtt.extension(), "vtt");
        assert_eq!(SubtitleFormat::Srt.extension(), "srt");
        assert_eq!(SubtitleFormat::Ass.extension(), "ass");
        assert_eq!(SubtitleFormat::Sub.extension(), "sub");
        assert_eq!(SubtitleFormat::Smil.extension(), "smi");

        let expected = ["vtt", "srt", "ass", "sub", "smi"];
        assert_eq!(SUBTITLE_EXTENSIONS, expected);
    }

    /// A folder that holds the sidecar of a film yields that sidecar.
    #[test]
    fn a_folder_that_holds_the_sidecar_yields_it() {
        let siblings = folder(&["Nightfall.mkv", "Nightfall.srt"]);
        let sidecar = sidecar_of("Nightfall", &siblings);

        assert_eq!(
            sidecar,
            Some(SubtitleSidecar {
                name: "Nightfall.srt".to_string(),
                format: SubtitleFormat::Srt,
                mime_type: "application/x-subrip".to_string(),
            })
        );
    }

    /// When a folder holds two sidecars, the one of the first format wins.
    ///
    /// WebVTT comes first because it is the format a browser-based client can
    /// read without a conversion.
    #[test]
    fn the_priority_order_decides_between_two_sidecars() {
        let siblings = folder(&["Nightfall.mkv", "Nightfall.srt", "Nightfall.vtt"]);
        let sidecar = sidecar_of("Nightfall", &siblings);
        let vtt = SubtitleFormat::Vtt;

        assert_eq!(sidecar.as_ref().map(|s| s.format), Some(vtt));
        assert_eq!(
            sidecar.as_ref().map(|s| s.name.as_str()),
            Some("Nightfall.vtt")
        );
    }

    /// A folder that holds no sidecar yields none.
    #[test]
    fn a_folder_without_a_sidecar_yields_none() {
        let siblings = folder(&["Nightfall.mkv", "Daybreak.mkv"]);

        assert_eq!(sidecar_of("Nightfall", &siblings), None);
    }

    /// An extension in upper case is still the sidecar, and the name is
    /// answered as it is on disk.
    #[test]
    fn the_extension_is_judged_without_its_case() {
        let siblings = folder(&["Nightfall.mkv", "Nightfall.SRT"]);
        let sidecar = sidecar_of("Nightfall", &siblings);
        let name = sidecar.as_ref().map(|s| s.name.as_str());
        let srt = SubtitleFormat::Srt;

        assert_eq!(name, Some("Nightfall.SRT"));
        assert_eq!(sidecar.as_ref().map(|s| s.format), Some(srt));
    }

    /// A name that merely contains the stem is not the sidecar of the film.
    #[test]
    fn a_name_that_merely_contains_the_stem_is_no_sidecar() {
        let siblings = folder(&["Nightfall.mkv", "Nightfall.forced.srt"]);

        assert_eq!(sidecar_of("Nightfall", &siblings), None);
    }

    /// A path of a subtitle file answers its format and its MIME type; a path
    /// without a subtitle extension answers nothing.
    #[test]
    fn a_path_answers_its_format_and_its_mime_type() {
        let srt = SubtitleFormat::Srt;
        let vtt = SubtitleFormat::Vtt;
        let subrip = "application/x-subrip";

        assert_eq!(format_at("Nightfall.srt"), Some(srt));
        assert_eq!(mime_at("Nightfall.srt"), Some(subrip));
        assert_eq!(format_at("Nightfall.vtt"), Some(vtt));
        assert_eq!(mime_at("Nightfall.vtt"), Some("text/vtt"));
        assert_eq!(format_at("Nightfall.mkv"), None);
        assert_eq!(mime_at("Nightfall.mkv"), None);
        assert_eq!(format_at("Nightfall"), None);
        assert_eq!(mime_at("Nightfall"), None);
    }
}
