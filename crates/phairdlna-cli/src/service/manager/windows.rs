//! Windows service manager commands and status parsing.
//!
//! What: command templates for `sc.exe` and PowerShell service management,
//!       and status interpretation.
//! Why: isolates Windows-specific service manager CLI syntax so `manager.rs`
//!      stays within budget and cross-platform logic remains orthogonal.

use crate::service::layout::ServiceLayout;
use crate::service::manager::ServiceCommands;
use crate::service::runner::CommandOutcome;

/// Builds the service commands for Windows Service Control Manager (`sc.exe`).
pub fn commands(layout: &ServiceLayout) -> ServiceCommands {
    let service_name = layout.service_name();
    let bin_path = format!(
        "{} {}",
        layout.program.to_string_lossy(),
        layout.arguments.join(" ")
    );

    ServiceCommands {
        program: "sc.exe".to_owned(),
        load: vec![
            "create".to_owned(),
            service_name.clone(),
            format!("binPath={bin_path}"),
            "start=auto".to_owned(),
            "DisplayName=PhairDLNA media server".to_owned(),
        ],
        unload: vec!["delete".to_owned(), service_name.clone()],
        start: vec!["start".to_owned(), service_name.clone()],
        stop: vec!["stop".to_owned(), service_name.clone()],
        status: vec!["query".to_owned(), service_name],
        pid: None,
        reload: None,
    }
}

/// Renders a JSON service specification file for Windows installations.
pub fn render(layout: &ServiceLayout) -> String {
    format!(
        "{{\n  \"name\": \"{}\",\n  \"program\": \"{}\",\n  \"arguments\": {:?},\n  \"user\": \"{}\"\n}}\n",
        layout.service_name(),
        layout.program.display(),
        layout.arguments,
        layout.user
    )
}

/// Reads the answer of `sc.exe query`.
pub fn read_status(outcome: &CommandOutcome) -> (bool, bool, String) {
    if !outcome.is_success() {
        return (false, false, outcome.stderr.trim().to_owned());
    }
    let text = outcome.stdout.trim();
    let is_running = text.contains("RUNNING") || text.contains("START_PENDING");
    (true, is_running, text.to_owned())
}
