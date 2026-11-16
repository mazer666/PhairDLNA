//! Tests of the administration surface.
//!
//! What: proves that the surface answers the three routes it promises, and
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

/// The negative case: a request without a token is refused.
#[tokio::test]
async fn a_request_without_a_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, _token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[header("X-Phair-Admin", "1")],
    )
    .await;

    assert_eq!(answer.status, 401, "{}", answer.body);
}

/// The negative case: a request with a wrong token is refused.
#[tokio::test]
async fn a_request_with_a_wrong_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, _token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[
            header("Authorization", "Bearer 0000"),
            header("X-Phair-Admin", "1"),
        ],
    )
    .await;

    assert_eq!(answer.status, 401, "{}", answer.body);
}

/// The negative case: a request that names another host is refused.
///
/// This is the attack the rule exists for: a name the attacker controls points
/// the browser of the user at the machine of the user (`SEC-TH-23`).
#[tokio::test]
async fn a_request_that_names_another_host_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut stream = TcpStream::connect(address)
        .await
        .expect("the surface accepts");
    let text = format!(
        "GET /api/status HTTP/1.1\r\nHost: attacker.example\r\nAuthorization: Bearer {token}\r\n\
         X-Phair-Admin: 1\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(text.as_bytes())
        .await
        .expect("the request is sent");
    let mut raw = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut raw))
        .await
        .expect("the answer comes in time")
        .expect("the answer is read");

    let answer = parse(&raw);
    assert_eq!(answer.status, 421, "{}", answer.body);
}

/// The negative case: a request without the custom header is refused.
///
/// A form in a browser cannot send the header (`OPS-020`).
#[tokio::test]
async fn a_request_without_the_custom_header_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[header("Authorization", &format!("Bearer {token}"))],
    )
    .await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request from another origin is refused.
#[tokio::test]
async fn a_request_from_another_origin_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("Origin", "http://attacker.example"));
    let answer = request(address, "GET", "/api/status", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request that a browser fetched for another page is
/// refused.
#[tokio::test]
async fn a_request_for_another_page_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("Sec-Fetch-Site", "cross-site"));
    let answer = request(address, "GET", "/api/status", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: the answer never tells another origin that it may read
/// it.
#[tokio::test]
async fn the_answer_never_carries_a_cross_origin_header() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(answer.header("access-control-allow-origin"), None);
    assert_eq!(answer.header("access-control-allow-credentials"), None);
}

/// The negative case: a path the surface does not know is refused.
#[tokio::test]
async fn an_unknown_path_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/secret", &allowed(&token)).await;

    assert_eq!(answer.status, 404, "{}", answer.body);
}

/// The negative case: the surface answers no question about permissions.
#[tokio::test]
async fn a_preflight_question_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "OPTIONS", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 405, "{}", answer.body);
    assert_eq!(answer.header("access-control-allow-methods"), None);
}

/// The positive case: the status of the server is answered.
#[tokio::test]
async fn the_status_is_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(
        answer.header("content-type"),
        Some("application/json; charset=\"utf-8\"")
    );
    assert!(
        answer.body.contains("\"uptime_seconds\":42"),
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("\"active_streams\":1"),
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("\"system_update_id\":3"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"Films\""), "{}", answer.body);
}

/// The positive case: the scan of the library is answered.
#[tokio::test]
async fn the_rescan_is_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;
    let csrf = csrf_token(address, &token).await;

    let mut headers = allowed(&token);
    headers.push(header("X-Phair-Admin-Csrf", &csrf));
    let answer = request(address, "POST", "/api/catalogue/rescan", &headers).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert!(
        answer.body.contains("\"rescanned\":true"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"objects\":7"), "{}", answer.body);
}

/// The negative case: a request that changes something without the token of
/// the "cross-site request forgery" defence is refused.
#[tokio::test]
async fn a_change_without_the_csrf_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "POST", "/api/catalogue/rescan", &allowed(&token)).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request that changes something with a wrong token of
/// the "cross-site request forgery" defence is refused.
#[tokio::test]
async fn a_change_with_a_wrong_csrf_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("X-Phair-Admin-Csrf", "0000"));
    let answer = request(address, "POST", "/api/catalogue/rescan", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The positive case: the diagnostics are answered.
#[tokio::test]
async fn the_diagnostics_are_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/diagnostics", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert!(
        answer.body.contains("the library could not be read"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"warnings\""), "{}", answer.body);
}

/// The positive case: the diagnostics do not name the files of the user.
#[tokio::test]
async fn the_diagnostics_hide_the_paths() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/diagnostics", &allowed(&token)).await;

    assert!(
        !answer.body.contains("/Users/me"),
        "the path is gone: {}",
        answer.body
    );
    assert!(answer.body.contains("<path:1>"), "{}", answer.body);
}

/// The positive case: the token file is readable by its owner alone.
#[cfg(unix)]
#[tokio::test]
async fn the_token_file_is_private() {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().join("admin.token");
    let (_server, _token, _address) = start(&path).await;

    let mode = std::fs::metadata(&path)
        .expect("the token file is there")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "the file belongs to its owner alone");
}
