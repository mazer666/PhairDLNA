//! The playlists the server reads.
//!
//! What: a parser for the M3U, M3U8 and PLS playlist formats that turns
//!       playlist text into entries and problems.
//! Why: a client asks for a playlist by name, so the server has to know what
//!       it holds, and a broken playlist must never stop a scan (`FUN-015`).
//!       The parser reads text only: every entry stays an untrusted path
//!       until the containment rule has seen it (`SEC-068`).
//! Example:
//!
//! ```rust
//! use phairdlna_core::catalogue::playlist::{parse, PlaylistBounds, PlaylistFormat};
//! let playlist = parse("#EXTINF:1,Song\na.mp3\n", PlaylistFormat::M3u, PlaylistBounds::default());
//! assert_eq!(playlist.entries[0].title.as_deref(), Some("Song"));
//! ```

use std::collections::BTreeMap;

/// The format of a playlist file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistFormat {
    /// The M3U format: one path per line.
    M3u,
    /// The M3U8 format: the M3U format, written in UTF-8.
    M3u8,
    /// The PLS format: an INI-style file with a `[playlist]` section.
    Pls,
}

impl PlaylistFormat {
    /// The format of a file, judged by its extension (case-insensitive).
    pub fn of_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "m3u" => Some(Self::M3u),
            "m3u8" => Some(Self::M3u8),
            "pls" => Some(Self::Pls),
            _ => None,
        }
    }

    /// The MIME type the server knows the format by.
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Pls => "audio/x-scpls",
            _ => "audio/x-mpegurl",
        }
    }
}

/// One entry of a playlist, before it is resolved to a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntry {
    /// The reference the playlist carries: a path, a `file:` address or a network address.
    pub reference: String,
    /// The title the playlist gives the entry, when it gives one.
    pub title: Option<String>,
}

/// The bounds the parser keeps inside: every bound has a documented default
/// and a documented failure behaviour (`SEC-060`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaylistBounds {
    /// The largest file the reader may hand to the parser.
    pub max_bytes: usize,
    /// The largest number of entries the parser accepts.
    pub max_entries: usize,
    /// The largest length of a single line, in bytes.
    pub max_line_bytes: usize,
    /// The largest total length of all accepted references, in bytes.
    pub max_total_entry_bytes: usize,
}

impl Default for PlaylistBounds {
    fn default() -> Self {
        Self {
            max_bytes: 1_048_576,
            max_entries: 10_000,
            max_line_bytes: 4_096,
            max_total_entry_bytes: 1_048_576,
        }
    }
}

/// Why the parser could not use something it read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaylistProblem {
    /// The content does not match the format the name promises.
    InvalidFormat {
        /// What is wrong with the content.
        reason: String,
    },
    /// One line is longer than the bound allows.
    EntryTooLong {
        /// An excerpt of the line, never the whole line.
        reference: String,
    },
    /// One entry cannot be used at all.
    MalformedEntry {
        /// The text of the entry that cannot be used.
        reference: String,
        /// What is wrong with the entry.
        reason: String,
    },
    /// The parser stopped at a bound.
    Truncated {
        /// Which bound the parser stopped at.
        reason: String,
    },
}

/// What the parser found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedPlaylist {
    /// The entries of the playlist, in the order the playlist writes them.
    pub entries: Vec<PlaylistEntry>,
    /// What the parser could not use.
    pub problems: Vec<PlaylistProblem>,
    /// Whether the parser stopped at a bound.
    pub truncated: bool,
}

/// Parses the text of a playlist.
///
/// The text is read line by line; the parser never reads the filesystem and
/// never resolves a path (`SEC-068`). A file that breaks a bound keeps what it
/// read and says so in a `Truncated` problem.
pub fn parse(text: &str, format: PlaylistFormat, bounds: PlaylistBounds) -> ParsedPlaylist {
    if text.len() > bounds.max_bytes {
        let mut result = ParsedPlaylist::default();
        let limit = bounds.max_bytes;
        stop_at_bound(
            &mut result,
            format!("the playlist is longer than {limit} bytes"),
        );
        return result;
    }
    match format {
        PlaylistFormat::M3u | PlaylistFormat::M3u8 => parse_m3u(text, bounds),
        PlaylistFormat::Pls => parse_pls(text, bounds),
    }
}

/// The lines of a playlist that carry something, without the line ending.
fn lines(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.split('\n')
        .map(|raw| raw.strip_suffix('\r').unwrap_or(raw))
        .map(|line| (line, line.trim()))
        .filter(|(_, trimmed)| !trimmed.is_empty())
}

/// Reads an M3U or M3U8 playlist.
fn parse_m3u(text: &str, bounds: PlaylistBounds) -> ParsedPlaylist {
    let mut result = ParsedPlaylist::default();
    let mut total_bytes = 0usize;
    let mut pending_title: Option<String> = None;
    let mut seen_entry = false;

    for (line, trimmed) in lines(text) {
        if trimmed.starts_with('#') {
            if let Some(payload) = trimmed.strip_prefix("#EXTINF:") {
                pending_title = extinf_title(payload);
            }
            continue;
        }
        if !seen_entry && trimmed.starts_with('[') {
            invalid_format(
                &mut result,
                "the file is an INI playlist, not an M3U playlist",
            );
            return result;
        }
        seen_entry = true;
        if line.len() > bounds.max_line_bytes {
            too_long(&mut result, preview(trimmed));
            continue;
        }
        let title = pending_title.take();
        if !accept_entry(&mut result, &bounds, &mut total_bytes, trimmed, title) {
            break;
        }
    }
    result
}

/// The title an `#EXTINF:` payload carries for the next entry.
fn extinf_title(payload: &str) -> Option<String> {
    let (_, after) = payload.split_once(',')?;
    Some(after.trim().to_string()).filter(|title| !title.is_empty())
}

/// Reads a PLS playlist.
fn parse_pls(text: &str, bounds: PlaylistBounds) -> ParsedPlaylist {
    let mut result = ParsedPlaylist::default();
    let mut total_bytes = 0usize;
    let mut section: Option<String> = None;
    let mut saw_playlist = false;
    let mut entries = PlsEntries::default();

    for (line, trimmed) in lines(text) {
        if let Some(inner) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = Some(inner.trim().to_ascii_lowercase());
            saw_playlist |= section.as_deref() == Some("playlist");
            continue;
        }
        if section.as_deref() != Some("playlist") {
            continue;
        }
        if line.len() > bounds.max_line_bytes {
            too_long(&mut result, preview(trimmed));
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if !read_pls_key(&mut entries, key.trim(), value.trim(), bounds.max_entries) {
            let count = bounds.max_entries;
            stop_at_bound(
                &mut result,
                format!("the playlist has more than {count} entries"),
            );
            break;
        }
    }
    if !saw_playlist {
        invalid_format(&mut result, "the file has no [playlist] section");
        return result;
    }
    result.problems.append(&mut entries.problems);
    for (index, reference) in &entries.files {
        let title = entries.titles.get(index).cloned();
        if !accept_entry(&mut result, &bounds, &mut total_bytes, reference, title) {
            break;
        }
    }
    result
}

#[derive(Debug, Default)]
struct PlsEntries {
    files: BTreeMap<u32, String>,
    titles: BTreeMap<u32, String>,
    problems: Vec<PlaylistProblem>,
}

/// Reads one `key=value` line of the `[playlist]` section of a PLS file.
fn read_pls_key(entries: &mut PlsEntries, key: &str, value: &str, max_entries: usize) -> bool {
    let lower = key.to_ascii_lowercase();
    let Some(index) = numbered_suffix(&lower, "file") else {
        if let Some(Some(index)) = numbered_suffix(&lower, "title") {
            if !value.is_empty() && entries.titles.len() < max_entries {
                entries.titles.insert(index, value.to_string());
            }
        }
        return true;
    };
    let Some(index) = index else {
        let reference = if value.is_empty() { key } else { value };
        malformed(&mut entries.problems, reference, "the entry has no number");
        return true;
    };
    if value.is_empty() {
        malformed(&mut entries.problems, key, "the entry has no path");
        return true;
    }
    if entries.files.len() >= max_entries && !entries.files.contains_key(&index) {
        return false;
    }
    entries.files.insert(index, value.to_string());
    true
}

/// Records an entry the parser cannot use.
fn malformed(problems: &mut Vec<PlaylistProblem>, reference: &str, reason: &str) {
    problems.push(PlaylistProblem::MalformedEntry {
        reference: reference.to_string(),
        reason: reason.to_string(),
    });
}

/// Adds an entry to the result, or stops the parser at a bound.
fn accept_entry(
    result: &mut ParsedPlaylist,
    bounds: &PlaylistBounds,
    total_bytes: &mut usize,
    reference: &str,
    title: Option<String>,
) -> bool {
    if result.entries.len() >= bounds.max_entries {
        let count = bounds.max_entries;
        stop_at_bound(
            result,
            format!("the playlist has more than {count} entries"),
        );
        return false;
    }
    let bytes = bounds.max_total_entry_bytes;
    if *total_bytes + reference.len() > bytes {
        let reason = format!("the total length of the entries is more than {bytes} bytes");
        stop_at_bound(result, reason);
        return false;
    }
    *total_bytes += reference.len();
    result.entries.push(PlaylistEntry {
        reference: reference.to_string(),
        title,
    });
    true
}

/// Records that the content does not match the format the name promises.
fn invalid_format(result: &mut ParsedPlaylist, reason: &str) {
    result.problems.push(PlaylistProblem::InvalidFormat {
        reason: reason.to_string(),
    });
}

/// Records a line that is too long to accept.
fn too_long(result: &mut ParsedPlaylist, reference: String) {
    result
        .problems
        .push(PlaylistProblem::EntryTooLong { reference });
}

/// Records that the parser stopped at a bound.
fn stop_at_bound(result: &mut ParsedPlaylist, reason: String) {
    result.truncated = true;
    result.problems.push(PlaylistProblem::Truncated { reason });
}

/// The number behind a PLS key such as `File12`: `None` when the key has no
/// such prefix, `Some(None)` when the number behind it is not a number.
fn numbered_suffix(key: &str, prefix: &str) -> Option<Option<u32>> {
    let rest = key.strip_prefix(prefix)?;
    Some(rest.parse::<u32>().ok())
}

/// A short excerpt of a line that is too long: at most 64 bytes, never a whole
/// hostile line.
fn preview(text: &str) -> String {
    let cut = (0..=64)
        .rev()
        .find(|at| text.is_char_boundary(*at))
        .unwrap_or(0);
    format!("{}…", &text[..cut])
}

#[cfg(test)]
mod tests {
    use super::{parse, ParsedPlaylist, PlaylistBounds, PlaylistFormat, PlaylistProblem};

    /// Parses a playlist with the default bounds.
    fn parsed(text: &str, format: PlaylistFormat) -> ParsedPlaylist {
        parse(text, format, PlaylistBounds::default())
    }

    /// The default bounds, with the change a test needs.
    fn with_bounds(change: impl FnOnce(&mut PlaylistBounds)) -> PlaylistBounds {
        let mut bounds = PlaylistBounds::default();
        change(&mut bounds);
        bounds
    }

    /// Two lines of an M3U file are two entries without a title.
    #[test]
    fn two_lines_are_two_entries() {
        let playlist = parsed("a.mp3\nb.mp3\n", PlaylistFormat::M3u);
        assert_eq!(playlist.entries.len(), 2);
        assert_eq!(playlist.entries[0].reference, "a.mp3");
        assert_eq!(playlist.entries[1].reference, "b.mp3");
        assert_eq!(playlist.entries[0].title, None);
        assert!(playlist.problems.is_empty());
    }

    /// An `#EXTINF:` line gives its title to the next entry.
    #[test]
    fn an_extinf_line_gives_the_title_to_the_next_entry() {
        let playlist = parsed("#EXTINF:123,A song\na.mp3\n", PlaylistFormat::M3u);
        assert_eq!(playlist.entries.len(), 1);
        assert_eq!(playlist.entries[0].title.as_deref(), Some("A song"));
    }

    /// A comment and an empty line are neither an entry nor a problem.
    #[test]
    fn a_comment_and_an_empty_line_are_skipped() {
        let playlist = parsed("#EXTM3U\n\n# a comment\na.mp3\n", PlaylistFormat::M3u);
        assert_eq!(playlist.entries.len(), 1);
        assert!(playlist.problems.is_empty());
    }

    /// An M3U8 file is an M3U file written in UTF-8.
    #[test]
    fn an_m3u8_file_is_read_like_an_m3u_file() {
        let text = "#EXTINF:1,Lied\nAusschnitt.mp3\n";
        let m3u = parsed(text, PlaylistFormat::M3u);
        assert_eq!(m3u, parsed(text, PlaylistFormat::M3u8));
    }

    /// A PLS file lists its entries by index, not by the order of the lines.
    #[test]
    fn a_pls_file_lists_its_entries_by_index() {
        let text = "[playlist]\nFile10=t.mp3\nFile2=s.mp3\n";
        let playlist = parsed(text, PlaylistFormat::Pls);
        assert_eq!(playlist.entries.len(), 2);
        assert_eq!(playlist.entries[0].reference, "s.mp3");
        assert_eq!(playlist.entries[1].reference, "t.mp3");
    }

    /// A file that does not match the format it was named after is invalid:
    /// a PLS file without a `[playlist]` section, and an M3U file that carries
    /// an INI section, are no playlists at all.
    #[test]
    fn a_file_of_the_wrong_format_is_invalid() {
        let pls = parsed("[general]\nFile1=a.mp3\n", PlaylistFormat::Pls);
        let m3u = parsed("[playlist]\nFile1=a.mp3\n", PlaylistFormat::M3u);
        assert!(pls.entries.is_empty() && m3u.entries.is_empty());
        assert_eq!(pls.problems.len(), 1);
        assert_eq!(m3u.problems.len(), 1);
        assert!(format!("{:?}", pls.problems[0]).contains("no [playlist] section"));
        assert!(format!("{:?}", m3u.problems[0]).contains("INI playlist"));
    }

    /// A line that is too long is refused, and the lines around it survive.
    #[test]
    fn a_too_long_line_is_refused() {
        let bounds = with_bounds(|b| b.max_line_bytes = 16);
        let long = "a".repeat(40);
        let text = format!("short.mp3\n{long}\nother.mp3\n");
        let playlist = parse(&text, PlaylistFormat::M3u, bounds);
        let problem = &playlist.problems[0];
        assert_eq!(playlist.entries.len(), 2);
        assert_eq!(playlist.entries[0].reference, "short.mp3");
        assert_eq!(playlist.entries[1].reference, "other.mp3");
        assert_eq!(playlist.problems.len(), 1);
        assert!(matches!(problem, PlaylistProblem::EntryTooLong { .. }));
    }

    /// A playlist with more entries than the bound allows stops at the bound.
    #[test]
    fn more_entries_than_the_bound_are_truncated() {
        let bounds = with_bounds(|b| b.max_entries = 2);
        let playlist = parse("a.mp3\nb.mp3\nc.mp3\n", PlaylistFormat::M3u, bounds);
        let problem = &playlist.problems[0];
        assert!(playlist.truncated);
        assert_eq!(playlist.entries.len(), 2);
        assert_eq!(playlist.problems.len(), 1);
        assert!(format!("{problem:?}").contains("more than 2 entries"));
    }

    /// The references of a playlist cannot grow past the total bound.
    #[test]
    fn the_total_length_of_the_references_is_bounded() {
        let bounds = with_bounds(|b| b.max_total_entry_bytes = 10);
        let playlist = parse("aaaa.mp3\nbbbbbb.mp3\n", PlaylistFormat::M3u, bounds);
        let problem = &playlist.problems[0];
        assert!(playlist.truncated);
        assert_eq!(playlist.entries.len(), 1);
        assert_eq!(playlist.problems.len(), 1);
        assert!(format!("{problem:?}").contains("more than 10 bytes"));
    }

    /// A PLS entry without a path cannot be used.
    #[test]
    fn a_pls_entry_without_a_path_is_malformed() {
        let playlist = parsed("[playlist]\nFile1=\n", PlaylistFormat::Pls);
        let problem = &playlist.problems[0];
        assert!(playlist.entries.is_empty());
        assert_eq!(playlist.problems.len(), 1);
        assert!(format!("{problem:?}").contains("the entry has no path"));
    }

    /// The title of a PLS entry is read from its `Title` key.
    #[test]
    fn a_pls_entry_takes_its_title_from_the_title_key() {
        let text = "[playlist]\nFile1=a.mp3\nTitle1=A song\n";
        let playlist = parsed(text, PlaylistFormat::Pls);
        assert_eq!(playlist.entries.len(), 1);
        assert_eq!(playlist.entries[0].title.as_deref(), Some("A song"));
    }

    /// A playlist of nothing but comments holds no entry and no problem.
    #[test]
    fn a_playlist_of_comments_holds_nothing() {
        let playlist = parsed("#EXTM3U\n#EXTINF:1,A song\n", PlaylistFormat::M3u);
        assert!(playlist.entries.is_empty());
        assert!(playlist.problems.is_empty());
    }

    /// A file with Windows line endings holds the same entries as the same
    /// file with Unix line endings.
    #[test]
    fn a_crlf_file_holds_the_same_entries() {
        let lf = parsed("a.mp3\nb.mp3\n", PlaylistFormat::M3u);
        let crlf = parsed("a.mp3\r\nb.mp3\r\n", PlaylistFormat::M3u);
        assert_eq!(lf, crlf);
    }
}
