//! The pressure of the machine, and the hysteresis that keeps the server calm.
//!
//! What: the signals the server reads about the machine it runs on, the
//!       thresholds at which the background work yields, and the backoff a
//!       refused task is asked to wait.
//! Why: `RES-003` asks the indexing to yield when the machine is busy — and
//!      to yield *with hysteresis*, because a scanner that flaps is worse for
//!      a playback than a scanner that never starts.
//! Example: `PressureMonitor::observe` reads the signals and answers whether
//!       the background work may run.
//!
//! **Where the numbers come from.** The *policy* — which signal matters, at
//! which value the work yields, and how long a refused task waits — lives
//! here, in the core. A platform adapter supplies the *hints* and nothing
//! else (`RES-005`). A platform that cannot measure a signal says so, and the
//! signal counts as "nothing is wrong".

use std::time::{Duration, Instant};

/// What the server observes about the machine it runs on.
///
/// Every signal is a hint of the platform. A signal that the platform cannot
/// measure is zero, and zero never crosses a threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSignals {
    /// How many transfers are running right now.
    pub active_streams: usize,
    /// How long the last stream needed before its first byte, in milliseconds.
    pub streaming_latency_ms: u64,
    /// How busy the processor is, in percent of one logical core.
    pub cpu_percent: u8,
    /// How much of the memory is in use, in percent.
    pub memory_percent: u8,
    /// How busy the disk is, in percent of the time.
    pub disk_busy_percent: u8,
    /// Whether the machine is battery powered with low battery or thermally constrained (RES-008).
    pub energy_constrained: bool,
}

impl ResourceSignals {
    /// The signals of a machine about which nothing is known.
    pub fn unknown() -> Self {
        Self::default()
    }
}

/// One threshold, with its hysteresis band.
///
/// The work yields when the value rises **above** `yield_above`, and it starts
/// again when the value falls **below** `resume_below`. The space between the
/// two is the band in which nothing changes: that is the hysteresis that
/// keeps the scanner from flapping (`RES-003`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threshold {
    /// The value above which the work yields.
    pub yield_above: u32,
    /// The value below which the work starts again.
    pub resume_below: u32,
}

impl Threshold {
    /// Builds a threshold.
    pub fn new(yield_above: u32, resume_below: u32) -> Self {
        Self {
            yield_above,
            resume_below,
        }
    }

    /// Whether the band of the threshold is valid.
    ///
    /// A band that is upside down would make the work yield and start again
    /// at the same value, which is exactly the flapping the band exists to
    /// prevent.
    pub fn is_valid(&self) -> bool {
        self.resume_below <= self.yield_above
    }
}

/// The thresholds of the whole server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PressureThresholds {
    /// The latency at which the background work yields.
    pub streaming_latency: Threshold,
    /// The processor usage at which the background work yields.
    pub cpu: Threshold,
    /// The memory usage at which the background work yields.
    pub memory: Threshold,
    /// The disk usage at which the background work yields.
    pub disk: Threshold,
    /// How long the machine has to be calm before the background work starts.
    ///
    /// The quiet period is the second half of the hysteresis: the value of a
    /// signal must be below its resuming threshold *and* the machine must stay
    /// there for a while. A machine that is busy for a moment therefore does
    /// not start the scanner again, and then stop it again (`RES-003`).
    pub quiet_period: Duration,
}

impl PressureThresholds {
    /// The thresholds of an ordinary machine.
    ///
    /// The numbers are documented defaults: they are the values of the
    /// question F18-Q-01 asks, and they are written down here until a
    /// measurement on the reference hardware replaces them.
    pub fn ordinary_machine() -> Self {
        Self {
            streaming_latency: Threshold::new(500, 200),
            cpu: Threshold::new(85, 70),
            memory: Threshold::new(90, 80),
            disk: Threshold::new(90, 75),
            quiet_period: Duration::from_secs(2),
        }
    }

    /// Whether every band of the thresholds is valid.
    pub fn is_valid(&self) -> bool {
        self.streaming_latency.is_valid()
            && self.cpu.is_valid()
            && self.memory.is_valid()
            && self.disk.is_valid()
    }
}

impl Default for PressureThresholds {
    /// The thresholds of an ordinary machine.
    fn default() -> Self {
        Self::ordinary_machine()
    }
}

/// Whether the machine is busy enough to make the background work yield.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressure {
    /// The machine is calm: the background work may run.
    Calm,
    /// The machine is busy: the background work yields (`RES-003`).
    Yielding,
}

impl Pressure {
    /// The name of the state, as it appears in a report.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Calm => "calm",
            Self::Yielding => "yielding",
        }
    }
}

/// The state of the pressure, with the hysteresis.
#[derive(Debug, Clone)]
pub struct PressureMonitor {
    thresholds: PressureThresholds,
    state: Pressure,
    signals: ResourceSignals,
    calm_since: Option<Instant>,
}

impl PressureMonitor {
    /// Builds a monitor with the given thresholds.
    pub fn new(thresholds: PressureThresholds) -> Self {
        Self {
            thresholds,
            state: Pressure::Calm,
            signals: ResourceSignals::unknown(),
            calm_since: None,
        }
    }

    /// Reads the signals and answers whether the background work may run.
    ///
    /// The state changes only when a threshold is crossed in the right
    /// direction *and* the machine has been calm for the quiet period: a
    /// value inside the band of a threshold changes nothing.
    pub fn observe(&mut self, signals: &ResourceSignals, now: Instant) -> Pressure {
        self.signals = *signals;
        if self.is_pressured(signals) {
            self.state = Pressure::Yielding;
            self.calm_since = None;
            return self.state;
        }
        if self.state == Pressure::Calm {
            self.calm_since = Some(now);
            return self.state;
        }
        if self.is_still_pressured(signals) {
            // The value is inside the band: the machine is calmer than it was,
            // but it is not calm yet.
            self.calm_since = None;
            return self.state;
        }
        let since = *self.calm_since.get_or_insert(now);
        if now.saturating_duration_since(since) >= self.thresholds.quiet_period {
            self.state = Pressure::Calm;
        }
        self.state
    }

    /// The state as it is now.
    pub fn state(&self) -> Pressure {
        self.state
    }

    /// The signals of the last reading.
    pub fn signals(&self) -> &ResourceSignals {
        &self.signals
    }

    /// Whether a signal crosses the threshold at which the work yields.
    fn is_pressured(&self, signals: &ResourceSignals) -> bool {
        signals.energy_constrained
            || signals.streaming_latency_ms
                > u64::from(self.thresholds.streaming_latency.yield_above)
            || u32::from(signals.cpu_percent) > self.thresholds.cpu.yield_above
            || u32::from(signals.memory_percent) > self.thresholds.memory.yield_above
            || u32::from(signals.disk_busy_percent) > self.thresholds.disk.yield_above
    }

    /// Whether the machine is still busy after the work yielded.
    ///
    /// The signals are compared with the *lower* edge of the band, so that the
    /// work starts again only when the machine has really calmed down.
    fn is_still_pressured(&self, signals: &ResourceSignals) -> bool {
        signals.energy_constrained
            || signals.streaming_latency_ms
                > u64::from(self.thresholds.streaming_latency.resume_below)
            || u32::from(signals.cpu_percent) > self.thresholds.cpu.resume_below
            || u32::from(signals.memory_percent) > self.thresholds.memory.resume_below
            || u32::from(signals.disk_busy_percent) > self.thresholds.disk.resume_below
    }
}

/// A source of hints about the machine.
///
/// The trait carries **hints**, never a decision: what the thresholds are and
/// what happens when they are crossed is the policy of the core (`RES-005`).
pub trait ResourceProbe: Send + Sync {
    /// The signals of the machine right now.
    fn signals(&self) -> ResourceSignals;
}

/// A probe that knows nothing.
///
/// It is the right probe for a platform that cannot measure anything: the
/// signals are zero, no threshold is crossed, and the background work runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProbe;

impl ResourceProbe for NoProbe {
    /// The signals of a machine about which nothing is known.
    fn signals(&self) -> ResourceSignals {
        ResourceSignals::unknown()
    }
}

/// How long a refused task waits before it tries again.
///
/// The wait grows with every refusal, so that a task that is refused again
/// and again does not knock on the door every few milliseconds, and it never
/// grows beyond the bound (`RES-002`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffPolicy {
    /// The wait after the first refusal.
    pub base: Duration,
    /// The longest wait.
    pub max: Duration,
}

impl BackoffPolicy {
    /// The wait of the given attempt.
    ///
    /// The attempt counts from one: the first refusal waits the base time.
    pub fn delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let factor = 1u32
            .checked_shl(attempt.saturating_sub(1))
            .unwrap_or(u32::MAX);
        self.base.saturating_mul(factor).min(self.max)
    }
}

impl Default for BackoffPolicy {
    /// A wait of fifty milliseconds that never grows beyond five seconds.
    fn default() -> Self {
        Self {
            base: Duration::from_millis(50),
            max: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackoffPolicy, NoProbe, Pressure, PressureMonitor, PressureThresholds, ResourceProbe,
        ResourceSignals, Threshold,
    };
    use crate::scheduler::clock::{Clock, ManualClock};
    use std::time::Duration;

    /// A monitor that sees a calm machine stays calm.
    #[test]
    fn a_calm_machine_stays_calm() {
        let mut monitor = PressureMonitor::new(PressureThresholds::default());
        let clock = ManualClock::new();
        let signals = ResourceSignals {
            cpu_percent: 10,
            ..ResourceSignals::unknown()
        };
        assert_eq!(monitor.observe(&signals, clock.now()), Pressure::Calm);
    }

    /// A machine above a threshold makes the work yield.
    #[test]
    fn a_busy_machine_makes_the_work_yield() {
        let mut monitor = PressureMonitor::new(PressureThresholds::default());
        let clock = ManualClock::new();
        let signals = ResourceSignals {
            cpu_percent: 95,
            ..ResourceSignals::unknown()
        };
        assert_eq!(monitor.observe(&signals, clock.now()), Pressure::Yielding);
    }

    /// The work starts again only when the machine has calmed down.
    #[test]
    fn the_work_starts_again_when_the_machine_calms_down() {
        let mut monitor = PressureMonitor::new(PressureThresholds::default());
        let clock = ManualClock::new();
        let busy = ResourceSignals {
            cpu_percent: 95,
            ..ResourceSignals::unknown()
        };
        let calm = ResourceSignals {
            cpu_percent: 10,
            ..ResourceSignals::unknown()
        };
        assert_eq!(monitor.observe(&busy, clock.now()), Pressure::Yielding);
        assert_eq!(monitor.observe(&calm, clock.now()), Pressure::Yielding);
        // The machine has to stay calm for the quiet period.
        clock.advance(Duration::from_secs(3));
        assert_eq!(monitor.observe(&calm, clock.now()), Pressure::Calm);
    }

    /// The band between the two thresholds changes nothing: that is the
    /// hysteresis that keeps the scanner from flapping.
    #[test]
    fn a_value_inside_the_band_changes_nothing() {
        let mut monitor = PressureMonitor::new(PressureThresholds::default());
        let clock = ManualClock::new();
        let busy = ResourceSignals {
            cpu_percent: 95,
            ..ResourceSignals::unknown()
        };
        let inside = ResourceSignals {
            cpu_percent: 75,
            ..ResourceSignals::unknown()
        };
        assert_eq!(monitor.observe(&busy, clock.now()), Pressure::Yielding);
        // 75 % is below the yielding threshold but above the resuming one.
        assert_eq!(monitor.observe(&inside, clock.now()), Pressure::Yielding);
        assert_eq!(monitor.state(), Pressure::Yielding);
    }

    /// A band that is upside down is refused.
    #[test]
    fn an_upside_down_band_is_refused() {
        assert!(!Threshold::new(10, 20).is_valid());
        assert!(Threshold::new(20, 10).is_valid());
        assert!(PressureThresholds::default().is_valid());
    }

    /// The wait grows with the number of refusals and never passes the bound.
    #[test]
    fn the_wait_grows_and_stays_inside_its_bound() {
        let policy = BackoffPolicy {
            base: Duration::from_millis(50),
            max: Duration::from_millis(400),
        };
        assert_eq!(policy.delay(0), Duration::ZERO);
        assert_eq!(policy.delay(1), Duration::from_millis(50));
        assert_eq!(policy.delay(2), Duration::from_millis(100));
        assert_eq!(policy.delay(3), Duration::from_millis(200));
        assert_eq!(policy.delay(4), Duration::from_millis(400));
        assert_eq!(policy.delay(9), Duration::from_millis(400));
    }

    /// A probe that knows nothing says that nothing is wrong.
    #[test]
    fn a_probe_that_knows_nothing_says_so() {
        assert_eq!(NoProbe.signals(), ResourceSignals::unknown());
    }

    /// An energy or thermally constrained machine yields background work (RES-008).
    #[test]
    fn an_energy_constrained_machine_yields_background_work() {
        let mut monitor = PressureMonitor::new(PressureThresholds::default());
        let clock = ManualClock::new();
        let signals = ResourceSignals {
            energy_constrained: true,
            ..ResourceSignals::unknown()
        };
        assert_eq!(monitor.observe(&signals, clock.now()), Pressure::Yielding);
    }
}
