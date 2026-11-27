//! The on-demand scan command of the command line (`OPS-003`).
//!
//! What: triggers a library scan on demand, either via the running daemon's
//!       administration surface (`/api/catalogue/rescan`) or in-process when
//!       no daemon is running.
//! Why: lets operators and automated jobs refresh the catalogue and inspect
//!      progress, seen/changed/skipped counts and errors (`AC-04`).

use std::path::PathBuf;
use std::time::Instant;

use phairdlna_core::catalogue::Catalogue;
use phairdlna_core::config::Config;
use phairdlna_core::json::Json;
use phairdlna_core::platform::LocalFileSystem;

use crate::Outcome;

/// Options for the scan command.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScanOptions {
    /// The configuration file to use.
    pub path: Option<PathBuf>,
    /// Optional specific root name or path to scan.
    pub root: Option<String>,
}

/// Executes the on-demand scan command.
pub fn scan(options: &ScanOptions) -> Outcome {
    let path = crate::resolve(options.path.as_deref());
    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::configuration_error(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ));
        }
    };

    // Attempt to trigger rescan via running daemon first
    if let Ok(rescan_outcome) = trigger_daemon_rescan(&config) {
        return Outcome::ok(format!(
            "Daemon rescan completed: {}\nObjects indexed: {}\n",
            rescan_outcome.message, rescan_outcome.objects
        ));
    }

    // Fall back to in-process scanning when no daemon is running (F-13 §9 table, AC-04)
    run_in_process_scan(&config, options.root.as_deref())
}

/// Result of triggering a daemon rescan.
struct RescanResult {
    objects: usize,
    message: String,
}

/// Sends a POST /api/catalogue/rescan request to the running daemon.
fn trigger_daemon_rescan(config: &Config) -> Result<RescanResult, String> {
    if !config.security.admin_enabled {
        return Err("admin surface disabled".to_owned());
    }
    // Read token
    let token = std::fs::read_to_string(&config.security.admin_token_file)
        .map_err(|e| format!("cannot read token: {e}"))?;
    let token = token.trim();
    if token.is_empty() {
        return Err("empty token".to_owned());
    }

    // First obtain CSRF token by querying GET /api/status
    let csrf_token = fetch_csrf_token(config.security.admin_port, token)?;

    // Connect to admin surface for POST /api/catalogue/rescan
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpStream};
    use std::time::Duration;

    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, config.security.admin_port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
        .map_err(|e| format!("connect failed: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;

    let req = format!(
        "POST /api/catalogue/rescan HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Authorization: Bearer {}\r\n\
         X-Phair-Admin: 1\r\n\
         X-Phair-Admin-Csrf: {}\r\n\
         Connection: close\r\n\
         Content-Length: 0\r\n\
         \r\n",
        config.security.admin_port, token, csrf_token
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut response = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buf[..n]);
    }
    let resp_str = String::from_utf8_lossy(&response);
    let (head, body) = resp_str.split_once("\r\n\r\n").ok_or("invalid response")?;
    if !head.contains("200 OK") {
        return Err("daemon refused rescan".to_owned());
    }

    let parsed = Json::parse(body).map_err(|e| e.to_string())?;
    let objects = parsed.get("objects").and_then(Json::as_u64).unwrap_or(0) as usize;
    let message = parsed
        .get("message")
        .and_then(Json::as_str)
        .unwrap_or("library rescanned")
        .to_owned();

    Ok(RescanResult { objects, message })
}

/// Fetches the CSRF token from the server's GET /api/status response header.
fn fetch_csrf_token(port: u16, token: &str) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpStream};
    use std::time::Duration;

    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
        .map_err(|e| format!("status connect failed: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;

    let req = format!(
        "GET /api/status HTTP/1.1\r\n\
         Host: 127.0.0.1:{port}\r\n\
         Authorization: Bearer {token}\r\n\
         X-Phair-Admin: 1\r\n\
         Connection: close\r\n\
         \r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut response = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buf[..n]);
    }
    let resp_str = String::from_utf8_lossy(&response);
    for line in resp_str.lines() {
        if let Some((header, value)) = line.split_once(':') {
            if header.trim().eq_ignore_ascii_case("x-phairdlna-admin-csrf") {
                return Ok(value.trim().to_owned());
            }
        }
    }
    Err("csrf token header missing".to_owned())
}

/// Performs the scan in the current CLI process.
fn run_in_process_scan(config: &Config, root_filter: Option<&str>) -> Outcome {
    let started = Instant::now();
    let roots: Vec<_> = if let Some(filter) = root_filter {
        config
            .library
            .roots
            .iter()
            .filter(|r| r.name == filter || r.path.contains(filter))
            .cloned()
            .collect()
    } else {
        config.library.roots.clone()
    };

    if roots.is_empty() {
        return Outcome::ok("No matching library roots configured to scan.\n");
    }

    let catalogue = Catalogue::build(&roots, config.server.symlink_policy, &LocalFileSystem);
    let duration = started.elapsed();
    let objects = catalogue.object_count();
    let errors = catalogue.diagnostics();

    let mut out = String::new();
    out.push_str(&format!(
        "Scan complete: {} objects discovered across {} roots in {:.2}s\n",
        objects,
        roots.len(),
        duration.as_secs_f64()
    ));

    if !errors.is_empty() {
        out.push_str("Errors and warnings encountered during scan:\n");
        for err in errors {
            out.push_str(&format!("  - {err}\n"));
        }
    }

    Outcome::ok(out)
}
