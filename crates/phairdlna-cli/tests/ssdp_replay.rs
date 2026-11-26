//! Stored M-SEARCH replay diagnostics acceptance test.
//!
//! What: verifies replaying stored discovery requests and inspecting response datagrams (`INT-103`).
//! Why: lets operators diagnose client discovery problems without capturing live network traffic.

use std::fs;
use tempfile::tempdir;

use phairdlna_cli::{run, Command};

#[test]
fn replay_stored_msearch_fixture() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    let corpus_dir = dir.path().join("corpus");
    fs::create_dir_all(&corpus_dir).expect("corpus dir");

    let request_file = corpus_dir.join("vlc.http");
    fs::write(
        &request_file,
        "M-SEARCH * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         MAN: \"ssdp:discover\"\r\n\
         MX: 3\r\n\
         ST: ssdp:all\r\n\r\n",
    )
    .expect("write request");

    fs::write(
        &config_path,
        "schema_version = 1\n\
         [server]\n\
         device_name = \"PhairDLNA Test\"\n\
         http_port = 8200\n\
         \n\
         [library]\n\
         roots = []\n",
    )
    .expect("write config");

    let outcome = run(&Command::SsdpReplay {
        corpus: request_file,
        path: Some(config_path),
    });

    assert_eq!(
        outcome.exit_code, 0,
        "replay should succeed: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("Search Target: ssdp:all"),
        "should show ST: ssdp:all: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("Responses: 5"),
        "should produce 5 UPnP responses: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("LOCATION: http://192.168.0.10:8200/description.xml"),
        "should contain location URL: {}",
        outcome.stdout
    );
}

#[test]
fn replay_empty_corpus_directory() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    let empty_dir = dir.path().join("empty_corpus");
    fs::create_dir_all(&empty_dir).expect("empty dir");

    fs::write(
        &config_path,
        "schema_version = 1\n\
         [server]\n\
         device_name = \"PhairDLNA Test\"\n\
         http_port = 8200\n\
         \n\
         [library]\n\
         roots = []\n",
    )
    .expect("write config");

    let outcome = run(&Command::SsdpReplay {
        corpus: empty_dir,
        path: Some(config_path),
    });

    assert_eq!(
        outcome.exit_code, 0,
        "replay empty dir should succeed: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("no stored requests found in replay corpus"),
        "should report empty corpus clearly (OPS-014): {}",
        outcome.stdout
    );
}
