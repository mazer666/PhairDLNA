//! The index of the catalogue on disk, in the daemon.
//!
//! What: the opening of the index of the library, the catalogue that is read
//!       out of it and the writing of a catalogue into it.
//! Why: the index is a cache of the catalogue. The daemon takes the catalogue
//!       of the index at its start and reads the library again in the
//!       background, so that a client never meets an empty server because the
//!       index is empty (`FUN-008`, `FUN-009`).
//! Example: `Index::open` opens the index of the library and says what was
//!       wrong with it.

use std::sync::{Arc, Mutex};

use phairdlna_core::catalogue::index::CatalogueIndex;
use phairdlna_core::catalogue::Catalogue;
use phairdlna_core::config::Config;

use super::log::Logger;

/// The index of the catalogue of the daemon.
#[derive(Clone)]
pub struct Index {
    /// The index itself, when it could be opened.
    inner: Option<Arc<CatalogueIndex>>,
    /// What the user has to know about it.
    warnings: Arc<Mutex<Vec<String>>>,
}

impl Index {
    /// An index that does not exist, for a library that has none.
    pub fn empty() -> Self {
        Self {
            inner: None,
            warnings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Opens the index of the library and says what was wrong with it.
    ///
    /// An index that cannot be opened at all is not a reason to refuse the
    /// start: the daemon reads the library as it always did, and the index is
    /// written again at the next read (`FUN-009`, `FUN-010`).
    pub fn open(config: &Config, logger: &Logger) -> Self {
        let path = config.library.database_file();
        let mut warnings = Vec::new();
        let inner = match CatalogueIndex::open(&path) {
            Ok(opened) => {
                if let Some(repair) = opened.recovery {
                    logger.warn(
                        "the index of the catalogue was damaged and was moved aside",
                        &[
                            ("reason", &repair.reason),
                            ("file", &repair.moved_to.display().to_string()),
                        ],
                    );
                    warnings.push(format!(
                        "the index of the catalogue was damaged and was rebuilt: {} (the damaged \
                         file was kept as {})",
                        repair.reason,
                        repair.moved_to.display()
                    ));
                }
                Some(Arc::new(opened.index))
            }
            Err(error) => {
                logger.warn(
                    "the index of the catalogue cannot be used",
                    &[("reason", &error.to_string())],
                );
                warnings.push(format!(
                    "the index of the catalogue cannot be used: {error}"
                ));
                None
            }
        };
        Self {
            inner,
            warnings: Arc::new(Mutex::new(warnings)),
        }
    }

    /// The catalogue of the last run, when the index holds one.
    pub fn restored(&self) -> Option<Catalogue> {
        self.inner
            .as_ref()
            .and_then(|index| index.load().ok().flatten())
    }

    /// Writes a catalogue into the index.
    ///
    /// The identifier of the catalogue is settled by the index: a catalogue that
    /// did not change keeps the identifier it had, and a catalogue that changed
    /// gets one that is higher than every identifier before it (`INT-037`).
    pub fn store(&self, catalogue: &mut Catalogue) {
        let Some(index) = &self.inner else {
            return;
        };
        if let Err(error) = index.store(catalogue) {
            self.warn(format!(
                "the catalogue cannot be written into the index: {error}"
            ));
        }
    }

    /// What the user has to know about the index.
    pub fn warnings(&self) -> Vec<String> {
        self.warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Keeps a warning for the status of the server.
    fn warn(&self, warning: String) {
        self.warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(warning);
    }
}
