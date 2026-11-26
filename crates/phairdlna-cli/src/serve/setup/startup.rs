//! The ordered startup sequence and construction of the server context.

use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use phairdlna_core::config::Config;
use phairdlna_core::identity;
use phairdlna_core::security::{ClientAccess, HostPolicy, Limits, Redactor};
use phairdlna_core::upnp::BaseUrl;
use phairdlna_core::upnp::DeviceDescription;
use tokio::net::TcpListener;

#[cfg(feature = "admin")]
use super::super::admin::{self, AdminStartup};
use super::super::guard::Guard;
use super::super::http;
use super::super::library::{build_library, Library};
use super::super::lifecycle::LifecycleController;
use super::super::limit::CappedListener;
use super::super::log::{Level, Logger};
use super::super::media;
use super::super::pictures::Pictures;
use super::super::resources::Resources;
use super::super::transfers::Transfers;
use super::super::watch;
use super::super::{Daemon, ServeOptions};
use super::configuration::{address_and_port, device_info, load_config, load_identity, roots_of};
use super::discovery::start_ssdp;
use super::http_server::start_http;

/// Starts the daemon: the configuration, the identity, the catalogue, the
/// HTTP server and the discovery socket.
pub async fn start(options: ServeOptions) -> Result<Daemon, String> {
    let config = load_config(&options.config_path)?;
    let prepared = prepare(&config, &options)?;
    let logger = prepared.logger.clone();
    let identity = prepared.identity.clone();
    let library = prepared.library.clone();
    let resources = prepared.resources.clone();
    let lifecycle = LifecycleController::new(Arc::clone(&prepared.redactor), logger.clone());
    let (host, port) = address_and_port(&options, &config);
    let discovery_available = Arc::new(AtomicBool::new(false));
    let (listener, address, base_url) = open_the_socket(host, port).await?;
    let transfers = Transfers::new();
    let (context, limits) = build_server_context(&config, &prepared, base_url, transfers.clone())?;
    let server = start_capped_http(listener, &limits, context.clone());
    #[cfg(feature = "admin")]
    let admin = start_admin(AdminInputs {
        config: &config,
        prepared: &prepared,
        context: &context,
        logger: &logger,
        host,
        port: options.admin_port,
        transfers,
        discovery_available: &discovery_available,
        lifecycle: lifecycle.clone(),
        resources: resources.clone(),
    })
    .await?;
    let (ssdp, stop_ssdp, ssdp_task) = start_ssdp(
        &options,
        &identity,
        &config,
        host,
        address.port(),
        &logger,
        discovery_available,
    )
    .await;
    lifecycle.transition(
        phairdlna_core::lifecycle::LifecycleState::Ready,
        format!("the server is listening on {address}"),
    );
    let (stop_watch, watch_task) =
        watch::spawn(library, logger.clone(), resources, lifecycle.clone());
    announce(&logger, &address, &context.base_url);
    Ok(Daemon {
        context,
        address,
        ssdp,
        logger,
        server: Some(server),
        stop_ssdp,
        ssdp_task,
        stop_watch: Some(stop_watch),
        watch_task: Some(watch_task),
        #[cfg(feature = "admin")]
        admin,
    })
}

/// Starts the HTTP server with bounded connections and timeouts.
fn start_capped_http(
    listener: TcpListener,
    limits: &Limits,
    context: Arc<http::ServerContext>,
) -> super::http_server::HttpServer {
    let capped = CappedListener::new(
        listener,
        limits.connections,
        limits.read_timeout,
        limits.write_timeout,
    );
    start_http(capped, context)
}

/// Starts the optional authenticated administration API.
#[cfg(feature = "admin")]
async fn start_admin(
    inputs: AdminInputs<'_>,
) -> Result<Option<phairdlna_admin::AdminServer>, String> {
    let AdminInputs {
        config,
        prepared,
        context,
        logger,
        host,
        port,
        transfers,
        discovery_available,
        lifecycle,
        resources,
    } = inputs;
    let server = admin::start(AdminStartup {
        config,
        library: prepared.library.clone(),
        transfers,
        pictures: context.pictures.clone(),
        logger,
        redactor: Arc::clone(&prepared.redactor),
        device_name: config.server.device_name.clone(),
        device_address: (!host.is_loopback() && !host.is_unspecified()).then(|| host.to_string()),
        discovery_available: Arc::clone(discovery_available),
        lifecycle,
        resources,
        port,
    })
    .await?;
    explain_the_admin_surface(config, logger, server.as_ref());
    Ok(server)
}

/// Inputs shared by the optional administration API during daemon startup.
#[cfg(feature = "admin")]
struct AdminInputs<'a> {
    config: &'a Config,
    prepared: &'a Prepared,
    context: &'a Arc<http::ServerContext>,
    logger: &'a Logger,
    host: Ipv4Addr,
    port: Option<u16>,
    transfers: Transfers,
    discovery_available: &'a Arc<AtomicBool>,
    lifecycle: LifecycleController,
    resources: super::super::resources::Resources,
}

/// Says that the server is listening.
fn announce(logger: &Logger, address: &std::net::SocketAddr, base_url: &str) {
    logger.info(
        "the server is listening",
        &[("address", &address.to_string()), ("base", base_url)],
    );
}

/// The parts of the daemon that are not sockets.
///
/// The pseudonyms of the log belong to the daemon: every component names the
/// same path with the same pseudonym (`SEC-094`).
struct Prepared {
    /// The pseudonyms that replace the private values.
    redactor: Arc<Redactor>,
    /// Where the daemon writes what it has to say.
    logger: Logger,
    /// The identity of the installation.
    identity: identity::InstalledIdentity,
    /// The library of the daemon.
    library: Library,
    /// The scheduler of the work of the daemon.
    resources: Resources,
}

/// Reads the identity of the installation and builds the library.
fn prepare(config: &Config, options: &ServeOptions) -> Result<Prepared, String> {
    let redactor = Arc::new(Redactor::new());
    let logger = Logger::with_redactor(
        Level::from_config(config.logging.level),
        "daemon",
        Arc::clone(&redactor),
    )
    .redacting(config.logging.redact);
    let identity = load_identity(config, options.identity_path.as_deref())?;
    let library = build_library(config, &logger);
    // The scheduler of the work of the daemon: the control plane, the covers
    // and the scan of the library all run in it, and the answer of a client
    // comes first (`RES-001`).
    let resources = Resources::new(config, &logger);
    Ok(Prepared {
        redactor,
        logger,
        identity,
        library,
        resources,
    })
}

/// Opens the socket of the media server and builds its base URL.
///
/// The base URL carries the port the operating system gave, not the one that
/// was asked for: the port `0` means "any free port", and a client that reads
/// the description must learn the port that is really open (`INT-022`,
/// `INT-006`).
async fn open_the_socket(
    host: Ipv4Addr,
    port: u16,
) -> Result<(TcpListener, std::net::SocketAddr, BaseUrl), String> {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .map_err(|error| format!("the port {port} cannot be opened: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("the address of the server is not known: {error}"))?;
    let base_url = BaseUrl::new(&format!("http://{host}:{}", address.port()))
        .map_err(|error| format!("the base URL is not usable: {error}"))?;
    Ok((listener, address, base_url))
}

/// Tells the user where the administration surface is and where its token is.
///
/// The surface is useless without the token, and the token is useless without
/// the address. The daemon therefore says both, once, when it starts
/// (`OPS-026`).
#[cfg(feature = "admin")]
fn explain_the_admin_surface(
    config: &Config,
    logger: &Logger,
    admin: Option<&phairdlna_admin::AdminServer>,
) {
    let Some(server) = admin else {
        return;
    };
    for failure in server.failures() {
        logger.warn(
            "the administration surface could not listen on an address",
            &[("reason", failure)],
        );
    }
    let Some(address) = server.addresses().first() else {
        return;
    };
    logger.info(
        "the administration surface is listening",
        &[
            ("address", &address.to_string()),
            ("token", &config.security.admin_token_file),
        ],
    );
}

/// Builds the shared HTTP server context and extracts resource limits.
fn build_server_context(
    config: &Config,
    prepared: &Prepared,
    base_url: BaseUrl,
    transfers: Transfers,
) -> Result<(Arc<http::ServerContext>, Limits), String> {
    let description = DeviceDescription::new(
        &base_url,
        &prepared.identity.as_device_identity().uuid(),
        device_info(config),
    )
    .map_err(|error| format!("the device description is not usable: {error}"))?;
    let media = media::MediaLibrary::new(prepared.library.clone(), roots_of(config));
    let pictures = Pictures::new(config);
    let limits = Limits::from_section(&config.limits);
    let hosts = HostPolicy::new(config.security.allowed_host_names.clone());
    // The clients that may talk to the server are the ones of the
    // configuration; a list that cannot be read stops the server before a
    // socket is opened (`CFG-002`, `CFG-009`).
    let access = ClientAccess::new(&config.security.allowed_clients)
        .map_err(|error| format!("the list of the allowed clients is not usable: {error}"))?;
    let guard = Arc::new(Guard::with_access(
        limits,
        hosts,
        access,
        Arc::clone(&prepared.redactor),
    ));
    let context = Arc::new(http::ServerContext {
        library: prepared.library.clone(),
        media,
        pictures,
        description,
        base_url: base_url.origin().to_owned(),
        logger: prepared.logger.clone(),
        guard,
        transfers,
        resources: prepared.resources.clone(),
    });
    Ok((context, limits))
}
