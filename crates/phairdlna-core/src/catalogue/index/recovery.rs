//! The recovery from a damaged index.
//!
//! What: the inspection of the file of the index, the moving of a damaged file
//!       aside and the report of what was done.
//! Why: the index is a cache, and a damaged cache must never stop the server.
//!       The damaged file is kept — the user may want to look at it — and a new
//!       one takes its place, and the library is read again (`FUN-010`).
//! Example: `damage` says what is wrong with a file, and `move_aside` puts the
//!       file next to the new one under the name `.corrupt.<moment>`.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::IndexError;

/// What the server did with a damaged file of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recovery {
    /// The file the damaged database was moved to.
    pub moved_to: PathBuf,
    /// What was wrong with the database.
    pub reason: String,
}

/// What is wrong with the file, when something is.
pub(super) fn damage(connection: &Connection) -> Option<String> {
    match integrity_of(connection) {
        Ok(value) if value == "ok" => {}
        Ok(value) => return Some(format!("the integrity check says \"{value}\"")),
        Err(error) => return Some(format!("the integrity check could not be run: {error}")),
    }
    match super::schema::is_empty(connection) {
        Ok(true) => return None,
        Ok(false) => {}
        Err(error) => return Some(format!("the schema of the file cannot be read: {error}")),
    }
    match super::schema::matches(connection) {
        Ok(true) => None,
        Ok(false) => Some("the schema of the file is not the one of this build".to_owned()),
        Err(error) => Some(format!("the schema of the file cannot be read: {error}")),
    }
}

/// The answer of the integrity check of SQLite.
fn integrity_of(connection: &Connection) -> Result<String, rusqlite::Error> {
    connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))
}

/// Moves a damaged file aside and creates a new one in its place.
///
/// The write-ahead log belongs to the damaged file and goes with it: a log that
/// stays behind would be read into the new database.
pub(super) fn move_aside(path: &Path, reason: &str) -> Result<Recovery, IndexError> {
    let target = aside_path(path);
    std::fs::rename(path, &target).map_err(|error| IndexError::Unusable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    for suffix in ["-wal", "-shm"] {
        let _ = std::fs::remove_file(sidecar(path, suffix));
    }
    super::permissions::prepare(path)?;
    Ok(Recovery {
        moved_to: target,
        reason: reason.to_owned(),
    })
}

/// The name the damaged file is given.
///
/// The moment is part of the name, so that two damaged files do not fight for
/// the same name and so that the user can tell when the damage happened.
fn aside_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".corrupt.{}", moment()));
    path.with_file_name(name)
}

/// The file that belongs to the database beside it.
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// The moment of the damage, written so that it may be part of a file name.
fn moment() -> String {
    crate::time::now_utc().replace(':', "-")
}
