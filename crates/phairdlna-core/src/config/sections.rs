//! The configuration sections.
//!
//! What: the typed shape of the configuration document, one struct per section.
//! Why: a typed document is what makes `CFG-002` possible — the validation
//!      rules can then be written against the types instead of against strings.
//! Example: `[server] http_port = 8200` becomes `ServerSection { http_port: 8200 }`.

use std::path::PathBuf;

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
    /// The UDP port the discovery socket listens on.
    ///
    /// The specification names the port 1900, and that is the default. A
    /// machine that already runs a server on that port can move the socket of
    /// this one (`INT-004`).
    pub ssdp_port: u16,
    /// Whether SSDP discovery runs at all.
    pub discovery_enabled: bool,
    /// How symbolic links inside a media root are treated.
    pub symlink_policy: SymlinkPolicy,
}

/// Which network interfaces the server speaks on.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
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
    /// How symbolic links inside this root are treated.
    ///
    /// `None` means the setting of the server applies. A root may therefore
    /// follow links that another root does not follow (`CFG-007`).
    #[serde(default)]
    pub symlink_policy: Option<SymlinkPolicy>,
}

/// The media library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct LibrarySection {
    /// The folders to share.
    pub roots: Vec<MediaRoot>,
    /// Whether the pictures of the library are looked for and served.
    ///
    /// The setting is on by default: a control point asks for the cover of an
    /// album before it plays anything. A library with many files may turn it
    /// off, and then the scan reads no file to look for a picture (`PERF-004`).
    #[serde(default = "yes_by_default")]
    pub extract_artwork: bool,
    /// Where the disposable cache of the produced pictures is kept.
    ///
    /// `None` means the data directory of the user. The cache may be deleted
    /// at any time; it is rebuilt when it is needed again (`FUN-018`).
    #[serde(default)]
    pub cache_directory: Option<String>,
    /// Where the catalogue index on disk is kept.
    ///
    /// `None` means the data directory of the user. The index is a cache of
    /// the catalogue: it may be deleted at any time, and the library is then
    /// read again (`FUN-009`).
    #[serde(default)]
    pub database_path: Option<String>,
    /// The scan schedule for periodic automatic indexing (FUN-024).
    #[serde(default)]
    pub scan_schedule: super::schedule::ScanSchedule,
}

impl LibrarySection {
    /// The file of the catalogue index.
    ///
    /// The setting wins when it is there; the data directory of the user holds
    /// the file when it is not. The file belongs to the user alone, and the
    /// server never writes to a media root (`FUN-009`, `SEC-095`).
    pub fn database_file(&self) -> PathBuf {
        match self.database_path.as_deref() {
            Some(path) => PathBuf::from(path),
            None => super::paths::default_database_path(),
        }
    }
}

impl Default for LibrarySection {
    /// A library without roots and with the pictures switched on.
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            extract_artwork: true,
            cache_directory: None,
            database_path: None,
            scan_schedule: super::schedule::ScanSchedule::default(),
        }
    }
}

/// The answer of a setting that is on unless the user turns it off.
fn yes_by_default() -> bool {
    true
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
    /// The largest request line the server reads, in bytes (`SEC-011`).
    pub max_request_line_bytes: usize,
    /// The largest header block the server reads, in bytes (`SEC-011`).
    pub max_header_block_bytes: usize,
    /// The largest number of headers the server reads (`SEC-011`).
    pub max_header_count: usize,
    /// The largest body the server reads, in bytes (`SEC-011`).
    pub max_body_bytes: usize,
    /// How many connections may be open at the same time (`SEC-012`).
    pub max_concurrent_connections: usize,
    /// The largest DIDL-Lite document the server writes, in bytes (`SEC-061`).
    pub max_didl_bytes: usize,
    /// How long a connection may stay idle before it is closed (`SEC-016`).
    pub read_timeout_seconds: u64,
    /// How long the server may take to write an answer (`SEC-016`).
    pub write_timeout_seconds: u64,
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
    /// The TCP port the administration surface listens on.
    ///
    /// The port is not the port of the media server: the two surfaces are two
    /// origins, and the browser of the user keeps them apart (`OPS-020`).
    pub admin_port: u16,
    /// The addresses the administration surface binds to.
    ///
    /// An empty list means the loopback addresses of the machine, `127.0.0.1`
    /// and `::1`. A name is never accepted here: a name is what an attacker
    /// can point somewhere else (`OPS-020`, `SEC-071`).
    pub admin_bind_addresses: Vec<String>,
    /// Whether the administration surface may bind to a non-loopback address.
    ///
    /// The surface is meant for the machine the server runs on. A user who
    /// really wants it on the network has to say so, and the server refuses to
    /// start without that word (`CFG-005`, `OPS-020`).
    pub admin_allow_non_loopback: bool,
    /// The host names the server answers to (`SEC-071`).
    ///
    /// A name is accepted only when it is written down here: a name is what
    /// an attacker can point somewhere else, an address is not. The list is
    /// empty by default, and a server without a name answers to addresses only.
    pub allowed_host_names: Vec<String>,
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
