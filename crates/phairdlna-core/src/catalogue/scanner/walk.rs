//! The walk that reads a root.
//!
//! What: the state of one walk over a root and the rules it follows.
//! Why: the walk touches the filesystem of the owner, so its rules are worth
//!       keeping in one place where they can be read (`SEC-003`, `FUN-018`).
//! Example: a walk reads a folder, visits what is in it and reports what it
//!       could not read.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::SymlinkPolicy;
use crate::platform::identity_of_with;
use crate::platform::{FileId, FileSystem};

use super::super::media::{is_indexable, mime_of_extension};
use super::super::sniff::{sniff, HEAD_LENGTH};
use super::{RootSpec, ScanEntry, ScanOptions, ScanOutcome};

/// The state of one walk.
pub(super) struct Walk<'a> {
    root: &'a RootSpec,
    options: &'a ScanOptions,
    filesystem: &'a dyn FileSystem,
    canonical_root: PathBuf,
    pub(super) seen_files: HashSet<FileId>,
    pub(super) outcome: ScanOutcome,
    pub(super) visited: HashSet<PathBuf>,
}

/// What a symbolic link resolved to.
enum Resolution {
    /// The entry is its own entry, and the path to read is the one given.
    Own { path: PathBuf },
    /// The link is transparent: the target is indexed where it lives.
    Transparent,
}

impl<'a> Walk<'a> {
    /// Starts a walk over a root.
    pub(super) fn new(
        root: &'a RootSpec,
        options: &'a ScanOptions,
        filesystem: &'a dyn FileSystem,
        canonical: Option<PathBuf>,
    ) -> Self {
        Self {
            root,
            options,
            filesystem,
            canonical_root: canonical.unwrap_or_else(|| root.path.clone()),
            visited: HashSet::new(),
            seen_files: HashSet::new(),
            outcome: ScanOutcome::default(),
        }
    }

    /// Reads one folder and visits what is in it.
    pub(super) fn walk(&mut self, directory: &Path, relative: &Path, depth: usize) {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => {
                self.diagnose(format!(
                    "the folder \"{}\" cannot be read: {error}",
                    display_relative(relative)
                ));
                return;
            }
        };
        for entry in entries {
            match entry {
                Ok(entry) => self.visit(&entry.path(), relative, depth),
                Err(error) => self.diagnose(format!(
                    "an entry of the folder \"{}\" cannot be read: {error}",
                    display_relative(relative)
                )),
            }
        }
    }

    /// Visits one entry of a folder.
    fn visit(&mut self, path: &Path, relative: &Path, depth: usize) {
        let Some(name) = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            return;
        };
        if self.is_hidden(path, &name) {
            return;
        }
        let child = relative.join(&name);
        if self.is_excluded(&child) {
            return;
        }
        let Some(resolution) = self.resolve(path, &child) else {
            return;
        };
        match resolution {
            Resolution::Transparent => {}
            Resolution::Own { path: target } => {
                if target.is_dir() {
                    self.visit_folder(path, &child, &name, depth);
                } else {
                    self.visit_file(path, &child, &name, &target);
                }
            }
        }
    }

    /// Records a folder and walks into it.
    fn visit_folder(&mut self, path: &Path, child: &Path, name: &str, depth: usize) {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !self.visited.insert(canonical) {
            self.diagnose(format!(
                "the folder \"{}\" is reached twice",
                display_relative(child)
            ));
            return;
        }
        self.outcome.entries.push(ScanEntry {
            path: path.to_path_buf(),
            identity: self.identity(child),
            parent_identity: self.identity(child.parent().unwrap_or(Path::new(""))),
            title: name.to_owned(),
            is_folder: true,
            mime_type: None,
            size: None,
            modified: None,
        });
        if depth < self.options.max_depth {
            self.walk(path, child, depth + 1);
        } else {
            self.diagnose(format!(
                "the folder \"{}\" is deeper than the limit of {} and was not read",
                display_relative(child),
                self.options.max_depth
            ));
        }
    }

    /// Records a file, when the catalogue can place it.
    ///
    /// The `entry` is the name the owner gave the file; the `target` is the
    /// file on disk that is read. For a link that is indexed as its own entry
    /// the two differ.
    fn visit_file(&mut self, entry: &Path, child: &Path, name: &str, target: &Path) {
        let Some(mime_type) = self.mime_of(entry, target) else {
            return;
        };
        if !is_indexable(&mime_type) {
            return;
        }
        if entry == target && self.is_second_name(target) {
            self.diagnose(format!(
                "the file \"{}\" is another name for a file that is already indexed",
                display_relative(child)
            ));
            return;
        }
        let metadata = std::fs::metadata(target).ok();
        self.outcome.entries.push(ScanEntry {
            path: target.to_path_buf(),
            identity: self.identity(child),
            parent_identity: self.identity(child.parent().unwrap_or(Path::new(""))),
            title: name.to_owned(),
            is_folder: false,
            mime_type: Some(mime_type),
            size: metadata.as_ref().map(std::fs::Metadata::len),
            modified: metadata.as_ref().and_then(|meta| meta.modified().ok()),
        });
    }

    /// The format of a file: by its name first, by its content second.
    ///
    /// The name is the one the owner gave the file; the content is read from
    /// the file on disk. For a link that is indexed as its own entry the two
    /// are not the same file.
    fn mime_of(&self, entry: &Path, target: &Path) -> Option<String> {
        let extension = entry
            .extension()
            .map(|value| value.to_string_lossy().into_owned());
        if let Some(mime_type) = extension.as_deref().and_then(mime_of_extension) {
            return Some(mime_type.to_owned());
        }
        let head = read_head(target);
        sniff(&head).map(str::to_owned)
    }

    /// Whether the file is already indexed under another name.
    ///
    /// Two names of one file — a hard link — are one object, not two
    /// (`FUN-011`). A file system that gives no identity to its files is not
    /// asked to answer.
    fn is_second_name(&mut self, target: &Path) -> bool {
        let Ok(identity) = self.filesystem.file_identity(target) else {
            return false;
        };
        if identity.device == 0 && identity.inode == 0 {
            return false;
        }
        !self.seen_files.insert(identity)
    }

    /// Resolves a symbolic link according to the policy.
    ///
    /// `None` means the entry is not indexed: either the policy forbids it, or
    /// its target lies outside the root, or the target cannot be read.
    fn resolve(&mut self, path: &Path, child: &Path) -> Option<Resolution> {
        let metadata = std::fs::symlink_metadata(path).ok()?;
        if !metadata.file_type().is_symlink() {
            return Some(Resolution::Own {
                path: path.to_path_buf(),
            });
        }
        match self.options.symlink_policy {
            SymlinkPolicy::Skip => None,
            SymlinkPolicy::Follow => self
                .follow_link(path, child)
                .map(|_| Resolution::Transparent),
            SymlinkPolicy::IndexAsLink => self
                .follow_link(path, child)
                .map(|target| Resolution::Own { path: target }),
        }
    }

    /// The target of a symbolic link, when it lies inside the root.
    fn follow_link(&mut self, path: &Path, child: &Path) -> Option<PathBuf> {
        let target = std::fs::canonicalize(path).ok()?;
        if !target.starts_with(&self.canonical_root) {
            self.diagnose(format!(
                "the link \"{}\" points outside the root and is not indexed",
                display_relative(child)
            ));
            return None;
        }
        Some(target)
    }

    /// Whether the platform hides this entry.
    fn is_hidden(&self, path: &Path, name: &str) -> bool {
        self.filesystem.is_hidden(path) || name.starts_with("._")
    }

    /// Whether the configuration excludes this path.
    ///
    /// The comparison uses the identity of the path, so that the exclusion
    /// behaves like the identity: `private` excludes `Private` on a volume
    /// that does not tell the two apart.
    fn is_excluded(&self, child: &Path) -> bool {
        let identity = identity_of_with(child, self.root.case_sensitivity);
        let text = identity.to_lowercase();
        self.options.excludes.iter().any(|excluded| {
            let excluded = excluded.trim_matches('/').to_lowercase();
            !excluded.is_empty() && (text == excluded || text.starts_with(&format!("{excluded}/")))
        })
    }

    /// The identity of a path relative to the root.
    fn identity(&self, relative: &Path) -> String {
        identity_of_with(relative, self.root.case_sensitivity)
    }

    /// Records what could not be done.
    pub(super) fn diagnose(&mut self, message: String) {
        self.outcome.diagnostics.push(message);
    }
}

/// The name of a path below the root, for a diagnostic.
///
/// The diagnostics of the server carry the path below the root and never the
/// path of the disk (`SEC-094`).
fn display_relative(relative: &Path) -> String {
    let text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        ".".to_owned()
    } else {
        text
    }
}

/// Reads the first bytes of a file.
fn read_head(path: &Path) -> Vec<u8> {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    let mut head = vec![0u8; HEAD_LENGTH];
    let Ok(read) = file.read(&mut head) else {
        return Vec::new();
    };
    head.truncate(read);
    head
}
