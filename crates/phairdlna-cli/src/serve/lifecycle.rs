//! The lifecycle controller and recovery event recorder of the daemon.
//!
//! What: tracks the operational lifecycle state of the server and records transitions
//!       into a bounded, redacted timeline file.
//! Why: `LIFE-001` requires exposing distinct states (Starting, Ready, Degraded, etc.)
//!      with reasons, and `LIFE-011` requires a local redacted timeline of recovery events.
//! Example: `lifecycle.transition(LifecycleState::Ready, "listeners bound")` records the
//!          transition and informs diagnostics/status.

use std::sync::{Arc, Mutex};

use phairdlna_core::lifecycle::{is_allowed, LifecycleState, Timeline, Transition};
use phairdlna_core::security::Redactor;

use super::log::Logger;

/// The active lifecycle controller of the running server.
#[derive(Debug, Clone)]
pub struct LifecycleController {
    current: Arc<Mutex<Transition>>,
    last_recovery: Arc<Mutex<Option<String>>>,
    timeline: Arc<Timeline>,
    logger: Logger,
}

impl LifecycleController {
    /// Builds a new lifecycle controller starting in `Starting`.
    pub fn new(redactor: Arc<Redactor>, logger: Logger) -> Self {
        let timeline_path = Timeline::default_path();
        let timeline = Arc::new(Timeline::new(timeline_path, redactor));
        let initial = Transition::new(LifecycleState::Starting, "the server is starting");
        let _ = timeline.append(&initial);
        Self {
            current: Arc::new(Mutex::new(initial)),
            last_recovery: Arc::new(Mutex::new(None)),
            timeline,
            logger,
        }
    }

    /// The current state of the server.
    pub fn state(&self) -> LifecycleState {
        self.current_transition().state
    }

    /// The current transition details.
    pub fn current_transition(&self) -> Transition {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Records a new transition if permitted by the lifecycle rules.
    pub fn transition(&self, next: LifecycleState, reason: impl Into<String>) {
        let reason_str = reason.into();
        let mut guard = self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if guard.state == next {
            return;
        }

        if !is_allowed(guard.state, next) {
            self.logger.warn(
                "disallowed lifecycle transition ignored",
                &[
                    ("from", guard.state.as_str()),
                    ("to", next.as_str()),
                    ("reason", &reason_str),
                ],
            );
            return;
        }

        let transition = Transition::new(next, reason_str.clone());
        let _ = self.timeline.append(&transition);
        if next == LifecycleState::Recovering || guard.state == LifecycleState::Recovering {
            let mut rec = self
                .last_recovery
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *rec = Some(format!("{}: {reason_str}", next.as_str()));
        }
        *guard = transition;

        self.logger.info(
            "lifecycle state changed",
            &[("state", next.as_str()), ("reason", &reason_str)],
        );
    }

    /// The lines of the timeline for diagnostics export.
    pub fn read_timeline(&self) -> std::io::Result<String> {
        self.timeline.read()
    }

    /// The last observed recovery event, if any (LIFE-009).
    pub fn last_recovery(&self) -> Option<String> {
        self.last_recovery
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Records an explicit recovery event note (LIFE-009, LIFE-011).
    pub fn record_recovery(&self, event: &str) {
        let _ = self.timeline.append_note(event);
        let mut rec = self
            .last_recovery
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *rec = Some(event.to_owned());
    }
}
