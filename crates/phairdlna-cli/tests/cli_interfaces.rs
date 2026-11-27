//! Integration tests for the interfaces command (`OPS-004`).
//!
//! What: verifies that `phairdlna interfaces` lists known network interfaces,
//!       evaluates roles ("used for discovery", "used for media", "excluded"),
//!       and names exclusion reasons faithfully.

use std::fs;
use std::path::PathBuf;

use phairdlna_cli::{run, Command};

fn write_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    Box::leak(Box::new(dir));
    path
}

#[test]
fn default_configuration_evaluates_interfaces() {
    let toml = r#"
schema_version = 1

[server]
device_name = "Interface Tester"
"#;
    let path = write_config(toml);
    let outcome = run(&Command::Interfaces {
        path: Some(path),
        json: false,
    });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome
        .stdout
        .contains("Network Interfaces Evaluation (OPS-004):"));
    assert!(outcome.stdout.contains("Interface \"lo0\":"));
    // lo0 lacks multicast, so it must be excluded from discovery
    assert!(outcome
        .stdout
        .contains("interface does not support multicast"));
}

#[test]
fn interfaces_command_outputs_json() {
    let toml = r#"
schema_version = 1

[server]
device_name = "JSON Interface Tester"
"#;
    let path = write_config(toml);
    let outcome = run(&Command::Interfaces {
        path: Some(path),
        json: true,
    });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.starts_with("[\n"));
    assert!(outcome.stdout.contains("\"name\": \"lo0\""));
    assert!(outcome.stdout.contains("\"supports_multicast\": false"));
}

#[test]
fn interfaces_fails_on_broken_configuration() {
    let path = write_config("schema_version = 1\n[server]\nhttp_port = 0\n");
    let outcome = run(&Command::Interfaces {
        path: Some(path),
        json: false,
    });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("not usable"));
}
