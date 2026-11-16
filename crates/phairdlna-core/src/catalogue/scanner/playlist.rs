//! The reading of a playlist of a root.
//!
//! What: the bounded reading of a playlist file, the parsing of its entries and
//!       the resolution of every entry against the folder of the playlist.
//! Why: a playlist is a file the owner wrote by hand, and every entry in it is
//!       an untrusted path (`SEC-068`). The reader resolves the entries inside
//!       the root, refuses the network addresses, and reports every entry it
//!       cannot use — a bad playlist must never take a scan down (`UX-005`,
//!       `FUN-015`).
//! Example: an entry that names a network address is refused and reported; the
//!       rest of the playlist is still played.

use std::io::Read;
use std::path::Path;

use crate::catalogue::playlist::{
    self, PlaylistBounds, PlaylistEntry, PlaylistFormat, PlaylistProblem,
};
use crate::platform::{identity_of_with, CaseSensitivity};
use crate::security::containment::{resolve_reference, ContainmentError};

use super::{PlaylistPlan, PlaylistTarget, TargetState};

/// What the reader needs to know about the root it is reading for.
pub(super) struct PlaylistContext<'a> {
    /// The path of the root, as the owner wrote it.
    pub(super) root: &'a Path,
    /// The path of the root, resolved.
    pub(super) canonical_root: &'a Path,
    /// Whether the volume of the root tells two names apart.
    pub(super) case_sensitivity: CaseSensitivity,
    /// The bounds the reader keeps inside.
    pub(super) bounds: PlaylistBounds,
}

/// Reads a playlist and resolves every entry it names.
///
/// The answer is the plan of the playlist and the sentences that tell what
/// could not be used. The plan is never empty for a playlist that could be
/// read: a playlist with a broken entry is still a playlist the client opens.
pub(super) fn read(
    path: &Path,
    format: PlaylistFormat,
    base: &Path,
    label: &str,
    context: &PlaylistContext<'_>,
) -> (PlaylistPlan, Vec<String>) {
    let text = match read_text(path, context.bounds.max_bytes) {
        Ok(text) => text,
        Err(reason) => {
            let sentence = format!("the playlist \"{label}\" cannot be read: {reason}");
            return (
                PlaylistPlan {
                    targets: Vec::new(),
                },
                vec![sentence],
            );
        }
    };
    let parsed = playlist::parse(&text, format, context.bounds);
    let mut sentences = Vec::new();
    for problem in &parsed.problems {
        sentences.push(sentence_for(label, problem));
    }
    let targets: Vec<PlaylistTarget> = parsed
        .entries
        .iter()
        .map(|entry| resolve_entry(entry, base, context))
        .collect();
    for target in &targets {
        match &target.state {
            TargetState::Refused(reason) => sentences.push(format!(
                "the playlist \"{label}\" has an entry that was refused: {reason}"
            )),
            TargetState::Missing(reason) => sentences.push(format!(
                "the playlist \"{label}\" names \"{}\", which cannot be played: {reason}",
                target.name
            )),
            TargetState::Resolved => {}
        }
    }
    (PlaylistPlan { targets }, sentences)
}

/// The sentence that tells what the parser could not use.
fn sentence_for(label: &str, problem: &PlaylistProblem) -> String {
    match problem {
        PlaylistProblem::InvalidFormat { reason } => {
            format!("the playlist \"{label}\" is not a valid playlist: {reason}")
        }
        PlaylistProblem::EntryTooLong { .. } => {
            format!("the playlist \"{label}\" has an entry that is too long and was left out")
        }
        PlaylistProblem::MalformedEntry { .. } => {
            format!("the playlist \"{label}\" has an entry that cannot be used and was left out")
        }
        PlaylistProblem::Truncated { reason } => {
            format!("the playlist \"{label}\" was not read entirely: {reason}")
        }
    }
}

/// Resolves one entry of a playlist against the folder of the playlist.
///
/// The entry is kept even when it cannot be played: the user learns that the
/// track is missing instead of wondering where it went (`UX-004`).
fn resolve_entry(
    entry: &PlaylistEntry,
    base: &Path,
    context: &PlaylistContext<'_>,
) -> PlaylistTarget {
    let name = name_of(&entry.reference);
    let roots = [context.root.to_path_buf()];
    let resolved = match resolve_reference(&entry.reference, base, &roots) {
        Ok(path) => path,
        Err(error) => {
            return PlaylistTarget {
                identity: String::new(),
                name,
                title: entry.title.clone(),
                state: TargetState::Refused(reason_of(&error)),
            };
        }
    };
    let identity = identity_of_resolved(&resolved, context);
    let state = state_of(&resolved, context);
    PlaylistTarget {
        identity,
        name,
        title: entry.title.clone(),
        state,
    }
}

/// The identity of a resolved entry, relative to the root.
///
/// The identity is the one the catalogue gives the file, so that the playlist
/// and the tree of the root agree about which file an entry names.
fn identity_of_resolved(resolved: &Path, context: &PlaylistContext<'_>) -> String {
    let relative = resolved.strip_prefix(context.root).unwrap_or(resolved);
    identity_of_with(relative, context.case_sensitivity)
}

/// What became of a resolved entry.
///
/// A file that is not there is missing: the entry stays in the playlist, so
/// that the user learns where the track went. A file that is there but lies
/// outside the root is **refused**, whatever the name of the link that points
/// to it says: the server never follows a link out of the root (`SEC-003`,
/// `SEC-068`).
fn state_of(resolved: &Path, context: &PlaylistContext<'_>) -> TargetState {
    match std::fs::canonicalize(resolved) {
        Err(_) => TargetState::Missing("the file does not exist".to_owned()),
        Ok(canonical) => {
            let roots = [context.canonical_root.to_path_buf()];
            if crate::security::containment::is_inside_roots(&canonical, &roots) {
                TargetState::Resolved
            } else {
                TargetState::Refused("the file lies outside the shared folder".to_owned())
            }
        }
    }
}

/// The reason an entry was refused.
fn reason_of(error: &ContainmentError) -> String {
    match error {
        ContainmentError::Empty => "the entry is empty".to_owned(),
        ContainmentError::NetworkAddress => {
            "a network address is not allowed in a playlist".to_owned()
        }
        ContainmentError::UnsupportedScheme(_) => {
            "the entry uses a scheme that is not allowed".to_owned()
        }
        ContainmentError::OutsideRoots => "the file lies outside the shared folder".to_owned(),
    }
}

/// The name of the file an entry names.
///
/// The name is the last part of the reference: it is what the user recognises,
/// and it never carries the path of the disk (`SEC-094`).
fn name_of(reference: &str) -> String {
    let trimmed = reference.trim();
    let without_scheme = trimmed.split_once(':').map_or(trimmed, |(_, rest)| rest);
    let last = without_scheme
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or(without_scheme);
    last.chars().take(120).collect()
}

/// Reads the text of a playlist, within the bounds.
///
/// The file is read through a limit, so that a file that grows between the
/// moment it was listed and the moment it is read cannot make the server
/// allocate without limit (`SEC-060`, `SEC-011`).
fn read_text(path: &Path, max_bytes: usize) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err(format!("the file is larger than {max_bytes} bytes"));
    }
    Ok(decode(bytes))
}

/// Decodes the bytes of a playlist.
///
/// A playlist is UTF-8 in the M3U8 form; the other two forms are written in
/// the encoding of the system that wrote them. A file that is not valid UTF-8
/// is read as Latin-1, which never fails and never loses a byte (`SEC-007`).
fn decode(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => error
            .into_bytes()
            .iter()
            .map(|byte| *byte as char)
            .collect(),
    }
}
