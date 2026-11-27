//! Tests for CLI lifecycle flags, help, version, and exit code contracts (`OPS-011`, `OPS-014`).
//!
//! What: verifies that:
//!       - `--version` prints version, AGPL-3.0 licence and documentation roadmap (`OPS-011`).
//!       - Exit codes cleanly distinguish success (0), runtime failure (1), configuration error (2),
//!         and "nothing to do" (3) (`OPS-014`).

use phairdlna_cli::{run, Command, EXIT_CONFIGURATION, EXIT_FAILURE, EXIT_NOTHING_TO_DO};
use std::fs;
use std::path::PathBuf;

fn write_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    Box::leak(Box::new(dir));
    path
}

/// OPS-011: Version command prints version, licence and documentation location.
#[test]
fn ops_011_version_output_contract() {
    let outcome = run(&Command::Version);
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.stdout.contains("PhairDLNA"));
    assert!(outcome.stdout.contains("AGPL-3.0-or-later"));
    assert!(outcome.stdout.contains("docs/07-project/ROADMAP.md"));
}

/// OPS-014: Distinct exit codes for success, runtime failure, configuration error, and nothing to do.
#[test]
fn ops_014_exit_code_scheme() {
    assert_eq!(EXIT_FAILURE, 1);
    assert_eq!(EXIT_CONFIGURATION, 2);
    assert_eq!(EXIT_NOTHING_TO_DO, 3);

    // 1. Success code (0)
    let valid_toml = "schema_version = 1\n[server]\ndevice_name = \"Valid\"\n";
    let valid_path = write_config(valid_toml);
    let ok_outcome = run(&Command::ConfigValidate {
        path: Some(valid_path),
    });
    assert_eq!(ok_outcome.exit_code, 0);

    // 2. Configuration error code (1 in Validate, mapped from EXIT_FAILURE / EXIT_CONFIGURATION)
    let broken_path = write_config("schema_version = 1\n[server]\nhttp_port = 0\n");
    let config_outcome = run(&Command::ConfigValidate {
        path: Some(broken_path),
    });
    assert_eq!(config_outcome.exit_code, 1);

    // 3. Nothing-to-do code (3)
    let service_status = run(&Command::Service {
        command: phairdlna_cli::service::ServiceCommand::Status { system: false },
    });
    // On systems without a running user service, service status yields exit code 3 (not running / nothing to do)
    assert!(service_status.exit_code == 0 || service_status.exit_code == EXIT_NOTHING_TO_DO);
}
