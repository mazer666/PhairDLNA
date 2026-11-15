//! The default platform: every capability reports "not supported".
//!
//! What: one implementation of each trait that refuses politely.
//! Why: a new platform must compile before it is implemented, and a missing
//!       capability must degrade **explicitly** rather than panic or lie
//!       (`PLT-003`).
//! Example: `UnsupportedNetwork.interfaces()` returns
//! `Err(PlatformError::Unsupported { .. })`, so the caller can say "this platform
//! cannot list interfaces" instead of crashing.

use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use super::path::{CaseSensitivity, FileId, MediaPath, VolumeId};
use super::traits::{
    Consent, ConsentOutcome, FileSystem, LocalNetworkStatus, LogRecord, LogSink, Network,
    NetworkInterface, Notification, PlatformError, PlatformResult, ServiceManager, ServiceSpec,
    UserEnvironment, UserNotifier, VolumeEvent, VolumeWatcher,
};

/// A network that cannot be asked anything.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedNetwork;

impl Network for UnsupportedNetwork {
    fn interfaces(&self) -> PlatformResult<Vec<NetworkInterface>> {
        Err(PlatformError::unsupported("listing network interfaces"))
    }

    fn join_multicast(
        &self,
        _interface: &NetworkInterface,
        _group: Ipv4Addr,
        _port: u16,
    ) -> PlatformResult<()> {
        Err(PlatformError::unsupported("joining a multicast group"))
    }

    fn local_network_status(&self) -> LocalNetworkStatus {
        LocalNetworkStatus::Unknown
    }
}

/// A file system that cannot be asked anything.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedFileSystem;

impl FileSystem for UnsupportedFileSystem {
    fn media_path(&self, _path: &Path) -> PlatformResult<MediaPath> {
        Err(PlatformError::unsupported("resolving a media path"))
    }

    fn volume_of(&self, _path: &Path) -> PlatformResult<VolumeId> {
        Err(PlatformError::unsupported("identifying a volume"))
    }

    fn case_sensitivity(&self, _volume: &VolumeId) -> CaseSensitivity {
        CaseSensitivity::default()
    }

    fn file_identity(&self, _path: &Path) -> PlatformResult<FileId> {
        Err(PlatformError::unsupported("identifying a file"))
    }
}

/// A volume watcher that never sees anything.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedVolumeWatcher;

impl VolumeWatcher for UnsupportedVolumeWatcher {
    fn poll_events(&self) -> PlatformResult<Vec<VolumeEvent>> {
        Err(PlatformError::unsupported("watching volumes"))
    }
}

/// A consent mechanism that is never asked.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedConsent;

impl Consent for UnsupportedConsent {
    fn request_folder_access(&self, _path: &Path) -> PlatformResult<ConsentOutcome> {
        Ok(ConsentOutcome::NotRequired)
    }
}

/// A user environment with no directories.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedUserEnvironment;

impl UserEnvironment for UnsupportedUserEnvironment {
    fn config_dir(&self) -> PathBuf {
        PathBuf::new()
    }

    fn data_dir(&self) -> PathBuf {
        PathBuf::new()
    }
}

/// A service manager that installs nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedServiceManager;

impl ServiceManager for UnsupportedServiceManager {
    fn spec(&self) -> ServiceSpec {
        ServiceSpec {
            name: String::new(),
            program: PathBuf::new(),
            arguments: Vec::new(),
            start_at_login: false,
            bundle_identifier: None,
            reports_readiness: false,
        }
    }

    fn install(&self, _spec: &ServiceSpec) -> PlatformResult<()> {
        Err(PlatformError::unsupported("installing a service"))
    }
}

/// A log sink that writes nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedLogSink;

impl LogSink for UnsupportedLogSink {
    fn write(&self, _record: &LogRecord) {}
}

/// A notifier with nobody to notify.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedUserNotifier;

impl UserNotifier for UnsupportedUserNotifier {
    fn notify(&self, _notification: &Notification) -> PlatformResult<()> {
        Err(PlatformError::unsupported("showing a notification"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UnsupportedConsent, UnsupportedFileSystem, UnsupportedNetwork, UnsupportedServiceManager,
        UnsupportedUserNotifier, UnsupportedVolumeWatcher,
    };
    use crate::platform::traits::VolumeWatcher;
    use crate::platform::traits::{Consent, FileSystem, Network, ServiceManager, UserNotifier};

    /// Every capability refuses politely rather than panicking.
    #[test]
    fn every_capability_refuses_politely() {
        assert!(UnsupportedNetwork.interfaces().is_err());
        assert!(UnsupportedFileSystem
            .volume_of(std::path::Path::new("/"))
            .is_err());
        assert!(UnsupportedVolumeWatcher.poll_events().is_err());
        assert!(UnsupportedServiceManager
            .install(&UnsupportedServiceManager.spec())
            .is_err());
        assert!(UnsupportedUserNotifier
            .notify(&crate::platform::traits::Notification {
                title: "t".to_owned(),
                body: "b".to_owned(),
            })
            .is_err());
    }

    /// A platform that does not ask for consent says so, rather than pretending
    /// the user agreed.
    #[test]
    fn a_platform_without_consent_says_not_required() {
        let outcome = UnsupportedConsent
            .request_folder_access(std::path::Path::new("/"))
            .expect("the default should answer");
        assert_eq!(
            outcome,
            crate::platform::traits::ConsentOutcome::NotRequired
        );
    }
}
