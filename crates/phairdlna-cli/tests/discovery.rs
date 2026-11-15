//! Tests of the discovery of the daemon.
//!
//! What: proves that a running daemon advertises itself, answers a search of
//!       a client and withdraws its advertisements when it stops.
//! Why: the discovery is how a television finds the server at all. A server
//!       that answers HTTP but is not discoverable is a server nobody uses.
//! Example: `a_search_is_answered` sends the `M-SEARCH` a television sends
//!       and reads the answer that comes back.
//!
//! The tests speak the real protocol over a real socket. The port of the
//! discovery socket is another one than 1900, so that a test run does not
//! compete with a server that is really running on the network.

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use phairdlna_cli::serve::{Daemon, ServeOptions};

/// The group of the SSDP discovery.
const GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// The positive case: the daemon advertises itself and answers a search.
///
/// The search is sent from a socket that is **not** a member of the group, so
/// the answer can only have reached it as a unicast datagram (`INT-003`).
#[test]
fn a_search_is_answered() {
    let port = free_udp_port();
    let listen_port = free_udp_port();
    let mut server = Server::start(port, listen_port);

    let mut group = receiver(listen_port);
    let advertisement = wait_for(&mut group, "NTS: ssdp:alive");
    assert!(
        advertisement.contains("NOTIFY * HTTP/1.1"),
        "the advertisement is a notification: {advertisement}"
    );
    assert!(
        advertisement.contains("NT: upnp:rootdevice"),
        "the advertisement names the root device: {advertisement}"
    );

    let mut searcher = plain_socket();
    let search = format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {GROUP}:{port}\r\nMAN: \"ssdp:discover\"\r\n\
         MX: 1\r\nST: ssdp:all\r\n\r\n"
    );
    searcher
        .send_to(search.as_bytes(), SocketAddr::new(GROUP.into(), port))
        .expect("the search is sent");

    let answer = wait_for(&mut searcher, "HTTP/1.1 200 OK");
    assert!(
        answer.contains("ST: upnp:rootdevice"),
        "the answer names the target: {answer}"
    );
    assert!(
        answer.contains("USN: uuid:"),
        "the answer names the device: {answer}"
    );
    assert!(
        answer.contains("::upnp:rootdevice"),
        "the USN carries the target of the specification: {answer}"
    );
    assert!(
        answer.contains("LOCATION: http://"),
        "the answer names the description: {answer}"
    );
    assert!(
        answer.contains("CACHE-CONTROL: max-age=1800"),
        "the answer names the lifetime: {answer}"
    );

    server.stop();
}

/// The positive case: the daemon withdraws its advertisements when it stops.
#[test]
fn the_advertisements_are_withdrawn() {
    let port = free_udp_port();
    let listen_port = free_udp_port();
    let mut server = Server::start(port, listen_port);
    let mut group = receiver(listen_port);
    wait_for(&mut group, "NTS: ssdp:alive");

    server.stop();

    let byebye = wait_for(&mut group, "NTS: ssdp:byebye");
    assert!(
        byebye.contains("NOTIFY * HTTP/1.1"),
        "the withdrawal is a notification: {byebye}"
    );
}

/// The negative case: a search for something the server does not advertise is
/// ignored.
#[test]
fn an_unknown_search_is_ignored() {
    let port = free_udp_port();
    let listen_port = free_udp_port();
    let mut server = Server::start(port, listen_port);
    let mut group = receiver(listen_port);
    wait_for(&mut group, "NTS: ssdp:alive");

    let searcher = plain_socket();
    let search = format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {GROUP}:{port}\r\nMAN: \"ssdp:discover\"\r\n\
         MX: 1\r\nST: urn:schemas-upnp-org:device:MediaRenderer:1\r\n\r\n"
    );
    searcher
        .send_to(search.as_bytes(), SocketAddr::new(GROUP.into(), port))
        .expect("the search is sent");

    let mut answered = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut buffer = vec![0u8; 2048];
    while std::time::Instant::now() < deadline {
        let Ok((read, _)) = searcher.recv_from(&mut buffer) else {
            break;
        };
        let text = String::from_utf8_lossy(&buffer[..read]).into_owned();
        if text.contains("HTTP/1.1 200 OK") {
            answered = true;
            break;
        }
    }
    assert!(
        !answered,
        "the server does not answer a search it does not match"
    );

    server.stop();
}

/// A daemon that serves a library and is discovered on the given port.
struct Server {
    /// The directory of the test, kept alive so that the configuration stays.
    _directory: tempfile::TempDir,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Server {
    /// Starts a daemon whose discovery socket listens on the given port.
    fn start(ssdp_port: u16, listen_port: u16) -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let library = directory.path().join("Library");
        std::fs::create_dir_all(&library).expect("the library is created");
        let config = directory.path().join("phairdlna.toml");
        std::fs::write(
            &config,
            format!(
                "schema_version = 1\n\n[server]\ndevice_name = \"Living room\"\n\n\
                 [library]\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n",
                library.display()
            ),
        )
        .expect("the configuration is written");
        let mut options = ServeOptions::new(config);
        options.http_port = Some(0);
        options.ssdp = true;
        options.host = Some(Ipv4Addr::LOCALHOST);
        options.ssdp_port = Some(ssdp_port);
        options.ssdp_advertise_port = Some(listen_port);
        options.identity_path = Some(directory.path().join("identity"));
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("the runtime starts");
            runtime.block_on(async move {
                let mut daemon = Daemon::start(options).await.expect("the daemon starts");
                let _ = stopped.await;
                daemon.shutdown().await.expect("the daemon stops");
            });
        });
        Self {
            _directory: directory,
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    /// Stops the daemon and waits for it.
    fn stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            // A daemon that panics has already said so on the standard error;
            // the test does not stop twice for the same reason.
            let _ = thread.join();
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Opens a socket that receives the datagrams of the group.
fn receiver(port: u16) -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).expect("the receiver binds");
    socket
        .join_multicast_v4(&GROUP, &Ipv4Addr::UNSPECIFIED)
        .expect("the receiver joins the group");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("the timeout is set");
    socket
}

/// Opens a socket that is **not** a member of the group.
///
/// A datagram that reaches this socket has been sent to its address and its
/// port: it is a unicast datagram.
fn plain_socket() -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("the socket binds");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("the timeout is set");
    socket
}

/// Waits for a datagram that carries the given text.
fn wait_for(socket: &mut UdpSocket, text: &str) -> String {
    let mut buffer = vec![0u8; 2048];
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        let Ok((read, _)) = socket.recv_from(&mut buffer) else {
            continue;
        };
        let message = String::from_utf8_lossy(&buffer[..read]).into_owned();
        if message.contains(text) {
            return message;
        }
    }
    panic!("no datagram with \"{text}\" arrived");
}

/// A UDP port that is free at the moment of the call.
fn free_udp_port() -> u16 {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("a free port");
    let port = socket.local_addr().expect("the address of the port").port();
    drop(socket);
    port
}
