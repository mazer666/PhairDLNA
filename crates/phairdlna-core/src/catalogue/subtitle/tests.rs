//! Tests for subtitle sidecars.

use std::collections::HashSet;
use std::path::Path;

use super::{
    format_of, mime_type_of, sidecar_of, SubtitleFormat, SubtitleSidecar, SUBTITLE_EXTENSIONS,
};

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
