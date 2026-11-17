//! The catalogue index on disk.
//!
//! What: the SQLite database that holds the catalogue of the library, the
//!       writing of a catalogue into it, the reading of it back and the
//!       recovery from a damaged file.
//! Why: the catalogue in memory is gone when the daemon stops, and a daemon
//!       that reads the whole library again at every start is slow. The index
//!       is a *cache*: it may be deleted at any time, and the library is then
//!       read again (`FUN-009`, ADR-0006).
//! Example: `CatalogueIndex::open` opens the index of a library and `store`
//!       writes the catalogue of a scan into it.
//!
//! **Two connections, one state.**
//!
//! The index has a connection that writes and a connection that reads. A read
//! during a write sees the state that was committed last and never a half
//! written one, so the catalogue stays usable while a scan runs (`FUN-008`).

mod connection;
mod load;
mod permissions;
mod recovery;
mod rows;
mod schema;
mod store;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use super::Catalogue;

pub use recovery::Recovery;

/// What can go wrong with the index.
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    /// The file of the index cannot be opened, read or written.
    #[error("the catalogue index at {path} cannot be used: {reason}")]
    Unusable {
        /// The file of the index.
        path: PathBuf,
        /// What went wrong.
        reason: String,
    },
}

/// The index of a library, on disk.
///
/// The index is opened once and used for as long as the daemon runs. It is not
/// thread-safe by itself: the two connections are guarded, and the guard is
/// taken for the length of one statement or one transaction.
pub struct CatalogueIndex {
    /// The file of the index.
    path: PathBuf,
    /// The connection that writes.
    writer: Mutex<Connection>,
    /// The connection that reads.
    reader: Mutex<Connection>,
}

/// The result of opening the index.
pub struct OpenedIndex {
    /// The index that was opened.
    pub index: CatalogueIndex,
    /// The damage that was found and repaired, when there was damage.
    pub recovery: Option<Recovery>,
}

impl CatalogueIndex {
    /// Opens the index of the library at the given path.
    ///
    /// A file that does not exist is created; a file that is damaged is moved
    /// aside and a new one takes its place, and the report of that is returned
    /// (`FUN-010`).
    pub fn open(path: &Path) -> Result<OpenedIndex, IndexError> {
        permissions::prepare(path)?;
        let (writer, recovery) = open_sound(path)?;
        let reader = connection::open_reader(path)?;
        Ok(OpenedIndex {
            index: CatalogueIndex {
                path: path.to_path_buf(),
                writer: Mutex::new(writer),
                reader: Mutex::new(reader),
            },
            recovery,
        })
    }

    /// The file of the index.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes the catalogue into the index.
    ///
    /// The whole catalogue is written in one transaction, so that a reader
    /// sees either the catalogue of before or the catalogue of now, and never
    /// a mixture of the two (`FUN-008`).
    ///
    /// The identifier of the catalogue is settled here: a catalogue that did
    /// not change since the last run keeps the identifier it had, and a
    /// catalogue that changed gets one that is higher than every identifier
    /// that was handed out before (`INT-037`).
    pub fn store(&self, catalogue: &mut Catalogue) -> Result<(), IndexError> {
        let mut writer = self.write_connection();
        store::store(&mut writer, catalogue, &self.path)
    }

    /// Reads the catalogue back.
    ///
    /// The answer is `None` when the index is empty: the library has then not
    /// been read yet, and the server reads it.
    pub fn load(&self) -> Result<Option<Catalogue>, IndexError> {
        let mut reader = self.read_connection();
        load::load(&mut reader, &self.path)
    }

    /// The connection that writes.
    fn write_connection(&self) -> MutexGuard<'_, Connection> {
        self.writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The connection that reads.
    fn read_connection(&self) -> MutexGuard<'_, Connection> {
        self.reader
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Opens the connection that writes, and repairs a damaged file.
///
/// A file that cannot be opened, that fails the integrity check or that
/// carries another schema is moved aside, and a new file takes its place.
fn open_sound(path: &Path) -> Result<(Connection, Option<Recovery>), IndexError> {
    match try_open(path) {
        Ok(writer) => Ok((writer, None)),
        Err(reason) => {
            let recovery = recovery::move_aside(path, &reason)?;
            let writer = try_open(path).map_err(|error| IndexError::Unusable {
                path: path.to_path_buf(),
                reason: error,
            })?;
            Ok((writer, Some(recovery)))
        }
    }
}

/// Opens the connection that writes and checks that the file is sound.
fn try_open(path: &Path) -> Result<Connection, String> {
    let writer = connection::open(path).map_err(|error| error.to_string())?;
    if let Some(reason) = recovery::damage(&writer) {
        return Err(reason);
    }
    schema::create(&writer).map_err(|error| error.to_string())?;
    Ok(writer)
}
