//! The library of the daemon: the catalogue and the rules that keep it fresh.
//!
//! What: the shared catalogue of the daemon, the memory of the volumes of the
//!       roots, and the way the catalogue is read again when a drive comes or
//!       goes.
//! Why: the daemon runs for hours, and the library of a person changes while it
//!       runs: a drive is plugged in, a network share comes back, a disk is
//!       replaced. The catalogue has to follow, and the protocol layer must
//!       still never touch the filesystem (`FUN-017`, `PLT-011`).
//! Example: a request reads the catalogue through `with_catalogue`; a while
//!       later the watch finds that a drive came back and reads the catalogue
//!       again.

use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use phairdlna_core::catalogue::{Catalogue, VolumeRegistry, VolumeReport};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::FileSystem;

/// The library of the daemon.
#[derive(Clone)]
pub struct Library {
    catalogue: Arc<RwLock<Catalogue>>,
    memory: Arc<Mutex<Memory>>,
}

/// The state the library keeps between two reads of the catalogue.
struct Memory {
    registry: VolumeRegistry,
    roots: Vec<MediaRoot>,
    policy: SymlinkPolicy,
}

impl Library {
    /// Builds the library and reads the roots.
    ///
    /// The memory of the volumes starts empty: the first read sees every root
    /// for the first time, and the second read can tell what changed
    /// (`FUN-007`, `PLT-011`).
    pub fn build(roots: &[MediaRoot], policy: SymlinkPolicy, filesystem: &dyn FileSystem) -> Self {
        let mut registry = VolumeRegistry::default();
        let outcome = Catalogue::build_with(roots, policy, filesystem, &mut registry);
        Self {
            catalogue: Arc::new(RwLock::new(outcome.catalogue)),
            memory: Arc::new(Mutex::new(Memory {
                registry,
                roots: roots.to_vec(),
                policy,
            })),
        }
    }

    /// The catalogue as it is now.
    ///
    /// The catalogue is read under a lock, because a drive may come back while
    /// a client is browsing. The lock is held for the length of the answer and
    /// for nothing else.
    pub fn catalogue(&self) -> RwLockReadGuard<'_, Catalogue> {
        // A panic while the catalogue was written leaves it as it was before the
        // panic; the lock must not keep the daemon from serving it.
        self.catalogue
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Reads the catalogue as it is now.
    pub fn with_catalogue<T>(&self, action: impl FnOnce(&Catalogue) -> T) -> T {
        let catalogue = self.catalogue();
        action(&catalogue)
    }

    /// Whether anything changed at the roots since the last read.
    ///
    /// The check looks at the roots only and reads no file of the library, so
    /// that the daemon may ask the question often (`FUN-017`).
    pub fn changed(&self, filesystem: &dyn FileSystem) -> bool {
        let memory = self.memory();
        memory.registry.differs(&memory.roots, filesystem)
    }

    /// Reads the catalogue again and says what changed at the roots.
    ///
    /// The catalogue that was there is replaced as a whole: a root that is not
    /// there any more takes its entries with it, and a root whose volume was
    /// replaced is given new identifiers (`FUN-007`, `PLT-011`).
    pub fn refresh(&self, filesystem: &dyn FileSystem) -> VolumeReport {
        let mut memory = self.write_memory();
        let roots = memory.roots.clone();
        let policy = memory.policy;
        let outcome = Catalogue::build_with(&roots, policy, filesystem, &mut memory.registry);
        let report = outcome.report;
        *self.write_catalogue() = outcome.catalogue;
        report
    }

    /// The roots the library reads.
    pub fn roots(&self) -> Vec<MediaRoot> {
        self.memory().roots.clone()
    }

    /// Reads the memory of the volumes.
    fn memory(&self) -> MutexGuard<'_, Memory> {
        self.memory
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Writes the memory of the volumes.
    fn write_memory(&self) -> MutexGuard<'_, Memory> {
        self.memory
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Writes the catalogue.
    fn write_catalogue(&self) -> RwLockWriteGuard<'_, Catalogue> {
        self.catalogue
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::Library;
    use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
    use phairdlna_core::platform::LocalFileSystem;

    /// A root in a temporary folder.
    fn root(name: &str, path: &std::path::Path) -> MediaRoot {
        MediaRoot {
            name: name.to_owned(),
            path: path.to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        }
    }

    /// The catalogue of a library holds the roots it was built from.
    #[test]
    fn the_catalogue_holds_the_roots() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let roots = vec![root("Films", folder.path())];
        let library = Library::build(&roots, SymlinkPolicy::default(), &LocalFileSystem);
        let names = library.with_catalogue(|catalogue| {
            catalogue
                .roots()
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<Vec<_>>()
        });
        assert_eq!(names, vec!["Films".to_owned()]);
    }

    /// A library that was just built has nothing to report.
    #[test]
    fn a_fresh_library_has_nothing_to_refresh() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let roots = vec![root("Films", folder.path())];
        let library = Library::build(&roots, SymlinkPolicy::default(), &LocalFileSystem);
        assert!(!library.changed(&LocalFileSystem));
    }

    /// A library notices that a root went away and that it came back.
    #[test]
    fn a_library_notices_that_a_root_went_away() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let roots = vec![root("Films", folder.path())];
        let library = Library::build(&roots, SymlinkPolicy::default(), &LocalFileSystem);

        std::fs::remove_dir_all(folder.path()).expect("the drive is unplugged");
        assert!(
            library.changed(&LocalFileSystem),
            "the library notices that the root is gone"
        );
        library.refresh(&LocalFileSystem);
        assert!(
            !library.changed(&LocalFileSystem),
            "the library does not ask the question again after it has read the catalogue"
        );
    }
}
