//! The shared state of the workers.
//!
//! What: the queues of the classes, the counters, and the rules that decide
//!       which task a free worker takes.
//! Why: the arbitration of the scheduler is the one rule that decides what
//!      the server does first. It lives in one place, so that it can be read
//!      and proved as one piece (`RES-001`).
//! Example: `Inner::select_task` walks the classes from the most important to
//!       the least important and takes the first task that may run.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use super::clock::Clock;
use super::pool::{Overload, SchedulerOptions};
use super::pressure::{Pressure, PressureMonitor, ResourceProbe};
use super::priority::Priority;
use super::report::{ClassReport, ClassStats, ResourceReport, SchedulerStats};

/// How often a worker looks at the machine when it has nothing to do.
///
/// The worker cannot wait forever: the signals of the machine change while it
/// sleeps, and the work must start again as soon as the machine is calm.
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The state the workers share.
pub(super) struct Inner {
    /// The bounds, the thresholds and the backoff of the scheduler.
    pub(super) options: SchedulerOptions,
    /// The clock of the scheduler.
    pub(super) clock: Arc<dyn Clock>,
    /// The source of the hints about the machine.
    pub(super) probe: Mutex<Option<Arc<dyn ResourceProbe>>>,
    /// The queues and the counters.
    state: Mutex<State>,
    /// The wake-up of the workers.
    ready: Condvar,
    /// The state of the pressure.
    monitor: Mutex<PressureMonitor>,
    /// Whether the scheduler is stopping.
    closed: AtomicBool,
}

/// The queues, the counters and the places of the classes.
pub(super) struct State {
    queues: Vec<VecDeque<Task>>,
    running: Vec<usize>,
    completed: Vec<u64>,
    rejected: Vec<u64>,
    deferred: Vec<u64>,
    deferrals: u32,
}

/// A task that waits in a queue.
pub(super) struct Task {
    /// What the task does.
    pub(super) work: Box<dyn FnOnce() + Send + 'static>,
}

impl Inner {
    /// Builds the shared state of the workers.
    pub(super) fn new(
        options: SchedulerOptions,
        clock: Arc<dyn Clock>,
        probe: Option<Arc<dyn ResourceProbe>>,
    ) -> Self {
        let monitor = PressureMonitor::new(options.thresholds);
        Self {
            state: Mutex::new(State::new()),
            ready: Condvar::new(),
            monitor: Mutex::new(monitor),
            probe: Mutex::new(probe),
            closed: AtomicBool::new(false),
            options,
            clock,
        }
    }

    /// Puts a task in the queue of its class.
    pub(super) fn submit(
        &self,
        class: Priority,
        work: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), Overload> {
        if self.is_closed() {
            return Err(Overload::Closed);
        }
        let mut state = self.lock_state();
        let limits = self.options.limits.for_class(class);
        let index = class.index();
        if state.queues[index].len() >= limits.queue {
            state.rejected[index] += 1;
            return Err(Overload::QueueFull {
                class: class.as_str(),
                capacity: limits.queue,
            });
        }
        if class.is_sheddable() && self.pressure() == Pressure::Yielding {
            state.deferrals += 1;
            state.deferred[index] += 1;
            let delay = self.options.backoff.delay(state.deferrals);
            return Err(Overload::Busy {
                class: class.as_str(),
                millis: delay.as_millis() as u64,
            });
        }
        state.deferrals = 0;
        state.queues[index].push_back(Task { work });
        drop(state);
        self.ready.notify_one();
        Ok(())
    }

    /// The counters of the scheduler.
    pub(super) fn stats(&self) -> SchedulerStats {
        let state = self.lock_state();
        SchedulerStats {
            streaming: state.stats_of(Priority::Streaming),
            control_plane: state.stats_of(Priority::ControlPlane),
            media: state.stats_of(Priority::Media),
            background: state.stats_of(Priority::Background),
        }
    }

    /// The report of the resources.
    pub(super) fn report(&self) -> ResourceReport {
        let state = self.lock_state();
        let classes = Priority::ALL
            .iter()
            .map(|class| {
                let limits = self.options.limits.for_class(*class);
                ClassReport {
                    class: class.as_str(),
                    stats: state.stats_of(*class),
                    concurrent_limit: limits.concurrent,
                    queue_limit: limits.queue,
                }
            })
            .collect();
        let (pressure, signals) = self.monitor_state();
        ResourceReport {
            workers: self.options.limits.workers,
            classes,
            pressure: pressure.as_str(),
            signals,
        }
    }

    /// Whether the machine is calm or busy.
    pub(super) fn pressure(&self) -> Pressure {
        self.monitor_state().0
    }

    /// The state of the machine and the signals of the last reading.
    fn monitor_state(&self) -> (Pressure, super::pressure::ResourceSignals) {
        let monitor = self
            .monitor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (monitor.state(), *monitor.signals())
    }

    /// Whether nothing runs and nothing waits.
    pub(super) fn idle(&self) -> bool {
        let state = self.lock_state();
        state.queues.iter().all(|queue| queue.is_empty())
            && state.running.iter().all(|running| *running == 0)
    }

    /// Takes the next task, waiting for one if there is none.
    pub(super) fn take_task(&self) -> Option<(Priority, Task)> {
        let mut state = self.lock_state();
        loop {
            if self.is_closed() {
                return None;
            }
            self.refresh_pressure();
            if let Some((class, task)) = self.select_task(&mut state) {
                return Some((class, task));
            }
            let (guard, _timeout) = self
                .ready
                .wait_timeout(state, POLL_INTERVAL)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = guard;
        }
    }

    /// Reads the machine and updates the state of the pressure.
    fn refresh_pressure(&self) {
        let probe = {
            let current = self
                .probe
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            current.clone()
        };
        let Some(probe) = probe else {
            return;
        };
        let signals = probe.signals();
        let now = self.clock.now();
        let mut monitor = self
            .monitor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        monitor.observe(&signals, now);
    }

    /// The task a free worker should take.
    fn select_task(&self, state: &mut State) -> Option<(Priority, Task)> {
        for class in Priority::ALL {
            let index = class.index();
            if state.running[index] >= self.options.limits.for_class(class).concurrent {
                continue;
            }
            if class.is_sheddable() && self.pressure() == Pressure::Yielding {
                continue;
            }
            if let Some(task) = state.queues[index].pop_front() {
                state.running[index] += 1;
                return Some((class, task));
            }
        }
        None
    }

    /// Notes that a task of a class has ended.
    pub(super) fn finish_task(&self, class: Priority) {
        let mut state = self.lock_state();
        let index = class.index();
        state.running[index] = state.running[index].saturating_sub(1);
        state.completed[index] += 1;
        drop(state);
        self.ready.notify_all();
    }

    /// Stops the scheduler.
    pub(super) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.ready.notify_all();
    }

    /// Whether the scheduler is stopping.
    pub(super) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// The state of the queues, for a change.
    fn lock_state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl State {
    /// The state of a scheduler with empty queues.
    pub(super) fn new() -> Self {
        let count = Priority::ALL.len();
        Self {
            queues: (0..count).map(|_| VecDeque::new()).collect(),
            running: vec![0; count],
            completed: vec![0; count],
            rejected: vec![0; count],
            deferred: vec![0; count],
            deferrals: 0,
        }
    }

    /// The counters of one class.
    pub(super) fn stats_of(&self, class: Priority) -> ClassStats {
        let index = class.index();
        ClassStats {
            queued: self.queues[index].len(),
            running: self.running[index],
            completed: self.completed[index],
            rejected: self.rejected[index],
            deferred: self.deferred[index],
        }
    }
}
