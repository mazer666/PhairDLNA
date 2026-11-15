//! The daemon: the server that runs in the foreground.
//!
//! What: the daemon that loads the configuration, opens the sockets and serves
//!       the library until it is told to stop.
//! Why: the server is the product. Everything it does — the description, the
//!       discovery, the browse, the media — happens here, in one place, in the
//!       order the specification names (`F-13`, `OPS-001`, `OPS-008`).
//! Example: `phairdlna serve` starts the daemon; it answers a client on the
//!       local network and stops when it receives `SIGINT` or `SIGTERM`.
//!
//! **The order of the start.** The configuration is read and validated
//! **before** any socket is opened, so that a broken configuration never leaves
//! a half-started server behind (`CFG-002`, `F-13` § 9). The order of the
//! stop is the mirror image: the discovery withdraws its advertisements, the
//! HTTP server drains its transfers, and the process exits with code 0
//! (`OPS-008`).

pub mod address;
pub mod http;
pub mod log;
pub mod media;
pub mod setup;
pub mod signal;
pub mod ssdp;
pub mod stream;

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use std::time::Duration;

use tokio::task::JoinHandle;

use log::Logger;

/// The budget the shutdown gives the open transfers to end (`OPS-008`).
const DRAIN_BUDGET: Duration = Duration::from_secs(10);

/// Waits for the HTTP server to end, and describes how it ended.
///
/// The channel is a copy of the one of the server: the daemon keeps the
/// handle of the server for the shutdown, and only the *news* travels here.
async fn wait_for_the_server(finished: &mut Option<tokio::sync::watch::Receiver<bool>>) -> String {
    let Some(receiver) = finished.as_mut() else {
        return "there was no server".to_owned();
    };
    if receiver.changed().await.is_err() {
        return "the server ended without a result".to_owned();
    }
    "the server stopped".to_owned()
}

/// How the daemon is started.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// The configuration file to read.
    pub config_path: PathBuf,
    /// The port of the HTTP server, or `None` for the configured one.
    ///
    /// The port `0` asks the operating system for a free port, which is what
    /// the tests use.
    pub http_port: Option<u16>,
    /// Whether the discovery socket is opened.
    pub ssdp: bool,
    /// The address the base URL is built with, or `None` for the one the
    /// machine has.
    pub host: Option<Ipv4Addr>,
    /// The port of the discovery socket, or `None` for the one the
    /// specification names.
    ///
    /// The tests use another port, so that a test run does not compete with a
    /// real server on the network for the port 1900.
    pub ssdp_port: Option<u16>,
    /// The port the advertisements are sent to, or `None` for the port of the
    /// socket itself.
    ///
    /// The port is the same in a real server: a client listens on 1900. Only a
    /// test names another one, because a listener and the server cannot share
    /// a port.
    pub ssdp_advertise_port: Option<u16>,
    /// The file the device identity is read from, or `None` for the standard
    /// place.
    ///
    /// The tests point it at a temporary file, so that a test run never writes
    /// into the configuration directory of the user.
    pub identity_path: Option<PathBuf>,
}

impl ServeOptions {
    /// The options of a daemon that reads the given configuration.
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            http_port: None,
            ssdp: true,
            host: None,
            ssdp_port: None,
            ssdp_advertise_port: None,
            identity_path: None,
        }
    }
}

/// The daemon, with its sockets open.
pub struct Daemon {
    context: Arc<http::ServerContext>,
    address: SocketAddr,
    ssdp: Option<Arc<ssdp::SsdpSocket>>,
    logger: Logger,
    server: Option<setup::HttpServer>,
    stop_ssdp: Option<tokio::sync::watch::Sender<bool>>,
    ssdp_task: Option<JoinHandle<()>>,
}

impl Daemon {
    /// Starts the daemon: the configuration, the identity, the catalogue, the
    /// HTTP server and the discovery socket.
    ///
    /// The steps themselves live in `setup`, because their order is a rule of
    /// its own; this function is the door to them.
    pub async fn start(options: ServeOptions) -> Result<Self, String> {
        setup::start(options).await
    }

    /// The address the HTTP server listens on.
    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    /// The base URL the documents are built with.
    pub fn base_url(&self) -> &str {
        &self.context.base_url
    }

    /// Runs the daemon until the shutdown signal arrives.
    ///
    /// The order of the stop is the one of `OPS-008`: the discovery withdraws
    /// its advertisements, the HTTP server drains the open transfers for a
    /// bounded time, and the daemon returns.
    pub async fn run(mut self) -> Result<(), String> {
        // The channel that tells the daemon that the server ended. It is a
        // copy: the handle of the server stays in the daemon, so that the
        // shutdown can still ask the server to stop and drain its transfers.
        let mut finished = self.server.as_ref().map(|server| server.finished.clone());
        tokio::select! {
            _ = signal::wait_for_shutdown() => {
                self.logger.info("the daemon stops", &[]);
                let sequence = self.shutdown();
                tokio::select! {
                    result = sequence => result,
                    _ = signal::wait_for_shutdown() => {
                        self.logger.warn("the second signal arrived; the daemon stops at once", &[]);
                        Ok(())
                    }
                }
            }
            ended = wait_for_the_server(&mut finished) => {
                // The server ended without a signal: a daemon without a server
                // has nothing left to do, so it stops and says why.
                self.logger.error(
                    "the server stopped; the daemon stops as well",
                    &[("reason", &ended)],
                );
                self.shutdown().await
            }
        }
    }

    /// Stops the daemon in the order of `OPS-008`.
    ///
    /// The advertisements are withdrawn first, so that a client stops seeing
    /// the server before the server stops answering; the open transfers are
    /// then drained.
    pub async fn shutdown(&mut self) -> Result<(), String> {
        self.stop_sequence().await
    }

    /// The shutdown sequence: the advertisements are withdrawn, the discovery
    /// stops and the HTTP server drains its transfers.
    async fn stop_sequence(&mut self) -> Result<(), String> {
        self.withdraw_advertisements().await;
        self.stop_discovery().await;
        self.stop_http().await;
        Ok(())
    }

    /// Sends the advertisements that say the server is gone.
    async fn withdraw_advertisements(&mut self) {
        let Some(ssdp) = self.ssdp.as_ref() else {
            return;
        };
        if let Err(error) = ssdp.advertise_byebye().await {
            self.logger.warn(
                "the advertisements could not be withdrawn",
                &[("reason", &error)],
            );
        }
    }

    /// Stops the task that answers the searches.
    async fn stop_discovery(&mut self) {
        if let Some(stop) = self.stop_ssdp.take() {
            let _ = stop.send(true);
        }
        if let Some(task) = self.ssdp_task.take() {
            let _ = task.await;
        }
    }

    /// Stops the HTTP server and waits for the open transfers to drain.
    ///
    /// The drain has a budget: a transfer that does not end within
    /// `DRAIN_BUDGET` is cut off, so that a stuck client cannot keep the
    /// daemon alive forever (`OPS-008`).
    async fn stop_http(&mut self) {
        let Some(mut server) = self.server.take() else {
            return;
        };
        let _ = server.stop.send(());
        match tokio::time::timeout(DRAIN_BUDGET, &mut server.ended).await {
            Ok(Ok(Ok(()))) => self.logger.info("the server stopped", &[]),
            Ok(Ok(Err(error))) => self.logger.warn(
                "the server stopped with an error",
                &[("reason", &error.to_string())],
            ),
            Ok(Err(_)) => self.logger.warn("the server ended without a result", &[]),
            Err(_) => self.logger.warn(
                "the open transfers did not end within the budget",
                &[("budget", &format!("{} s", DRAIN_BUDGET.as_secs()))],
            ),
        }
    }
}

/// Runs the daemon in the foreground until it is stopped.
///
/// The runtime is a multi-thread one: the discovery, the HTTP server and the
/// wait for the signal run beside one another, and none of them may hold the
/// others up (`OPS-001`).
pub fn run(options: ServeOptions) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("the runtime cannot be started: {error}"))?;
    runtime.block_on(async {
        let daemon = setup::start(options).await?;
        daemon.run().await
    })
}
