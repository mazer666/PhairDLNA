//! The bounds of the scheduler.
//!
//! What: how many workers the scheduler has, and how many tasks of each class
//!       may run at the same time and wait in the queue.
//! Why: `RES-002` asks for a documented bound everywhere, and `RES-007` asks
//!      for a meaningful explanation when a bound is reached. A bound that is
//!      not written down is a bound nobody can rely on.
//! Example: `SchedulerLimits::default()` is the scheduler of an ordinary
//!       machine; a smaller one is the scheduler of a small one.
//!
//! **Every number here is a bound, not a hint.** The queue of a class that is
//! full refuses the next task of that class with an error that names the
//! bound; the class that runs more tasks than its bound does not start another
//! one until one of them ends.

use super::priority::Priority;

/// The bounds of one priority class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassLimits {
    /// How many tasks of the class may run at the same time.
    pub concurrent: usize,
    /// How many tasks of the class may wait in the queue.
    pub queue: usize,
}

impl ClassLimits {
    /// Builds the bounds of a class.
    pub fn new(concurrent: usize, queue: usize) -> Self {
        Self { concurrent, queue }
    }
}

/// The bounds of the whole scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerLimits {
    /// How many worker threads the scheduler has.
    ///
    /// The workers are shared by the classes: a class never uses more workers
    /// than it has tasks, and the most important class that is ready gets the
    /// free worker.
    pub workers: usize,
    /// The bounds of the streaming class.
    pub streaming: ClassLimits,
    /// The bounds of the control plane class.
    pub control_plane: ClassLimits,
    /// The bounds of the metadata class.
    pub media: ClassLimits,
    /// The bounds of the background class.
    pub background: ClassLimits,
}

impl SchedulerLimits {
    /// The bounds of one class.
    pub fn for_class(&self, class: Priority) -> ClassLimits {
        match class {
            Priority::Streaming => self.streaming,
            Priority::ControlPlane => self.control_plane,
            Priority::Media => self.media,
            Priority::Background => self.background,
        }
    }

    /// Checks that the bounds can be used.
    ///
    /// A bound of zero would mean a class that can never run; the scheduler
    /// refuses such a configuration instead of refusing every task later.
    pub fn validate(&self) -> Result<(), LimitsError> {
        if self.workers == 0 {
            return Err(LimitsError::NoWorker);
        }
        for class in Priority::ALL {
            let limits = self.for_class(class);
            if limits.concurrent == 0 {
                return Err(LimitsError::NoConcurrentTask {
                    class: class.as_str(),
                });
            }
            if limits.queue == 0 {
                return Err(LimitsError::NoQueue {
                    class: class.as_str(),
                });
            }
        }
        Ok(())
    }
}

impl Default for SchedulerLimits {
    /// The bounds of an ordinary machine.
    ///
    /// The numbers are small on purpose: a server that sits under a television
    /// must answer a client quickly, and the work it does besides that is
    /// small (`PERF-001`, `PERF-013`).
    fn default() -> Self {
        Self {
            workers: 8,
            streaming: ClassLimits::new(16, 64),
            control_plane: ClassLimits::new(8, 64),
            media: ClassLimits::new(4, 32),
            background: ClassLimits::new(2, 16),
        }
    }
}

/// A configuration of bounds that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LimitsError {
    /// The scheduler would have no worker at all.
    #[error("the scheduler needs at least one worker")]
    NoWorker,
    /// A class could never run a task.
    #[error("the {class} class may not run any task")]
    NoConcurrentTask {
        /// The name of the class.
        class: &'static str,
    },
    /// A class could never keep a task waiting.
    #[error("the {class} class may not keep any task waiting")]
    NoQueue {
        /// The name of the class.
        class: &'static str,
    },
    /// A worker of the scheduler could not be started.
    #[error("a worker of the scheduler could not be started: {reason}")]
    WorkerFailed {
        /// What the operating system said.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::{ClassLimits, LimitsError, SchedulerLimits};
    use crate::scheduler::Priority;

    /// The bounds of the default scheduler are usable.
    #[test]
    fn the_default_bounds_are_usable() {
        assert_eq!(SchedulerLimits::default().validate(), Ok(()));
    }

    /// A scheduler without a worker is refused.
    #[test]
    fn a_scheduler_without_a_worker_is_refused() {
        let limits = SchedulerLimits {
            workers: 0,
            ..SchedulerLimits::default()
        };
        assert_eq!(limits.validate(), Err(LimitsError::NoWorker));
    }

    /// A class that may never run a task is refused.
    #[test]
    fn a_class_without_a_place_to_run_is_refused() {
        let limits = SchedulerLimits {
            background: ClassLimits::new(0, 16),
            ..SchedulerLimits::default()
        };
        assert_eq!(
            limits.validate(),
            Err(LimitsError::NoConcurrentTask {
                class: "background"
            })
        );
    }

    /// A class that may never wait is refused.
    #[test]
    fn a_class_without_a_queue_is_refused() {
        let limits = SchedulerLimits {
            media: ClassLimits::new(4, 0),
            ..SchedulerLimits::default()
        };
        assert_eq!(
            limits.validate(),
            Err(LimitsError::NoQueue { class: "media" })
        );
    }

    /// Every class has bounds of its own.
    #[test]
    fn every_class_has_bounds_of_its_own() {
        let limits = SchedulerLimits::default();
        assert_eq!(limits.for_class(Priority::Streaming), limits.streaming);
        assert_eq!(
            limits.for_class(Priority::ControlPlane),
            limits.control_plane
        );
        assert_eq!(limits.for_class(Priority::Media), limits.media);
        assert_eq!(limits.for_class(Priority::Background), limits.background);
    }
}
