//! The server of the administration surface.
//!
//! What: the server that listens on the loopback interface and answers the
//!       three routes of the surface.
//! Why: the surface is a server of its own, with its own port, its own token
//!       and its own rules. It is not a path of the media server: the browser
//!       of the user keeps the two apart, and a page of the internet must not
//!       be able to reach the surface through the media server (`OPS-020`).
//! Example: a server that was started on the loopback interface answers
//!          `GET /api/status` on `http://127.0.0.1:8210`.
//!
//! **The token comes first.** The token is read, or written, before a socket
//! is opened: a server that cannot keep a secret does not open a door.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use phairdlna_core::security::Limits;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::backend::AdminBackend;
use crate::guard::AdminGuard;
use crate::routes::{self, AdminState};
use crate::token::{self, AdminToken, TokenError};

/// How the surface is started.
pub struct AdminOptions {
    /// The port the surface listens on.
    pub port: u16,
    /// The addresses the surface binds to.
    ///
    /// An empty list means the loopback addresses of the machine.
    pub bind_addresses: Vec<IpAddr>,
    /// The path of the file that holds the token.
    pub token_path: PathBuf,
    /// The bounds of the requests.
    pub limits: Limits,
}

impl AdminOptions {
    /// The options of a surface on the loopback interface of this machine.
    pub fn loopback(port: u16, token_path: PathBuf) -> Self {
        Self {
            port,
            bind_addresses: Vec::new(),
            token_path,
            limits: Limits::default(),
        }
    }
}

/// The server of the surface.
pub struct AdminServer {
    /// The addresses the surface listens on.
    addresses: Vec<SocketAddr>,
    /// The addresses the surface could not listen on, with the reason.
    failures: Vec<String>,
    /// The secret the requests must carry.
    token: AdminToken,
    /// The switch that tells the tasks to stop.
    shutdown: Option<watch::Sender<bool>>,
}

impl AdminServer {
    /// Starts the surface.
    ///
    /// The token is read, or written, first. Then every address is tried: a
    /// machine without IPv6 keeps the surface on IPv4, and a surface that
    /// cannot listen at all is an error the caller has to hear about.
    pub async fn start(
        options: AdminOptions,
        backend: Arc<dyn AdminBackend>,
    ) -> Result<Self, AdminError> {
        let token = token::load_or_create(&options.token_path)?;
        let guard = Arc::new(AdminGuard::new(token.clone(), options.limits));
        let state = Arc::new(AdminState::new(backend, guard));
        let (shutdown, _) = watch::channel(false);
        let mut addresses = Vec::new();
        let mut failures = Vec::new();
        for address in addresses_of(&options) {
            match TcpListener::bind(address).await {
                Ok(listener) => {
                    // The address of the socket is the one the client reaches:
                    // a port of 0 means that the machine chose the port.
                    let bound = listener.local_addr().unwrap_or(address);
                    spawn(listener, state.clone(), shutdown.subscribe());
                    addresses.push(bound);
                }
                Err(error) => failures.push(format!("{address}: {error}")),
            }
        }
        if addresses.is_empty() {
            return Err(AdminError::NoAddress {
                reasons: failures.join("; "),
            });
        }
        Ok(Self {
            addresses,
            failures,
            token,
            shutdown: Some(shutdown),
        })
    }

    /// The addresses the surface listens on.
    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }

    /// The addresses the surface could not listen on, with the reason.
    pub fn failures(&self) -> &[String] {
        &self.failures
    }

    /// The secret the requests must carry.
    pub fn token(&self) -> &AdminToken {
        &self.token
    }

    /// Stops the surface.
    ///
    /// The sockets are closed and the tasks end; the token file stays where it
    /// is, so that the next start uses the same secret.
    pub fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
    }
}

/// The addresses the surface listens on.
fn addresses_of(options: &AdminOptions) -> Vec<SocketAddr> {
    let addresses = if options.bind_addresses.is_empty() {
        vec![
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ]
    } else {
        options.bind_addresses.clone()
    };
    addresses
        .into_iter()
        .map(|address| SocketAddr::new(address, options.port))
        .collect()
}

/// Serves the surface on one listener.
fn spawn(listener: TcpListener, state: Arc<AdminState>, shutdown: watch::Receiver<bool>) {
    tokio::spawn(async move {
        let router = routes::router(state);
        let serving =
            axum::serve(listener, router).with_graceful_shutdown(wait_for_the_stop(shutdown));
        let _ = serving.await;
    });
}

/// Waits until the surface is asked to stop.
async fn wait_for_the_stop(mut shutdown: watch::Receiver<bool>) {
    while !*shutdown.borrow() {
        if shutdown.changed().await.is_err() {
            break;
        }
    }
}

/// Why the surface did not start.
#[derive(Debug)]
pub enum AdminError {
    /// The token could not be read or written.
    Token(TokenError),
    /// The surface could not listen on any address.
    NoAddress {
        /// What every attempt said.
        reasons: String,
    },
}

impl fmt::Display for AdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(error) => write!(formatter, "{error}"),
            Self::NoAddress { reasons } => {
                write!(
                    formatter,
                    "the administration surface could not listen: {reasons}"
                )
            }
        }
    }
}

impl std::error::Error for AdminError {}

impl From<TokenError> for AdminError {
    fn from(error: TokenError) -> Self {
        Self::Token(error)
    }
}
