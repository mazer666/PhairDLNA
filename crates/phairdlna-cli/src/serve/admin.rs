//! The administration surface of the daemon.
//!
//! What: the part of the daemon that answers the questions of the
//!       administration surface — the state of the server, the diagnostics and
//!       the scan of the library.
//! Why: the surface is a crate of its own, and it knows the server only
//!       through a trait. The daemon fills that trait in here, with the
//!       library, the transfers and the log it has (`ADR-0012`).
//! Example: the surface asks for the status, and the daemon answers how many
//!          streams are running and what the roots of the library look like.
//!
//! **What the surface never gets.** The surface never gets the path of a file
//! of the user: the roots are shown with their pseudonyms, and the records of
//! the log carry the pseudonyms the log has already given them (`SEC-094`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use phairdlna_admin::{
    AdminBackend, AdminOptions, AdminServer, Diagnostics, LogRecord, RescanOutcome, RootState,
    Status, TransferInfo,
};
use phairdlna_core::catalogue::ContentSource;
use phairdlna_core::config::Config;
use phairdlna_core::platform::{LocalFileSystem, LocalProcessMemory, ProcessMemory};
use phairdlna_core::security::Redactor;

use super::library::Library;
use super::lifecycle::LifecycleController;
use super::log::Logger;
use super::pictures::Pictures;
use super::resources::Resources;
use super::transfers::Transfers;

/// How many records the diagnostics show.
const DIAGNOSTIC_RECORDS: usize = 50;

/// The daemon, seen by the administration surface.
pub struct DaemonAdmin {
    /// The friendly name advertised in the UPnP description.
    device_name: String,
    /// The network address advertised to media clients, absent in local-only mode.
    device_address: Option<String>,
    /// Whether the SSDP listener and its task are running.
    discovery_available: Arc<AtomicBool>,
    /// The operational lifecycle controller.
    lifecycle: LifecycleController,
    /// The scheduler resources of the daemon.
    resources: Resources,
    /// The library of the daemon.
    library: Library,
    /// The transfers that are running.
    transfers: Transfers,
    /// The pictures the daemon keeps.
    pictures: Pictures,
    /// Where the daemon writes what it has to say.
    logger: Logger,
    /// The pseudonyms that replace the private values.
    redactor: Arc<Redactor>,
    /// When the daemon started.
    started: Instant,
}

/// Inputs for starting the optional local administration API.
pub struct AdminStartup<'a> {
    /// Validated server configuration.
    pub config: &'a Config,
    /// The catalogue and its roots.
    pub library: Library,
    /// Active media transfers.
    pub transfers: Transfers,
    /// Cached pictures.
    pub pictures: Pictures,
    /// The daemon logger.
    pub logger: &'a Logger,
    /// Redaction for private filesystem paths.
    pub redactor: Arc<Redactor>,
    /// UPnP device name.
    pub device_name: String,
    /// Advertised address, when available.
    pub device_address: Option<String>,
    /// Whether discovery is currently available.
    pub discovery_available: Arc<AtomicBool>,
    /// Operational lifecycle controller.
    pub lifecycle: LifecycleController,
    /// Scheduler resources.
    pub resources: Resources,
    /// Optional port override.
    pub port: Option<u16>,
}

impl DaemonAdmin {
    /// Builds the view of the daemon for the surface.
    /// The roots of the library, as the surface shows them.
    fn roots(&self) -> Vec<RootState> {
        self.library.with_catalogue(|catalogue| {
            catalogue
                .roots()
                .iter()
                .map(|root| RootState {
                    name: root.name.clone(),
                    path: self.redactor.path(&root.path),
                    mounted: root.present,
                    objects: root.entries,
                })
                .collect()
        })
    }

    /// The number of objects the catalogue holds.
    fn objects(&self) -> usize {
        self.library
            .with_catalogue(|catalogue| catalogue.object_count())
    }

    /// The identifier that changes when the catalogue changes.
    fn update_id(&self) -> u32 {
        self.library
            .with_catalogue(|catalogue| catalogue.system_update_id())
    }
}

impl AdminBackend for DaemonAdmin {
    /// The state of the server.
    fn status(&self) -> Status {
        let current_transition = self.lifecycle.current_transition();
        Status {
            lifecycle_state: current_transition.state,
            lifecycle_reason: current_transition.reason,
            device_name: self.device_name.clone(),
            device_address: self.device_address.clone(),
            discovery_available: self.discovery_available.load(Ordering::Acquire),
            uptime_seconds: self.started.elapsed().as_secs(),
            version: phairdlna_core::PROJECT_VERSION.to_owned(),
            source_revision: phairdlna_core::SOURCE_REVISION.to_owned(),
            active_streams: self.transfers.count(),
            catalogue_objects: self.objects(),
            resident_bytes: LocalProcessMemory.resident_bytes(),
            cache_bytes: self.pictures.cache().size(),
            system_update_id: self.update_id(),
            roots: self.roots(),
            effective_interface: self.effective_interface(),
            effective_address: self.device_address.clone(),
            listener_state: Some(self.lifecycle.state().as_str().to_owned()),
            last_recovery_event: self.lifecycle.last_recovery(),
        }
    }

    /// What the server has to say about itself.
    fn diagnostics(&self) -> Diagnostics {
        let records = self
            .logger
            .recent(DIAGNOSTIC_RECORDS)
            .into_iter()
            .map(|record| LogRecord {
                time: record.time,
                level: record.level,
                component: record.component,
                message: record.message,
                fields: record.fields,
            })
            .collect();
        let mut warnings = self
            .library
            .with_catalogue(|catalogue| catalogue.diagnostics().to_vec());
        // The warnings of the library itself are shown as well: a damaged index
        // is told to the user in the log *and* in the status of the server
        // (`FUN-010`, `F-09` § 9).
        warnings.extend(self.library.warnings());
        Diagnostics {
            version: phairdlna_core::PROJECT_VERSION.to_owned(),
            source_revision: phairdlna_core::SOURCE_REVISION.to_owned(),
            uptime_seconds: self.started.elapsed().as_secs(),
            catalogue_objects: self.objects(),
            resident_bytes: LocalProcessMemory.resident_bytes(),
            system_update_id: self.update_id(),
            roots: self.roots(),
            transfers: self.transfers_of_the_clients(),
            records,
            warnings,
            resources: Some(self.resources.report().summary()),
            timeline: self.lifecycle.read_timeline().ok(),
            last_scan: None,
            effective_interface: self.effective_interface(),
            effective_address: self.device_address.clone(),
            listener_state: Some(self.lifecycle.state().as_str().to_owned()),
            last_recovery_event: self.lifecycle.last_recovery(),
        }
    }

    /// Reads the library again.
    fn rescan(&self) -> RescanOutcome {
        let started = Instant::now();
        self.library.refresh(&LocalFileSystem);
        let objects = self.objects();
        let duration = started.elapsed().as_secs();
        self.logger.info(
            "the library was read again",
            &[("objects", &objects.to_string())],
        );
        RescanOutcome {
            rescanned: true,
            objects,
            duration_seconds: duration,
            message: format!("the library holds {objects} objects"),
        }
    }

    /// Notes that a request of the surface was refused.
    fn note_refusal(&self, reason: &str) {
        self.logger.warn(
            "a request of the administration surface was refused",
            &[("reason", reason)],
        );
    }
}

impl DaemonAdmin {
    /// The effective interface if an address is advertised.
    fn effective_interface(&self) -> Option<String> {
        self.device_address.as_ref().map(|_| "default".to_owned())
    }

    /// The transfers that are running, as the surface shows them.
    fn transfers_of_the_clients(&self) -> Vec<TransferInfo> {
        self.transfers
            .list()
            .into_iter()
            .map(|transfer| TransferInfo {
                client: transfer.client,
                address: String::new(),
                title: transfer.title,
                running_seconds: transfer.running_seconds,
            })
            .collect()
    }
}

/// Starts the administration surface of the daemon.
///
/// The surface is started only when the configuration asks for it. A
/// configuration that does not ask for it leaves the machine with no
/// administration socket at all (`OPS-020`, `OPS-025`).
pub async fn start(startup: AdminStartup<'_>) -> Result<Option<AdminServer>, String> {
    let AdminStartup {
        config,
        library,
        transfers,
        pictures,
        logger,
        redactor,
        device_name,
        device_address,
        discovery_available,
        lifecycle,
        resources,
        port,
    } = startup;
    if !config.security.admin_enabled {
        return Ok(None);
    }
    let options = AdminOptions {
        port: port.unwrap_or(config.security.admin_port),
        bind_addresses: addresses_of(config),
        token_path: std::path::PathBuf::from(&config.security.admin_token_file),
        limits: phairdlna_core::security::Limits::from_section(&config.limits),
    };
    let backend = Arc::new(DaemonAdmin {
        device_name,
        device_address,
        discovery_available,
        lifecycle,
        resources,
        library,
        transfers,
        pictures,
        logger: logger.clone(),
        redactor,
        started: Instant::now(),
    });
    let server = AdminServer::start(options, backend)
        .await
        .map_err(|error| format!("the administration surface cannot start: {error}"))?;
    Ok(Some(server))
}

/// The addresses the surface binds to.
fn addresses_of(config: &Config) -> Vec<std::net::IpAddr> {
    config
        .security
        .admin_bind_addresses
        .iter()
        .filter_map(|address| address.trim().parse::<std::net::IpAddr>().ok())
        .collect()
}
