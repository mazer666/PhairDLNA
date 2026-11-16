//! The pictures of the daemon.
//!
//! What: the disposable cache of the pictures that were produced, and the
//!       folder it lives in.
//! Why: a picture is produced from a file of the library, and a control point
//!       asks for the same picture again and again. The cache keeps the
//!       picture, and it is bounded, so that it cannot fill the disk of the
//!       user (`FUN-018`, `SEC-064`).
//! Example: `Pictures::new` builds the cache of a configuration, and
//!       `Pictures::cache` hands it to the route that answers the requests.
//!
//! **The cache is disposable.** It may be deleted at any time; the daemon
//! writes the pictures again when a client asks for them.

use std::path::PathBuf;

use phairdlna_core::config::{user_data_dir, Config};
use phairdlna_core::media::{ArtworkFinder, PictureCache, PictureLimits, RootBoundary};

use super::library::Library;

/// The pictures the daemon keeps.
pub struct Pictures {
    cache: PictureCache,
}

impl Pictures {
    /// The cache of the pictures of a server.
    ///
    /// The bound of the cache is the bound the configuration names: the cache
    /// never uses more disk than the user allowed (`SEC-064`).
    pub fn new(config: &Config) -> Self {
        let cache = PictureCache::new(cache_directory(config), config.limits.max_cache_bytes);
        Self { cache }
    }

    /// The cache of the produced pictures.
    pub fn cache(&self) -> &PictureCache {
        &self.cache
    }
}

/// Gives the library its finder of the pictures, when the user wants them.
///
/// A library with many files may switch the pictures off, and then no file of
/// the library is read to look for a cover (`PERF-004`).
pub fn attach(library: Library, config: &Config) -> Library {
    if config.library.extract_artwork {
        library.with_pictures(finder(config))
    } else {
        library
    }
}

/// The finder of the pictures of the library.
///
/// The roots are resolved before they are given to the finder: a picture is
/// only inside the library when its target is inside the root after the links
/// are resolved (`SEC-003`).
pub fn finder(config: &Config) -> ArtworkFinder {
    let roots = config
        .library
        .roots
        .iter()
        .map(|root| RootBoundary {
            path: std::fs::canonicalize(&root.path).unwrap_or_else(|_| PathBuf::from(&root.path)),
            symlink_policy: root.symlink_policy.unwrap_or(config.server.symlink_policy),
        })
        .collect();
    ArtworkFinder::new(roots, PictureLimits::default())
}

/// The folder the cache of the pictures lives in.
///
/// The setting of the configuration wins; without it the data directory of the
/// user is used, and the cache is a folder of its own inside it (`FUN-018`).
fn cache_directory(config: &Config) -> PathBuf {
    match config.library.cache_directory.as_deref() {
        Some(directory) => PathBuf::from(directory),
        None => user_data_dir().join("cache").join("pictures"),
    }
}
