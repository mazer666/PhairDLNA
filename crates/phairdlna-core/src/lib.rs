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

pub mod catalogue;
pub mod config;
pub mod didl;
pub mod error;
pub mod http;
pub mod identity;
pub mod lifecycle;
pub mod media;
pub mod platform;
pub mod scheduler;
pub mod security;
pub mod ssdp;
pub mod time;
pub mod upnp;
pub mod xml;

pub use config::{Config, ConfigError};
pub use error::CoreError;

/// The name of the project, used in logs and in the device description.
pub const PROJECT_NAME: &str = "PhairDLNA";

/// The version of the project, taken from the crate metadata.
pub const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The source revision the binary was built from.
///
/// The revision is read when the binary is built, so that the running server
/// and the source it offers cannot disagree (`OPS-017`, `ADR-0010`). A build
/// without a repository behind it reports `unknown`.
pub const SOURCE_REVISION: &str = env!("PHAIRDLNA_SOURCE_REVISION");

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
