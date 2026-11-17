//! The adaptive resource management of the server.
//!
//! What: the one scheduler that arbitrates between the work of the server, the
//!       bounds of its pools and queues, and the rule that makes the
//!       background work yield when the machine is busy.
//! Why: the server runs on a desk, next to the television of the living room.
//!      A film that plays while the library is being indexed must not stutter,
//!      and the indexing must not stop the film (`RES-001`, `RES-003`). The
//!      policy of that trade-off belongs in the core and not in a user
//!      interface, so that the headless server and the graphical one behave
//!      the same way (`RES-005`).
//! Example: `Scheduler::submit` puts the scan of the library in the queue of
//!       the background class, and the worker that becomes free takes the
//!       answer of a client first.
//!
//! **The classes.** Streaming is the most important: a client waits for it.
//! The control plane comes next: a television that waits too long for a
//! browse gives up. The metadata and the thumbnails come after that. The scan
//! of the library is the least important, and it is the only class that yields
//! when the machine is busy.
//!
//! **The bounds.** Every queue has a bound, every class has a bound on the
//! tasks that run at the same time, and a task that does not fit is refused
//! with an error that names the bound (`RES-002`, `RES-007`).
//!
//! **The privacy.** The counters of the scheduler are asked for and never sent
//! anywhere. A report names no file, no client and no address (`RES-006`).

pub mod clock;
mod inner;
pub mod limits;
pub mod pool;
pub mod pressure;
pub mod priority;
pub mod report;

pub use clock::{Clock, ManualClock, SystemClock};
pub use limits::{ClassLimits, LimitsError, SchedulerLimits};
pub use pool::{Overload, Scheduler, SchedulerOptions};
pub use pressure::{
    BackoffPolicy, NoProbe, Pressure, PressureMonitor, PressureThresholds, ResourceProbe,
    ResourceSignals, Threshold,
};
pub use priority::Priority;
pub use report::{ClassReport, ClassStats, ResourceReport, SchedulerStats};

#[cfg(test)]
mod tests {
    use super::{
        ClassLimits, Overload, Pressure, Priority, Scheduler, SchedulerLimits, SchedulerOptions,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// A scheduler that is built and stopped again releases its workers.
    #[test]
    fn a_scheduler_releases_its_workers() {
        let scheduler = Scheduler::new(SchedulerOptions::default())
            .expect("the bounds of the default scheduler are usable");
        scheduler.shutdown();
        assert!(scheduler.idle());
    }

    /// A task that is submitted runs exactly once.
    #[test]
    fn a_submitted_task_runs() {
        let scheduler = Scheduler::new(SchedulerOptions::default())
            .expect("the bounds of the default scheduler are usable");
        let ran = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&ran);
        scheduler
            .submit(Priority::Background, move || {
                flag.store(true, Ordering::SeqCst);
            })
            .expect("the task fits in the queue");
        wait_until(|| ran.load(Ordering::SeqCst));
        assert!(ran.load(Ordering::SeqCst));
        assert_eq!(scheduler.stats().background.completed, 1);
    }

    /// A scheduler that is stopping takes no new task.
    #[test]
    fn a_stopping_scheduler_takes_no_new_task() {
        let scheduler = Scheduler::new(SchedulerOptions::default())
            .expect("the bounds of the default scheduler are usable");
        scheduler.shutdown();
        let outcome = scheduler.submit(Priority::Background, || {});
        assert_eq!(outcome, Err(Overload::Closed));
    }

    /// A machine without a probe is calm, and the background work runs.
    #[test]
    fn a_machine_without_a_probe_is_calm() {
        let scheduler = Scheduler::new(SchedulerOptions::default())
            .expect("the bounds of the default scheduler are usable");
        assert_eq!(scheduler.pressure(), Pressure::Calm);
    }

    /// The bounds of the classes are the ones of the configuration.
    #[test]
    fn the_bounds_of_the_classes_are_kept() {
        let options = SchedulerOptions {
            limits: SchedulerLimits {
                workers: 2,
                streaming: ClassLimits::new(4, 8),
                ..SchedulerLimits::default()
            },
            ..SchedulerOptions::default()
        };
        let scheduler = Scheduler::new(options).expect("the bounds are usable");
        let report = scheduler.report();
        let streaming = report
            .class(Priority::Streaming)
            .expect("the streaming class is in the report");
        assert_eq!(streaming.concurrent_limit, 4);
        assert_eq!(streaming.queue_limit, 8);
    }

    /// Waits until the answer is true, or until the budget is over.
    fn wait_until(answer: impl Fn() -> bool) -> bool {
        for _ in 0..200 {
            if answer() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        false
    }
}
