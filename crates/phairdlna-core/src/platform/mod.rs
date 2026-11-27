//! The boundary between the core and the operating system.
//!
//! What: the nine capability traits, the path model, and the default
//!       implementation that refuses politely.
//! Why: the core must compile unchanged on macOS, Linux and Windows. The only
//!      way to keep that true is to put every operating-system question behind
//!      a trait (ADR-0003).
//! Example:
//!
//! ```no_run
//! use phairdlna_core::platform::{Network, UnsupportedNetwork};
//!
//! let network = UnsupportedNetwork;
//! match network.interfaces() {
//!     Ok(interfaces) => println!("{} interfaces", interfaces.len()),
//!     Err(error) => println!("not on this platform: {error}"),
//! }
//! ```

mod change;
mod fs_watcher;
mod local;
mod memory;
pub mod network;
mod path;
mod traits;
mod unsupported;

pub use change::{ChangeReliability, ChangeWatcher, FileChange};
pub use fs_watcher::FsChangeWatcher;
pub use local::{LocalFileSystem, LOCAL_VOLUME};
pub use memory::{LocalProcessMemory, ProcessMemory};
pub use network::{select_interfaces, InterfaceDecision, InterfaceRole};
pub use path::{
    display_name, identity_of, identity_of_with, is_readable_folder, CaseSensitivity, FileId,
    MediaPath, VolumeFingerprint, VolumeId,
};
pub use traits::{
    Consent, ConsentOutcome, FileSystem, LocalNetworkStatus, LogLevel, LogRecord, LogSink, Network,
    NetworkInterface, Notification, PlatformError, PlatformResult, ServiceManager, ServiceSpec,
    UserEnvironment, UserNotifier, VolumeEvent, VolumeWatcher,
};
pub use unsupported::{
    UnsupportedChangeWatcher, UnsupportedConsent, UnsupportedFileSystem, UnsupportedLogSink,
    UnsupportedNetwork, UnsupportedServiceManager, UnsupportedUserEnvironment,
    UnsupportedUserNotifier, UnsupportedVolumeWatcher,
};
