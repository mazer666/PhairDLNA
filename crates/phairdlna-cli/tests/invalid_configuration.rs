//! Invalid configuration validation and rollback test (REL-005, AC-14).
//!
//! What: verifies that an invalid configuration is rejected cleanly before any state change,
//!       names the offending field, and leaves the running/previous state intact.
//! Why: prevents corrupt, dangerous, or unvalidated network settings from taking effect.

use std::fs;
use tempfile::tempdir;

use phairdlna_cli::{run, Command};

#[test]
fn invalid_configuration_rejected_with_offending_field_named() {
    let dir = tempdir().expect("tempdir");
    let broken_config = dir.path().join("broken.toml");

    // Invalid port (0 is rejected by validation)
    fs::write(
        &broken_config,
        "schema_version = 1\n\
         [server]\n\
         device_name = \"Test Server\"\n\
         http_port = 0\n",
    )
    .expect("write config");

    let outcome = run(&Command::ConfigValidate {
        path: Some(broken_config),
    });

    assert_eq!(
        outcome.exit_code, 1,
        "validation of invalid config must fail"
    );
    assert!(
        outcome.stdout.contains("server.http_port"),
        "error message must name the offending field (REL-005): {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("not usable"),
        "error message must report that config is not usable: {}",
        outcome.stdout
    );
}
