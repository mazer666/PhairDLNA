//! Where the configuration and the token live on disk.
//!
//! What: the one place that knows how the user's configuration directory is
//!      found.
//! Why: two modules that each guess the home directory will eventually guess
//!      it differently, and the user will find two configuration files.
//! Example: `user_config_dir()` returns `~/.config/phairdlna` on a Unix system.

use std::path::PathBuf;

/// The directory name the project uses inside the user's configuration
/// directory.
pub const PROJECT_DIRECTORY: &str = "phairdlna";

/// The user's home directory.
///
/// Falls back to the temporary directory when the environment does not say,
/// which is the only sensible answer in a container without a home.
pub fn home_directory() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// The project's own directory inside the user's configuration directory.
pub fn user_config_dir() -> PathBuf {
    home_directory().join(".config").join(PROJECT_DIRECTORY)
}
