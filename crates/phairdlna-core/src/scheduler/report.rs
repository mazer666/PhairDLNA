//! The numbers the scheduler keeps, and the report it hands out.
//!
//! What: the counters of the scheduler and the report that describes them.
//! Why: `RES-006` asks that the resource metrics are opt-in, local and free
//!      of private data. A report that names a file of the user or the address
//!      of a client is a leak, and a report that nobody can ask for is not
//!      observable.
//! Example: `Scheduler::report` answers what the scheduler did, and the
//!       answer carries no path and no address.
//!
//! **What is never in a report.** The name of a client, the path of a file,
//! the address of a machine. The report says how much work ran and how much
//! was refused; it says nothing about *what* the work was.

use super::pressure::ResourceSignals;
use super::priority::Priority;

/// The counters of one priority class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClassStats {
    /// How many tasks wait in the queue right now.
    pub queued: usize,
    /// How many tasks run right now.
    pub running: usize,
    /// How many tasks have ended since the scheduler was built.
    pub completed: u64,
    /// How many tasks were refused because the queue was full.
    pub rejected: u64,
    /// How many tasks were deferred because the machine was busy.
    pub deferred: u64,
}

/// The counters of every priority class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SchedulerStats {
    /// The counters of the streaming class.
    pub streaming: ClassStats,
    /// The counters of the control plane class.
    pub control_plane: ClassStats,
    /// The counters of the metadata class.
    pub media: ClassStats,
    /// The counters of the background class.
    pub background: ClassStats,
}

impl SchedulerStats {
    /// The counters of one class.
    pub fn of(&self, class: Priority) -> &ClassStats {
        match class {
            Priority::Streaming => &self.streaming,
            Priority::ControlPlane => &self.control_plane,
            Priority::Media => &self.media,
            Priority::Background => &self.background,
        }
    }
}

/// The report of one class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassReport {
    /// The name of the class.
    pub class: &'static str,
    /// The counters of the class.
    pub stats: ClassStats,
    /// How many tasks of the class may run at the same time.
    pub concurrent_limit: usize,
    /// How many tasks of the class may wait in the queue.
    pub queue_limit: usize,
}

/// The report of the resources of the server.
///
/// The report is asked for and never pushed anywhere: it leaves the machine
/// only when the user asks for it (`RES-006`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceReport {
    /// How many workers the scheduler has.
    pub workers: usize,
    /// The report of every class.
    pub classes: Vec<ClassReport>,
    /// Whether the machine is calm or busy: `calm` or `yielding`.
    pub pressure: &'static str,
    /// The signals of the last reading.
    pub signals: ResourceSignals,
}

impl ResourceReport {
    /// The report of one class, when there is one.
    pub fn class(&self, class: Priority) -> Option<&ClassReport> {
        self.classes
            .iter()
            .find(|entry| entry.class == class.as_str())
    }

    /// A sentence that describes the report.
    ///
    /// The sentence is the plain-language explanation the user sees
    /// (`UX-004`): it names the numbers and never the files.
    pub fn summary(&self) -> String {
        format!(
            "{} workers, machine {}{}",
            self.workers,
            self.pressure,
            self.classes
                .iter()
                .map(|entry| {
                    format!(
                        ", {}: {} queued, {} running, {} done, {} refused",
                        entry.class,
                        entry.stats.queued,
                        entry.stats.running,
                        entry.stats.completed,
                        entry.stats.rejected
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ClassReport, ClassStats, ResourceReport, SchedulerStats};
    use crate::scheduler::{Priority, ResourceSignals};

    /// The counters of a class are found by the name of the class.
    #[test]
    fn the_counters_of_a_class_are_found() {
        let stats = SchedulerStats {
            streaming: ClassStats {
                completed: 3,
                ..ClassStats::default()
            },
            ..SchedulerStats::default()
        };
        assert_eq!(stats.of(Priority::Streaming).completed, 3);
        assert_eq!(stats.of(Priority::Background).completed, 0);
    }

    /// The report describes itself without naming a file or an address.
    #[test]
    fn the_report_describes_itself() {
        let report = ResourceReport {
            workers: 4,
            classes: vec![ClassReport {
                class: "streaming",
                stats: ClassStats {
                    completed: 2,
                    ..ClassStats::default()
                },
                concurrent_limit: 8,
                queue_limit: 16,
            }],
            pressure: "calm",
            signals: ResourceSignals::unknown(),
        };
        assert_eq!(
            report
                .class(Priority::Streaming)
                .map(|entry| entry.stats.completed),
            Some(2)
        );
        assert!(report.class(Priority::Media).is_none());
        assert!(report.summary().contains("streaming"));
    }
}
