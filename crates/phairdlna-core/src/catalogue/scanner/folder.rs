//! The folder the walk is reading.
//!
//! What: the names of the files of a folder and the names whose identity is
//!       claimed by more than one of them.
//! Why: the walk reads a folder once, and everything it needs to know about
//!       the folder — which file is the sidecar of a film, which names are two
//!       spellings of one name — is answered by that one reading (`FUN-016`,
//!       `FUN-020`, `PLT-005`).
//! Example: a folder that holds `Nightfall.mkv` and `Nightfall.vtt` names the
//!       sidecar of the film.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::platform::{identity_of_with, CaseSensitivity};

/// What the walk knows about the folder it is reading.
pub(super) struct Folder {
    /// The names of the files of the folder.
    ///
    /// The names are what a subtitle is recognised by: the sidecar of a film
    /// is the file that lies next to it and carries the same base name
    /// (`FUN-016`).
    pub(super) names: HashSet<String>,
    /// The names whose identity is claimed by more than one file.
    ///
    /// Two names that differ only in the Unicode normalisation form share one
    /// identity, but they are two files on a volume that keeps the bytes
    /// (`FUN-020`, `PLT-005`). Such a file keeps the identity the platform
    /// gives it, so that the two files have two object identifiers and
    /// neither of them is lost.
    pub(super) colliding: HashSet<String>,
}

impl Folder {
    /// Reads the folder once.
    pub(super) fn of(
        entries: &[std::fs::DirEntry],
        relative: &Path,
        sensitivity: CaseSensitivity,
    ) -> Self {
        let names: HashSet<String> = entries
            .iter()
            .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
            .collect();
        let colliding = colliding_names(entries, relative, sensitivity);
        Self { names, colliding }
    }

    /// Whether the name of the file is claimed by another file of the folder.
    pub(super) fn collides(&self, name: &str) -> bool {
        self.colliding.contains(name)
    }
}

/// The names of the folder whose identity is claimed by more than one file.
///
/// The identity of a path is normalised to Unicode NFC, so two names that
/// differ only in the normalisation form share one identity — whatever the
/// volume does with them (`FUN-020`, `PLT-005`).
fn colliding_names(
    entries: &[std::fs::DirEntry],
    relative: &Path,
    sensitivity: CaseSensitivity,
) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let identity = identity_of_with(&relative.join(name), sensitivity);
        *counts.entry(identity).or_insert(0) += 1;
    }
    let shared: HashSet<String> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(identity, _)| identity.clone())
        .collect();
    entries
        .iter()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let identity = identity_of_with(&relative.join(name), sensitivity);
            shared.contains(&identity).then(|| name.to_owned())
        })
        .collect()
}
