//! State transitions for answers from the local administration surface.
//!
//! What: methods that replace or update the server snapshot shown by the app.
//! Why: a failed refresh must clear stale server data and explain the failure.
//! Example: `take_failure` maps a refused token to the Refused state.

use std::time::Duration;

use crate::app::{App, RefreshOutcome};
use crate::ipc::client::ClientError;
use crate::ipc::snapshots::RescanReport;
use crate::model::{ServerSnapshot, ServerState};
use phairdlna_core::config::Config;

/// Maximum time to wait for a rescan response before clearing its progress.
const SCAN_DEADLINE: Duration = Duration::from_secs(180);

impl App {
    /// Whether a scan is running.
    pub fn is_scanning(&self) -> bool {
        self.scanning
    }

    /// Ends a scan that nobody answered for.
    pub fn end_of_a_silent_scan(&mut self) -> bool {
        let Some(since) = self.scanning_since else {
            return false;
        };
        if since.elapsed() < SCAN_DEADLINE {
            return false;
        }
        self.scanning = false;
        self.scanning_since = None;
        self.snapshot.state = ServerState::Running;
        self.notice = Some(self.strings.text("error.scan_silent").to_owned());
        true
    }

    /// Replaces the snapshot with the latest answer from the surface.
    pub fn take_state(&mut self, mut snapshot: ServerSnapshot) -> RefreshOutcome {
        let before = self.snapshot.clone();
        // Older daemons omit the friendly name from `/api/status`; the app
        // and daemon share this validated configuration path.
        if snapshot.device_name.trim().is_empty() {
            if let Ok(config) = Config::load(&self.config_path) {
                snapshot.device_name = config.server.device_name;
            }
        }
        if self.scanning {
            snapshot.state = ServerState::Scanning;
        }
        self.snapshot = snapshot;
        if matches!(before.state, ServerState::Stopped | ServerState::Refused)
            && self.snapshot.state == ServerState::Running
        {
            self.notice = None;
        }
        changed_or_unchanged(before != self.snapshot)
    }

    /// Clears stale server details when the surface cannot be reached.
    pub fn take_failure(&mut self, error: &ClientError) -> RefreshOutcome {
        let before_snapshot = self.snapshot.clone();
        let before_notice = self.notice.clone();
        self.snapshot = ServerSnapshot {
            state: match error {
                ClientError::Unauthorised | ClientError::Refused { .. } => ServerState::Refused,
                _ => ServerState::Stopped,
            },
            ..ServerSnapshot::default()
        };
        self.scanning = false;
        self.scanning_since = None;
        self.notice = Some(self.message_of(error));
        let changed = before_snapshot != self.snapshot || before_notice != self.notice;
        changed_or_unchanged(changed)
    }

    /// Updates the item count after a completed scan.
    pub fn take_rescan(&mut self, report: &RescanReport) -> RefreshOutcome {
        self.scanning = false;
        self.scanning_since = None;
        self.snapshot.catalogue_objects = report.objects;
        let objects = report.objects.to_string();
        self.notice = Some(
            self.strings
                .format("notice.rescan_done", &[("objects", &objects)]),
        );
        RefreshOutcome::Changed
    }

    /// Ends scan progress and shows why the scan failed.
    pub fn take_failed_rescan(&mut self, error: &ClientError) -> RefreshOutcome {
        self.scanning = false;
        self.scanning_since = None;
        self.notice = Some(self.message_of(error));
        RefreshOutcome::Changed
    }

    fn message_of(&self, error: &ClientError) -> String {
        match error {
            ClientError::Unreachable { .. } => {
                self.strings.text("error.surface_unreachable").to_owned()
            }
            ClientError::Unreadable { .. } => {
                self.strings.text("error.surface_unreadable").to_owned()
            }
            ClientError::Invalid { .. } => self.strings.text("error.answer_invalid").to_owned(),
            ClientError::Unexpected { .. } => {
                self.strings.text("error.answer_unexpected").to_owned()
            }
            ClientError::Unauthorised => self.strings.text("error.token_refused").to_owned(),
            ClientError::Refused { .. } => self.strings.text("error.request_refused").to_owned(),
            ClientError::Token { .. } => self.strings.text("error.token_unreadable").to_owned(),
            ClientError::NoToken => self.strings.text("error.server_not_started").to_owned(),
            ClientError::Busy => self.strings.text("error.busy").to_owned(),
        }
    }
}

fn changed_or_unchanged(changed: bool) -> RefreshOutcome {
    if changed {
        RefreshOutcome::Changed
    } else {
        RefreshOutcome::Unchanged
    }
}
