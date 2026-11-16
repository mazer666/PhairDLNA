//! The life of the server.
//!
//! What: the states the server is in, the transitions between them and the
//!       local timeline that remembers them.
//! Why: a server is not a program that runs, it is a thing with a life. It
//!      starts, it serves, the machine sleeps, the network changes, it stops —
//!      and the user must be able to see *which* of those it is doing right
//!      now, and why (`LIFE-001`, `LIFE-009`, `LIFE-011`).
//! Example: `Transition::new(LifecycleState::Ready, "the listeners are open")`
//!      describes the moment the server became ready.
//!
//! The state is **one value**, never a set of flags: the server is *Ready* or
//! it is not. A transition always carries a reason, a moment and the sentence
//! the user would read (`LIFE-001`).

pub mod state;
pub mod timeline;

pub use state::{is_allowed, LifecycleState, Transition};
pub use timeline::{Timeline, DEFAULT_TIMELINE_LIMIT};
