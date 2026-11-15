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

/// The honest case: the server refuses to start with a broken configuration,
/// and says why.
#[test]
fn the_server_refuses_a_broken_configuration() {
    let path = write("schema_version = 1\n[server]\nhttp_port = 0\n");
    let outcome = run(&Command::Serve {
        path: Some(path),
        port: None,
    });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("not usable"), "{}", outcome.stdout);
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

/// The positive case: the scan shows the tree of the library with the
/// identifiers a client would see.
#[test]
fn the_catalogue_scan_shows_the_tree() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let films = directory.path().join("Films");
    std::fs::create_dir_all(&films).expect("the folder is created");
    std::fs::write(films.join("King Kong.mkv"), b"a film").expect("the file is written");
    let config = format!(
        "schema_version = 1\n\n[library]\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n",
        directory.path().display()
    );
    let path = write(&config);

    let outcome = run(&Command::CatalogueScan { path: Some(path) });

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("King Kong.mkv"));
    assert!(outcome.stdout.contains("video/x-matroska"));
    assert!(outcome.stdout.contains("present"));
}

/// The negative case: a container that is not in the catalogue is reported.
#[test]
fn the_catalogue_browse_reports_an_unknown_container() {
    let path = write(VALID);
    let outcome = run(&Command::CatalogueBrowse {
        object_id: "does-not-exist".to_owned(),
        path: Some(path),
    });
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.stdout.contains("is not in the catalogue"));
}

/// The positive case: the browse shows the document a client would receive.
#[test]
fn the_catalogue_browse_shows_the_document() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");
    let config = format!(
        "schema_version = 1\n\n[library]\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n",
        directory.path().display()
    );
    let path = write(&config);

    let scan = run(&Command::CatalogueScan {
        path: Some(path.clone()),
    });
    let identifier = scan
        .stdout
        .lines()
        .find(|line| line.trim_start().starts_with("id:"))
        .and_then(|line| line.split_whitespace().last())
        .expect("the scan names the root")
        .to_owned();

    let outcome = run(&Command::CatalogueBrowse {
        object_id: identifier,
        path: Some(path),
    });

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("DIDL-Lite"));
    assert!(outcome.stdout.contains("King Kong.mkv"));
}

/// The positive case: the browse of an item shows the document of that item.
#[test]
fn the_catalogue_browse_shows_an_item() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::write(directory.path().join("King Kong.mkv"), b"a film").expect("the file is written");
    let config = format!(
        "schema_version = 1\n\n[library]\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n",
        directory.path().display()
    );
    let path = write(&config);

    let scan = run(&Command::CatalogueScan {
        path: Some(path.clone()),
    });
    let identifier = scan
        .stdout
        .lines()
        .find(|line| line.contains("King Kong.mkv"))
        .and_then(|line| line.split('[').next())
        .and_then(|line| line.split_whitespace().last())
        .expect("the scan names the file")
        .to_owned();

    let outcome = run(&Command::CatalogueBrowse {
        object_id: identifier,
        path: Some(path),
    });

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("DIDL-Lite"));
    assert!(outcome.stdout.contains("King Kong.mkv"));
}
