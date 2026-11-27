//! Tests for the CLI commands `status` and `scan` (`OPS-002`, `OPS-003`, `OPS-014`).
//!
//! What: verifies that `status` and `scan` report the server and library facts correctly,
//!       interact with the running daemon or handle the stopped daemon gracefully.
//! Why: `AC-02`, `AC-03`, `AC-04` of F-13 require automated testing for status reporting,
//!      exit codes and on-demand library scans.

use std::fs;
use std::path::PathBuf;

use phairdlna_cli::{run, Command, EXIT_CONFIGURATION, EXIT_NOTHING_TO_DO};

#[path = "support/mod.rs"]
mod support;
use support::Server;

fn write_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    Box::leak(Box::new(dir));
    path
}

fn write_server_config(server: &Server) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let token_path = dir.path().join("admin.token");
    fs::write(&token_path, server.admin_token()).expect("write token");
    let config_path = dir.path().join("phairdlna.toml");
    let toml = format!(
        r#"
schema_version = 1

[server]
device_name = "Living room"
http_port = {}

[[library.roots]]
name = "Films"
path = "{}"

[security]
admin_enabled = true
admin_port = {}
admin_token_file = "{}"
"#,
        server.address().port(),
        server.library().display(),
        server.admin_address().port(),
        token_path.display()
    );
    fs::write(&config_path, toml).expect("write config");
    Box::leak(Box::new(dir));
    config_path
}

/// The negative/stopped case: `status` with no running daemon exits with EXIT_NOTHING_TO_DO (AC-03, OPS-002, OPS-014).
#[test]
fn status_with_no_daemon_running_exits_nothing_to_do() {
    let toml = r#"
schema_version = 1

[server]
device_name = "Test Media Server"
http_port = 8200

[security]
admin_enabled = true
admin_port = 65432
"#;
    let path = write_config(toml);
    let outcome = run(&Command::Status {
        path: Some(path),
        json: false,
    });

    assert_eq!(outcome.exit_code, EXIT_NOTHING_TO_DO, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("PhairDLNA daemon is not running"));
}

/// The invalid configuration case: `status` with invalid config exits with EXIT_CONFIGURATION.
#[test]
fn status_with_invalid_config_exits_configuration_error() {
    let toml = "schema_version = 999\n";
    let path = write_config(toml);
    let outcome = run(&Command::Status {
        path: Some(path),
        json: false,
    });

    assert_eq!(outcome.exit_code, EXIT_CONFIGURATION);
    assert!(outcome.stdout.contains("is not usable"));
}

/// The positive case: `status` against a running daemon prints version, uptime, roots, transfers (AC-02, OPS-002).
#[test]
fn status_with_running_daemon_reports_status() {
    let server = Server::start_with_admin();
    let path = write_server_config(&server);

    // Human-readable text format
    let outcome = run(&Command::Status {
        path: Some(path.clone()),
        json: false,
    });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("PhairDLNA"));
    assert!(outcome.stdout.contains("Uptime:"));
    assert!(outcome.stdout.contains("Catalogue objects:"));
    assert!(outcome.stdout.contains("Active transfers:"));
    assert!(outcome.stdout.contains("Configured roots:"));

    // JSON format
    let outcome_json = run(&Command::Status {
        path: Some(path),
        json: true,
    });
    assert_eq!(outcome_json.exit_code, 0, "{}", outcome_json.stdout);
    assert!(outcome_json.stdout.contains("\"version\""));
    assert!(outcome_json.stdout.contains("\"uptime_seconds\""));
    assert!(outcome_json.stdout.contains("\"catalogue_objects\""));
    assert!(outcome_json.stdout.contains("\"active_transfers\""));
}

/// The positive case: `scan` without a running daemon performs in-process scan (AC-04, OPS-003).
#[test]
fn scan_without_running_daemon_performs_in_process_scan() {
    let media_dir = tempfile::tempdir().expect("media dir");
    let file1 = media_dir.path().join("song1.mp3");
    let file2 = media_dir.path().join("video1.mp4");
    fs::write(&file1, b"ID3dummy mp3 content").expect("write mp3");
    fs::write(&file2, b"dummy mp4 content").expect("write mp4");

    let toml = format!(
        r#"
schema_version = 1

[server]
device_name = "Scan Test Server"

[[library.roots]]
name = "TestMedia"
path = "{}"
"#,
        media_dir.path().display()
    );
    let config_path = write_config(&toml);

    let outcome = run(&Command::Scan {
        path: Some(config_path),
        root: None,
    });

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("Scan complete:"));
    assert!(outcome.stdout.contains("objects discovered across 1 roots"));
}

/// The positive case: `scan` with a running daemon triggers daemon rescan (AC-04, OPS-003).
#[test]
fn scan_with_running_daemon_triggers_rescan() {
    let server = Server::start_with_admin();
    let path = write_server_config(&server);

    let outcome = run(&Command::Scan {
        path: Some(path),
        root: None,
    });

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("Daemon rescan completed"));
    assert!(outcome.stdout.contains("Objects indexed:"));
}
