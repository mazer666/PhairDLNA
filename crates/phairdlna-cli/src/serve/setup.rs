//! The start of the daemon.
//!
//! What: the steps that bring the daemon up — the configuration, the identity,
//!       the catalogue, the sockets.
//! Why: the order of these steps is a rule, not a preference: a broken
//!       configuration must be refused before a socket is opened, so that a
//!       failed start leaves nothing behind (`CFG-002`, `F-13` § 9).
//! Example: `start_http` opens the listener and hands it to the server task.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use phairdlna_core::config::Config;
use phairdlna_core::identity;
use phairdlna_core::security::{HostPolicy, Limits, Redactor};
use phairdlna_core::upnp::BaseUrl;
use phairdlna_core::upnp::DeviceDescription;
use phairdlna_core::upnp::DeviceInfo;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::address;
use super::guard::Guard;
use super::http;
use super::library::{build_library, Library};
use super::limit::CappedListener;
use super::log::{Level, Logger};
use super::media;
use super::pictures::Pictures;
use super::ssdp;
use super::watch;
use super::Daemon;
use super::ServeOptions;

/// Reads and validates the configuration.
///
/// A configuration that cannot be used stops the daemon before any socket is
/// opened, and the user is told what is wrong (`CFG-002`, `F-13` § 9).
pub fn load_config(path: &std::path::Path) -> Result<Config, String> {
    Config::load(path).map_err(|error| {
        format!(
            "the configuration \"{}\" is not usable:\n{}",
            path.display(),
            error.message()
        )
    })
}

/// Loads the identity of the installation, creating it when it is missing.
pub fn load_identity(
    config: &Config,
    path: Option<&std::path::Path>,
) -> Result<identity::InstalledIdentity, String> {
    let path = match path {
        Some(path) => path.to_path_buf(),
        None => identity::identity_path(),
    };
    identity::load_or_create(&path, &config.server.device_name)
        .map_err(|error| format!("the device identity cannot be read: {error}"))
}

/// The identity strings of the device description.
pub fn device_info(config: &Config) -> DeviceInfo {
    DeviceInfo::new(config.server.device_name.clone())
}

/// The roots the media may not leave.
pub fn roots_of(config: &Config) -> Vec<PathBuf> {
    config
        .library
        .roots
        .iter()
        .map(|root| PathBuf::from(&root.path))
        .collect()
}

/// The address and the port of the server.
pub fn address_and_port(options: &ServeOptions, config: &Config) -> (Ipv4Addr, u16) {
    let host = options
        .host
        .unwrap_or_else(address::local_address_or_loopback);
    let port = options.http_port.unwrap_or(config.server.http_port);
    (host, port)
}

/// The HTTP server of the daemon, and the way to stop it.
///
/// The server reports its end through a channel of its own: the daemon has to
/// know when the server stops, both to drain the open transfers and to notice
/// a server that died on its own.
pub struct HttpServer {
    /// Asks the server to stop.
    pub(super) stop: oneshot::Sender<()>,
    /// Carries the result of the server when it ends.
    pub(super) ended: oneshot::Receiver<Result<(), std::io::Error>>,
    /// Tells the daemon that the server ended, whoever waits for it.
    ///
    /// The daemon watches this channel while it waits for a signal, and it
    /// keeps the handle of the server for the shutdown. The two must not
    /// fight over the same handle: a shutdown that cannot stop the server
    /// cuts every open transfer.
    pub(super) finished: tokio::sync::watch::Receiver<bool>,
}

/// Starts the HTTP server in its own task.
pub fn start_http(listener: CappedListener, context: Arc<http::ServerContext>) -> HttpServer {
    let (stop, stopped) = oneshot::channel::<()>();
    let (report, ended) = oneshot::channel::<Result<(), std::io::Error>>();
    let (done, finished) = tokio::sync::watch::channel(false);
    let app = http::router(context);
    let server = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = stopped.await;
    });
    tokio::spawn(async move {
        let _ = report.send(server.await);
        let _ = done.send(true);
    });
    HttpServer {
        stop,
        ended,
        finished,
    }
}

/// Opens the discovery socket and starts the tasks that use it.
pub async fn start_ssdp(
    options: &ServeOptions,
    identity: &identity::InstalledIdentity,
    config: &Config,
    host: Ipv4Addr,
    port: u16,
    logger: &Logger,
) -> (
    Option<Arc<ssdp::SsdpSocket>>,
    Option<tokio::sync::watch::Sender<bool>>,
    Option<JoinHandle<()>>,
) {
    if !options.ssdp || !config.server.discovery_enabled {
        return (None, None, None);
    }
    let server = phairdlna_core::ssdp::ServerIdentity {
        friendly_name: config.server.device_name.clone(),
        product: identity::PRODUCT_NAME.to_owned(),
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
    };
    let multicast_port = options.ssdp_port.unwrap_or(config.server.ssdp_port);
    let advertise_port = options.ssdp_advertise_port.unwrap_or(multicast_port);
    let socket = match ssdp::SsdpSocket::bind(
        &identity.as_device_identity(),
        server,
        host,
        port,
        multicast_port,
        advertise_port,
        logger.for_component("ssdp"),
    )
    .await
    {
        Ok(socket) => socket,
        Err(error) => {
            logger.warn("the server cannot be discovered", &[("reason", &error)]);
            return (None, None, None);
        }
    };
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let socket = Arc::new(socket);
    let task_socket = Arc::clone(&socket);
    let task = tokio::spawn(async move {
        task_socket.run(stopped).await;
    });
    (Some(socket), Some(stop), Some(task))
}

/// Starts the daemon: the configuration, the identity, the catalogue, the
/// HTTP server and the discovery socket.
pub async fn start(options: ServeOptions) -> Result<super::Daemon, String> {
    let config = load_config(&options.config_path)?;
    // The pseudonyms of the log belong to the daemon: every component names
    // the same path with the same pseudonym (`SEC-094`).
    let redactor = Arc::new(Redactor::new());
    let logger = Logger::with_redactor(
        Level::from_config(config.logging.level),
        "daemon",
        Arc::clone(&redactor),
    )
    .redacting(config.logging.redact);
    let identity = load_identity(&config, options.identity_path.as_deref())?;
    let library = build_library(&config, &logger);
    let (host, port) = address_and_port(&options, &config);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .map_err(|error| format!("the port {port} cannot be opened: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("the address of the server is not known: {error}"))?;
    // The base URL carries the port the operating system gave, not the one
    // that was asked for: the port `0` means "any free port", and a client
    // that reads the description must learn the port that is really open
    // (`INT-022`, `INT-006`).
    let base_url = BaseUrl::new(&format!("http://{host}:{}", address.port()))
        .map_err(|error| format!("the base URL is not usable: {error}"))?;
    let (context, limits) = build_server_context(
        &config,
        &identity,
        base_url,
        library.clone(),
        &logger,
        redactor,
    )?;
    // The listener keeps the connections inside the bounds and gives every
    // connection a deadline (`SEC-012`, `SEC-016`).
    let listener = CappedListener::new(
        listener,
        limits.connections,
        limits.read_timeout,
        limits.write_timeout,
    );
    let server = start_http(listener, context.clone());
    let (ssdp, stop_ssdp, ssdp_task) =
        start_ssdp(&options, &identity, &config, host, address.port(), &logger).await;
    // The watch keeps the catalogue in step with the drives of the library: a
    // removable drive that is unplugged, and a network share that comes back,
    // are seen by the running server and not only by the next start
    // (`FUN-017`, `PLT-011`).
    let (stop_watch, watch_task) = watch::spawn(library, logger.clone());
    logger.info(
        "the server is listening",
        &[
            ("address", &address.to_string()),
            ("base", &context.base_url),
        ],
    );
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
    })
}

/// Builds the shared HTTP server context and extracts resource limits.
fn build_server_context(
    config: &Config,
    identity: &identity::InstalledIdentity,
    base_url: BaseUrl,
    library: Library,
    logger: &Logger,
    redactor: Arc<Redactor>,
) -> Result<(Arc<http::ServerContext>, Limits), String> {
    let description = DeviceDescription::new(
        &base_url,
        &identity.as_device_identity().uuid(),
        device_info(config),
    )
    .map_err(|error| format!("the device description is not usable: {error}"))?;
    let media = media::MediaLibrary::new(library.clone(), roots_of(config));
    let pictures = Pictures::new(config);
    let limits = Limits::from_section(&config.limits);
    let hosts = HostPolicy::new(config.security.allowed_host_names.clone());
    let guard = Arc::new(Guard::new(limits, hosts, redactor));
    let context = Arc::new(http::ServerContext {
        library,
        media,
        pictures,
        description,
        base_url: base_url.origin().to_owned(),
        logger: logger.clone(),
        guard,
    });
    Ok((context, limits))
}
