//! The cache of produced pictures.
//!
//! What: a bounded directory on disk that holds the pictures that were
//!      produced, one file per picture.
//! Why: a control point asks for the same picture again and again, and a
//!      picture is expensive to produce. The cache is disposable: it may be
//!      deleted at any time, and it never grows beyond the bound it was given,
//!      so that a library of large photos can never fill a disk.
//! Example: `PictureCache::put` writes a picture and `PictureCache::get`
//!       reads it back.
//!
//! **The trust boundary.** The key of a picture is the name of a file, and a
//! checked name cannot leave the directory of the cache (`SEC-003`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The longest key a cache accepts.
const MAX_KEY_LENGTH: usize = 64;

/// A cache of produced pictures on disk.
///
/// The cache is disposable: it may be deleted at any time, and it is rebuilt
/// when it is needed again. It never grows beyond the bound it was given.
#[derive(Debug, Clone)]
pub struct PictureCache {
    directory: PathBuf,
    max_bytes: u64,
}

impl PictureCache {
    /// A cache that keeps its pictures in `directory` and never uses more than
    /// `max_bytes`.
    pub fn new(directory: PathBuf, max_bytes: u64) -> Self {
        Self {
            directory,
            max_bytes,
        }
    }

    /// The type and the bytes of a cached picture, or `None` when the cache
    /// does not hold it.
    ///
    /// A broken file gives `None` and is not an error: a damaged cache is a
    /// cache that is empty.
    pub fn get(&self, key: &str) -> Option<(String, Vec<u8>)> {
        if !is_valid_key(key) {
            return None;
        }
        let bytes = fs::read(self.directory.join(key)).ok()?;
        let (first, rest) = bytes.split_first()?;
        let length = usize::from(*first);
        if rest.len() < length {
            return None;
        }
        let (mime_type, picture) = rest.split_at(length);
        let mime_type = std::str::from_utf8(mime_type).ok()?;
        Some((mime_type.to_owned(), picture.to_vec()))
    }

    /// Puts a picture into the cache.
    ///
    /// The picture is refused when it is larger than the bound of the cache.
    pub fn put(&self, key: &str, mime_type: &str, bytes: &[u8]) -> io::Result<()> {
        if !is_valid_key(key) {
            return Err(refused("the key is not a valid name for a file"));
        }
        if mime_type.len() > usize::from(u8::MAX) {
            return Err(refused("the type of the picture is too long"));
        }
        let content = content_of(mime_type, bytes);
        if content.len() as u64 > self.max_bytes {
            return Err(refused("the picture is larger than the whole cache"));
        }
        fs::create_dir_all(&self.directory)?;
        let target = self.directory.join(key);
        let temporary = self.directory.join(temporary_name(key));
        if let Err(error) = fs::write(&temporary, &content) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        // The rename is the moment the picture appears: a reader sees either
        // the old picture or the new one, never half of it.
        fs::rename(&temporary, &target)?;
        self.evict(&target);
        Ok(())
    }

    /// How many bytes the cache holds.
    pub fn size(&self) -> u64 {
        self.files().iter().map(|file| file.length).sum()
    }

    /// How many pictures the cache holds.
    pub fn entries(&self) -> usize {
        self.files().len()
    }

    /// The directory the cache keeps its pictures in.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The pictures the cache holds, with the moment they were written.
    ///
    /// A cache whose directory was deleted is empty and not an error.
    fn files(&self) -> Vec<CachedFile> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let mut files = Vec::new();
        for entry in entries.flatten() {
            // A name that is not text cannot be a key, and the lossy reading
            // of such a name is refused by the check below.
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_valid_key(&name) {
                continue;
            }
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            // A file whose moment is unknown is the oldest one: a picture of
            // unknown age is the one that is most likely to be obsolete.
            let written = metadata.modified().ok().unwrap_or(UNIX_EPOCH);
            files.push(CachedFile {
                path: entry.path(),
                length: metadata.len(),
                written,
            });
        }
        files
    }

    /// Removes the oldest pictures until the cache is inside its bound again.
    ///
    /// The picture just written is kept: a caller that paid for it reads it back.
    fn evict(&self, keep: &Path) {
        let mut files = self.files();
        let mut total: u64 = files.iter().map(|file| file.length).sum();
        files.sort_by_key(|file| file.written);
        for file in files {
            if total <= self.max_bytes {
                return;
            }
            if file.path == keep {
                continue;
            }
            if fs::remove_file(&file.path).is_ok() {
                total = total.saturating_sub(file.length);
            }
        }
    }
}

/// One picture in the cache.
struct CachedFile {
    path: PathBuf,
    length: u64,
    written: SystemTime,
}

/// The error of a picture the cache refuses, with a reason for the log.
fn refused(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, reason)
}

/// Whether the key may be used as the name of a file.
///
/// Only the letters, the digits, the dash and the underline are allowed, so a
/// key can never name another directory.
fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LENGTH
        && key.chars().all(|letter| {
            letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-' || letter == '_'
        })
}

/// The bytes of a picture as they are written to the disk: the first byte
/// gives the length of the type, so the reader knows where the picture begins.
fn content_of(mime_type: &str, bytes: &[u8]) -> Vec<u8> {
    let mut content = Vec::with_capacity(bytes.len() + mime_type.len() + 1);
    content.push(mime_type.len() as u8);
    content.extend_from_slice(mime_type.as_bytes());
    content.extend_from_slice(bytes);
    content
}

/// The name of the temporary file a picture is written to.
///
/// The name starts with a dot, which is not a valid key, so the file is never
/// counted as a picture of the cache and never evicted by it.
fn temporary_name(key: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let number = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(".tmp-{key}-{}-{number}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::PictureCache;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use std::thread::sleep;
    use std::time::Duration;

    /// The type of the pictures the tests store.
    const TYPE: &str = "image/jpeg";

    /// A cache in a fresh directory.
    fn cache(max_bytes: u64) -> (tempfile::TempDir, PictureCache) {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let cache = PictureCache::new(PathBuf::from(directory.path()), max_bytes);
        (directory, cache)
    }

    /// Puts a picture of the given length into the cache.
    fn store(cache: &PictureCache, key: &str, length: usize) {
        let picture = vec![7u8; length];
        cache
            .put(key, TYPE, &picture)
            .expect("the picture is stored");
    }

    #[test]
    fn a_picture_is_stored_and_read_back() {
        let (_directory, cache) = cache(10_000);
        store(&cache, "cover-1", 64);
        let read = cache.get("cover-1").expect("the picture is there");
        assert_eq!(read.0, TYPE);
        assert_eq!(read.1, vec![7u8; 64]);
        assert_eq!(cache.entries(), 1);
        assert!(cache.directory().is_absolute());
    }

    #[test]
    fn a_key_that_is_not_a_file_name_is_refused() {
        let (_directory, cache) = cache(10_000);
        for key in ["../escape", "UPPER", "dots.here", "a b", ""] {
            assert!(cache.get(key).is_none(), "{key} was read");
            let error = cache.put(key, TYPE, &[7u8; 8]).expect_err("refused");
            assert_eq!(error.kind(), ErrorKind::InvalidInput, "{key}");
        }
        let long = "x".repeat(65);
        assert!(cache.get(&long).is_none(), "a long key was read");
        assert!(cache.put(&long, TYPE, &[7u8; 8]).is_err());
        assert_eq!(cache.entries(), 0);
    }

    #[test]
    fn the_cache_stays_inside_its_bound() {
        let (_directory, cache) = cache(300);
        store(&cache, "first", 100);
        sleep(Duration::from_millis(20));
        store(&cache, "second", 100);
        assert!(cache.size() <= 300, "the cache is too large");
        assert_eq!(cache.entries(), 2);
        sleep(Duration::from_millis(20));
        store(&cache, "third", 100);
        assert!(cache.size() <= 300, "the cache is too large");
        assert_eq!(cache.entries(), 2, "the oldest picture was not removed");
        assert!(cache.get("first").is_none(), "the oldest picture is gone");
        assert!(cache.get("third").is_some(), "the newest picture is there");
    }

    #[test]
    fn a_picture_that_is_larger_than_the_bound_is_refused() {
        let (_directory, cache) = cache(100);
        let error = cache.put("huge", TYPE, &[7u8; 200]).expect_err("refused");
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(cache.entries(), 0);
        assert!(cache.get("huge").is_none());
    }

    #[test]
    fn a_deleted_directory_is_recreated() {
        let (directory, cache) = cache(10_000);
        store(&cache, "cover", 32);
        std::fs::remove_dir_all(cache.directory()).expect("the directory is deleted");
        assert_eq!(cache.entries(), 0);
        assert_eq!(cache.size(), 0);
        store(&cache, "cover", 32);
        assert_eq!(cache.entries(), 1);
        assert!(cache.directory().starts_with(directory.path()));
    }

    #[test]
    fn a_broken_file_gives_no_picture() {
        let (_directory, cache) = cache(10_000);
        store(&cache, "broken", 32);
        std::fs::write(cache.directory().join("broken"), [9u8, 1, 2]).expect("broken");
        assert!(cache.get("broken").is_none(), "a broken file gives nothing");
    }
}
