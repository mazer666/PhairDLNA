//! The picture that belongs to a media file.
//!
//! What: the search for the picture of a file — the cover that is embedded in
//!      an audio file, the image that lies next to it, the thumbnail of a
//!      photo — and the reading of that picture.
//! Why: the picture is the part of the library a client asks for first, and it
//!      is also the part a damaged file breaks first. The search therefore
//!      never trusts a file: it stays inside the roots, it reads bounded
//!      amounts of data, and a file it cannot understand gives no picture
//!      instead of an error (`F-10`, `SEC-067`, `UX-004`).
//! Example: `ArtworkFinder::picture_of` answers the bytes of the picture of a
//!       track, or nothing at all.
//!
//! The data of the pictures — the bounds, the roots and the sources — lives in
//! `types.rs`.
//!
//! **The order of the search.** A sidecar wins over an embedded cover, and a
//! media file that is a picture itself is its own picture (`F-10` § 3).

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::config::SymlinkPolicy;
use crate::http::path::is_inside_roots;

pub use self::types::{Artwork, ArtworkSource, PictureLimits, RootBoundary};
use super::sidecar;
use super::{id3, mp4, thumbnail};

mod types;

/// The finder of the pictures of a library.
///
/// The finder remembers what it has found, because a browse asks for the same
/// pictures again and again, and because a file that is read again and again
/// is a file that wears out a disk (`F-10` § 5).
///
/// The memory is shared by every copy of the finder: the catalogue and the
/// server hold one each, and they must see the same answers.
#[derive(Debug, Clone, Default)]
pub struct ArtworkFinder {
    roots: Vec<RootBoundary>,
    limits: PictureLimits,
    found: Arc<Mutex<HashMap<String, Option<ArtworkSource>>>>,
}

impl ArtworkFinder {
    /// A finder that looks inside the given roots.
    pub fn new(roots: Vec<RootBoundary>, limits: PictureLimits) -> Self {
        Self {
            roots,
            limits,
            found: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The bounds the finder works with.
    pub fn limits(&self) -> PictureLimits {
        self.limits
    }

    /// Where the picture of a media file comes from.
    ///
    /// The answer is `None` when the file has no picture at all: a client that
    /// asks for it then gets a truthful answer instead of a broken image
    /// (`F-10` § 6).
    pub fn source_of(&self, id: &str, path: &Path) -> Option<ArtworkSource> {
        let key = self.key(id, path);
        if let Some(cached) = self.remembered(&key) {
            return cached;
        }
        let source = self.search(path);
        self.remember(key, source.clone());
        source
    }

    /// The picture of a media file.
    ///
    /// The bytes are read only when a client asks for them; a file that cannot
    /// be read gives no picture and never an error (`UX-004`).
    pub fn picture_of(&self, id: &str, path: &Path) -> Option<Artwork> {
        let source = self.source_of(id, path)?;
        self.read(path, &source)
    }

    /// The key under which the answer for a file is remembered.
    ///
    /// The key carries the size and the modification time of the file, so that
    /// a file that changed is looked at again (`F-10` § 5).
    fn key(&self, id: &str, path: &Path) -> String {
        cache_key(id, path)
    }

    /// The answer the finder remembers for a key.
    fn remembered(&self, key: &str) -> Option<Option<ArtworkSource>> {
        let found = self.found.lock().ok()?;
        found.get(key).cloned()
    }

    /// Remembers the answer for a key.
    fn remember(&self, key: String, source: Option<ArtworkSource>) {
        if let Ok(mut found) = self.found.lock() {
            found.insert(key, source);
        }
    }

    /// The root a file belongs to.
    ///
    /// A file that lies in no root has no picture: the pipeline never looks
    /// outside the library (`SEC-003`).
    fn root_of(&self, path: &Path) -> Option<&RootBoundary> {
        let canonical = std::fs::canonicalize(path).ok()?;
        let paths: Vec<PathBuf> = self.roots.iter().map(|root| root.path.clone()).collect();
        if !is_inside_roots(&canonical, &paths) {
            return None;
        }
        self.roots
            .iter()
            .find(|root| canonical.starts_with(&root.path))
    }

    /// The search for the picture of a file.
    fn search(&self, path: &Path) -> Option<ArtworkSource> {
        let root = self.root_of(path)?;
        if let Some(mime_type) = picture_mime_type(path) {
            return Some(ArtworkSource::Thumbnail {
                mime_type: mime_type.to_owned(),
            });
        }
        if let Some(sidecar) = self.sidecar(path, root) {
            return Some(sidecar);
        }
        self.embedded(path)
    }

    /// The sidecar of a file, when the sidecar is inside the library.
    fn sidecar(&self, path: &Path, root: &RootBoundary) -> Option<ArtworkSource> {
        for candidate in sidecar::candidates(path) {
            let mime_type = match sidecar::mime_type_of(&candidate) {
                Some(mime_type) => mime_type,
                None => continue,
            };
            let Ok(metadata) = std::fs::symlink_metadata(&candidate) else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            // A link is only followed when the root allows it: under the
            // policy that skips links, a link is not a sidecar (SEC-003).
            if metadata.file_type().is_symlink() && root.symlink_policy == SymlinkPolicy::Skip {
                continue;
            }
            let Ok(target) = std::fs::canonicalize(&candidate) else {
                continue;
            };
            if !target.starts_with(&root.path) {
                continue;
            }
            if metadata.len() > self.limits.max_sidecar_bytes {
                continue;
            }
            return Some(ArtworkSource::Sidecar {
                path: candidate,
                mime_type: mime_type.to_owned(),
            });
        }
        None
    }

    /// The picture that is embedded in an audio file.
    ///
    /// A cover that is larger than the limit is not a cover of this server: it
    /// is discarded, and the search goes on with the other sources (`F-10`
    /// § 12).
    fn embedded(&self, path: &Path) -> Option<ArtworkSource> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        let mut file = File::open(path).ok()?;
        let (mime_type, length) = match extension.as_str() {
            "mp3" => {
                let info = id3::picture_info(&mut file)?;
                (info.mime_type, info.length)
            }
            "m4a" | "m4b" | "mp4" => {
                let info = mp4::picture_info(&mut file)?;
                (info.mime_type, info.length)
            }
            _ => return None,
        };
        if length > self.limits.max_embedded_bytes {
            return None;
        }
        Some(ArtworkSource::Embedded { mime_type })
    }

    /// Reads the picture of a file.
    fn read(&self, path: &Path, source: &ArtworkSource) -> Option<Artwork> {
        match source {
            ArtworkSource::Sidecar { path, mime_type } => {
                let bytes = read_bounded(path, self.limits.max_sidecar_bytes)?;
                Some(Artwork {
                    mime_type: mime_type.clone(),
                    bytes,
                })
            }
            ArtworkSource::Embedded { mime_type } => {
                let mut file = File::open(path).ok()?;
                let extension = path.extension()?.to_str()?.to_ascii_lowercase();
                let bytes = match extension.as_str() {
                    "mp3" => id3::read_picture(&mut file, self.limits.max_embedded_bytes)?.bytes,
                    "m4a" | "m4b" | "mp4" => {
                        mp4::read_picture(&mut file, self.limits.max_embedded_bytes)?.bytes
                    }
                    _ => return None,
                };
                Some(Artwork {
                    mime_type: mime_type.clone(),
                    bytes,
                })
            }
            ArtworkSource::Thumbnail { .. } => {
                let bytes = read_bounded(path, self.limits.max_sidecar_bytes)?;
                let produced = thumbnail::thumbnail(&bytes, &self.limits).ok()?;
                Some(Artwork {
                    mime_type: "image/jpeg".to_owned(),
                    bytes: produced,
                })
            }
        }
    }
}

/// The key under which the picture of a file is kept.
///
/// The key carries the identifier of the object, the size of the file and the
/// moment it was changed, so that a picture of a file that changed is produced
/// again and a picture of a file that did not change is not (`F-10` § 5).
///
/// The key is a digest: it is a name of a file in the cache, and it must not
/// carry a path of the disk (`PLT-008`).
pub fn cache_key(id: &str, path: &Path) -> String {
    let metadata = std::fs::metadata(path).ok();
    let size = metadata.as_ref().map(|data| data.len()).unwrap_or(0);
    let modified = metadata
        .as_ref()
        .and_then(|data| data.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|time| time.as_nanos())
        .unwrap_or(0);
    let mut hasher = blake3::Hasher::new();
    hasher.update(id.as_bytes());
    hasher.update(&size.to_le_bytes());
    hasher.update(&modified.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

/// The type of a file that is a picture itself.
///
/// Only the formats the catalogue indexes are pictures here; every other file
/// is looked at for a sidecar or for an embedded cover.
fn picture_mime_type(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        _ => None,
    }
}

/// Reads a file, but only when it is small enough.
///
/// A file that grew behind the back of the catalogue is refused instead of
/// being read into the memory of the server (`SEC-067`).
fn read_bounded(path: &Path, max_bytes: u64) -> Option<Vec<u8>> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() as u64 > max_bytes {
        return None;
    }
    Some(bytes)
}
