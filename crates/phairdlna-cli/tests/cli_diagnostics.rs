//! Tests of the CLI describe command (`OPS-005`, `INT-015`).
//!
//! What: verifies that `describe` prints the device description XML and the exact
//!       SSDP advertisement (`NOTIFY`) and response (`M-SEARCH 200 OK`) header blocks.
//! Why: lets users diagnose interoperability problems without packet capture, even
//!      on a host with no network.

use std::fs;
use std::path::PathBuf;

use phairdlna_cli::{run, Command};

fn write_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    // Leak directory so path remains valid for the test
    Box::leak(Box::new(dir));
    path
}

/// The positive case: describe prints UPnP device description XML and SSDP headers.
#[test]
fn describe_prints_xml_and_headers() {
    let toml = r#"
schema_version = 1

[server]
device_name = "Living Room Streamer"
http_port = 8200
"#;
    let path = write_config(toml);
    let outcome = run(&Command::Describe { path: Some(path) });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);

    // Verifies XML device description
    assert!(outcome
        .stdout
        .contains("=== device description (UPnP XML) ==="));
    assert!(outcome
        .stdout
        .contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"));
    assert!(outcome
        .stdout
        .contains("<friendlyName>Living Room Streamer</friendlyName>"));
    assert!(outcome
        .stdout
        .contains("<serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>"));
    assert!(outcome
        .stdout
        .contains("<serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>"));

    // Verifies SSDP advertisement header block (NOTIFY)
    assert!(outcome
        .stdout
        .contains("=== SSDP advertisement header block (NOTIFY) ==="));
    assert!(outcome.stdout.contains("NOTIFY * HTTP/1.1\r\n"));
    assert!(outcome.stdout.contains("NT: upnp:rootdevice\r\n"));
    assert!(outcome.stdout.contains("NTS: ssdp:alive\r\n"));
    assert!(outcome
        .stdout
        .contains("LOCATION: http://192.168.0.10:8200/description.xml\r\n"));
    assert!(outcome.stdout.contains("SERVER: "));
    assert!(outcome.stdout.contains("USN: uuid:"));

    // Verifies SSDP response header block (200 OK)
    assert!(outcome
        .stdout
        .contains("=== SSDP response header block (M-SEARCH 200 OK) ==="));
    assert!(outcome.stdout.contains("HTTP/1.1 200 OK\r\n"));
    assert!(outcome.stdout.contains("ST: upnp:rootdevice\r\n"));
    assert!(outcome.stdout.contains("EXT:\r\n"));
}

/// The negative case: an invalid config returns a failure exit code.
#[test]
fn describe_fails_on_broken_config() {
    let path = write_config("schema_version = 1\n[server]\nhttp_port = 0\n");
    let outcome = run(&Command::Describe { path: Some(path) });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("not usable"));
}
