//! Socket-level tests for the local API deadlines.

use super::*;

#[tokio::test]
async fn a_partial_header_is_closed_by_the_socket_read_deadline() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut options = AdminOptions::loopback(0, folder.path().join("admin.token"));
    options.limits.read_timeout = Duration::from_millis(80);
    let backend = Arc::new(Fake {
        rescans: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    });
    let server = AdminServer::start(options, backend)
        .await
        .expect("the surface starts");
    let mut stream = TcpStream::connect(server.addresses()[0])
        .await
        .expect("the surface accepts a connection");
    stream
        .write_all(b"GET /api/status HTTP/1.1\r\nHost: localhost\r\n")
        .await
        .expect("the partial header is sent");

    let mut byte = [0u8; 1];
    let closed = tokio::time::timeout(Duration::from_millis(500), stream.read(&mut byte))
        .await
        .expect("the server closes the idle request in time");

    assert!(matches!(closed, Ok(0) | Err(_)));
}
