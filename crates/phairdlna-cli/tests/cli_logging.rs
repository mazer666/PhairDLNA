//! Tests for CLI logging and pseudonym redaction (`OPS-006`, `OPS-007`, `SEC-094`, `AC-07`, `AC-08`).
//!
//! What: verifies that log records carry timestamp, level, component and structured message,
//!       and that paths, addresses and device identifiers are consistently pseudonymized.
//! Why: required by AC-07 and AC-08 of feature F-13.

use phairdlna_cli::serve::log::{Level, Logger};
use phairdlna_core::security::Redactor;
use std::sync::Arc;

/// AC-07: Log record carrying timestamp, level, component and structured fields.
#[test]
fn log_record_carries_timestamp_level_component_and_fields() {
    let logger = Logger::new(Level::Info, "test_comp");
    let line = logger.record(
        Level::Info,
        "a client connected",
        &[("client", "192.168.1.50"), ("count", "1")],
    );

    // Formatted line: <timestamp> <level> <component> <message> <field1>=<val1> ...
    let parts: Vec<&str> = line.split_whitespace().collect();
    assert!(parts.len() >= 4);
    assert_eq!(parts[1], "info");
    assert_eq!(parts[2], "test_comp");
    assert_eq!(parts[3], "a");
    assert!(line.contains("client=<address:1>"));
    assert!(line.contains("count=1"));
}

/// AC-08: Redaction of filesystem paths, client addresses and device identifiers.
#[test]
fn log_record_redacts_paths_addresses_and_device_identifiers() {
    let redactor = Arc::new(Redactor::new());
    let logger = Logger::with_redactor(Level::Info, "redact_test", redactor);

    let path_val = "/Users/secret/Media/video.mp4";
    let addr_val = "192.168.1.100";
    let device_val = "00000000-1111-2222-3333-444444444444";

    let line1 = logger.record(
        Level::Info,
        "media transfer started",
        &[
            ("path", path_val),
            ("client", addr_val),
            ("udn", device_val),
        ],
    );

    // Verify original sensitive values do NOT appear in the rendered log string
    assert!(!line1.contains(path_val));
    assert!(!line1.contains(addr_val));
    assert!(!line1.contains(device_val));

    // Verify pseudonyms are present
    assert!(line1.contains("path=<path:1>"));
    assert!(line1.contains("client=<address:1>"));
    assert!(line1.contains("udn=<device:1>"));

    // Verify consistent pseudonym assignment across repeated entries
    let line2 = logger.record(
        Level::Warn,
        "repeated access",
        &[("path", path_val), ("client", addr_val)],
    );
    assert!(line2.contains("path=<path:1>"));
    assert!(line2.contains("client=<address:1>"));
}
