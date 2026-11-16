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

use super::super::identity::{disambiguated_identity, identity_key};
use super::super::media::{is_indexable, mime_of_extension};
use super::super::playlist::PlaylistFormat;
use super::super::sniff::{sniff, HEAD_LENGTH};
use super::super::subtitle;
use super::encoding::charset_of;
use super::folder::Folder;
use super::playlist::{self as playlists, PlaylistContext};
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
    ///
    /// The names of the files of the folder are read once: a subtitle is
    /// recognised by the name that lies next to the film, and the folder is
    /// the only place that knows it (`FUN-016`).
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
        let mut readable = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => readable.push(entry),
                Err(error) => self.diagnose(format!(
                    "an entry of the folder \"{}\" cannot be read: {error}",
                    display_relative(relative)
                )),
            }
        }
        let folder = Folder::of(&readable, relative, self.root.case_sensitivity);
        for entry in readable {
            self.visit(&entry.path(), relative, depth, &folder);
        }
    }

    /// Visits one entry of a folder.
    fn visit(&mut self, path: &Path, relative: &Path, depth: usize, folder: &Folder) {
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
                    self.visit_file(path, &child, &name, &target, folder);
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
            identity_key: self.identity_key(child),
            parent_identity: self.identity(child.parent().unwrap_or(Path::new(""))),
            title: name.to_owned(),
            is_folder: true,
            mime_type: None,
            size: None,
            modified: None,
            file_identity: None,
            subtitle: None,
            playlist: None,
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
    ///
    /// A playlist is read here, and a subtitle is paired with its film here:
    /// both are facts about the disk, and a browse must not have to look for
    /// them again (`FUN-015`, `FUN-016`, `PERF-008`).
    fn visit_file(
        &mut self,
        entry: &Path,
        child: &Path,
        name: &str,
        target: &Path,
        folder: &Folder,
    ) {
        let Some(mime_type) = self.mime_of(entry, target) else {
            return;
        };
        if !is_indexable(&mime_type) {
            return;
        }
        let file_identity = if entry == target {
            self.filesystem.file_identity(target).ok()
        } else {
            // A link that is indexed as its own entry is an object of its own;
            // the identity of the file it points at belongs to that file.
            None
        };
        if entry == target && self.is_second_name(target, file_identity.as_ref()) {
            self.diagnose(format!(
                "the file \"{}\" is another name for a file that is already indexed",
                display_relative(child)
            ));
            return;
        }
        let metadata = std::fs::metadata(target).ok();
        let (playlist, subtitle) = self.extras_of(entry, child, name, target, folder);
        let identity = self.identity_of_file(child, name, folder, file_identity.as_ref());
        let identity_key = self.identity_key_of_file(child, name, folder, file_identity.as_ref());
        self.outcome.entries.push(ScanEntry {
            path: target.to_path_buf(),
            identity,
            identity_key,
            parent_identity: self.identity(child.parent().unwrap_or(Path::new(""))),
            title: name.to_owned(),
            is_folder: false,
            mime_type: Some(mime_type),
            size: metadata.as_ref().map(std::fs::Metadata::len),
            modified: metadata.as_ref().and_then(|meta| meta.modified().ok()),
            file_identity,
            subtitle,
            playlist,
        });
    }

    /// The identity of a file.
    ///
    /// The identity of the path is the identity of the file, unless another
    /// file of the folder claims it: two names that differ only in the Unicode
    /// normalisation form are two files on a volume that keeps the bytes, and
    /// such a file keeps the identity the platform gives it (`FUN-020`,
    /// `PLT-005`).
    fn identity_of_file(
        &self,
        child: &Path,
        name: &str,
        folder: &Folder,
        file: Option<&FileId>,
    ) -> String {
        let identity = self.identity(child);
        if !folder.collides(name) {
            return identity;
        }
        match file {
            Some(file) => disambiguated_identity(&identity, file),
            None => identity,
        }
    }

    /// The identity key of a path below the root.
    ///
    /// The key is scoped by the volume and built from the path the owner gave,
    /// so that the same file reached through two roots of one volume has one
    /// key (`FUN-020`).
    fn identity_key(&self, child: &Path) -> String {
        identity_key(
            &self.root.volume,
            &self.root.path.join(child),
            self.root.case_sensitivity,
        )
    }

    /// The identity key of a file.
    ///
    /// A file that shares its path with another file keeps the identity the
    /// platform gives it, so that the two files have two keys and two object
    /// identifiers (`FUN-020`, `PLT-005`).
    fn identity_key_of_file(
        &self,
        child: &Path,
        name: &str,
        folder: &Folder,
        file: Option<&FileId>,
    ) -> String {
        let key = self.identity_key(child);
        if !folder.collides(name) {
            return key;
        }
        match file {
            Some(file) => disambiguated_identity(&key, file),
            None => key,
        }
    }

    /// The facts about the disk that a file carries: its playlist and its
    /// subtitle.
    ///
    /// A file that is a playlist is read and resolved; a file that has a
    /// subtitle next to it is paired with it. Every other file carries nothing
    /// and costs nothing.
    fn extras_of(
        &mut self,
        entry: &Path,
        child: &Path,
        name: &str,
        target: &Path,
        folder: &Folder,
    ) -> (Option<super::PlaylistPlan>, Option<super::SubtitleEntry>) {
        if let Some(format) = playlist_format_of(entry) {
            let label = display_relative(child);
            let base = target.parent().unwrap_or(&self.root.path).to_path_buf();
            let context = PlaylistContext {
                root: &self.root.path,
                canonical_root: &self.canonical_root,
                case_sensitivity: self.root.case_sensitivity,
                bounds: self.options.playlist_bounds,
            };
            let (plan, sentences) = playlists::read(target, format, &base, &label, &context);
            for sentence in sentences {
                self.diagnose(sentence);
            }
            return (Some(plan), None);
        }
        (None, self.subtitle_of(entry, child, name, folder))
    }

    /// The subtitle that lies next to a media file.
    ///
    /// The sidecar is the file that carries the same base name with one of the
    /// subtitle extensions. When several match, the one with the highest
    /// priority extension wins (`FUN-016`).
    fn subtitle_of(
        &self,
        entry: &Path,
        child: &Path,
        name: &str,
        folder: &Folder,
    ) -> Option<super::SubtitleEntry> {
        let stem = Path::new(name).file_stem()?.to_string_lossy().into_owned();
        let sidecar = subtitle::sidecar_of(&stem, &folder.names)?;
        let folder = entry.parent().unwrap_or(Path::new(""));
        let path = folder.join(&sidecar.name);
        let relative = child.parent().unwrap_or(Path::new("")).join(&sidecar.name);
        let size = std::fs::metadata(&path).ok().map(|metadata| metadata.len());
        let charset = charset_of(&path);
        Some(super::SubtitleEntry {
            name: sidecar.name,
            identity: self.identity(&relative),
            path,
            mime_type: sidecar.mime_type,
            charset,
            size,
        })
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
    fn is_second_name(&mut self, _target: &Path, identity: Option<&FileId>) -> bool {
        let Some(identity) = identity else {
            return false;
        };
        if identity.device == 0 && identity.inode == 0 {
            return false;
        }
        !self.seen_files.insert(identity.clone())
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

/// The format of a playlist, judged by the name of the file.
///
/// A file whose name says nothing about a playlist is not one, whatever it
/// contains: the catalogue never guesses the format of a file (`FUN-019`).
fn playlist_format_of(entry: &Path) -> Option<PlaylistFormat> {
    let extension = entry.extension()?.to_str()?;
    PlaylistFormat::of_extension(extension)
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
