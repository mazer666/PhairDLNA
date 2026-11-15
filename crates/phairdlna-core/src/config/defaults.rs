//! The configuration document and its defaults.
//!
//! What: the top-level configuration type, the schema version and the defaults
//!       of every section.
//! Why: `CFG-004` requires that an empty configuration file yields a working,
//!      secure server. That is only true if every default is written down here
//!      and nowhere else.
//! Example: `Config::default()` is the server a user gets without touching a
//!      file.

use serde::{Deserialize, Serialize};

use super::sections::{
    LibrarySection, LimitsSection, LogLevel, LoggingSection, NetworkSection, SecuritySection,
    ServerSection, SymlinkPolicy,
};

/// The schema version this build writes.
///
/// A configuration file carries its own `schema_version`. A file with an older
/// known version is migrated in memory (`CFG-003`); a file with a newer version
/// is refused, because this build cannot know what the newer one means.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The schema version a document without an explicit version is assumed to use.
pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// The default TCP port of the media server.
///
/// 8200 is the conventional port for a DLNA media server, so a client that
/// remembers the port finds it where it expects.
pub const DEFAULT_HTTP_PORT: u16 = 8200;

/// The port the discovery socket listens on by default.
///
/// The specification names the port 1900 for SSDP.
pub const DEFAULT_SSDP_PORT: u16 = 1900;

/// The default name a client sees.
pub const DEFAULT_DEVICE_NAME: &str = "PhairDLNA";

/// The whole configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The version of the schema this document was written against.
    ///
    /// A document that does not say is a document written against the current
    /// version: an empty file must yield a working server (`CFG-004`).
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,

    /// The server identity and listener.
    #[serde(default)]
    pub server: ServerSection,

    /// The network interfaces.
    #[serde(default)]
    pub network: NetworkSection,

    /// The media library.
    #[serde(default)]
    pub library: LibrarySection,

    /// The resource ceilings.
    #[serde(default)]
    pub limits: LimitsSection,

    /// The access rules.
    #[serde(default)]
    pub security: SecuritySection,

    /// The logging behaviour.
    #[serde(default)]
    pub logging: LoggingSection,
}

impl Default for Config {
    /// The default configuration: a working, safe server.
    ///
    /// Every value here is a decision, not an accident. The ones that matter
    /// most: discovery is on, the administration surface is **off**, the
    /// non-local address is refused, symbolic links are **skipped**, and the log
    /// is redacted.
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            server: ServerSection::default(),
            network: NetworkSection::default(),
            library: LibrarySection::default(),
            limits: LimitsSection::default(),
            security: SecuritySection::default(),
            logging: LoggingSection::default(),
        }
    }
}

impl Default for ServerSection {
    /// A server that announces itself and listens on the conventional port.
    fn default() -> Self {
        Self {
            device_name: DEFAULT_DEVICE_NAME.to_owned(),
            http_port: DEFAULT_HTTP_PORT,
            ssdp_port: DEFAULT_SSDP_PORT,
            discovery_enabled: true,
            symlink_policy: SymlinkPolicy::Skip,
        }
    }
}

impl Default for LimitsSection {
    /// The ceilings that keep the server small on an ordinary machine.
    fn default() -> Self {
        Self {
            max_concurrent_streams: 10,
            max_cache_bytes: 512 * 1024 * 1024,
            metadata_timeout_seconds: 5,
            index_workers: 2,
        }
    }
}

impl Default for SecuritySection {
    /// A server that is closed to the administration surface by default.
    fn default() -> Self {
        Self {
            allowed_clients: Vec::new(),
            trusted_clients: Vec::new(),
            admin_enabled: false,
            admin_token_file: default_admin_token_file(),
        }
    }
}

impl Default for LoggingSection {
    /// A log that is informative but does not leak the user's paths.
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            redact: true,
        }
    }
}

/// The default location of the administration token.
///
/// The token lives beside the configuration, not inside it: a token inside a
/// configuration file would be copied around with it (`CFG-014`).
pub fn default_admin_token_file() -> String {
    super::paths::user_config_dir()
        .join("admin.token")
        .to_string_lossy()
        .into_owned()
}
