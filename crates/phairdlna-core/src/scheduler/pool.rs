//! The one scheduler of the server.
//!
//! What: the pool of workers that runs the work of the server, the priority
//!       that decides which work runs next, and the refusal that names the
//!       bound that was reached.
//! Why: `RES-001` asks for **one** scheduler with explicit priority classes.
//!      A second arbitration mechanism would make the priority model
//!      unverifiable, and an unbounded queue would be a denial of service
//!      waiting to happen (`RES-002`).
//! Example: `Scheduler::submit` puts a task in the queue of its class, and
//!       the worker that becomes free takes the task of the most important
//!       class that is ready.
//!
//! **The rules of the arbitration.**
//!
//! 1. A worker takes the task of the most important class that has a task
//!    waiting and that may run one. The order is streaming, control plane,
//!    metadata, background.
//! 2. A class never runs more tasks at the same time than its bound says.
//! 3. The background work yields while the machine is busy, and it starts
//!    again when the machine has been calm for the quiet period (`RES-003`).
//! 4. A queue that is full refuses the next task with an error that names the
//!    class and the bound (`RES-007`).
//! 5. The shutdown drops the tasks that have not started and lets the task
//!    that runs finish, so that no worker and no file handle is left behind
//!    (`RES-009`).

use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use super::clock::{Clock, SystemClock};
use super::inner::{Inner, Task};
use super::limits::{LimitsError, SchedulerLimits};
use super::pressure::{BackoffPolicy, Pressure, PressureThresholds, ResourceProbe};
use super::priority::Priority;
use super::report::{ResourceReport, SchedulerStats};

/// How the scheduler is configured.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerOptions {
    /// The bounds of the workers and of the queues.
    pub limits: SchedulerLimits,
    /// The thresholds at which the background work yields.
    pub thresholds: PressureThresholds,
    /// How long a refused task waits before it tries again.
    pub backoff: BackoffPolicy,
}

impl Default for SchedulerOptions {
    /// The configuration of an ordinary machine.
    fn default() -> Self {
        Self {
            limits: SchedulerLimits::default(),
            thresholds: PressureThresholds::default(),
            backoff: BackoffPolicy::default(),
        }
    }
}

/// A task the scheduler refused.
///
/// Every variant names the bound that was reached, so that the caller can
/// tell the user what happened instead of failing in silence (`RES-007`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Overload {
    /// The queue of the class was full.
    #[error("the {class} queue of the scheduler is full: it holds {capacity} tasks")]
    QueueFull {
        /// The name of the class.
        class: &'static str,
        /// The bound of the queue.
        capacity: usize,
    },
    /// The machine is busy, so the work has to wait.
    #[error("the machine is busy: the {class} work must wait {millis} ms")]
    Busy {
        /// The name of the class.
        class: &'static str,
        /// How long the work should wait, in milliseconds.
        millis: u64,
    },
    /// The scheduler is shutting down.
    #[error("the scheduler is shutting down and takes no new task")]
    Closed,
}

/// The scheduler of the server.
pub struct Scheduler {
    inner: Arc<Inner>,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

impl Scheduler {
    /// Builds a scheduler and starts its workers.
    ///
    /// The bounds are checked first: a scheduler that could never run a task
    /// is refused here and not on every task later.
    pub fn new(options: SchedulerOptions) -> Result<Self, LimitsError> {
        Self::build(options, Arc::new(SystemClock), None)
    }

    /// Builds a scheduler that reads the machine through a probe.
    ///
    /// The probe supplies hints; the policy stays in the core (`RES-005`).
    pub fn with_probe(
        options: SchedulerOptions,
        probe: Arc<dyn ResourceProbe>,
    ) -> Result<Self, LimitsError> {
        Self::build(options, Arc::new(SystemClock), Some(probe))
    }

    /// Builds a scheduler with a clock of its own.
    ///
    /// The clock is an input: a test moves it by hand and reads the effect on
    /// the hysteresis without waiting for a wall clock.
    pub fn with_clock(
        options: SchedulerOptions,
        clock: Arc<dyn Clock>,
        probe: Option<Arc<dyn ResourceProbe>>,
    ) -> Result<Self, LimitsError> {
        Self::build(options, clock, probe)
    }

    /// Builds the scheduler and starts the workers.
    fn build(
        options: SchedulerOptions,
        clock: Arc<dyn Clock>,
        probe: Option<Arc<dyn ResourceProbe>>,
    ) -> Result<Self, LimitsError> {
        options.limits.validate()?;
        let inner = Arc::new(Inner::new(options, clock, probe));
        let mut workers = Vec::new();
        for number in 0..inner.options.limits.workers {
            let worker_inner = Arc::clone(&inner);
            let handle = std::thread::Builder::new()
                .name(format!("phairdlna-scheduler-{number}"))
                .spawn(move || worker_loop(worker_inner))
                .map_err(|error| LimitsError::WorkerFailed {
                    reason: error.to_string(),
                })?;
            workers.push(handle);
        }
        Ok(Self {
            inner,
            workers: Mutex::new(workers),
        })
    }

    /// Replaces the source of the hints about the machine.
    ///
    /// The source may be given late: a server that builds its scheduler
    /// before it knows itself can hand the source over afterwards.
    pub fn set_probe(&self, probe: Arc<dyn ResourceProbe>) {
        let mut current = self
            .inner
            .probe
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Some(probe);
    }

    /// Puts a task in the queue of its class.
    ///
    /// The task runs as soon as a worker is free and the class may run one.
    /// A task that cannot be accepted is refused with an error that names the
    /// bound (`RES-002`, `RES-007`).
    pub fn submit<F>(&self, class: Priority, work: F) -> Result<(), Overload>
    where
        F: FnOnce() + Send + 'static,
    {
        self.inner.submit(class, Box::new(work))
    }

    /// The counters of the scheduler.
    pub fn stats(&self) -> SchedulerStats {
        self.inner.stats()
    }

    /// The report of the resources.
    ///
    /// The report is asked for; it is never sent anywhere (`RES-006`).
    pub fn report(&self) -> ResourceReport {
        self.inner.report()
    }

    /// Whether the machine is calm or busy.
    pub fn pressure(&self) -> Pressure {
        self.inner.pressure()
    }

    /// Whether nothing runs and nothing waits.
    pub fn idle(&self) -> bool {
        self.inner.idle()
    }

    /// Stops the scheduler and releases its workers.
    ///
    /// The tasks that have not started are dropped, and the task that runs
    /// finishes. The workers are joined, so that no thread and no file handle
    /// is left behind (`RES-009`).
    pub fn shutdown(&self) {
        self.inner.close();
        let mut workers = self
            .workers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for worker in workers.drain(..) {
            let _ = worker.join();
        }
    }
}

impl Drop for Scheduler {
    /// Stops the scheduler when the last owner lets it go.
    ///
    /// The workers are **not** waited for here: a task that never ends would
    /// keep the owner of the scheduler waiting forever, and the owner has
    /// already given the scheduler up. The daemon calls `shutdown` while it
    /// stops, and that call waits for the workers (`RES-009`).
    fn drop(&mut self) {
        self.inner.close();
    }
}

/// The loop of a worker.
fn worker_loop(inner: Arc<Inner>) {
    while let Some((class, task)) = inner.take_task() {
        run_task(task);
        inner.finish_task(class);
    }
}

/// Runs a task and keeps a panic of the task from taking the worker down.
///
/// A task belongs to a caller and may fail in any way. A worker that died
/// would take its place in the pool with it, and the class of the task would
/// keep a place that is never given back.
fn run_task(task: Task) {
    let Task { work } = task;
    let _ = std::panic::catch_unwind(AssertUnwindSafe(work));
}
