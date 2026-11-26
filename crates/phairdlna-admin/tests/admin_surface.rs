//! Tests of the administration surface.
//!
//! What: proves that the surface answers its authenticated API routes, and
//!       that it refuses every request that does not come from the owner of
//!       the machine.
//! Why: the surface is the part of the server that a browser of the machine
//!       can be pointed at. A rule that is not tested is a rule that does not
//!       hold (`SEC-071`, `SEC-TH-23`).
//! Example: `a_request_without_a_token_is_refused` sends a request without a
//!          token and expects the answer `401`.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use phairdlna_admin::{
    AdminBackend, AdminOptions, AdminServer, Diagnostics, LogRecord, RescanOutcome, RootState,
    Status, TransferInfo,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// The answer of a request.
struct Answer {
    /// The status code of the answer.
    status: u16,
    /// The headers of the answer.
    headers: Vec<(String, String)>,
    /// The body of the answer.
    body: String,
}

impl Answer {
    /// The value of a header, in the case of a name.
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// A server that answers with fixed values.
struct Fake {
    /// How many times the library was read again.
    rescans: Arc<std::sync::atomic::AtomicUsize>,
}

impl AdminBackend for Fake {
    /// The state of the server.
    fn status(&self) -> Status {
        Status {
            lifecycle_state: phairdlna_core::lifecycle::LifecycleState::Ready,
            lifecycle_reason: "The server is running and clients can reach it.".to_owned(),
            device_name: "Living room".to_owned(),
            device_address: Some("192.168.1.20".to_owned()),
            discovery_available: false,
            uptime_seconds: 42,
            version: "0.1.0".to_owned(),
            source_revision: "abcdef123456".to_owned(),
            active_streams: 1,
            catalogue_objects: 7,
            resident_bytes: Some(4096),
            cache_bytes: 128,
            system_update_id: 3,
            roots: vec![RootState {
                name: "Films".to_owned(),
                path: "/Users/me/Films".to_owned(),
                mounted: true,
                objects: 7,
            }],
        }
    }

    /// What the server has to say about itself.
    fn diagnostics(&self) -> Diagnostics {
        Diagnostics {
            version: "0.1.0".to_owned(),
            source_revision: "abcdef123456".to_owned(),
            uptime_seconds: 42,
            catalogue_objects: 7,
            resident_bytes: Some(4096),
            system_update_id: 3,
            roots: vec![RootState {
                name: "Films".to_owned(),
                path: "/Users/me/Films".to_owned(),
                mounted: true,
                objects: 7,
            }],
            transfers: vec![TransferInfo {
                client: "Living Room TV".to_owned(),
                address: String::new(),
                title: "A film".to_owned(),
                running_seconds: 5,
            }],
            records: vec![LogRecord {
                time: "2026-11-15T19:30:00Z".to_owned(),
                level: "warn".to_owned(),
                component: "daemon".to_owned(),
                message: "the library could not be read".to_owned(),
                fields: vec![("path".to_owned(), "/Users/me/Films/a.mkv".to_owned())],
            }],
            warnings: vec!["a root could not be read".to_owned()],
            resources: Some("4 workers, machine calm".to_owned()),
            timeline: Some("2026-11-26T08:00:00Z ready reason=\"listening\"".to_owned()),
            last_scan: Some("7 objects".to_owned()),
        }
    }

    /// Reads the library again.
    fn rescan(&self) -> RescanOutcome {
        self.rescans
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        RescanOutcome {
            rescanned: true,
            objects: 7,
            duration_seconds: 1,
            message: "the library holds 7 objects".to_owned(),
        }
    }

    /// Notes that a request was refused.
    fn note_refusal(&self, _reason: &str) {}
}

/// Starts the surface on a free port and hands back its address and token.
async fn start(path: &Path) -> (AdminServer, String, SocketAddr) {
    let options = AdminOptions::loopback(0, path.to_path_buf());
    let backend = Arc::new(Fake {
        rescans: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    });
    let server = AdminServer::start(options, backend)
        .await
        .expect("the surface starts");
    let address = server.addresses()[0];
    let token = server.token().as_str().to_owned();
    (server, token, address)
}

/// Sends a request to the surface and reads the answer.
async fn request(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(String, String)],
) -> Answer {
    let mut text = format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\n");
    for (name, value) in headers {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    text.push_str("Connection: close\r\nContent-Length: 0\r\n\r\n");
    let mut stream = TcpStream::connect(address)
        .await
        .expect("the surface accepts");
    stream
        .write_all(text.as_bytes())
        .await
        .expect("the request is sent");
    let mut raw = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut raw))
        .await
        .expect("the answer comes in time")
        .expect("the answer is read");
    parse(&raw)
}

/// Reads an answer of the surface.
fn parse(raw: &[u8]) -> Answer {
    let separator = b"\r\n\r\n";
    let end = raw
        .windows(separator.len())
        .position(|part| part == separator)
        .expect("the answer has a head and a body");
    let head = String::from_utf8_lossy(&raw[..end]).into_owned();
    let body = String::from_utf8_lossy(&raw[end + separator.len()..]).into_owned();
    let mut lines = head.lines();
    let status_line = lines.next().expect("the answer has a status line");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("the status line carries a code");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Answer {
        status,
        headers,
        body,
    }
}

/// The token of the "cross-site request forgery" defence of the surface.
///
/// The client reads it from the answer of a request that changes nothing.
async fn csrf_token(address: SocketAddr, token: &str) -> String {
    let answer = request(address, "GET", "/api/status", &allowed(token)).await;
    answer
        .header("x-phairdlna-admin-csrf")
        .expect("the answer carries the token")
        .to_owned()
}

/// The headers of a request of the owner of the machine.
fn allowed(token: &str) -> Vec<(String, String)> {
    vec![
        ("Authorization".to_owned(), format!("Bearer {token}")),
        ("X-Phair-Admin".to_owned(), "1".to_owned()),
    ]
}

/// One header of a request.
fn header(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}

#[path = "admin_surface/api.rs"]
mod api;
#[path = "admin_surface/privacy.rs"]
mod privacy;
#[path = "admin_surface/security.rs"]
mod security;
#[path = "admin_surface/timeout.rs"]
mod timeout;
