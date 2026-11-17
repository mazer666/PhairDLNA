//! The connections to the database of the index.
//!
//! What: the connection that writes, the connection that reads, and the
//!       settings both of them carry.
//! Why: a reader must see the state that was committed last and never a
//!       half-written one. The two connections are therefore apart, and the
//!       journal of the database is the one that lets a reader read while a
//!       writer writes (`FUN-008`, ADR-0006).
//! Example: `open` gives the connection that writes; `open_reader` gives the
//!       one that reads and may not write at all.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::IndexError;

/// Opens the connection that writes the index.
///
/// The journal of the database is the write-ahead log, so that a reader is
/// never blocked by a writer and sees the state of the last commit.
pub(super) fn open(path: &Path) -> Result<Connection, IndexError> {
    let connection =
        Connection::open_with_flags(path, write_flags()).map_err(|error| unusable(path, error))?;
    set_journal(&connection).map_err(|error| unusable(path, error))?;
    Ok(connection)
}

/// Opens the connection that reads the index.
///
/// The connection may not write: a mistake in the read path must not be able to
/// change the index.
pub(super) fn open_reader(path: &Path) -> Result<Connection, IndexError> {
    let connection =
        Connection::open_with_flags(path, read_flags()).map_err(|error| unusable(path, error))?;
    connection
        .execute_batch("PRAGMA query_only = ON;")
        .map_err(|error| unusable(path, error))?;
    Ok(connection)
}

/// The flags of the connection that writes.
fn write_flags() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
}

/// The flags of the connection that reads.
///
/// The connection may write to the file as far as SQLite is concerned — it has
/// to be able to read the write-ahead log — but it is told to refuse every
/// change.
fn read_flags() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX
}

/// The journal of the database: the write-ahead log.
///
/// The log lets a reader read the state of the last commit while the writer is
/// at work, which is what keeps the catalogue usable while a scan runs
/// (`FUN-008`).
fn set_journal(connection: &Connection) -> Result<(), rusqlite::Error> {
    let mode: String = connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        return Err(rusqlite::Error::InvalidParameterName(mode));
    }
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

/// The error of a file that cannot be opened.
fn unusable(path: &Path, error: rusqlite::Error) -> IndexError {
    IndexError::Unusable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
