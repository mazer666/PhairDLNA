//! Token source and permission checks for the local administration client.
//!
//! What: read the installation token from its private file before each request.
//! Why: the app must see a server started after the app and must not expose its token.
//! Example: a missing token reports that the server has not been started.

use std::path::{Path, PathBuf};

use super::ClientError;

/// Where the token of the installation comes from.
#[derive(Clone, Debug)]
pub(super) enum TokenSource {
    /// A token that the caller knows.
    Fixed(String),
    /// The file of the token of the installation.
    ///
    /// The file is read before every request: the server may be started
    /// while the application runs, and the application must find it then.
    File(PathBuf),
}

/// Reads the token of the installation out of its file.
///
/// The file must be readable by its owner alone: a token that another program
/// of the machine can read is no token at all (`SEC-071`).
pub fn read_token(path: &Path) -> Result<String, ClientError> {
    if !path.exists() {
        return Err(ClientError::NoToken);
    }
    check_private(path)?;
    let text = std::fs::read_to_string(path).map_err(|error| ClientError::Token {
        reason: error.to_string(),
    })?;
    let token = text.trim().to_owned();
    if token.is_empty() {
        return Err(ClientError::Token {
            reason: String::new(),
        });
    }
    Ok(token)
}

/// Refuses a token file that another program of the machine may read.
#[cfg(unix)]
fn check_private(path: &Path) -> Result<(), ClientError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|error| ClientError::Token {
        reason: error.to_string(),
    })?;
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(ClientError::Token {
            reason: String::new(),
        });
    }
    Ok(())
}

/// The rule of the private file only exists on the systems that have file
/// permissions; elsewhere the token file is read as it is.
#[cfg(not(unix))]
fn check_private(_path: &Path) -> Result<(), ClientError> {
    Ok(())
}
