//! The library of the daemon: the catalogue and the rules that keep it fresh.
//!
//! What: the building of the shared catalogue of the daemon, the memory of the
//!       volumes of the roots, the index of the catalogue on disk, and the way
//!       the catalogue is read again when a drive comes or goes.
//! Why: the daemon runs for hours, and the library of a person changes while it
//!       runs: a drive is plugged in, a network share comes back, a disk is
//!       replaced. The catalogue has to follow, and the protocol layer must
//!       still never touch the filesystem (`FUN-017`, `PLT-011`).
//! Example: a request reads the catalogue through `with_catalogue`; a while
//!       later the watch finds that a drive came back and reads the catalogue
//!       again.

use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use phairdlna_core::catalogue::index::CatalogueIndex;
use phairdlna_core::catalogue::{Catalogue, VolumeRegistry, VolumeReport};
use phairdlna_core::config::{Config, MediaRoot, SymlinkPolicy};
use phairdlna_core::media::ArtworkFinder;
use phairdlna_core::platform::{FileSystem, LocalFileSystem};

use super::log::Logger;
use super::pictures;

/// The library of the daemon.
#[derive(Clone)]
pub struct Library {
    catalogue: Arc<RwLock<Catalogue>>,
    memory: Arc<Mutex<Memory>>,
    pictures: Option<ArtworkFinder>,
    index: Option<Arc<CatalogueIndex>>,
    warnings: Arc<Mutex<Vec<String>>>,
}

/// The state the library keeps between two reads of the catalogue.
struct Memory {
    registry: VolumeRegistry,
    roots: Vec<MediaRoot>,
    policy: SymlinkPolicy,
}

/// Builds the library of the daemon.
///
/// A file the scanner could not read is a warning, not a reason to refuse the
/// start: the rest of the library is still worth serving (`F-08`).
pub fn build_library(config: &Config, logger: &Logger) -> Library {
    let (index, warnings) = open_index(config, logger);
    let library = Library::build_with_index(
        &config.library.roots,
        config.server.symlink_policy,
        &LocalFileSystem,
        index,
        warnings,
    );
    // The pictures are looked for only when the user wants them: a library
    // with many files may switch them off, and then no file of the library is
    // read to look for a cover (`PERF-004`).
    let library = pictures::attach(library, config);
    library.with_catalogue(|catalogue| {
        for diagnostic in catalogue.diagnostics() {
            logger.warn(
                "the catalogue could not read everything",
                &[("reason", diagnostic)],
            );
        }
    });
    library
}

/// Opens the index of the catalogue and says what was wrong with it.
///
/// An index that cannot be opened at all is not a reason to refuse the start:
/// the daemon reads the library as it always did, and the index is written
/// again at the next read (`FUN-009`, `FUN-010`).
fn open_index(config: &Config, logger: &Logger) -> (Option<Arc<CatalogueIndex>>, Vec<String>) {
    let path = config.library.database_file();
    match CatalogueIndex::open(&path) {
        Ok(opened) => {
            let mut warnings = Vec::new();
            if let Some(repair) = opened.recovery {
                logger.warn(
                    "the index of the catalogue was damaged and was moved aside",
                    &[
                        ("reason", &repair.reason),
                        ("file", &repair.moved_to.display().to_string()),
                    ],
                );
                warnings.push(format!(
                    "the index of the catalogue was damaged and was rebuilt: {} (the damaged file \
                     was kept as {})",
                    repair.reason,
                    repair.moved_to.display()
                ));
            }
            (Some(Arc::new(opened.index)), warnings)
        }
        Err(error) => {
            logger.warn(
                "the index of the catalogue cannot be used",
                &[("reason", &error.to_string())],
            );
            (
                None,
                vec![format!(
                    "the index of the catalogue cannot be used: {error}"
                )],
            )
        }
    }
}

impl Library {
    /// Builds the library and reads the roots.
    ///
    /// The memory of the volumes starts empty: the first read sees every root
    /// for the first time, and the second read can tell what changed
    /// (`FUN-007`, `PLT-011`).
    pub fn build(roots: &[MediaRoot], policy: SymlinkPolicy, filesystem: &dyn FileSystem) -> Self {
        Self::build_with_index(roots, policy, filesystem, None, Vec::new())
    }

    /// Builds the library, and takes the catalogue from the index when it holds
    /// one.
    ///
    /// The index holds the catalogue of the last run: the server has it at once
    /// and reads the library again in the background, and the client sees the
    /// library of a moment ago instead of an empty server (`FUN-008`,
    /// `FUN-009`). An index that holds nothing is a library that was never
    /// read, and it is read at once.
    pub fn build_with_index(
        roots: &[MediaRoot],
        policy: SymlinkPolicy,
        filesystem: &dyn FileSystem,
        index: Option<Arc<CatalogueIndex>>,
        warnings: Vec<String>,
    ) -> Self {
        let mut registry = VolumeRegistry::default();
        let restored = index.as_ref().and_then(|index| index.load().ok().flatten());
        let (catalogue, read) = match restored {
            Some(catalogue) => (catalogue, false),
            None => (
                Catalogue::build_with(roots, policy, filesystem, &mut registry).catalogue,
                true,
            ),
        };
        let library = Self {
            catalogue: Arc::new(RwLock::new(catalogue)),
            memory: Arc::new(Mutex::new(Memory {
                registry,
                roots: roots.to_vec(),
                policy,
            })),
            pictures: None,
            index,
            warnings: Arc::new(Mutex::new(warnings)),
        };
        if read {
            library.write_to_the_index();
        }
        library
    }

    /// Lets the library look for the pictures of the catalogue.
    ///
    /// The finder is kept beside the catalogue, so that a catalogue that is
    /// read again keeps looking for the pictures (`F-10` § 5).
    pub fn with_pictures(self, finder: ArtworkFinder) -> Self {
        self.write_catalogue().with_pictures(finder.clone());
        Self {
            pictures: Some(finder),
            ..self
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
    ///
    /// The catalogue of before stays readable while the library is read: it is
    /// replaced only when the new one is ready, and the index is written
    /// before the swap, in one transaction (`FUN-008`).
    pub fn refresh(&self, filesystem: &dyn FileSystem) -> VolumeReport {
        let mut memory = self.write_memory();
        let roots = memory.roots.clone();
        let policy = memory.policy;
        let outcome = Catalogue::build_with(&roots, policy, filesystem, &mut memory.registry);
        let report = outcome.report;
        let mut catalogue = outcome.catalogue;
        self.write_catalogue_to_the_index(&mut catalogue);
        let mut current = self.write_catalogue();
        *current = catalogue;
        // The finder belongs to the library and not to the scan: the pictures
        // are looked for again after every read of the catalogue.
        if let Some(finder) = &self.pictures {
            current.with_pictures(finder.clone());
        }
        report
    }

    /// The roots the library reads.
    pub fn roots(&self) -> Vec<MediaRoot> {
        self.memory().roots.clone()
    }

    /// What the library has to tell the user.
    ///
    /// The warnings are the ones that belong to the library itself and not to
    /// a catalogue: a damaged index, an index that cannot be written. They are
    /// shown in the status of the server (`FUN-010`).
    pub fn warnings(&self) -> Vec<String> {
        self.warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Writes the catalogue of the server into the index.
    fn write_to_the_index(&self) {
        let mut catalogue = self.write_catalogue();
        self.write_catalogue_to_the_index(&mut catalogue);
    }

    /// Writes a catalogue into the index, if there is one.
    ///
    /// The identifier of the catalogue is settled by the index: a catalogue
    /// that did not change keeps the identifier it had, and a catalogue that
    /// changed gets one that is higher than every identifier before it
    /// (`INT-037`).
    fn write_catalogue_to_the_index(&self, catalogue: &mut Catalogue) {
        let Some(index) = &self.index else {
            return;
        };
        if let Err(error) = index.store(catalogue) {
            self.warn(format!(
                "the catalogue cannot be written into the index: {error}"
            ));
        }
    }

    /// Keeps a warning for the status of the server.
    fn warn(&self, warning: String) {
        self.warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(warning);
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
