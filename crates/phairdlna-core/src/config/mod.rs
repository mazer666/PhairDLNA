//! The configuration module.
//!
//! What: the typed, validated, versioned configuration of PhairDLNA.
//! Why: one place reads the configuration, one place validates it, and one
//!      place decides what a missing setting means. Two modules that read the
//!      same file disagree with each other sooner or later.
//! Example:
//!
//! ```no_run
//! use phairdlna_core::config::Config;
//! use std::path::Path;
//!
//! let config = Config::load(Path::new("/etc/phairdlna/config.toml"))?;
//! println!("listening on port {}", config.server.http_port);
//! # Ok::<(), phairdlna_core::config::ConfigError>(())
//! ```

mod defaults;
mod error;
mod load;
mod paths;
pub mod schema;
mod sections;
mod validate;

pub use defaults::{
    current_schema_version, default_admin_token_file, Config, CURRENT_SCHEMA_VERSION,
    DEFAULT_ADMIN_PORT, DEFAULT_DEVICE_NAME, DEFAULT_HTTP_PORT, DEFAULT_SSDP_PORT,
};
pub use error::ConfigError;
pub use load::{resolve_path, CONFIG_PATH_ENV, DEFAULT_CONFIG_FILE_NAME};
pub use paths::{
    default_database_path, home_directory, user_config_dir, user_data_dir, PROJECT_DIRECTORY,
};
pub use sections::{
    LibrarySection, LimitsSection, LogLevel, LoggingSection, MediaRoot, NetworkSection,
    SecuritySection, ServerSection, SymlinkPolicy,
};
pub use validate::validate;
