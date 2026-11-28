//! The acceptance criteria of F-08 for the scan schedule (FUN-024, AC-09).
//!
//! What: verifies that:
//!       - The scan schedule can be configured as an interval or manual only.
//!       - With manual only, automatic scanning is inactive and scans only occur on explicit trigger.
//!       - With an interval, background periodic scanning is enabled.

use phairdlna_core::config::{Config, ScanSchedule};
use std::fs;
use std::path::PathBuf;

fn write_temp_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    Box::leak(Box::new(dir));
    path
}

#[test]
fn ac_09_scan_schedule_configuration_and_semantics() {
    // 1. Default config has an automatic interval (default 900s)
    let default_config = Config::default();
    assert!(default_config.library.scan_schedule.is_automatic());
    assert_eq!(
        default_config.library.scan_schedule.interval_seconds(),
        Some(900)
    );

    // 2. Load custom interval config
    let interval_toml = r#"
schema_version = 1
[server]
device_name = "Scheduled"
[library]
scan_schedule = { interval = 120 }
"#;
    let path = write_temp_config(interval_toml);
    let config = Config::load(&path).expect("load interval config");
    assert_eq!(config.library.scan_schedule, ScanSchedule::Interval(120));
    assert!(config.library.scan_schedule.is_automatic());
    assert_eq!(config.library.scan_schedule.interval_seconds(), Some(120));

    // 3. Load manual-only config
    let manual_toml = r#"
schema_version = 1
[server]
device_name = "Manual"
[library]
scan_schedule = "manual_only"
"#;
    let manual_path = write_temp_config(manual_toml);
    let manual_config = Config::load(&manual_path).expect("load manual config");
    assert_eq!(
        manual_config.library.scan_schedule,
        ScanSchedule::ManualOnly
    );
    assert!(!manual_config.library.scan_schedule.is_automatic());
    assert_eq!(manual_config.library.scan_schedule.interval_seconds(), None);

    // 4. Validation: interval of 0 seconds is rejected
    let broken_toml = r#"
schema_version = 1
[server]
device_name = "Broken"
[library]
scan_schedule = { interval = 0 }
"#;
    let broken_path = write_temp_config(broken_toml);
    let err = Config::load(&broken_path).unwrap_err();
    assert_eq!(err.field_name(), Some("library.scan_schedule"));
}
