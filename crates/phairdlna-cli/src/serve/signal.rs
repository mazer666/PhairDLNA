//! The signals that stop the daemon.
//!
//! What: the wait for `SIGINT` and for `SIGTERM`.
//! Why: a server that is killed rather than stopped leaves a television showing
//!       a dead entry. The signal starts the shutdown sequence of `OPS-008`,
//!       and a second signal ends it at once (`OPS-008`, AC-10).
//! Example: `Ctrl-C` in the terminal stops the daemon; a second `Ctrl-C`
//!       leaves immediately.

/// Why the daemon is stopping.
///
/// The wait ends on the first signal. A second signal is caught by the
/// shutdown sequence itself, which then ends at once (`AC-10`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The first signal arrived; the shutdown sequence runs.
    First,
}

/// Waits for the signal that stops the daemon.
///
/// The wait ends on `SIGINT` and on `SIGTERM`, on every platform the daemon
/// runs on.
pub async fn wait_for_shutdown() -> Reason {
    #[cfg(unix)]
    {
        wait_for_unix_signal().await
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        Reason::First
    }
}

/// Waits for `SIGINT` or `SIGTERM` on a Unix system.
#[cfg(unix)]
async fn wait_for_unix_signal() -> Reason {
    use tokio::signal::unix::{signal, SignalKind};
    let mut interrupt = signal(SignalKind::interrupt()).ok();
    let mut terminate = signal(SignalKind::terminate()).ok();
    match (interrupt.as_mut(), terminate.as_mut()) {
        (Some(interrupt), Some(terminate)) => {
            tokio::select! {
                _ = interrupt.recv() => Reason::First,
                _ = terminate.recv() => Reason::First,
            }
        }
        (Some(interrupt), None) => {
            interrupt.recv().await;
            Reason::First
        }
        (None, Some(terminate)) => {
            terminate.recv().await;
            Reason::First
        }
        (None, None) => {
            let _ = tokio::signal::ctrl_c().await;
            Reason::First
        }
    }
}
