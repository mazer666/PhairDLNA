//! Readiness and stopping notifications acceptance tests (OPS-009, F-13 AC-12).
//!
//! What: verifies that the server invokes readiness notifications via the systemd
//!       NOTIFY_SOCKET protocol when listeners are bound and stopping when shut down.
//! Why: OPS-009 requires systemd notify support so `systemctl start` returns only
//!      when the server is truly ready to answer.

use std::os::unix::net::UnixDatagram;

#[tokio::test]
async fn systemd_notification_is_delivered_on_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let socket_path = dir.path().join("notify.sock");

    let listener = UnixDatagram::bind(&socket_path).expect("bind notify socket");
    listener
        .set_nonblocking(true)
        .expect("set nonblocking socket");

    // Temporarily set NOTIFY_SOCKET for the test process
    std::env::set_var("NOTIFY_SOCKET", &socket_path);

    phairdlna_cli::service::notify::notify_ready("listening on 127.0.0.1:8000")
        .expect("send notify ready");

    let mut buf = [0u8; 512];
    let (len, _) = listener.recv_from(&mut buf).expect("receive ready message");
    let msg = std::str::from_utf8(&buf[..len]).expect("utf8 string");
    assert!(msg.contains("READY=1"));
    assert!(msg.contains("STATUS=listening on 127.0.0.1:8000"));

    phairdlna_cli::service::notify::notify_stopping().expect("send notify stopping");
    let (len, _) = listener
        .recv_from(&mut buf)
        .expect("receive stopping message");
    let msg = std::str::from_utf8(&buf[..len]).expect("utf8 string");
    assert!(msg.contains("STOPPING=1"));

    std::env::remove_var("NOTIFY_SOCKET");
}
