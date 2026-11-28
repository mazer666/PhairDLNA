//! Tests for configuration resolution and precedence (CFG-001, AC-01).
//!
//! CFG-001: "The server shall read its configuration from a single, versioned
//! configuration file whose path is given by an environment variable or a command-line switch."
//! AC-01: "Given a configuration file named by --config, when the server starts,
//! then that file is used; given only PHAIRDLNA_CONFIG, then that file is used;
//! given both, then the switch wins."

use phairdlna_core::config::{resolve_path, CONFIG_PATH_ENV};
use std::path::{Path, PathBuf};

#[test]
fn cli_switch_takes_precedence_over_env_and_default() {
    let cli_path = Path::new("/custom/cli/config.toml");
    std::env::set_var(CONFIG_PATH_ENV, "/custom/env/config.toml");

    let resolved = resolve_path(Some(cli_path));
    assert_eq!(resolved, PathBuf::from("/custom/cli/config.toml"));

    std::env::remove_var(CONFIG_PATH_ENV);
}

#[test]
fn env_var_takes_precedence_when_no_cli_switch() {
    std::env::set_var(CONFIG_PATH_ENV, "/custom/env/config.toml");

    let resolved = resolve_path(None);
    assert_eq!(resolved, PathBuf::from("/custom/env/config.toml"));

    std::env::remove_var(CONFIG_PATH_ENV);
}

#[test]
fn default_path_used_when_neither_cli_nor_env_set() {
    std::env::remove_var(CONFIG_PATH_ENV);

    let resolved = resolve_path(None);
    assert!(
        resolved.ends_with("config.toml"),
        "default config must be named config.toml: {}",
        resolved.display()
    );
}
