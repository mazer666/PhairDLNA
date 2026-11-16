#![allow(dead_code)]
//! Fixtures for the tests of the daemon.
//!
//! What: a daemon that runs in a thread of the test, and a small HTTP client
//!       that speaks to it over a socket.
//! Why: the daemon is the product, and the only honest way to test it is to
//!       speak to it as a client does. The helpers live here so that the test
//!       files stay readable.
//! Example: `Server::start()` brings a daemon up on a free port and hands out
//!       its address.

#[cfg(feature = "admin")]
mod admin;
pub mod artwork;
pub mod soap;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use phairdlna_cli::serve::{Daemon, ServeOptions};
use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;

/// The bytes of the file the tests serve.
pub const FILM: &[u8] = b"a film of eleven bytes";

/// The answer of a request.
pub struct Answer {
    /// The status code of the answer.
    pub status: u16,
    /// The headers of the answer, in the order they came.
    pub headers: Vec<(String, String)>,
    /// The bytes of the body.
    pub body: Vec<u8>,
}

impl Answer {
    /// The value of a header, in the case of a name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The body as text.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// A daemon that runs in a thread of the test.
pub struct Server {
    /// The directory of the test, kept alive so that the library stays.
    _directory: tempfile::TempDir,
    address: SocketAddr,
    library: PathBuf,
    /// The address of the administration surface, when it runs.
    #[cfg(feature = "admin")]
    admin: Option<SocketAddr>,
    /// The token of the administration surface.
    #[cfg(feature = "admin")]
    admin_token: Option<String>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Server {
    /// Starts a daemon with a library of one film.
    pub fn start() -> Self {
        Self::start_with_film(FILM)
    }

    /// Starts a daemon whose access-control list names the given clients.
    ///
    /// The daemon answers only the clients of the list; a test uses the list to
    /// refuse the client of the test itself (`CFG-009`).
    pub fn start_with_clients(clients: &[&str]) -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let library = directory.path().join("Library");
        std::fs::create_dir_all(&library).expect("the library is created");
        std::fs::write(library.join("King Kong.mkv"), FILM).expect("the file is written");
        let list: Vec<String> = clients
            .iter()
            .map(|client| format!("\"{client}\""))
            .collect();
        let security = format!("[security]\nallowed_clients = [{}]\n", list.join(", "));
        let config = write_config_with(directory.path(), &library, &security);
        let identity = directory.path().join("identity");
        let mut options = ServeOptions::new(config);
        options.http_port = Some(0);
        options.ssdp = false;
        options.host = Some(std::net::Ipv4Addr::LOCALHOST);
        options.identity_path = Some(identity);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("the runtime starts");
            runtime.block_on(async move {
                let mut daemon = Daemon::start(options).await.expect("the daemon starts");
                ready_tx.send(daemon.local_addr()).ok();
                let _ = stopped.await;
                daemon.shutdown().await.expect("the daemon stops");
            });
        });
        let address = ready_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the daemon announces its address");
        Self {
            _directory: directory,
            address,
            library,
            #[cfg(feature = "admin")]
            admin: None,
            #[cfg(feature = "admin")]
            admin_token: None,
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    /// Starts a daemon with a library of one film of the given size.
    pub fn start_with_film(film: &[u8]) -> Self {
        Self::start_with_files(&[("King Kong.mkv", film)])
    }

    /// Starts a daemon with a library of the given files.
    ///
    /// The names are the names of the files, and the bytes are their content.
    /// A test that needs a picture in the library writes the file itself.
    pub fn start_with_files(files: &[(&str, &[u8])]) -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let library = directory.path().join("Library");
        std::fs::create_dir_all(&library).expect("the library is created");
        for (name, bytes) in files {
            std::fs::write(library.join(name), bytes).expect("the file is written");
        }
        let config = write_config(directory.path(), &library);
        let identity = directory.path().join("identity");
        let mut options = ServeOptions::new(config);
        options.http_port = Some(0);
        options.ssdp = false;
        options.host = Some(std::net::Ipv4Addr::LOCALHOST);
        options.identity_path = Some(identity);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("the runtime starts");
            runtime.block_on(async move {
                let mut daemon = Daemon::start(options).await.expect("the daemon starts");
                ready_tx.send(daemon.local_addr()).ok();
                let _ = stopped.await;
                daemon.shutdown().await.expect("the daemon stops");
            });
        });
        let address = ready_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the daemon announces its address");
        Self {
            _directory: directory,
            address,
            library,
            #[cfg(feature = "admin")]
            admin: None,
            #[cfg(feature = "admin")]
            admin_token: None,
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    /// The address the daemon listens on.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// The folder the library lives in.
    pub fn library(&self) -> &Path {
        &self.library
    }

    /// The identifier of the film in the catalogue.
    pub fn media_id(&self) -> String {
        self.item_id("King Kong.mkv")
    }

    /// The identifier of an item of the catalogue, by its title.
    pub fn item_id(&self, title: &str) -> String {
        let catalogue = catalogue_of(&self.library);
        find_item(&catalogue, title).expect("the item is in the catalogue")
    }

    /// The identifier of the folder that holds the film.
    pub fn library_id(&self) -> String {
        let catalogue = catalogue_of(&self.library);
        catalogue
            .container("1")
            .expect("the root is in the catalogue")
            .children
            .first()
            .expect("the library is in the catalogue")
            .id
            .clone()
    }

    /// The folder of the library, so that a test can put a file into it.
    pub fn library_folder(&self) -> &Path {
        &self.library
    }

    /// Sends a `GET`.
    pub fn get(&self, path: &str) -> Answer {
        self.get_with(path, &[])
    }

    /// Sends a `GET` with headers.
    pub fn get_with(&self, path: &str, headers: &[(&str, &str)]) -> Answer {
        self.send("GET", path, headers, None)
    }

    /// Sends a `HEAD`.
    pub fn head(&self, path: &str) -> Answer {
        self.send("HEAD", path, &[], None)
    }

    /// Sends a `POST`.
    pub fn post(&self, path: &str, body: &str, action: &str) -> Answer {
        self.send("POST", path, &[("SOAPACTION", action)], Some(body))
    }

    /// Sends a request and reads the answer.
    pub fn send(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> Answer {
        let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {}\r\n", self.address);
        for (name, value) in headers {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str("Connection: close\r\n");
        let body = body.unwrap_or_default();
        request.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        request.push_str(body);

        let mut stream = TcpStream::connect(self.address).expect("the daemon accepts connections");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("the timeout is set");
        stream
            .write_all(request.as_bytes())
            .expect("the request is sent");
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).expect("the answer is read");
        parse(&raw)
    }

    /// Stops the daemon and waits for it.
    pub fn stop(&mut self) {
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

/// Reads an answer of the daemon.
///
/// The head is text and the body may not be: a picture is a picture, and its
/// bytes must reach the test unchanged. The two are therefore separated on
/// the raw bytes and not on a decoded text.
pub fn parse(raw: &[u8]) -> Answer {
    let separator = b"\r\n\r\n";
    let end = raw
        .windows(separator.len())
        .position(|part| part == separator)
        .expect("the answer has a head and a body");
    let head = String::from_utf8_lossy(&raw[..end]).into_owned();
    let body = raw[end + separator.len()..].to_vec();
    let mut lines = head.lines();
    let status_line = lines.next().expect("the answer has a status line");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("the status line carries a code");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Answer {
        status,
        headers,
        body,
    }
}

/// Builds the catalogue of a library.
pub fn catalogue_of(library: &Path) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: library.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// Finds an item of the catalogue by its title.
///
/// The catalogue has two levels: the root, and the folder the user shared.
pub fn find_item(catalogue: &Catalogue, title: &str) -> Option<String> {
    let root = catalogue.container("1")?;
    for child in &root.children {
        if child.title == title {
            return Some(child.id.clone());
        }
        let container = catalogue.container(&child.id)?;
        for item in &container.children {
            if item.title == title {
                return Some(item.id.clone());
            }
        }
    }
    None
}

/// Writes a configuration file with one root.
pub fn write_config(directory: &Path, library: &Path) -> PathBuf {
    write_config_with(directory, library, "")
}

/// Writes a configuration file with one root and a security section.
pub fn write_config_with(directory: &Path, library: &Path, security: &str) -> PathBuf {
    let path = directory.join("phairdlna.toml");
    // The cache of the pictures lives in the folder of the test, so that a test
    // run never writes into the data directory of the user.
    let text = format!(
        "schema_version = 1\n\n[server]\ndevice_name = \"Living room\"\n\n\
         [library]\ncache_directory = \"{}\"\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n\n{security}",
        directory.join("cache").display(),
        library.display()
    );
    std::fs::write(&path, text).expect("the configuration is written");
    path
}
