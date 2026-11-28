//! Transfer state reporting for media delivery (FUN-045).
//!
//! What: tracks the active transfer position, total duration, bytes sent,
//!       and MIME type, and reports them as required for AVTransport / event bus.
//! Why: FUN-045 specifies: "The server shall report the position and duration of
//!       the active transfer through the AVTransport service when that service is
//!       implemented." AC-13 defines: "Given an active transfer, when the position
//!       and duration are read through the event bus, then they match the bytes sent
//!       and the length of the file, and the MIME type is the one in the res element."
//! Example:
//!
//! ```
//! use std::time::Duration;
//! use phairdlna_core::http::transfer::TransferState;
//!
//! let mut state = TransferState::new("video/mp4", 1_000_000, Some(Duration::from_secs(120)));
//! state.advance(500_000);
//! assert_eq!(state.bytes_sent(), 500_000);
//! assert_eq!(state.position(), Duration::from_secs(60));
//! ```

use std::time::Duration;

/// The state of an active media transfer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferState {
    /// The MIME type of the active transfer.
    mime_type: String,
    /// Total bytes of the media resource (file size).
    total_bytes: u64,
    /// Bytes sent so far in this transfer.
    bytes_sent: u64,
    /// The media duration, if known from container probing / metadata.
    duration: Option<Duration>,
    /// Optional bandwidth limit in bytes per second for pacing the transfer (FUN-044).
    bandwidth_limit_bytes_per_sec: Option<u64>,
}

impl TransferState {
    /// Creates a new transfer state.
    pub fn new(mime_type: impl Into<String>, total_bytes: u64, duration: Option<Duration>) -> Self {
        Self {
            mime_type: mime_type.into(),
            total_bytes,
            bytes_sent: 0,
            duration,
            bandwidth_limit_bytes_per_sec: None,
        }
    }

    /// Sets the bandwidth limit in bytes per second for this transfer (FUN-044).
    pub fn with_bandwidth_limit(mut self, limit_bytes_per_sec: Option<u64>) -> Self {
        self.bandwidth_limit_bytes_per_sec = limit_bytes_per_sec;
        self
    }

    /// The bandwidth limit in bytes per second, if configured.
    pub fn bandwidth_limit(&self) -> Option<u64> {
        self.bandwidth_limit_bytes_per_sec
    }

    /// Calculates the minimum required duration to transfer the bytes sent under the configured bandwidth limit.
    ///
    /// If a limit of `L` bytes/sec is configured, sending `N` bytes must take at least `N / L` seconds.
    pub fn minimum_paced_duration(&self) -> Duration {
        match self.bandwidth_limit_bytes_per_sec {
            Some(limit) if limit > 0 => {
                let secs = (self.bytes_sent as f64) / (limit as f64);
                Duration::from_secs_f64(secs)
            }
            _ => Duration::ZERO,
        }
    }

    /// The MIME type of the stream.
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// Total bytes of the media stream.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Bytes sent so far.
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent
    }

    /// The total duration of the media, if known.
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Advances the transfer progress by the given number of bytes.
    pub fn advance(&mut self, bytes: u64) {
        self.bytes_sent = self.bytes_sent.saturating_add(bytes).min(self.total_bytes);
    }

    /// Calculates the current playback position based on duration and bytes sent.
    ///
    /// If duration is known and total_bytes > 0, position is interpolated proportionally:
    /// `position = duration * (bytes_sent / total_bytes)`.
    /// Otherwise, if no duration is known, returns `Duration::ZERO`.
    pub fn position(&self) -> Duration {
        match self.duration {
            Some(duration) if self.total_bytes > 0 => {
                let fraction = (self.bytes_sent as f64) / (self.total_bytes as f64);
                let secs = duration.as_secs_f64() * fraction;
                Duration::from_secs_f64(secs)
            }
            _ => Duration::ZERO,
        }
    }

    /// Formats the position as UPnP / AVTransport duration string `HH:MM:SS`.
    pub fn position_upnp_string(&self) -> String {
        format_upnp_duration(self.position())
    }

    /// Formats the total duration as UPnP / AVTransport duration string `HH:MM:SS`.
    pub fn duration_upnp_string(&self) -> String {
        self.duration
            .map(format_upnp_duration)
            .unwrap_or_else(|| "00:00:00".to_owned())
    }
}

/// Formats a duration as UPnP timestamp `HH:MM:SS`.
fn format_upnp_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_position_and_formats_upnp_strings() {
        let mut state = TransferState::new(
            "video/mp4",
            2_000_000,
            Some(Duration::from_secs(3665)), // 1h 1m 5s
        );
        assert_eq!(state.bytes_sent(), 0);
        assert_eq!(state.position(), Duration::ZERO);
        assert_eq!(state.duration_upnp_string(), "01:01:05");
        assert_eq!(state.position_upnp_string(), "00:00:00");

        // Advance 50%
        state.advance(1_000_000);
        assert_eq!(state.bytes_sent(), 1_000_000);
        assert_eq!(state.position().as_secs(), 1832);
        assert_eq!(state.position_upnp_string(), "00:30:32");

        // Advance to 100%
        state.advance(1_000_000);
        assert_eq!(state.bytes_sent(), 2_000_000);
        assert_eq!(state.position().as_secs(), 3665);
        assert_eq!(state.position_upnp_string(), "01:01:05");
    }

    #[test]
    fn handles_unknown_duration() {
        let mut state = TransferState::new("audio/mpeg", 500_000, None);
        state.advance(250_000);
        assert_eq!(state.bytes_sent(), 250_000);
        assert_eq!(state.position(), Duration::ZERO);
        assert_eq!(state.duration_upnp_string(), "00:00:00");
        assert_eq!(state.position_upnp_string(), "00:00:00");
    }

    #[test]
    fn calculates_paced_duration_for_bandwidth_limit() {
        let mut state =
            TransferState::new("video/mp4", 10_000_000, None).with_bandwidth_limit(Some(1_000_000)); // 1 MB/s limit

        assert_eq!(state.bandwidth_limit(), Some(1_000_000));
        assert_eq!(state.minimum_paced_duration(), Duration::ZERO);

        state.advance(2_500_000);
        assert_eq!(state.minimum_paced_duration(), Duration::from_millis(2500));
    }
}
