//! The hysteresis of the background work.
//!
//! What: the tests of the rule that the indexing yields while the machine is
//!       busy and starts again when the machine has been calm for a while.
//! Why: `RES-003` asks for hysteresis, and a scanner that flaps is worse for
//!      a playback than a scanner that never starts. The clock of these tests
//!      is a synthetic one, so that the rule can be proved without waiting
//!      for a wall clock (`AC-08`).
//! Example: the test makes the machine busy, watches the scan yield, calms
//!       the machine down and moves the clock, and watches the scan start.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairdlna_core::scheduler::{
    ManualClock, Pressure, PressureThresholds, Priority, ResourceProbe, ResourceSignals, Scheduler,
    SchedulerLimits, SchedulerOptions,
};

/// How long a test waits for the scheduler to do something.
const BUDGET: Duration = Duration::from_secs(5);

/// A probe whose signals the test decides.
struct TestProbe {
    signals: Mutex<ResourceSignals>,
}

impl TestProbe {
    /// A probe that says the machine is calm.
    fn calm() -> Arc<Self> {
        Arc::new(Self {
            signals: Mutex::new(ResourceSignals::unknown()),
        })
    }

    /// Makes the machine as busy as the test wants it to be.
    fn set(&self, signals: ResourceSignals) {
        let mut current = self
            .signals
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = signals;
    }
}

impl ResourceProbe for TestProbe {
    /// The signals the test wrote down.
    fn signals(&self) -> ResourceSignals {
        *self
            .signals
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// The scheduler of the tests: one worker and a synthetic clock.
fn scheduler_with(clock: Arc<ManualClock>, probe: Arc<TestProbe>) -> Scheduler {
    Scheduler::with_clock(
        SchedulerOptions {
            limits: SchedulerLimits {
                workers: 1,
                ..SchedulerLimits::default()
            },
            thresholds: PressureThresholds::default(),
            ..SchedulerOptions::default()
        },
        clock,
        Some(probe),
    )
    .expect("the bounds of the test are usable")
}

/// Waits until the answer is true, or until the budget is over.
fn wait_until(answer: impl Fn() -> bool) -> bool {
    let deadline = std::time::Instant::now() + BUDGET;
    while std::time::Instant::now() < deadline {
        if answer() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    answer()
}

/// The scan yields while the machine is busy.
#[test]
fn the_scan_yields_while_the_machine_is_busy() {
    let clock = Arc::new(ManualClock::new());
    let probe = TestProbe::calm();
    let scheduler = scheduler_with(Arc::clone(&clock), Arc::clone(&probe));
    let ran = Arc::new(AtomicBool::new(false));

    // The machine is busy before the scan is submitted.
    probe.set(ResourceSignals {
        cpu_percent: 95,
        ..ResourceSignals::unknown()
    });
    let flag = Arc::clone(&ran);
    scheduler
        .submit(Priority::Background, move || {
            flag.store(true, Ordering::SeqCst);
        })
        .expect("the scan fits in the queue");

    assert!(
        wait_until(|| scheduler.pressure() == Pressure::Yielding),
        "the machine is busy and the background work yields"
    );
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !ran.load(Ordering::SeqCst),
        "the scan does not run while the machine is busy"
    );
}

/// The scan starts again when the machine has been calm for the quiet period.
#[test]
fn the_scan_starts_again_when_the_machine_has_been_calm() {
    let clock = Arc::new(ManualClock::new());
    let probe = TestProbe::calm();
    let scheduler = scheduler_with(Arc::clone(&clock), Arc::clone(&probe));
    let ran = Arc::new(AtomicBool::new(false));

    probe.set(ResourceSignals {
        cpu_percent: 95,
        ..ResourceSignals::unknown()
    });
    let flag = Arc::clone(&ran);
    scheduler
        .submit(Priority::Background, move || {
            flag.store(true, Ordering::SeqCst);
        })
        .expect("the scan fits in the queue");
    assert!(wait_until(|| scheduler.pressure() == Pressure::Yielding));

    // The machine calms down, but the quiet period has not passed yet.
    probe.set(ResourceSignals::unknown());
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !ran.load(Ordering::SeqCst),
        "the scan waits for the quiet period"
    );

    // The clock moves past the quiet period, and the scan starts.
    clock.advance(Duration::from_secs(5));
    assert!(
        wait_until(|| ran.load(Ordering::SeqCst)),
        "the scan runs once the machine has been calm long enough"
    );
}

/// A value inside the band of the threshold changes nothing.
#[test]
fn a_value_inside_the_band_does_not_start_the_scan() {
    let clock = Arc::new(ManualClock::new());
    let probe = TestProbe::calm();
    let scheduler = scheduler_with(Arc::clone(&clock), Arc::clone(&probe));
    let ran = Arc::new(AtomicBool::new(false));

    probe.set(ResourceSignals {
        cpu_percent: 95,
        ..ResourceSignals::unknown()
    });
    let flag = Arc::clone(&ran);
    scheduler
        .submit(Priority::Background, move || {
            flag.store(true, Ordering::SeqCst);
        })
        .expect("the scan fits in the queue");
    assert!(wait_until(|| scheduler.pressure() == Pressure::Yielding));

    // 75 % is below the threshold of yielding and above the threshold of
    // resuming: the machine is calmer, but it is not calm.
    probe.set(ResourceSignals {
        cpu_percent: 75,
        ..ResourceSignals::unknown()
    });
    clock.advance(Duration::from_secs(10));
    std::thread::sleep(Duration::from_millis(200));

    assert_eq!(
        scheduler.pressure(),
        Pressure::Yielding,
        "the state does not change inside the band of the threshold"
    );
    assert!(
        !ran.load(Ordering::SeqCst),
        "the scan does not start inside the band of the threshold"
    );
}

/// The state changes once and does not oscillate while the machine is busy.
#[test]
fn the_state_does_not_oscillate_while_the_machine_is_busy() {
    let clock = Arc::new(ManualClock::new());
    let probe = TestProbe::calm();
    let scheduler = scheduler_with(Arc::clone(&clock), Arc::clone(&probe));

    probe.set(ResourceSignals {
        cpu_percent: 95,
        ..ResourceSignals::unknown()
    });
    assert!(wait_until(|| scheduler.pressure() == Pressure::Yielding));

    // The machine stays busy, and the clock moves: the state stays the same.
    for _ in 0..10 {
        clock.advance(Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(
            scheduler.pressure(),
            Pressure::Yielding,
            "the state does not change while the machine is busy"
        );
    }
}
