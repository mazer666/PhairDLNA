//! The permissions of the file of the index.
//!
//! What: the creation of the directory and of the file of the index, and the
//!       permissions they carry.
//! Why: the index holds the paths and the names of the files of the user. It
//!       belongs to the user alone: no other account of the machine may read
//!       it, and no other program of the user may write to it (SEC-095,
//!       ADR-0006).
//! Example: `prepare` makes sure that the file exists and that only its owner
//!       may read it.

use std::io;
use std::path::Path;

use super::IndexError;

/// The permissions of the file: the owner reads and writes, nobody else does.
#[cfg(unix)]
const FILE_MODE: u32 = 0o600;

/// The permissions of the directory: the owner alone may enter it.
#[cfg(unix)]
const DIRECTORY_MODE: u32 = 0o700;

/// Makes sure that the file of the index exists and that it is private.
///
/// The file is created empty with the permissions 600; SQLite turns an empty
/// file into a database when it opens it. The directory of the file is created
/// as well, with the permissions 700, so that the file never comes to lie in a
/// folder that another account may enter.
pub(super) fn prepare(path: &Path) -> Result<(), IndexError> {
    if let Some(directory) = path.parent() {
        create_directory(directory)?;
    }
    if !path.exists() {
        create_file(path)?;
    }
    Ok(())
}

/// Creates the file of the index with the permissions of its owner.
fn create_file(path: &Path) -> Result<(), IndexError> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE);
    }
    options
        .open(path)
        .map(|_| ())
        .map_err(|error| unusable(path, error))
}

/// Creates the directory of the index with the permissions of its owner.
fn create_directory(path: &Path) -> Result<(), IndexError> {
    if path.as_os_str().is_empty() || path.is_dir() {
        return Ok(());
    }
    let mut options = std::fs::DirBuilder::new();
    options.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        options.mode(DIRECTORY_MODE);
    }
    options.create(path).map_err(|error| unusable(path, error))
}

/// The error of a file that cannot be created.
fn unusable(path: &Path, error: io::Error) -> IndexError {
    IndexError::Unusable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
