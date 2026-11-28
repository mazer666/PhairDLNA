//! Acceptance test AC-13 / FUN-045 for HTTP transfer state reporting.
//!
//! AC-13: "Given an active transfer, when the position and duration are read
//! through the event bus, then they match the bytes sent and the length of the
//! file, and the MIME type is the one in the res element."

use phairdlna_core::http::transfer::TransferState;
use std::time::Duration;

#[test]
fn active_transfer_position_and_duration_reporting() {
    // Media with 120s duration, 24,000,000 bytes (~24 MB) video/mp4
    let mime = "video/mp4";
    let file_length = 24_000_000;
    let duration = Some(Duration::from_secs(120));

    let mut transfer = TransferState::new(mime, file_length, duration);

    // Initial state: 0 bytes sent, position 0s
    assert_eq!(transfer.mime_type(), mime);
    assert_eq!(transfer.total_bytes(), file_length);
    assert_eq!(transfer.bytes_sent(), 0);
    assert_eq!(transfer.duration(), duration);
    assert_eq!(transfer.position(), Duration::ZERO);
    assert_eq!(transfer.position_upnp_string(), "00:00:00");
    assert_eq!(transfer.duration_upnp_string(), "00:02:00");

    // Client streams 6 MB (25% progress)
    transfer.advance(6_000_000);
    assert_eq!(transfer.bytes_sent(), 6_000_000);
    assert_eq!(transfer.position(), Duration::from_secs(30));
    assert_eq!(transfer.position_upnp_string(), "00:00:30");

    // Client streams another 12 MB (now 18 MB / 75% progress)
    transfer.advance(12_000_000);
    assert_eq!(transfer.bytes_sent(), 18_000_000);
    assert_eq!(transfer.position(), Duration::from_secs(90));
    assert_eq!(transfer.position_upnp_string(), "00:01:30");

    // Client finishes remaining 6 MB (100% progress)
    transfer.advance(6_000_000);
    assert_eq!(transfer.bytes_sent(), 24_000_000);
    assert_eq!(transfer.position(), Duration::from_secs(120));
    assert_eq!(transfer.position_upnp_string(), "00:02:00");
}
