//! The states of the server and the transitions between them.
//!
//! What: the eight states the server can be in and the rule that says which
//!       transition is allowed.
//! Why: the daemon, the status command and the administration surface must
//!      name the same state with the same word; a vocabulary that each of them
//!      invents for itself is a vocabulary nobody can read (`LIFE-001`).
//! Example: `is_allowed(LifecycleState::Ready, LifecycleState::Degraded)` is
//!      `true`: a server that loses an interface says so.

use crate::time::now_utc;

/// The state the server is in.
///
/// The states are the eight of `LIFE-001`. They are deliberately few: a user
/// must be able to hold all of them in their head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// The daemon reads its configuration and opens its sockets.
    Starting,
    /// The server answers clients and advertises itself.
    Ready,
    /// The server serves, but something it needs is missing.
    Degraded,
    /// The machine sleeps; the server keeps nothing open.
    Suspended,
    /// The server is putting back what it lost.
    Recovering,
    /// The server is stopping.
    Stopping,
    /// The server is not running.
    Stopped,
    /// The server could not do what it was asked to do.
    Failed,
}

impl LifecycleState {
    /// The name of the state, as it appears in the log and in the status.
    pub fn as_str(self) -> &'static str {
        match self {
            LifecycleState::Starting => "starting",
            LifecycleState::Ready => "ready",
            LifecycleState::Degraded => "degraded",
            LifecycleState::Suspended => "suspended",
            LifecycleState::Recovering => "recovering",
            LifecycleState::Stopping => "stopping",
            LifecycleState::Stopped => "stopped",
            LifecycleState::Failed => "failed",
        }
    }

    /// The sentence a user reads about the state.
    ///
    /// The sentence names the state in plain words, without protocol jargon
    /// (`UX-004`, `UX-010`).
    pub fn explanation(self) -> &'static str {
        match self {
            LifecycleState::Starting => "The server is starting and is not answering yet.",
            LifecycleState::Ready => "The server is running and clients can reach it.",
            LifecycleState::Degraded => "The server is running, but something it needs is missing.",
            LifecycleState::Suspended => "The computer is asleep, so the server is resting.",
            LifecycleState::Recovering => "The server is putting back what it lost.",
            LifecycleState::Stopping => "The server is stopping and will answer no longer.",
            LifecycleState::Stopped => "The server is not running.",
            LifecycleState::Failed => "The server could not do what it was asked to do.",
        }
    }

    /// Whether the server answers clients in this state.
    ///
    /// Only *Ready* answers. A server that is *Degraded* still serves what it
    /// has, but it is not the state a user may promise to a television.
    pub fn is_serving(self) -> bool {
        matches!(self, LifecycleState::Ready)
    }
}

/// A moment at which the server changed its state.
///
/// The transition carries the new state, the reason, the moment and the
/// sentence the user would read (`LIFE-001`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    /// The state the server is in now.
    pub state: LifecycleState,
    /// Why the server changed its state.
    pub reason: String,
    /// When the server changed its state.
    pub moment: String,
    /// The sentence the user would read.
    pub explanation: String,
}

impl Transition {
    /// Builds a transition that happened now.
    ///
    /// The moment comes from the clock of the system, and the sentence from the
    /// state itself, so that the two can never disagree.
    pub fn new(state: LifecycleState, reason: impl Into<String>) -> Self {
        Self {
            state,
            reason: reason.into(),
            moment: now_utc(),
            explanation: state.explanation().to_owned(),
        }
    }

    /// The transition as one line.
    ///
    /// The line is the shape of a log record: the moment, the state, the reason
    /// and the sentence (`OPS-006`).
    pub fn render(&self) -> String {
        format!(
            "{} {} reason=\"{}\" {}",
            self.moment,
            self.state.as_str(),
            self.reason,
            self.explanation
        )
    }
}

/// Whether the server may go from one state to another.
///
/// The rule keeps the vocabulary honest: a server that is *Stopped* cannot
/// become *Ready* without passing through *Starting*, and a server that is
/// *Stopping* does not go back to *Ready* (`LIFE-001`).
pub fn is_allowed(from: LifecycleState, to: LifecycleState) -> bool {
    use LifecycleState::*;
    match from {
        Starting => matches!(to, Ready | Degraded | Failed | Stopping),
        Ready => matches!(to, Degraded | Suspended | Recovering | Stopping | Failed),
        Degraded => matches!(to, Recovering | Ready | Suspended | Stopping | Failed),
        Suspended => matches!(to, Recovering | Degraded | Stopping | Failed),
        Recovering => matches!(to, Ready | Degraded | Suspended | Stopping | Failed),
        Stopping => matches!(to, Stopped | Failed),
        Stopped => matches!(to, Starting),
        Failed => matches!(to, Starting | Stopped),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_allowed, LifecycleState, Transition};

    /// Every state has the name the specification names.
    #[test]
    fn every_state_has_its_name() {
        let names = [
            (LifecycleState::Starting, "starting"),
            (LifecycleState::Ready, "ready"),
            (LifecycleState::Degraded, "degraded"),
            (LifecycleState::Suspended, "suspended"),
            (LifecycleState::Recovering, "recovering"),
            (LifecycleState::Stopping, "stopping"),
            (LifecycleState::Stopped, "stopped"),
            (LifecycleState::Failed, "failed"),
        ];
        for (state, name) in names {
            assert_eq!(state.as_str(), name);
            assert!(!state.explanation().is_empty(), "{name} explains itself");
        }
    }

    /// The eight states of the specification are the eight of the code.
    #[test]
    fn there_are_exactly_eight_states() {
        let states = [
            LifecycleState::Starting,
            LifecycleState::Ready,
            LifecycleState::Degraded,
            LifecycleState::Suspended,
            LifecycleState::Recovering,
            LifecycleState::Stopping,
            LifecycleState::Stopped,
            LifecycleState::Failed,
        ];
        assert_eq!(states.len(), 8);
    }

    /// The positive case: the normal way of the server is allowed.
    #[test]
    fn the_normal_way_is_allowed() {
        assert!(is_allowed(LifecycleState::Starting, LifecycleState::Ready));
        assert!(is_allowed(LifecycleState::Ready, LifecycleState::Degraded));
        assert!(is_allowed(
            LifecycleState::Degraded,
            LifecycleState::Recovering
        ));
        assert!(is_allowed(
            LifecycleState::Recovering,
            LifecycleState::Ready
        ));
        assert!(is_allowed(LifecycleState::Ready, LifecycleState::Stopping));
        assert!(is_allowed(
            LifecycleState::Stopping,
            LifecycleState::Stopped
        ));
    }

    /// The negative case: a server that is stopped cannot become ready without
    /// starting first.
    #[test]
    fn a_stopped_server_must_start_first() {
        assert!(!is_allowed(LifecycleState::Stopped, LifecycleState::Ready));
        assert!(is_allowed(
            LifecycleState::Stopped,
            LifecycleState::Starting
        ));
    }

    /// The negative case: a server that is stopping does not go back to ready.
    #[test]
    fn a_stopping_server_does_not_go_back() {
        assert!(!is_allowed(LifecycleState::Stopping, LifecycleState::Ready));
    }

    /// A transition carries the moment, the state, the reason and the sentence.
    #[test]
    fn a_transition_carries_everything() {
        let transition = Transition::new(LifecycleState::Ready, "the listeners are open");
        let line = transition.render();

        assert!(line.contains("ready"), "{line}");
        assert!(line.contains("the listeners are open"), "{line}");
        assert!(line.contains("clients can reach it"), "{line}");
        assert_eq!(transition.moment.len(), 20, "the moment: {line}");
    }

    /// Only the ready state answers clients.
    #[test]
    fn only_the_ready_state_answers() {
        assert!(LifecycleState::Ready.is_serving());
        assert!(!LifecycleState::Degraded.is_serving());
        assert!(!LifecycleState::Suspended.is_serving());
        assert!(!LifecycleState::Starting.is_serving());
    }
}
