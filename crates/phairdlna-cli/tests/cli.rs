//! Tests of the command line.
//!
//! What: proves that each command does what it says and fails as it should.
//! Why: the command line is the product's primary interface; a command that
//!      fails silently is worse than a command that does not exist.
//! Example: `a_broken_configuration_is_reported` shows that the user is told
//!      what is wrong and where.

use std::path::PathBuf;

use phairdlna_cli::{run, Command};

/// A configuration file that is valid.
const VALID: &str = r#"
schema_version = 1

[server]
device_name = "Living room"
"#;

/// The positive case: a valid configuration passes.
#[test]
fn a_valid_configuration_passes() {
    let path = write(VALID);
    let outcome = run(&Command::ConfigValidate { path: Some(path) });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("is valid"));
}

/// The negative case: an invalid configuration is reported, not accepted.
#[test]
fn a_broken_configuration_is_reported() {
    let path = write("schema_version = 1\n[server]\nhttp_port = 0\n");
    let outcome = run(&Command::ConfigValidate { path: Some(path) });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("not usable"));
    assert!(outcome.stdout.contains("server.http_port"));
}

/// The negative case: a missing file is reported as missing.
#[test]
fn a_missing_file_is_reported() {
    let outcome = run(&Command::ConfigValidate {
        path: Some(PathBuf::from("/nonexistent/phairdlna.toml")),
    });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("cannot read"));
}

/// The positive case: the effective configuration can be shown.
#[test]
fn the_effective_configuration_is_shown() {
    let path = write(VALID);
    let outcome = run(&Command::ConfigShow { path: Some(path) });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("Living room"));
}

/// The positive case: the version names the project and the licence.
#[test]
fn the_version_names_the_project_and_the_licence() {
    let outcome = run(&Command::Version);
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.stdout.contains("PhairDLNA"));
    assert!(outcome.stdout.contains("AGPL-3.0-or-later"));
}

/// The honest case: the server says that it does not exist yet, rather than
/// pretending to start.
#[test]
fn the_server_says_it_is_not_implemented() {
    let path = write(VALID);
    let outcome = run(&Command::Serve { path: Some(path) });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("not implemented yet"));
}

/// Writes a configuration file into a temporary directory and returns its path.
fn write(text: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("config.toml");
    std::fs::write(&path, text).expect("the file should be written");
    // The directory is removed when it is dropped, so the file is copied out of
    // a directory that outlives this function.
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let target = std::env::temp_dir().join(format!(
        "phairdlna-test-{}-{}-{}.toml",
        std::process::id(),
        unique(),
        count
    ));
    std::fs::copy(&path, &target).expect("the file should be copied");
    target
}

/// A number that makes the temporary file names unique.
fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or_default()
}

/// The positive case: the SSDP report names the device and shows the bytes.
#[test]
fn the_ssdp_report_shows_the_bytes() {
    let path = write(VALID);
    let outcome = run(&Command::SsdpHeaders { path: Some(path) });
    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("Device identity: uuid:"));
    assert!(outcome.stdout.contains("NOTIFY * HTTP/1.1"));
    assert!(outcome.stdout.contains("NTS: ssdp:alive"));
    assert!(outcome.stdout.contains("upnp:rootdevice"));
    assert!(outcome
        .stdout
        .contains("urn:schemas-upnp-org:device:MediaServer:1"));
}
