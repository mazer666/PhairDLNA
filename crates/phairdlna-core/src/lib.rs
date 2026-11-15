//! The portable core of PhairDLNA.
//!
//! What: the domain logic that does not depend on an operating system — the
//!      configuration, the protocol machinery, the catalogue and the streaming
//!      path.
//! Why: the core must compile unchanged on macOS, Linux and Windows, and that
//!      claim is only true if the platform-specific parts are separable. The
//!      crate boundary is checked by `scripts/check-crate-boundaries.sh`.
//! Example:
//!
//! ```no_run
//! use phairdlna_core::config::Config;
//! use std::path::Path;
//!
//! let config = Config::load(Path::new("config.toml"))?;
//! # Ok::<(), phairdlna_core::config::ConfigError>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod identity;
pub mod platform;
pub mod ssdp;

pub use config::{Config, ConfigError};
pub use error::CoreError;

/// The name of the project, used in logs and in the device description.
pub const PROJECT_NAME: &str = "PhairDLNA";

/// The version of the project, taken from the crate metadata.
pub const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::{PROJECT_NAME, PROJECT_VERSION};

    #[test]
    fn project_name_is_stable() {
        assert_eq!(PROJECT_NAME, "PhairDLNA");
    }

    #[test]
    fn project_version_is_not_empty() {
        assert!(!PROJECT_VERSION.is_empty());
    }
}
