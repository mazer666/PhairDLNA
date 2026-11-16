//! The readiness notification of `systemd`.
//!
//! What: the message the daemon sends to `systemd` when its listeners are
//!       open.
//! Why: a unit of the type `notify` makes `systemctl start` wait for the
//!      message. Without it the service manager would report a server that
//!      cannot answer yet as a server that runs (`OPS-009`, `AC-12`).
//! Example: `notify_ready()` writes `READY=1` into the socket of the
//!      environment variable `NOTIFY_SOCKET`.
//!
//! The notification is sent **after** the listeners are open, never before:
//! a service that says it is ready before it can answer is a service that
//! lies.

/// The environment variable that names the socket of the notification.
const NOTIFY_SOCKET: &str = "NOTIFY_SOCKET";

/// Tells the service manager that the server is ready.
///
/// The function does nothing when no service manager asked for a
/// notification: the daemon runs in the foreground just as well.
pub fn notify_ready(status: &str) -> Result<(), String> {
    let message = format!("READY=1\nSTATUS={status}\n");
    send(&message)
}

/// Tells the service manager that the server is stopping.
pub fn notify_stopping() -> Result<(), String> {
    send("STOPPING=1\n")
}

/// Sends a message into the socket of the service manager.
fn send(message: &str) -> Result<(), String> {
    let Ok(socket) = std::env::var(NOTIFY_SOCKET) else {
        return Ok(());
    };
    if socket.is_empty() {
        return Ok(());
    }
    send_to(&socket, message)
}

/// Writes a message into a unix socket of the notification.
#[cfg(unix)]
fn send_to(socket: &str, message: &str) -> Result<(), String> {
    use std::os::unix::net::UnixDatagram;
    if socket.starts_with('@') {
        return Err(
            "the notification socket of the system is an abstract socket, which this build \
             cannot address"
                .to_owned(),
        );
    }
    let sender = UnixDatagram::unbound().map_err(|error| error.to_string())?;
    sender
        .send_to(message.as_bytes(), socket_path(socket))
        .map_err(|error| format!("the notification cannot be sent: {error}"))?;
    Ok(())
}

/// The path of the socket of the notification.
#[cfg(unix)]
fn socket_path(socket: &str) -> &str {
    socket
}

/// The notification is not sent on a platform without unix sockets.
#[cfg(not(unix))]
fn send_to(_socket: &str, _message: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::notify_ready;

    /// The positive case: a daemon that runs in the foreground sends nothing
    /// and says so by succeeding.
    #[test]
    fn a_daemon_without_a_service_manager_sends_nothing() {
        // The test process has no NOTIFY_SOCKET, so the call is a no-op.
        if std::env::var("NOTIFY_SOCKET").is_ok() {
            return;
        }
        notify_ready("the server is listening").expect("nothing is sent");
    }
}
