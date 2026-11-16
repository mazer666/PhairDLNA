//! Tests of the configuration module.
//!
//! What: proves that a configuration is loaded, validated and refused correctly.
//! Why: `CFG-002` and `CFG-013` are promises about *refusing* bad input; a
//!      validation that is never tested against a bad file is a validation that
//!      has never been shown to work.
//! Example: `an_unknown_key_is_refused` shows that a typo cannot silently
//!      disable a security setting.

use phairdlna_core::config::{
    Config, LogLevel, SymlinkPolicy, CURRENT_SCHEMA_VERSION, DEFAULT_HTTP_PORT,
};

/// A configuration file that is valid.
const VALID: &str = r#"
schema_version = 1

[server]
device_name = "Living room"

[[library.roots]]
name = "Films"
path = "/Users/me/Movies"
"#;

/// The positive case: a valid file is accepted.
#[test]
fn a_valid_configuration_is_accepted() {
    let config = Config::from_str(VALID, "test").expect("the configuration should be valid");
    assert_eq!(config.server.device_name, "Living room");
    assert_eq!(config.server.http_port, DEFAULT_HTTP_PORT);
    assert_eq!(config.library.roots.len(), 1);
}

/// The negative case: an empty device name is refused, with the field named.
#[test]
fn an_empty_device_name_is_refused() {
    let text = "schema_version = 1\n[server]\ndevice_name = \"\"\n";
    let error = Config::from_str(text, "test").expect_err("the configuration should be refused");
    assert_eq!(error.field_name(), Some("server.device_name"));
    assert!(error.message().contains("server.device_name"));
}

/// The negative case: an unknown key is an error, not a silent no-op.
///
/// This is the test behind `CFG-013`: a typo must never quietly disable a
/// security setting.
#[test]
fn an_unknown_key_is_refused() {
    let text = "schema_version = 1\n[server]\ndevice_nmae = \"Living room\"\n";
    let error = Config::from_str(text, "test").expect_err("the configuration should be refused");
    assert!(error.message().contains("device_nmae") || error.message().contains("unknown field"));
}

/// The negative case: a document that is not TOML at all is refused.
#[test]
fn a_broken_document_is_refused() {
    let error = Config::from_str("this is not toml", "test").expect_err("it should be refused");
    assert!(error.message().contains("cannot be parsed"));
}

/// The negative case: a discovery port of zero is refused.
#[test]
fn a_discovery_port_of_zero_is_refused() {
    let document = "schema_version = 1\n[server]\nssdp_port = 0\n";
    let error = Config::parse(document).expect_err("the port is refused");

    assert_eq!(error.field_name(), Some("server.ssdp_port"));
}

/// The positive case: the discovery port may be moved.
#[test]
fn the_discovery_port_may_be_moved() {
    let document = "schema_version = 1\n[server]\nssdp_port = 1901\n";
    let config = Config::parse(document).expect("the configuration is valid");

    assert_eq!(config.server.ssdp_port, 1901);
}

/// The boundary case: the port bounds are 1 and 65535, and 0 is refused.
#[test]
fn the_port_bounds_are_enforced() {
    let lowest = "schema_version = 1\n[server]\nhttp_port = 1\n";
    let highest = "schema_version = 1\n[server]\nhttp_port = 65535\n";
    let zero = "schema_version = 1\n[server]\nhttp_port = 0\n";

    assert!(Config::from_str(lowest, "test").is_ok());
    assert!(Config::from_str(highest, "test").is_ok());
    let error = Config::from_str(zero, "test").expect_err("port 0 should be refused");
    assert_eq!(error.field_name(), Some("server.http_port"));
}

/// The boundary case: a section that is written only partly uses the defaults.
#[test]
fn a_partial_section_uses_the_documented_defaults() {
    let text = "schema_version = 1\n[server]\ndevice_name = \"Studio\"\n";
    let config = Config::from_str(text, "test").expect("the configuration should be valid");
    assert_eq!(config.server.device_name, "Studio");
    assert_eq!(config.server.http_port, DEFAULT_HTTP_PORT);
    assert!(config.server.discovery_enabled);
}

/// The default case: an empty document yields a working, safe server.
#[test]
fn an_empty_document_yields_a_safe_server() {
    let config = Config::from_str("", "test").expect("an empty document should be valid");
    assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(config.server.discovery_enabled);
    assert!(!config.security.admin_enabled);
    assert!(!config.network.allow_non_local_addresses);
    assert!(config.logging.redact);
    assert_eq!(config.server.symlink_policy, SymlinkPolicy::Skip);
}

/// The default case: the administration surface is off and the token is outside
/// the configuration file.
#[test]
fn the_administration_surface_is_off_by_default() {
    let config = Config::default();
    assert!(!config.security.admin_enabled);
    assert!(config.security.admin_token_file.contains("admin.token"));
}

/// The versioning case: a newer schema version is refused rather than guessed.
#[test]
fn a_newer_schema_version_is_refused() {
    let text = "schema_version = 99\n";
    let error = Config::from_str(text, "test").expect_err("a newer version should be refused");
    assert!(error.message().contains("schema version 99"));
}

/// The versioning case: an older schema version is migrated forward.
#[test]
fn an_older_schema_version_is_migrated() {
    let text = "schema_version = 0\n";
    let config = Config::from_str(text, "test").expect("an older version should be migrated");
    assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
}

/// The recovery case: a configuration file that does not exist is reported as a
/// missing file, not as an invalid configuration.
#[test]
fn a_missing_file_is_reported_as_missing() {
    let path = std::path::Path::new("/nonexistent/phairdlna/config.toml");
    let error = Config::load(path).expect_err("the file should not be found");
    assert!(error.message().contains("cannot read"));
}

/// The recovery case: saving and loading again gives the same configuration.
#[test]
fn a_saved_configuration_reads_back_identically() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("config.toml");
    let original = Config::from_str(VALID, "test").expect("the configuration should be valid");

    original
        .save(&path)
        .expect("the configuration should be saved");
    let loaded = Config::load(&path).expect("the configuration should be read back");

    assert_eq!(original, loaded);
}

/// The rendering case: the configuration renders to a document that parses again.
#[test]
fn a_rendered_configuration_parses_again() {
    let original = Config::from_str(VALID, "test").expect("the configuration should be valid");
    let text = original.render().expect("the configuration should render");
    let reparsed = Config::from_str(&text, "test").expect("the rendered document should be valid");
    assert_eq!(original, reparsed);
}

/// The logging case: the default level is the documented one.
#[test]
fn the_default_log_level_is_info() {
    assert_eq!(Config::default().logging.level, LogLevel::Info);
}

/// The negative case: two roots with one name are refused, with the field named.
#[test]
fn two_roots_with_one_name_are_refused() {
    let text = r#"
schema_version = 1

[[library.roots]]
name = "Films"
path = "/Users/me/Movies"

[[library.roots]]
name = "Films"
path = "/Users/me/Archive"
"#;
    let error = Config::from_str(text, "test").expect_err("the configuration should be refused");
    assert_eq!(error.field_name(), Some("library.roots[1].name"));
}

/// The negative case: two roots with one path are refused, with the field named.
#[test]
fn two_roots_with_one_path_are_refused() {
    let text = r#"
schema_version = 1

[[library.roots]]
name = "Films"
path = "/Users/me/Movies"

[[library.roots]]
name = "Archive"
path = "/Users/me/Movies"
"#;
    let error = Config::from_str(text, "test").expect_err("the configuration should be refused");
    assert_eq!(error.field_name(), Some("library.roots[1].path"));
}
