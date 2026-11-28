//! Acceptance tests for configuration schema and validation (CFG-010, CFG-015, AC-13, AC-17).
//!
//! What: verifies that:
//!       - AC-17: environment variable syntax remains literal without unintended expansion.
//!       - AC-13 (CFG-010): trusted-client entries naming known profiles are accepted,
//!         while unknown profiles abort validation naming the entry and known profile IDs.

use phairdlna_core::config::Config;

#[test]
fn ac_17_environment_variables_remain_literal_without_expansion() {
    let toml = r#"
schema_version = 1
[server]
device_name = "${USER_NAME}'s DLNA"
"#;
    let config = Config::parse(toml).expect("parse config with literal variable");
    assert_eq!(config.server.device_name, "${USER_NAME}'s DLNA");
}

#[test]
fn ac_13_trusted_client_profile_validation() {
    // Valid entry naming known device profile (CFG-010, AC-13)
    let valid_toml = r#"
schema_version = 1
[security]
trusted_clients = ["smart-tv-2018", "192.168.1.50=quirky-client"]
"#;
    let config = Config::parse(valid_toml).expect("parse valid trusted_clients");
    assert_eq!(
        config.security.trusted_clients,
        vec!["smart-tv-2018", "192.168.1.50=quirky-client"]
    );

    // Invalid entry naming an unknown profile must abort naming the unknown profile
    let invalid_toml = r#"
schema_version = 1
[security]
trusted_clients = ["smart-tv-2018", "unknown-gadget-v1"]
"#;
    let err = Config::parse(invalid_toml).expect_err("unknown profile must fail validation");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("unknown device profile \"unknown-gadget-v1\""),
        "error message must name the unknown profile: {err_msg}"
    );
    assert!(
        err_msg.contains("known profiles are:"),
        "error message must list known profiles: {err_msg}"
    );
    assert!(
        err_msg.contains("smart-tv-2018"),
        "error message must include known profiles like smart-tv-2018: {err_msg}"
    );
}
