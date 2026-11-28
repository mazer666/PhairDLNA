//! The scan schedule configuration for the media library (FUN-024).
//!
//! What: determines whether background catalogue scans occur periodically
//!       or only when manually triggered via explicit command / API.
//! Why: allows users to choose between automatic periodic rescans (interval)
//!      and manual-only refreshes (FUN-024, AC-09).

use serde::{Deserialize, Serialize};

/// The default scan interval in seconds when automatic schedule is active (15 minutes).
pub const DEFAULT_SCAN_INTERVAL_SECS: u64 = 900;

/// The schedule for background media rescans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanSchedule {
    /// Rescans automatically at a fixed interval of seconds.
    Interval(u64),
    /// Never rescans automatically; refreshes only on explicit scan commands or drive events.
    ManualOnly,
}

impl ScanSchedule {
    /// Whether automatic periodic scans are scheduled.
    pub fn is_automatic(self) -> bool {
        matches!(self, Self::Interval(_))
    }

    /// The interval duration in seconds if automatic, or None if manual only.
    pub fn interval_seconds(self) -> Option<u64> {
        match self {
            Self::Interval(secs) => Some(secs),
            Self::ManualOnly => None,
        }
    }
}

impl Default for ScanSchedule {
    fn default() -> Self {
        Self::Interval(DEFAULT_SCAN_INTERVAL_SECS)
    }
}
