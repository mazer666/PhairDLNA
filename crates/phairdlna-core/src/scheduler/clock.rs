//! The clock of the scheduler.
//!
//! What: the source of the time the scheduler reads.
//! Why: the hysteresis and the backoff are about *time*, and a test that has
//!      to wait for a wall clock is a slow test. The clock is therefore a
//!      trait, and the tests use a clock they can move by hand.
//! Example: `ManualClock::advance` moves the clock of a test by a known
//!       amount, and the test does not sleep at all.
//!
//! **The clock is an input, not a global.** The scheduler never asks the
//! operating system for the time itself; it asks its clock.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The source of the time.
pub trait Clock: Send + Sync {
    /// The time of now.
    fn now(&self) -> Instant;
}

/// The clock of the machine.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    /// The time of now.
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock that a test moves by hand.
///
/// The clock starts at the moment it was built and moves only when the test
/// says so. A test of the hysteresis therefore asks the question "what
/// happens when the machine calms down?" without waiting for the machine to
/// calm down.
#[derive(Debug)]
pub struct ManualClock {
    /// The moment the clock started at.
    start: Instant,
    /// How far the clock has been moved.
    offset: Mutex<Duration>,
}

impl ManualClock {
    /// Builds a clock that stands still.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            offset: Mutex::new(Duration::ZERO),
        }
    }

    /// Moves the clock forward.
    pub fn advance(&self, by: Duration) {
        let mut offset = self
            .offset
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *offset += by;
    }

    /// How far the clock has been moved.
    pub fn elapsed(&self) -> Duration {
        let offset = self
            .offset
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *offset
    }
}

impl Default for ManualClock {
    /// A clock that stands still.
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    /// The time of now.
    fn now(&self) -> Instant {
        self.start + self.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, ManualClock, SystemClock};
    use std::time::Duration;

    /// The clock of the machine moves on its own.
    #[test]
    fn the_clock_of_the_machine_moves() {
        let clock = SystemClock;
        let first = clock.now();
        assert!(clock.now() >= first);
    }

    /// The clock of a test stands still until the test moves it.
    #[test]
    fn the_clock_of_a_test_stands_still() {
        let clock = ManualClock::new();
        let first = clock.now();
        assert_eq!(clock.now(), first);
        clock.advance(Duration::from_secs(5));
        assert_eq!(clock.now(), first + Duration::from_secs(5));
        assert_eq!(clock.elapsed(), Duration::from_secs(5));
    }
}
