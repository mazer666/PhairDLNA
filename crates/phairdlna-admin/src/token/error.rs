//! Errors returned while generating, reading or validating an admin token.

use std::fmt;
use std::path::PathBuf;

/// Why the token of the surface could not be used.
#[derive(Debug)]
pub enum TokenError {
    /// The file that holds the token could not be read.
    Unreadable {
        /// The path of the file.
        path: PathBuf,
        /// What the file system said.
        reason: String,
    },
    /// The file that holds the token could not be written.
    Unwritable {
        /// The path of the file.
        path: PathBuf,
        /// What the file system said.
        reason: String,
    },
    /// The file that holds the token is readable by others.
    NotPrivate {
        /// The path of the file.
        path: PathBuf,
    },
    /// The text of the file is not a token.
    Malformed,
}

impl fmt::Display for TokenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => {
                write!(
                    formatter,
                    "the token file \"{path:?}\" could not be read: {reason}"
                )
            }
            Self::Unwritable { path, reason } => {
                write!(
                    formatter,
                    "the token file \"{path:?}\" could not be written: {reason}"
                )
            }
            Self::NotPrivate { path } => write!(
                formatter,
                "the token file \"{path:?}\" is readable by others; expected the permissions 600"
            ),
            Self::Malformed => write!(formatter, "the text of the token file is not a token"),
        }
    }
}

impl std::error::Error for TokenError {}
