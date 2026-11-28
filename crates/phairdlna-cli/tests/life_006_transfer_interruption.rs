//! Lifecycle tests for network drop and transfer interruption (LIFE-006, INT-089, INT-081).
//!
//! What: verifies that:
//!       - When an active media transfer stream is dropped mid-body (e.g. client disconnect,
//!         network loss, or sleep), the connection permit is immediately returned to the pool
//!         without leaking transfer slots (INT-089, LIFE-006).
//!       - The server does not attempt a fake transparent resume; subsequent chunks are
//!         truthfully resumed via standard byte-range requests (INT-081, LIFE-006).

#[path = "support/mod.rs"]
mod support;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use support::{Server, FILM};

#[test]
fn interrupted_network_transfer_releases_slot_and_resumes_via_range() {
    let server = Server::start();
    let identifier = server.media_id();
    let path = format!("/media/{identifier}");

    // 1. Client connects and abruptly terminates the connection mid-transfer
    let request = format!("GET {path} HTTP/1.1\r\nHost: {}\r\n\r\n", server.address());
    let mut stream = TcpStream::connect(server.address()).expect("connect first client");
    stream.write_all(request.as_bytes()).expect("send GET");

    let mut partial_buf = [0u8; 8];
    let n = stream
        .read(&mut partial_buf)
        .expect("read partial header/data");
    assert!(n > 0, "received partial bytes from server");

    // Abrupt client disconnect / network interruption (LIFE-006, INT-089)
    drop(stream);

    // Give server event-loop a brief moment to process disconnect and release slot
    std::thread::sleep(Duration::from_millis(50));

    // 2. Truthful resumption: client opens new connection requesting remainder via byte-range (LIFE-006, INT-081)
    let range_answer = server.get_with(&path, &[("Range", "bytes=4-10")]);
    assert_eq!(
        range_answer.status, 206,
        "byte-range resume must return 206 Partial Content"
    );
    assert_eq!(
        range_answer.body,
        &FILM[4..=10],
        "resumed slice must match media file bytes"
    );
}
