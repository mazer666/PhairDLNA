//! The bytes of the requests of the administration surface.
//!
//! What: the request that is built, the answer that is read and the way the
//!       bytes travel.
//! Why: the client of the surface must speak HTTP/1.1 over a socket of the
//!      loopback interface, and the parts of that are kept apart from the
//!      parts that know what the answers mean (`SEC-071`).
//! Example: a request that carries the token of the installation asks
//!          `GET /api/status` on `http://127.0.0.1:8210`.
//!
//! **The transport is a trait.** The bytes do not know how they travel; the
//! tests therefore answer the client with a recorded answer and need no
//! socket (`MB-002`).

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

/// The name of the header that names the length of a body.
const CONTENT_LENGTH: &str = "Content-Length";
/// The version of the protocol the client speaks.
pub(crate) const HTTP_VERSION: &str = "HTTP/1.1";

/// The largest answer the client reads, in bytes.
const MAX_ANSWER_BYTES: u64 = 1024 * 1024;

/// How long the client waits for the surface.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A request of the client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    /// The method of the request.
    pub method: String,
    /// The path the request names.
    pub path: String,
    /// The headers of the request.
    pub headers: Vec<(String, String)>,
    /// The body of the request.
    pub body: Vec<u8>,
    /// How long the request waits for the answer.
    ///
    /// The answer of a request that changes something can take longer than
    /// the answer of a request that only reads: the scan of the library
    /// reads every file of the library (`UX-013`).
    pub timeout: Option<Duration>,
}

impl HttpRequest {
    /// Builds a request.
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_owned(),
            path: path.to_owned(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout: None,
        }
    }

    /// Sets how long the request waits for the answer.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Adds a header to the request.
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// The bytes of the request, as they travel.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut head = String::new();
        head.push_str(&self.method);
        head.push(' ');
        head.push_str(&self.path);
        head.push(' ');
        head.push_str(HTTP_VERSION);
        head.push_str("\r\n");
        for (name, value) in &self.headers {
            head.push_str(name);
            head.push_str(": ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        head.push_str("\r\n");
        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

/// An answer of the surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    /// The number of the answer.
    pub status: u16,
    /// The headers of the answer.
    pub headers: Vec<(String, String)>,
    /// The body of the answer.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Builds an answer.
    pub fn new(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// The value of a header of the answer.
    ///
    /// The name of a header does not matter in its case, as the names of the
    /// headers are compared without regard to it.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The body of the answer, as a text.
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Reads an answer out of the bytes that travel.
    ///
    /// The answer is an error when the bytes are not an answer of the
    /// protocol: the surface speaks HTTP/1.1 and nothing else.
    pub fn parse(bytes: &[u8]) -> Result<HttpResponse, TransportError> {
        let text = String::from_utf8_lossy(bytes);
        let Some((head, body)) = text.split_once("\r\n\r\n") else {
            return Err(TransportError::Malformed {
                reason: "the answer has no empty line between the headers and the body".to_owned(),
            });
        };
        let mut lines = head.split("\r\n");
        let status = status_of(lines.next().unwrap_or_default())?;
        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
            .collect();
        let response = HttpResponse::new(status, headers, body.as_bytes().to_vec());
        Ok(response.with_body_length())
    }

    /// Cuts the body to the length the answer names.
    fn with_body_length(mut self) -> Self {
        let Some(length) = self.header(CONTENT_LENGTH) else {
            return self;
        };
        let Ok(length) = length.trim().parse::<usize>() else {
            return self;
        };
        if self.body.len() > length {
            self.body.truncate(length);
        }
        self
    }
}

/// The number of an answer, out of its first line.
fn status_of(line: &str) -> Result<u16, TransportError> {
    let mut parts = line.split_whitespace();
    let version = parts.next().unwrap_or_default();
    if !version.starts_with("HTTP/") {
        return Err(TransportError::Malformed {
            reason: "the answer does not begin with a version of the protocol".to_owned(),
        });
    }
    parts
        .next()
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| TransportError::Malformed {
            reason: "the answer carries no number".to_owned(),
        })
}

/// The way the bytes of a request travel.
pub trait Transport {
    /// Sends a request and reads the answer.
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, TransportError>;

    /// The address of the surface, as the browser may name it.
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
