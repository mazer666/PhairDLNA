//! The helpers of the administration surface.
//!
//! What: the part of the test support that starts a daemon with the
//!       administration surface and talks to it.
//! Why: the support module of the tests must stay small enough to be read;
//!       the helpers of the surface are their own file (`QG-13`).
//! Example: `Server::start_with_admin` starts a daemon whose surface listens
//!          on a free port of the loopback interface.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use phairdlna_cli::serve::{Daemon, ServeOptions};

use super::{parse, Answer, Server, FILM};

impl Server {
    /// Starts a daemon with the administration surface switched on.
    ///
    /// The surface listens on a free port of the loopback interface, and its
    /// token is written into the folder of the test.
    #[cfg(feature = "admin")]
    pub fn start_with_admin() -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let library = directory.path().join("Library");
        std::fs::create_dir_all(&library).expect("the library is created");
        std::fs::write(library.join("King Kong.mkv"), FILM).expect("the file is written");
        let config = write_admin_config(directory.path(), &library);
        let identity = directory.path().join("identity");
        let mut options = ServeOptions::new(config);
        options.http_port = Some(0);
        options.ssdp = false;
        options.host = Some(std::net::Ipv4Addr::LOCALHOST);
        options.identity_path = Some(identity);
        options.admin_port = Some(0);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("the runtime starts");
            runtime.block_on(async move {
                let mut daemon = Daemon::start(options).await.expect("the daemon starts");
                let address = daemon.local_addr();
                let admin = daemon.admin_addr();
                ready_tx.send((address, admin)).ok();
                let _ = stopped.await;
                daemon.shutdown().await.expect("the daemon stops");
            });
        });
        let (address, admin) = ready_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the daemon announces its address");
        let token = std::fs::read_to_string(directory.path().join("admin.token"))
            .expect("the token is written");
        Self {
            _directory: directory,
            address,
            library,
            admin,
            admin_token: Some(token.trim().to_owned()),
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    /// The address of the administration surface.
    #[cfg(feature = "admin")]
    pub fn admin_address(&self) -> SocketAddr {
        self.admin.expect("the administration surface runs")
    }

    /// The token of the administration surface.
    #[cfg(feature = "admin")]
    pub fn admin_token(&self) -> &str {
        self.admin_token.as_deref().expect("the token is there")
    }

    /// Sends a `GET` to the administration surface, with the token.
    #[cfg(feature = "admin")]
    pub fn admin_get(&self, path: &str) -> Answer {
        self.admin_send("GET", path)
    }

    /// Sends a `POST` to the administration surface, with the token.
    ///
    /// A request that changes something carries the token of the "cross-site
    /// request forgery" defence; the client reads it from the answer of a
    /// request that changes nothing.
    #[cfg(feature = "admin")]
    pub fn admin_post(&self, path: &str) -> Answer {
        let csrf = self.admin_csrf();
        let address = self.admin_address();
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {address}\r\n\
             Authorization: Bearer {}\r\nX-Phair-Admin: 1\r\n\
             X-Phair-Admin-Csrf: {csrf}\r\n\
             Connection: close\r\nContent-Length: 0\r\n\r\n",
            self.admin_token()
        );
        admin_raw(address, &request)
    }

    /// The token of the "cross-site request forgery" defence.
    #[cfg(feature = "admin")]
    pub fn admin_csrf(&self) -> String {
        let answer = self.admin_get("/api/status");
        answer
            .header("x-phairdlna-admin-csrf")
            .expect("the answer carries the token")
            .to_owned()
    }

    /// Sends a request to the administration surface, with the token.
    #[cfg(feature = "admin")]
    pub fn admin_send(&self, method: &str, path: &str) -> Answer {
        let address = self.admin_address();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {address}\r\n\
             Authorization: Bearer {}\r\nX-Phair-Admin: 1\r\n\
             Connection: close\r\nContent-Length: 0\r\n\r\n",
            self.admin_token()
        );
        admin_raw(address, &request)
    }

    /// Sends a request to the administration surface, exactly as it is written.
    ///
    /// The test writes the request itself, because a rule that is tested with a
    /// helper that adds the headers of its own is not tested at all.
    #[cfg(feature = "admin")]
    pub fn admin_request(&self, request: &str) -> Answer {
        admin_raw(self.admin_address(), request)
    }
}

/// Sends a request to a socket and reads the answer.
pub fn admin_raw(address: SocketAddr, request: &str) -> Answer {
    let mut stream = TcpStream::connect(address).expect("the surface accepts connections");
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

/// Writes a configuration file with the administration surface switched on.
pub fn write_admin_config(directory: &Path, library: &Path) -> PathBuf {
    let path = directory.join("phairdlna.toml");
    let text = format!(
        "schema_version = 1\n\n[server]\ndevice_name = \"Living room\"\n\n\
         [library]\ncache_directory = \"{}\"\n[[library.roots]]\nname = \"Films\"\npath = \"{}\"\n\n\
         [security]\nadmin_enabled = true\nadmin_token_file = \"{}\"\n",
        directory.join("cache").display(),
        library.display(),
        directory.join("admin.token").display()
    );
    std::fs::write(&path, text).expect("the configuration is written");
    path
}
