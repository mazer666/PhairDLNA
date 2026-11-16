//! The scanner that reads a root and reports what is in it.
//!
//! What: the walk over a media root that turns the files on disk into the
//!       entries of the catalogue.
//! Why: the catalogue is only as good as the walk. A walk that follows a
//!       symbolic link out of the root, or that reads a folder nobody asked
//!       for, is a walk that publishes files the owner did not offer
//!       (`SEC-003`, `FUN-003`, `FUN-004`, `FUN-006`).
//! Example: a root that holds `Films/King Kong.mkv` produces one folder entry
//!       and one file entry, and nothing else.
//!
//! **The rules of the walk.**
//!
//! - It **never writes**. It opens directories to read them and files to read
//!   their first bytes, and it writes nothing, ever (`FUN-018`).
//! - It **never leaves the root**. A symbolic link is followed only when the
//!   policy says so, and only when the target is inside the root (`SEC-003`).
//! - It **never guesses**. A file it cannot place stays out of the tree and is
//!   reported, rather than indexed under a format nobody verified
//!   (`FUN-005`, `FUN-019`).
//! - It **is bounded**. The depth of the walk and the number of entries are
//!   limited, so a deep tree or a cycle of symbolic links cannot make the
//!   server disappear.
//! - It **names no absolute path**. The diagnostics of the server carry the
//!   name of the root and the path below it, never the path of the disk
//!   (`SEC-094`, `UX-011`).

mod encoding;
mod folder;
mod playlist;
mod walk;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::config::SymlinkPolicy;
use crate::platform::{CaseSensitivity, FileId, FileSystem, VolumeId};

use super::playlist::PlaylistBounds;
use walk::Walk;

/// The deepest tree the scanner walks.
///
/// The limit is a constant of the scanner; the configuration has no key for it
/// yet (`FUN-014`).
pub const DEFAULT_MAX_DEPTH: usize = 32;

/// What the scanner needs to know about a root.
#[derive(Debug, Clone)]
pub struct RootSpec {
    /// The path of the root on disk.
    pub path: PathBuf,
    /// The name the client shows for the root.
    pub name: String,
    /// The volume the root lives on.
    pub volume: VolumeId,
    /// Whether the volume distinguishes names that differ only in case.
    pub case_sensitivity: CaseSensitivity,
}

/// What the scanner is allowed to do.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// What to do with a symbolic link.
    pub symlink_policy: SymlinkPolicy,
    /// The paths below the root that are not indexed.
    pub excludes: Vec<String>,
    /// The deepest tree the scanner walks.
    pub max_depth: usize,
    /// The bounds the playlist parser keeps inside.
    pub playlist_bounds: PlaylistBounds,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            symlink_policy: SymlinkPolicy::default(),
            excludes: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            playlist_bounds: PlaylistBounds::default(),
        }
    }
}

/// Scans a root and reports what is in it.
///
/// A root that cannot be read is not an error: the catalogue reports the root
/// as absent and the rest of the library stays browsable (`FUN-007`).
pub fn scan(root: &RootSpec, options: &ScanOptions, filesystem: &dyn FileSystem) -> ScanOutcome {
    let canonical = std::fs::canonicalize(&root.path).ok();
    let mut walk = Walk::new(root, options, filesystem, canonical.clone());
    match canonical {
        Some(path) if path.is_dir() => {
            walk.outcome.root_present = true;
            walk.visited.insert(path);
            walk.walk(&root.path, Path::new(""), 0);
        }
        Some(_) => walk.diagnose(format!("the root \"{}\" is not a folder", root.name)),
        None => walk.diagnose(format!("the root \"{}\" cannot be read", root.name)),
    }
    walk.outcome
}

/// One entry the scanner found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanEntry {
    /// The path of the file on disk.
    ///
    /// For a link that is indexed as its own entry, this is the **target** of
    /// the link and not the link: a link that is repointed after the scan must
    /// not change what the catalogue serves (`SEC-001`).
    pub path: PathBuf,
    /// The identity of the entry, relative to the root.
    pub identity: String,
    /// The identity of the folder the entry lives in.
    pub parent_identity: String,
    /// The name of the entry, as the owner wrote it.
    pub title: String,
    /// Whether the entry is a folder.
    pub is_folder: bool,
    /// The format of the file, when it could be placed.
    pub mime_type: Option<String>,
    /// The size of the file in bytes.
    pub size: Option<u64>,
    /// When the file was last changed.
    pub modified: Option<SystemTime>,
    /// The identity the platform gives the file, when it gives one.
    ///
    /// The identity is what says that two sightings are one file, whatever
    /// they are called and wherever they were reached (`FUN-020`, `FUN-022`).
    pub file_identity: Option<FileId>,
    /// The identity key of the file: the value that says "the same path, seen
    /// again".
    ///
    /// The key is scoped by the volume and built from the path the owner gave,
    /// so that the same file reached through two roots of one volume has one
    /// key (`FUN-020`).
    pub identity_key: String,
    /// The subtitle that lies next to the file, when there is one.
    ///
    /// The pairing happens while the root is read, so that a browse opens no
    /// additional file (`FUN-016`, `PERF-008`).
    pub subtitle: Option<SubtitleEntry>,
    /// What the playlist names, when the file is a playlist.
    pub playlist: Option<PlaylistPlan>,
}

/// The subtitle that lies next to a media file.
///
/// A sidecar belongs to a media file when it lies in the same folder and
/// carries the same base name with one of the subtitle extensions
/// (`FUN-016`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleEntry {
    /// The name of the sidecar file, as the owner wrote it.
    pub name: String,
    /// The identity of the sidecar, relative to the root.
    pub identity: String,
    /// The path of the sidecar on the disk.
    pub path: PathBuf,
    /// The MIME type the server sends for the file.
    pub mime_type: String,
    /// The character set the file is written in.
    pub charset: &'static str,
    /// The size of the file in bytes.
    pub size: Option<u64>,
}

/// What a playlist names.
///
/// The targets are in the order the playlist writes them: a playlist is a list,
/// and a list that is sorted is a different list (`FUN-015`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistPlan {
    /// The entries of the playlist, in the order the playlist writes them.
    pub targets: Vec<PlaylistTarget>,
}

impl PlaylistPlan {
    /// The number of entries the client is offered.
    ///
    /// A refused entry is not offered: the user is told about it in the
    /// diagnostics, and the playlist shows the tracks that can be played.
    pub fn offered(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| !matches!(target.state, TargetState::Refused(_)))
            .count()
    }
}

/// One entry of a playlist, after it was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistTarget {
    /// The identity of the file the entry names, relative to the root.
    pub identity: String,
    /// The name of the file the entry names, for an entry that cannot be played.
    pub name: String,
    /// The title the playlist gives the entry, when it gives one.
    pub title: Option<String>,
    /// What became of the entry.
    pub state: TargetState,
}

/// What became of an entry of a playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetState {
    /// The entry names a file of the root.
    Resolved,
    /// The entry names a file that is not there.
    ///
    /// The entry stays in the playlist, so that the user learns where the
    /// track went instead of wondering about it (`UX-004`).
    Missing(String),
    /// The entry was refused and is not offered.
    ///
    /// A network address and a path outside the root are refused; the user is
    /// told about it in the diagnostics (`SEC-068`).
    Refused(String),
}

/// What the scanner found, and what it could not read.
#[derive(Debug, Clone, Default)]
pub struct ScanOutcome {
    /// The entries of the tree, in the order the folders were read.
    pub entries: Vec<ScanEntry>,
    /// What the scanner could not do, in words a person can read.
    pub diagnostics: Vec<String>,
    /// Whether the root is a folder that could be read.
    pub root_present: bool,
}
