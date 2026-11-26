//! Status parsing for service managers across platforms.
//!
//! What: interprets the exit code and output of service manager status commands.
//! Why: separates status parsing from ServiceManager execution to maintain 300-line budget.

use super::windows;
use crate::service::layout::Platform;
use crate::service::runner::CommandOutcome;

/// Reads the answer of the manager about the service.
pub fn read_the_status(platform: Platform, outcome: &CommandOutcome) -> (bool, bool, String) {
    match platform {
        Platform::MacOs => read_the_status_of_launchd(outcome),
        Platform::Linux => read_the_status_of_systemd(outcome),
        Platform::Windows => windows::read_status(outcome),
    }
}

/// Reads the answer of `launchctl`.
fn read_the_status_of_launchd(outcome: &CommandOutcome) -> (bool, bool, String) {
    if !outcome.is_success() {
        return (false, false, the_words_of_the_manager(outcome));
    }
    let text = outcome.stdout.trim();
    if text.is_empty() {
        return (true, false, "the job is loaded and does not run".to_owned());
    }
    let row = text.lines().nth(1).unwrap_or(text);
    let number = row.split_whitespace().next().unwrap_or("");
    let running = number.parse::<u32>().is_ok();
    (true, running, text.lines().next().unwrap_or("").to_owned())
}

/// Reads the answer of `systemctl`.
fn read_the_status_of_systemd(outcome: &CommandOutcome) -> (bool, bool, String) {
    let text = outcome.stdout.trim();
    if !outcome.is_success() && text.is_empty() {
        return (false, false, the_words_of_the_manager(outcome));
    }
    let running = text == "active" || text == "activating";
    (true, running, text.to_owned())
}

/// The words of the manager about a service that is not there.
fn the_words_of_the_manager(outcome: &CommandOutcome) -> String {
    let reason = outcome.stderr.trim();
    if reason.is_empty() {
        "the service is not loaded".to_owned()
    } else {
        reason.to_owned()
    }
}
