//! The error type of the portable core.
//!
//! What: one error enum per bounded domain, so that a caller can match on the
//! failure without matching on a string.
//! Why: the project forbids silently swallowed errors, and an unmatchable error
//!      string is the first step towards swallowing one.
//! Example: a caller matches on `CoreError::Config { .. }` to report a bad
//!      configuration file separately from a runtime failure.

use crate::config::ConfigError;

/// Every failure the portable core can report.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The configuration could not be read, parsed or validated.
    #[error("configuration error")]
    Config(#[from] ConfigError),

    /// A platform capability was used that this platform does not provide.
    #[error("capability not supported on this platform: {capability}")]
    Unsupported {
        /// What was asked for, for example "enumerating network interfaces".
        capability: &'static str,
    },

    /// An operating-system call failed.
    #[error("{operation} failed: {source}")]
    Io {
        /// What the core was trying to do.
        operation: &'static str,
        /// The underlying failure.
        #[source]
        source: std::io::Error,
    },
}

impl CoreError {
    /// Wraps an I/O failure with the operation that produced it.
    pub fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io { operation, source }
    }
}
