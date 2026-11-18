//! Service control for the macOS launchd agent.
//!
//! What: starts and stops the background launchd service `org.phairdlna.server`.
//! Why: lets users start or stop the server directly from the GUI without
//!      having to run terminal commands.

use std::path::Path;
use std::process::Command;

const SERVICE_LABEL: &str = "org.phairdlna.server";
const CMD_LAUNCHCTL: &str = "launchctl";
const ARG_START: &str = "start";
const ARG_STOP: &str = "stop";
const ARG_LOAD: &str = "load";
const ARG_W: &str = "-w";

/// Starts the server service via launchctl.
pub fn start_service() -> Result<(), String> {
    ensure_agent_loaded()?;
    let output = Command::new(CMD_LAUNCHCTL)
        .arg(ARG_START)
        .arg(SERVICE_LABEL)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// Stops the server service via launchctl.
pub fn stop_service() -> Result<(), String> {
    let output = Command::new(CMD_LAUNCHCTL)
        .arg(ARG_STOP)
        .arg(SERVICE_LABEL)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// Ensures the launchd plist is loaded if it exists.
fn ensure_agent_loaded() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let plist_path = format!("{home}/Library/LaunchAgents/{SERVICE_LABEL}.plist");
    if Path::new(&plist_path).exists() {
        let _ = Command::new(CMD_LAUNCHCTL)
            .args([ARG_LOAD, ARG_W, &plist_path])
            .output();
    }
    Ok(())
}
