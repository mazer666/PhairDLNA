//! Tests of the web user interface and settings API.
//!
//! What: verifies that the embedded single-page web UI is served as HTML,
//!       and that the settings documentation API distinguishes standard and
//!       expert settings correctly.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use phairdlna_admin::{
    AdminBackend, AdminOptions, AdminServer, Diagnostics, RescanOutcome, Status,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct Answer {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Answer {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

struct EmptyFake;

impl AdminBackend for EmptyFake {
    fn status(&self) -> Status {
        Status::default()
    }

    fn rescan(&self) -> RescanOutcome {
        RescanOutcome {
            rescanned: true,
            objects: 0,
            duration_seconds: 0,
            message: String::new(),
        }
    }

    fn diagnostics(&self) -> Diagnostics {
        Diagnostics::default()
    }

    fn note_refusal(&self, _reason: &str) {}
}

async fn start(token_path: &Path) -> (AdminServer, String, SocketAddr) {
    let options = AdminOptions::loopback(0, token_path.to_path_buf());
    let server = AdminServer::start(options, Arc::new(EmptyFake))
        .await
        .expect("the surface binds");
    let address = server.addresses()[0];
    let token = server.token().as_str().to_owned();
    (server, token, address)
}

fn allowed(token: &str) -> Vec<(String, String)> {
    vec![
        ("Host".to_owned(), "127.0.0.1".to_owned()),
        ("Authorization".to_owned(), format!("Bearer {token}")),
        ("X-Phair-Admin".to_owned(), "1".to_owned()),
    ]
}

async fn request(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(String, String)],
) -> Answer {
    let mut stream = TcpStream::connect(address).await.expect("connect succeeds");
    let mut raw = format!("{method} {path} HTTP/1.1\r\n");
    for (name, value) in headers {
        raw.push_str(&format!("{name}: {value}\r\n"));
    }
    raw.push_str("Connection: close\r\n\r\n");
    stream.write_all(raw.as_bytes()).await.unwrap();
    let mut body = Vec::new();
    stream.read_to_end(&mut body).await.unwrap();
    parse_answer(&body)
}

fn parse_answer(raw: &[u8]) -> Answer {
    let text = String::from_utf8_lossy(raw);
    let mut parts = text.splitn(2, "\r\n\r\n");
    let head = parts.next().unwrap_or_default();
    let body = parts.next().unwrap_or_default().to_owned();
    let mut lines = head.lines();
    let first = lines.next().unwrap_or_default();
    let status = first
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, val)) = line.split_once(':') {
            headers.push((name.trim().to_owned(), val.trim().to_owned()));
        }
    }
    Answer {
        status,
        headers,
        body,
    }
}

/// The positive case: the web UI is answered with HTML.
#[tokio::test]
async fn the_web_ui_is_answered_with_html() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/", &allowed(&token)).await;

    assert_eq!(answer.status, 200);
    assert_eq!(
        answer.header("content-type"),
        Some("text/html; charset=\"utf-8\"")
    );
    assert!(answer.body.contains("PhairDLNA Administration"));
    assert!(answer.body.contains("Standard Mode"));
    assert!(answer.body.contains("Expert Mode"));
}

/// The positive case: config settings are answered via API.
#[tokio::test]
async fn the_config_settings_are_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/config/explain?expert=0",
        &allowed(&token),
    )
    .await;

    assert_eq!(answer.status, 200);
    assert!(answer.body.contains("server.device_name"));
    assert!(!answer.body.contains("server.ssdp_port"));

    let expert_answer = request(
        address,
        "GET",
        "/api/config/explain?expert=1",
        &allowed(&token),
    )
    .await;

    assert_eq!(expert_answer.status, 200);
    assert!(expert_answer.body.contains("server.device_name"));
    assert!(expert_answer.body.contains("server.ssdp_port"));
}
