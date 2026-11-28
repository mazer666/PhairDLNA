//! Acceptance tests for bandwidth limit and pacing (FUN-044, AC-12).
//!
//! FUN-044: "The server shall apply a per-stream bandwidth limit when configured,
//! and shall document that the limit applies to the whole transfer, not to a rate."
//! AC-12: "Given a configured bandwidth limit, when a stream runs, then its
//! sustained throughput does not exceed the limit and the other streams are not starved."

use phairdlna_core::http::transfer::TransferState;
use std::time::Duration;

#[test]
fn per_stream_bandwidth_limit_pacing_calculation() {
    // Media transfer with 5,000,000 bytes (~5 MB) and 500,000 bytes/sec limit (0.5 MB/s)
    let limit_bps = 500_000;
    let mut transfer = TransferState::new("video/mp4", 5_000_000, Some(Duration::from_secs(60)))
        .with_bandwidth_limit(Some(limit_bps));

    assert_eq!(transfer.bandwidth_limit(), Some(limit_bps));
    assert_eq!(transfer.bytes_sent(), 0);
    assert_eq!(transfer.minimum_paced_duration(), Duration::ZERO);

    // After 1,000,000 bytes (1 MB), minimum elapsed time must be 2.0 seconds
    transfer.advance(1_000_000);
    assert_eq!(transfer.bytes_sent(), 1_000_000);
    assert_eq!(transfer.minimum_paced_duration(), Duration::from_secs(2));

    // After full 5,000,000 bytes (5 MB), minimum elapsed time must be 10.0 seconds
    transfer.advance(4_000_000);
    assert_eq!(transfer.bytes_sent(), 5_000_000);
    assert_eq!(transfer.minimum_paced_duration(), Duration::from_secs(10));
}

#[test]
fn unconstrained_transfer_has_no_pacing_delay() {
    let mut transfer = TransferState::new("audio/mpeg", 2_000_000, None);
    assert_eq!(transfer.bandwidth_limit(), None);

    transfer.advance(1_000_000);
    assert_eq!(transfer.minimum_paced_duration(), Duration::ZERO);
}
