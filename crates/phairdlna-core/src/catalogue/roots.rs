//! The rules of the roots.
//!
//! What: what the catalogue knows about a root, the outcome of a build, and the
//!       two rules that concern the roots as a whole — the sentence that tells
//!       a person what happened to a drive, and the root that lies inside
//!       another root.
//! Why: the builder reads the roots one by one, and the rules that concern
//!       several roots at once would otherwise be spread over the file
//!       (`FUN-017`).
//! Example: a root inside another root is published once, under its own
//!       identifier.

use std::path::PathBuf;

use unicode_normalization::UnicodeNormalization;

use crate::config::MediaRoot;

use super::volumes::VolumeChange;

/// What the catalogue knows about a root.
#[derive(Debug, Clone)]
pub struct RootSummary {
    /// The name the client shows.
    pub name: String,
    /// The path of the root on disk.
    pub path: PathBuf,
    /// The identifier of the container of the root.
    pub id: String,
    /// Whether the root could be read.
    pub present: bool,
    /// The number of entries below the root.
    pub entries: usize,
}

/// The paths of the other roots that lie inside this root.
///
/// A root inside another root would be published twice: once below the outer
/// root and once below its own key. The outer root leaves the inner one
/// alone, so that every file of the library has one place and one identifier
/// (`FUN-017`).
///
/// The paths are compared after they are resolved, because a root that is
/// written as `/var/folders/…` is the same folder as `/private/var/folders/…`
/// on macOS, and the comparison of the written paths would not see it.
pub(super) fn excluded_roots(roots: &[MediaRoot], index: usize) -> Vec<String> {
    let Ok(outer) = std::fs::canonicalize(&roots[index].path) else {
        return Vec::new();
    };
    let mut excluded = Vec::new();
    for (other, root) in roots.iter().enumerate() {
        if other == index {
            continue;
        }
        let Ok(inner) = std::fs::canonicalize(&root.path) else {
            continue;
        };
        if inner == outer {
            continue;
        }
        let Ok(relative) = inner.strip_prefix(&outer) else {
            continue;
        };
        let text = relative
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .map(|part| part.nfc().collect::<String>())
            .collect::<Vec<_>>()
            .join("/");
        if !text.is_empty() {
            excluded.push(text);
        }
    }
    excluded
}

/// The sentence that tells a person what happened to a drive.
///
/// The sentence names the root and never a path of the disk (`SEC-094`).
pub(super) fn diagnostic_for(name: &str, change: &VolumeChange) -> Option<String> {
    let sentence = match change {
        VolumeChange::First | VolumeChange::Unchanged => return None,
        VolumeChange::Lost => format!("the drive \"{name}\" is not connected"),
        VolumeChange::Returned => format!("the drive \"{name}\" is connected again"),
        VolumeChange::Remounted => {
            format!("the drive \"{name}\" was mounted again and was read anew")
        }
        VolumeChange::Replaced { .. } => format!(
            "the drive \"{name}\" is now a different volume; its files were given new identifiers"
        ),
    };
    Some(sentence)
}

#[cfg(test)]
mod tests {
    use super::diagnostic_for;
    use crate::catalogue::VolumeChange;

    /// The sentence of a change names the root and not a path.
    #[test]
    fn the_sentence_names_the_root() {
        let sentence = diagnostic_for("Films", &VolumeChange::Lost).expect("there is a sentence");
        assert!(sentence.contains("Films"));
        assert!(!sentence.contains('/'));
        assert!(diagnostic_for("Films", &VolumeChange::Unchanged).is_none());
    }
}
