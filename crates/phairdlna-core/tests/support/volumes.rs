#![allow(dead_code)]
//! Fixtures for the tests of the volumes.
//!
//! What: a filesystem whose volume identities and fingerprints a test writes
//!       down, and everything else the portable adapter answers.
//! Why: the acceptance criteria of F-08 for a drive that is unplugged and
//!       plugged in again are written against a physical drive. No test may ask
//!       the owner of the machine to unplug one, so the test plays the part of
//!       the platform and says what the platform would say (`FUN-017`,
//!       `PLT-011`).
//! Example: `script.set_volume(path, "vol-2")` makes the platform report that
//!       the drive at the path is another volume than it was.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::MediaRoot;
use phairdlna_core::platform::{
    CaseSensitivity, FileId, FileSystem, LocalFileSystem, MediaPath, PlatformError, PlatformResult,
    VolumeFingerprint, VolumeId,
};

/// The volume identity the portable adapter reports.
const LOCAL_VOLUME: &str = "local";

/// A filesystem whose volumes a test can script.
#[derive(Debug, Default)]
pub struct ScriptedFileSystem {
    volumes: Mutex<HashMap<PathBuf, String>>,
    fingerprints: Mutex<HashMap<PathBuf, String>>,
    blind: Mutex<bool>,
}

impl ScriptedFileSystem {
    /// A filesystem that says "local" for every path.
    pub fn new() -> Self {
        Self::default()
    }

    /// The platform cannot report a volume for a path it cannot read.
    ///
    /// A real adapter has to look at the path to answer, and a path that is
    /// not there has no volume. The portable adapter answers `local` for every
    /// path, which hides the question; a test that wants to see the answer of
    /// a real adapter turns the blindness on.
    pub fn blind_to_missing_paths(&self) {
        let mut blind = self.blind.lock().unwrap_or_else(|error| error.into_inner());
        *blind = true;
    }

    /// The volume of a path is another one.
    ///
    /// The script is kept for the folder and for everything below it, so that
    /// a test may write down the volume of a root and need not name every file
    /// of it.
    pub fn set_volume(&self, path: &Path, volume: &str) {
        let mut volumes = self
            .volumes
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        volumes.insert(path.to_path_buf(), volume.to_owned());
    }

    /// The fingerprint of a path is another one.
    pub fn set_fingerprint(&self, path: &Path, fingerprint: &str) {
        let mut fingerprints = self
            .fingerprints
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        fingerprints.insert(path.to_path_buf(), fingerprint.to_owned());
    }

    /// The volume the script gives for a path.
    fn volume_of(&self, path: &Path) -> VolumeId {
        let volumes = self
            .volumes
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for ancestor in path.ancestors() {
            if let Some(volume) = volumes.get(ancestor) {
                return VolumeId::new(volume.clone());
            }
        }
        VolumeId::new(LOCAL_VOLUME)
    }

    /// The fingerprint the script gives for a path.
    fn fingerprint_of(&self, path: &Path) -> Option<VolumeFingerprint> {
        let fingerprints = self
            .fingerprints
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for ancestor in path.ancestors() {
            if let Some(fingerprint) = fingerprints.get(ancestor) {
                return Some(VolumeFingerprint::new(fingerprint.clone()));
            }
        }
        None
    }
}

impl FileSystem for ScriptedFileSystem {
    fn media_path(&self, path: &Path) -> PlatformResult<MediaPath> {
        LocalFileSystem.media_path(path)
    }

    fn volume_of(&self, path: &Path) -> PlatformResult<VolumeId> {
        let blind = *self.blind.lock().unwrap_or_else(|error| error.into_inner());
        if blind && !path.exists() {
            return Err(PlatformError::io(
                "stat a path",
                std::io::Error::new(std::io::ErrorKind::NotFound, "the path is not there"),
            ));
        }
        Ok(self.volume_of(path))
    }

    fn case_sensitivity(&self, volume: &VolumeId) -> CaseSensitivity {
        LocalFileSystem.case_sensitivity(volume)
    }

    fn file_identity(&self, path: &Path) -> PlatformResult<FileId> {
        LocalFileSystem.file_identity(path)
    }

    fn volume_fingerprint(&self, path: &Path) -> PlatformResult<VolumeFingerprint> {
        match self.fingerprint_of(path) {
            Some(fingerprint) => Ok(fingerprint),
            None => LocalFileSystem.volume_fingerprint(path),
        }
    }
}

/// The bytes of a file the sniffer recognises as an MP4.
pub const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

/// A root of the library.
pub fn root(name: &str, path: &Path) -> MediaRoot {
    MediaRoot {
        name: name.to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    }
}

/// Writes a file, creating the folders above it.
pub fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the folder is created");
    }
    std::fs::write(path, content).expect("the file is written");
}

/// Two roots of a library: one on a drive of its own, one on the disk of the
/// machine.
pub fn library() -> (tempfile::TempDir, tempfile::TempDir) {
    let drive = tempfile::tempdir().expect("a temporary directory");
    let disk = tempfile::tempdir().expect("another temporary directory");
    write_file(&drive.path().join("film.mkv"), MP4_HEAD);
    write_file(&disk.path().join("track.mp3"), b"ID3\0\0\0\0");
    (drive, disk)
}

/// The roots of the library.
pub fn roots_of(drive: &Path, disk: &Path) -> Vec<MediaRoot> {
    vec![root("Films", drive), root("Music", disk)]
}

/// The children of a container, with their titles.
pub fn children_of(catalogue: &Catalogue, container_id: &str) -> Vec<(String, String)> {
    catalogue
        .container(container_id)
        .expect("the container is in the catalogue")
        .children
        .iter()
        .map(|child| (child.id.clone(), child.title.clone()))
        .collect()
}

/// The identifier of the file with the given title below a container.
pub fn file_below(catalogue: &Catalogue, container_id: &str, title: &str) -> Option<String> {
    let container = catalogue.container(container_id)?;
    for child in &container.children {
        if child.title == title {
            return Some(child.id.clone());
        }
        if let Some(found) = file_below(catalogue, &child.id, title) {
            return Some(found);
        }
    }
    None
}
