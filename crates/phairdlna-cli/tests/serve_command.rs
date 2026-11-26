//! Tests of the `serve` command of the command line.
//!
//! What: proves that the command starts a server that answers a client, and
//!       that it stops gracefully when the process is asked to stop.
//! Why: the command line is how the server is started in practice. A command
//!       that fails to start, or that leaves a broken server behind, is a
//!       defect the user meets on the first day (`OPS-001`, `OPS-008`).
//! Example: `the_server_answers_a_client` starts the binary, reads the device
//!       description over a socket and ends the process.

use std::io::{Read, Write};
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Sets up the test directory and writes the test configuration.
fn setup_test_server(directory: &tempfile::TempDir, port: u16) -> std::path::PathBuf {
    let library = directory.path().join("Library");
    std::fs::create_dir_all(&library).expect("the library is created");
    std::fs::write(library.join("King Kong.mkv"), b"a film").expect("the film is written");
    let config = directory.path().join("phairdlna.toml");
    std::fs::write(
        &config,
        format!(
            "schema_version = 1\n\n[server]\ndevice_name = \"Living room\"\nhttp_port = {port}\n\
             discovery_enabled = false\n\n\
             [library]\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n",
            library.display()
        ),
    )
    .expect("the configuration is written");
    config
}

/// The positive case: the `serve` command starts a server that answers a
/// client on the network.
///
/// The daemon runs in a process of its own, because it only stops when it
/// receives a signal; the test asks it for the device description and then
/// ends it.
#[test]
fn the_server_answers_a_client() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let port = free_port();
    let config = setup_test_server(&directory, port);

    let mut child = Command::new(env!("CARGO_BIN_EXE_phairdlna-cli"))
        .arg("serve")
        .arg("--config")
        .arg(&config)
        .arg("--port")
        .arg(port.to_string())
        // The identity of the test lives in the temporary directory of the
        // test, not in the configuration directory of the user.
        .env("HOME", directory.path())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let mut errors = child.stderr.take().expect("the log of the server");

    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let document = wait_for_the_description(address);
    assert!(
        document.contains("urn:schemas-upnp-org:device:MediaServer:1"),
        "the device description names the device type: {document}"
    );
    assert!(
        document.contains("<UDN>uuid:"),
        "the device description names the device: {document}"
    );

    // The server stops when it is asked to, and it stops with the code of a
    // successful run (`OPS-008`).
    terminate(&child);
    let status = wait_for_the_end(&mut child);
    assert_eq!(status.code(), Some(0), "the server stops with the code 0");

    // The log records go to standard error, and every record carries the
    // moment, the level, the component and the message (`OPS-006`).
    let mut records = Vec::new();
    let _ = errors.read_to_end(&mut records);
    let records = String::from_utf8_lossy(&records).into_owned();
    let record = records
        .lines()
        .find(|line| line.contains("daemon the server is listening"))
        .expect("the server writes a record about itself");
    let mut parts = record.split_whitespace();
    let moment = parts.next().expect("the record has a moment");
    assert!(
        moment.ends_with('Z') && moment.contains('-'),
        "the record carries a timestamp: {record}"
    );
    assert_eq!(parts.next(), Some("info"), "the record carries the level");
    assert_eq!(
        parts.next(),
        Some("daemon"),
        "the record carries the component"
    );
    assert_eq!(
        parts.next(),
        Some("the"),
        "the record carries the message: {record}"
    );
}

/// Sends the signal that asks the server to stop.
fn terminate(child: &Child) {
    let signal = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    assert!(signal.is_ok(), "the signal could be sent");
}

/// Waits for the server to end, and returns its status.
fn wait_for_the_end(child: &mut Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("the state of the server is known") {
            return status;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    panic!("the server did not stop");
}

/// Waits for the server to answer, and reads the device description.
fn wait_for_the_description(address: SocketAddr) -> String {
    for _ in 0..100 {
        if let Some(text) = try_fetch_description(&address) {
            return text;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the server did not answer the device description");
}

/// Tries to fetch the device description once.
fn try_fetch_description(address: &SocketAddr) -> Option<String> {
    let mut stream =
        std::net::TcpStream::connect_timeout(address, Duration::from_millis(200)).ok()?;
    let request =
        format!("GET /description.xml HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).ok()?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    if text.contains("<root") {
        Some(text)
    } else {
        None
    }
}

/// A port that is free at the moment of the call.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("a free port");
    let port = listener
        .local_addr()
        .expect("the address of the port")
        .port();
    drop(listener);
    port
}
