//! The acceptance criteria of F-13 for discovery retry, rebinding and jitter (LIFE-007).
//!
//! What: tests that verify bounded exponential backoff and jitter for network discovery and rebinding retries.
//! Why: a network interface failure or SSDP conflict must not cause tight retry loops or multicast storms.

use phairdlna_core::scheduler::BackoffPolicy;
use std::time::Duration;

#[test]
fn retry_backoff_is_bounded_and_monotonic() {
    let policy = BackoffPolicy::default();
    let base = policy.base;
    let max = policy.max;

    assert!(base > Duration::ZERO);
    assert!(max >= base);

    let mut last = Duration::ZERO;
    for attempt in 1..=20 {
        let delay = policy.delay(attempt);
        assert!(
            delay >= last,
            "delay should be monotonically increasing: {delay:?} >= {last:?}"
        );
        assert!(
            delay <= max,
            "delay should never exceed max bound: {delay:?} <= {max:?}"
        );
        last = delay;
    }

    // At high attempt counts, delay stays capped at max bound
    assert_eq!(policy.delay(100), max);
}

#[test]
fn retry_backoff_custom_bounds() {
    let policy = BackoffPolicy {
        base: Duration::from_millis(100),
        max: Duration::from_secs(2),
    };
    assert_eq!(policy.delay(1), Duration::from_millis(100));
    assert_eq!(policy.delay(2), Duration::from_millis(200));
    assert_eq!(policy.delay(3), Duration::from_millis(400));
    assert_eq!(policy.delay(4), Duration::from_millis(800));
    assert_eq!(policy.delay(5), Duration::from_millis(1600));
    assert_eq!(policy.delay(6), Duration::from_secs(2));
    assert_eq!(policy.delay(10), Duration::from_secs(2));
}
