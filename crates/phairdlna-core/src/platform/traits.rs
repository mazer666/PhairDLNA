//! The traits that isolate the core from the operating system.
//!
//! What: the eight capabilities the core needs from a platform, each with a
//!       default implementation that reports "not supported".
//! Why: a core that calls the operating system directly stops being portable
//!       (ADR-0003). A core that asks a trait stays portable, and a platform
//!       that is not implemented yet degrades explicitly instead of failing in
//!       a surprising way.
//! Example: `UnsupportedNetwork` answers every call with
//! `PlatformError::Unsupported`; a real implementation answers with the actual
//! interfaces.

use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use super::path::{
    is_readable_folder, CaseSensitivity, FileId, MediaPath, VolumeFingerprint, VolumeId,
};

/// The result of a platform operation.
pub type PlatformResult<T> = Result<T, PlatformError>;

/// A platform operation that could not be carried out.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// The platform does not have this capability.
    #[error("this platform does not support {capability}")]
    Unsupported {
        /// What was asked for.
        capability: &'static str,
    },

    /// The operating system refused or failed.
    #[error("{operation} failed: {source}")]
    Io {
        /// What the core was trying to do.
        operation: &'static str,
        /// The underlying failure.
        #[source]
        source: std::io::Error,
    },
}

impl PlatformError {
    /// Builds an "unsupported" error.
    pub fn unsupported(capability: &'static str) -> Self {
        Self::Unsupported { capability }
    }

    /// Wraps an I/O failure with the operation that produced it.
    pub fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io { operation, source }
    }
}

/// A network interface the server can speak on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterface {
    /// The operating system's name for the interface.
    pub name: String,
    /// The addresses of the interface.
    pub addresses: Vec<std::net::IpAddr>,
    /// Whether the interface can carry multicast.
    pub supports_multicast: bool,
    /// Whether the address is on the local network.
    pub is_local: bool,
}

/// Whether the operating system permits local-network traffic.
///
/// The answer matters on macOS 15 and later, where the user can refuse it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocalNetworkStatus {
    /// The platform says nothing; the server must assume the best.
    #[default]
    Unknown,
    /// The operating system permits local-network traffic.
    Permitted,
    /// The user has refused local-network traffic.
    Refused,
}

/// The network capabilities of a platform.
pub trait Network {
    /// The interfaces the server may use.
    fn interfaces(&self) -> PlatformResult<Vec<NetworkInterface>>;

    /// Joins a multicast group on one interface.
    fn join_multicast(
        &self,
        interface: &NetworkInterface,
        group: Ipv4Addr,
        port: u16,
    ) -> PlatformResult<()>;

    /// Whether the operating system permits local-network traffic.
    fn local_network_status(&self) -> LocalNetworkStatus {
        LocalNetworkStatus::Unknown
    }
}

/// The file system capabilities of a platform.
pub trait FileSystem {
    /// Builds the path model for a path.
    fn media_path(&self, path: &Path) -> PlatformResult<MediaPath>;

    /// The volume a path lives on.
    fn volume_of(&self, path: &Path) -> PlatformResult<VolumeId>;

    /// Whether the volume distinguishes names that differ only in case.
    fn case_sensitivity(&self, volume: &VolumeId) -> CaseSensitivity;

    /// The stable identity of a file.
    fn file_identity(&self, path: &Path) -> PlatformResult<FileId>;

    /// Whether the path is a folder that the catalogue can read.
    ///
    /// The question is asked about a root of the library: a path that cannot be
    /// resolved, or that is not a folder, is a root that is not there
    /// (`FUN-007`).
    ///
    /// The default asks the operating system through the portable adapter. A
    /// platform that knows something more about its own folders — a mount
    /// point, a network share, a volume that is being mounted right now —
    /// answers in its own terms (`PLT-003`).
    fn is_readable_folder(&self, path: &Path) -> bool {
        is_readable_folder(path)
    }

    /// Whether the platform considers the path hidden.
    ///
    /// The default is the convention every platform understands: a name that
    /// begins with a dot. A platform that has a real "hidden" flag — the
    /// `UF_HIDDEN` flag of macOS — overrides it, because only the platform knows
    /// what its own flag means (`PLT-003`, `FUN-004`).
    ///
    /// The question is asked about the **name**, not about the path: a file is
    /// hidden because of its name, not because of the folder it sits in.
    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
    }

    /// What the platform observes at a path right now.
    ///
    /// The answer tells the catalogue whether the drive behind a root is still
    /// the drive it saw last time, or whether it was unplugged, was plugged in
    /// again or was replaced by another one (`FUN-007`, `FUN-017`, `PLT-011`).
    ///
    /// The default says "not supported": a platform that cannot see the volume
    /// of a path must not pretend that it can (`PLT-003`). The catalogue then
    /// knows the path only as far as it can read it.
    fn volume_fingerprint(&self, _path: &Path) -> PlatformResult<VolumeFingerprint> {
        Err(PlatformError::unsupported("the volume of a path"))
    }
}

/// What happened to a volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeEvent {
    /// A volume appeared.
    Mounted(VolumeId),
    /// A volume went away.
    Unmounted(VolumeId),
    /// A volume came back, possibly under a different identity.
    Remounted(VolumeId),
    /// The watcher lost track and a rescan is needed.
    RescanRequired,
}

/// The volume notifications of a platform.
pub trait VolumeWatcher {
    /// The events seen since the last call.
    ///
    /// A polling interface keeps the core free of any particular event loop and
    /// lets a platform deliver the events however it can.
    fn poll_events(&self) -> PlatformResult<Vec<VolumeEvent>>;
}

/// The outcome of a request for access to a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentOutcome {
    /// The user allowed access.
    Granted,
    /// The user refused access.
    Refused,
    /// The platform does not ask; access is allowed.
    NotRequired,
}

/// The consent capabilities of a platform.
pub trait Consent {
    /// Asks for access to a folder the operating system protects.
    fn request_folder_access(&self, path: &Path) -> PlatformResult<ConsentOutcome>;
}

/// Where the per-user directories are.
pub trait UserEnvironment {
    /// The directory holding the configuration.
    fn config_dir(&self) -> PathBuf;

    /// The directory holding the catalogue and the cache.
    fn data_dir(&self) -> PathBuf;
}

/// The description of a background service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    /// The name of the service.
    pub name: String,
    /// The command that starts it.
    pub program: PathBuf,
    /// The arguments it is started with.
    pub arguments: Vec<String>,
    /// Whether the service starts when the user logs in.
    pub start_at_login: bool,
    /// The bundle identifier, on platforms that have one.
    pub bundle_identifier: Option<String>,
    /// Whether the service reports its readiness to the operating system.
    pub reports_readiness: bool,
}

/// The service capabilities of a platform.
pub trait ServiceManager {
    /// The description of the service this build installs.
    fn spec(&self) -> ServiceSpec;

    /// Installs the service.
    fn install(&self, spec: &ServiceSpec) -> PlatformResult<()>;
}

/// A log record, in the shape the core produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    /// The severity.
    pub level: LogLevel,
    /// The component that produced it.
    pub component: &'static str,
    /// The message.
    pub message: String,
}

/// The severity of a log record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Detail that is only useful while debugging.
    Debug,
    /// The normal operating detail.
    Info,
    /// Something that needs attention.
    Warn,
    /// Something that failed.
    Error,
}

/// Where log records go.
pub trait LogSink {
    /// Writes one record.
    fn write(&self, record: &LogRecord);
}

/// A message for the person using the computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// The headline.
    pub title: String,
    /// The body.
    pub body: String,
}

/// The notification capabilities of a platform.
pub trait UserNotifier {
    /// Shows a notification.
    ///
    /// A headless server has no user interface, so the default implementation
    /// does nothing and says so.
    fn notify(&self, notification: &Notification) -> PlatformResult<()>;
}
