//! The bounded socket transport for the local administration API.
//!
//! What: connect to the local surface and exchange complete HTTP messages.
//! Why: network IO stays separate from HTTP byte parsing.
//! Example: `TcpTransport::loopback` connects to the local IPv4 interface.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use super::{HttpRequest, HttpResponse};

/// How long the client waits for the surface by default.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
/// The largest answer the client reads, in bytes.
const MAX_ANSWER_BYTES: u64 = 1024 * 1024;

/// The way the bytes of a request travel.
pub trait Transport {
    /// Sends a request and reads the answer.
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, TransportError>;

    /// The loopback address of the administration API.
    fn address(&self) -> String;

    /// The value of the header that names the host of the request.
    fn host(&self) -> String;

    /// How long a request waits for the answer.
    fn timeout(&self) -> Duration;
}

/// The transport that opens a socket to the loopback interface.
#[derive(Clone, Debug)]
pub struct TcpTransport {
    /// The address of the surface.
    address: IpAddr,
    /// The port of the surface.
    port: u16,
    /// How long the client waits.
    timeout: Duration,
}

impl TcpTransport {
    /// The transport of a surface on the loopback interface.
    pub fn loopback(port: u16) -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// The transport of a surface at a known address.
    ///
    /// The address is the one of the loopback interface in every use of this
    /// crate; the function exists for the tests and not to open a connection
    /// to the network.
    pub fn at(address: IpAddr, port: u16, timeout: Duration) -> Self {
        Self {
            address,
            port,
            timeout,
        }
    }
}

impl Transport for TcpTransport {
    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    fn host(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let timeout = request.timeout.unwrap_or(self.timeout);
        let address = SocketAddr::new(self.address, self.port);
        let mut stream = TcpStream::connect_timeout(&address, timeout).map_err(|error| {
            TransportError::Connect {
                reason: error.to_string(),
            }
        })?;
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        stream
            .write_all(&request.to_bytes())
            .map_err(|error| TransportError::Io {
                reason: error.to_string(),
            })?;
        let _ = stream.flush();
        let mut buffer = Vec::new();
        let mut limited = (&mut stream).take(MAX_ANSWER_BYTES);
        limited
            .read_to_end(&mut buffer)
            .map_err(|error| TransportError::Io {
                reason: error.to_string(),
            })?;
        HttpResponse::parse(&buffer)
    }
}

/// Why the bytes did not travel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportError {
    /// The socket could not be opened.
    Connect {
        /// What the system said.
        reason: String,
    },
    /// The bytes could not be read or written.
    Io {
        /// What the system said.
        reason: String,
    },
    /// The answer was not an answer of the protocol.
    Malformed {
        /// What was wrong with it.
        reason: String,
    },
}

impl TransportError {
    /// What was wrong, in the words of the system.
    pub fn reason(&self) -> &str {
        match self {
            Self::Connect { reason } | Self::Io { reason } | Self::Malformed { reason } => reason,
        }
    }

    /// The kind of the failure.
    pub fn kind(&self) -> TransportKind {
        match self {
            Self::Connect { .. } => TransportKind::Unreachable,
            Self::Io { .. } => TransportKind::Unreadable,
            Self::Malformed { .. } => TransportKind::Invalid,
        }
    }
}

/// The kind of a failure of the transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportKind {
    /// The surface cannot be reached.
    Unreachable,
    /// The surface cannot be read.
    Unreadable,
    /// The answer is not an answer of the protocol.
    Invalid,
}
