//! Acceptance tests for CFG-015 / AC-17:
//! The configuration file preserves environment variables as literal strings unless explicitly expanded.

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
