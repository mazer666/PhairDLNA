//! The configuration sections.
//!
//! What: the typed shape of the configuration document, one struct per section.
//! Why: a typed document is what makes `CFG-002` possible — the validation
//!      rules can then be written against the types instead of against strings.
//! Example: `[server] http_port = 8200` becomes `ServerSection { http_port: 8200 }`.

use serde::{Deserialize, Serialize};

/// How a symbolic link inside a media root is treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymlinkPolicy {
    /// The link is not followed and the file is not indexed. This is the default:
    /// it is the only policy that cannot expose a file outside the root.
    #[default]
    Skip,
    /// The link is resolved and the target is indexed, if the target is inside a root.
    Follow,
    /// The link itself is indexed as its own entry, pointing at the target.
    IndexAsLink,
}

/// The identity and the listener of the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ServerSection {
    /// The name a client sees in its device list.
    pub device_name: String,
    /// The TCP port the media server listens on.
    pub http_port: u16,
    /// Whether SSDP discovery runs at all.
    pub discovery_enabled: bool,
    /// How symbolic links inside a media root are treated.
    pub symlink_policy: SymlinkPolicy,
}

/// Which network interfaces the server speaks on.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkSection {
    /// The interface names to use. Empty means every eligible interface.
    pub interfaces: Vec<String>,
    /// Whether a non-local address may be used. Off by default: a media server
    /// belongs on the local network.
    pub allow_non_local_addresses: bool,
}

/// One folder the user shares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaRoot {
    /// The name a client sees for this root.
    pub name: String,
    /// The absolute path of the folder.
    pub path: String,
    /// Sub-folders of this root that are not indexed.
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// The media library.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibrarySection {
    /// The folders to share.
    pub roots: Vec<MediaRoot>,
}

/// The resource ceilings the server must not exceed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct LimitsSection {
    /// How many transfers may run at the same time.
    pub max_concurrent_streams: usize,
    /// How much disk the disposable cache may use.
    pub max_cache_bytes: u64,
    /// How long a single file may take to inspect.
    pub metadata_timeout_seconds: u64,
    /// How many files may be inspected in parallel.
    pub index_workers: usize,
}

/// Who may talk to the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SecuritySection {
    /// Client addresses that may browse and stream. Empty means every client on
    /// the local network.
    pub allowed_clients: Vec<String>,
    /// Client addresses whose quirks are handled by a device profile.
    pub trusted_clients: Vec<String>,
    /// Whether the loopback administration surface runs.
    pub admin_enabled: bool,
    /// The path of the file holding the administration token.
    pub admin_token_file: String,
}

/// How much the server says about itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct LoggingSection {
    /// The lowest level that is written.
    pub level: LogLevel,
    /// Whether paths and addresses are replaced by pseudonyms in the log.
    pub redact: bool,
}

/// The severity of a log record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Everything, including per-request detail.
    Debug,
    /// The normal operating detail.
    #[default]
    Info,
    /// Only the problems.
    Warn,
    /// Only the failures.
    Error,
}

impl LogLevel {
    /// The level's name as it appears in the configuration.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
