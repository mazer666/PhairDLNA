//! The resources of the daemon.
//!
//! What: the scheduler of the daemon, the hints the daemon gives it about
//!       the machine, and the way the work of the daemon is classified.
//! Why: the daemon does several things at the same time — it answers
//!       clients, it plays films, it produces the covers of the library and
//!       it reads the drives. The server answers a client first, always
//!       (`RES-001`), and the scan of the library waits when the machine is
//!       busy (`RES-003`).
//! Example: `Resources::submit_background` puts the reading of the catalogue
//!       in the background class, and `Resources::run` runs the work of a
//!       request and hands back its answer.
//!
//! **The policy is not here.** The thresholds, the bounds and the order of
//! the classes live in the core (`RES-005`); this module only says what the
//! daemon does and what it observes.

use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use phairdlna_core::config::Config;
use phairdlna_core::scheduler::{
    ClassLimits, Overload, PressureThresholds, Priority, ResourceProbe, ResourceReport,
    ResourceSignals, Scheduler, SchedulerLimits, SchedulerOptions,
};

use super::log::Logger;

/// Why a task of the scheduler did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    /// The scheduler refused the task: a bound was reached.
    Refused(Overload),
    /// The task ended without an answer.
    Lost,
}

impl fmt::Display for TaskError {
    /// The sentence that tells what went wrong.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(error) => write!(formatter, "the server is busy: {error}"),
            Self::Lost => write!(formatter, "the task of the server ended without an answer"),
        }
    }
}

impl std::error::Error for TaskError {}

impl From<Overload> for TaskError {
    /// A refusal of the scheduler is the reason of the failure.
    fn from(error: Overload) -> Self {
        Self::Refused(error)
    }
}

/// The resources of the daemon.
#[derive(Clone)]
pub struct Resources {
    scheduler: Arc<Scheduler>,
    streams: Arc<AtomicUsize>,
    latency_ms: Arc<AtomicU64>,
    logger: Logger,
}

impl Resources {
    /// Builds the resources of the daemon.
    ///
    /// The bounds of the scheduler come from the configuration of the user:
    /// the number of the workers of the index is the number of the tasks that
    /// may read the library at the same time (`CFG-008`).
    pub fn new(config: &Config, logger: &Logger) -> Self {
        let index_workers = config.limits.index_workers.max(1);
        let limits = SchedulerLimits {
            workers: index_workers + 2,
            streaming: ClassLimits::new(config.limits.max_concurrent_streams.max(1), 64),
            control_plane: ClassLimits::new(4, 64),
            media: ClassLimits::new(index_workers, 32),
            background: ClassLimits::new(index_workers, 16),
        };
        let options = SchedulerOptions {
            limits,
            thresholds: PressureThresholds::default(),
            ..SchedulerOptions::default()
        };
        let scheduler =
            Arc::new(Scheduler::new(options).expect("the bounds of the configuration are usable"));
        let resources = Self {
            scheduler,
            streams: Arc::new(AtomicUsize::new(0)),
            latency_ms: Arc::new(AtomicU64::new(0)),
            logger: logger.clone(),
        };
        // The daemon is the source of the hints about the machine: it knows
        // how many transfers are running and how long the last one needed.
        resources
            .scheduler
            .set_probe(Arc::new(resources.clone()) as Arc<dyn ResourceProbe>);
        resources
    }

    /// Puts a task in the queue of its class.
    ///
    /// The task runs as soon as a worker is free and the class may run one.
    /// A task that does not fit is refused with an error that names the bound
    /// (`RES-002`, `RES-007`).
    pub fn submit<F>(&self, class: Priority, work: F) -> Result<(), Overload>
    where
        F: FnOnce() + Send + 'static,
    {
        self.scheduler.submit(class, work)
    }

    /// Runs a task and waits for its answer.
    ///
    /// The work runs in the class it was given, so that the answer of a client
    /// comes before the work of the library (`RES-001`).
    pub async fn run<T, F>(&self, class: Priority, work: F) -> Result<T, TaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.submit(class, move || {
            let _ = sender.send(work());
        })?;
        receiver.await.map_err(|_| TaskError::Lost)
    }

    /// Puts the work of the library in the background class.
    pub fn submit_background<F>(&self, work: F) -> Result<(), Overload>
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(Priority::Background, work)
    }

    /// Says that a transfer is running.
    ///
    /// The token ends the transfer when it is dropped, so that a client that
    /// goes away does not keep a place of the server forever.
    pub fn stream_started(&self) -> StreamToken {
        self.streams.fetch_add(1, Ordering::SeqCst);
        StreamToken {
            resources: self.clone(),
        }
    }

    /// Says how long the server needed to prepare the last answer.
    ///
    /// The number is a hint of the machine, not a promise: it is the time the
    /// last transfer needed before its first byte.
    pub fn note_latency(&self, millis: u64) {
        self.latency_ms.store(millis, Ordering::SeqCst);
    }

    /// How many transfers are running.
    pub fn streams(&self) -> usize {
        self.streams.load(Ordering::SeqCst)
    }

    /// The report of the resources.
    ///
    /// The report is asked for and never sent anywhere (`RES-006`).
    pub fn report(&self) -> ResourceReport {
        self.scheduler.report()
    }

    /// Says that the work of the server has to wait, and why.
    pub fn explain(&self, error: &Overload) {
        self.logger.info(
            "the work of the server waits",
            &[("reason", &error.to_string())],
        );
    }

    /// Stops the scheduler and releases its workers.
    pub fn shutdown(&self) {
        self.scheduler.shutdown();
    }
}

impl ResourceProbe for Resources {
    /// The hints the daemon gives about the machine.
    ///
    /// The daemon knows how many transfers are running and how long the last
    /// answer needed. The processor, the memory and the disk are not measured
    /// here: a signal that is not measured stays at zero, and zero never
    /// crosses a threshold.
    fn signals(&self) -> ResourceSignals {
        ResourceSignals {
            active_streams: self.streams(),
            streaming_latency_ms: self.latency_ms.load(Ordering::SeqCst),
            ..ResourceSignals::unknown()
        }
    }
}

/// The token of a transfer that is running.
pub struct StreamToken {
    resources: Resources,
}

impl Drop for StreamToken {
    /// Ends the transfer.
    fn drop(&mut self) {
        self.resources.streams.fetch_sub(1, Ordering::SeqCst);
    }
}
