//! The state of the server as the application understands it.
//!
//! What: the state of the daemon, as the menu bar shows it.
//! Why: the application holds no state of the server of its own; it reads the
//!      state from the administration surface and keeps the last answer
//!      (`MB-001`).
//! Example: a snapshot whose state is `Running` and whose `active_streams` is
//!          two becomes the line "Server running — 2 active streams".

/// What the application knows about the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ServerState {
    /// The application has not received a status response yet.
    Checking,
    /// Nothing listens on the port of the administration surface.
    #[default]
    Stopped,
    /// The server answers and serves the network.
    Running,
    /// The server answers and the application is waiting for a scan.
    Scanning,
    /// Something answers, but it refuses the requests of the application.
    Refused,
}

/// How much the menu shows.
///
/// The mode is a preference of the *view* and not a setting of the server: it
/// changes what the menu shows and nothing else. It lives outside the
/// configuration of the server, it survives a restart of the application, and
/// it is reversible (`UX-015`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PresentationMode {
    /// The short view: the state, the folders and the actions.
    #[default]
    Simple,
    /// The long view: the state, the folders, the actions and the detail of
    /// the server.
    Advanced,
}

impl PresentationMode {
    /// The other mode.
    ///
    /// The action that switches the mode asks for the other one, so that the
    /// user never has to remember which mode is on.
    pub fn other(self) -> PresentationMode {
        match self {
            Self::Simple => Self::Advanced,
            Self::Advanced => Self::Simple,
        }
    }

    /// The name of the mode, as it is written in the file of the preference.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
        }
    }

    /// The mode of a name, if the name is one.
    pub fn of(name: &str) -> Option<PresentationMode> {
        match name.trim() {
            "simple" => Some(Self::Simple),
            "advanced" => Some(Self::Advanced),
            _ => None,
        }
    }
}

/// A snapshot of the server.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ServerSnapshot {
    /// The name media clients see during discovery.
    pub device_name: String,
    /// The address advertised to clients on the local network.
    pub device_address: Option<String>,
    /// Whether the daemon currently runs its SSDP discovery task.
    pub discovery_available: Option<bool>,
    /// The state of the server.
    pub state: ServerState,
    /// How long the server has been running, in seconds.
    pub uptime_seconds: u64,
    /// The number of the streams that are running.
    pub active_streams: u64,
    /// The number of the objects the catalogue holds.
    pub catalogue_objects: u64,
    /// The roots of the library.
    pub roots: Vec<RootInfo>,
    /// The warnings of the server.
    pub warnings: Vec<String>,
    /// The streams that are running.
    pub transfers: Vec<TransferInfo>,
    /// What the last scan of the library found.
    pub last_scan: Option<String>,
    /// The effective interface (LIFE-009).
    pub effective_interface: Option<String>,
    /// The effective address the server is reachable at (LIFE-009).
    pub effective_address: Option<String>,
    /// The state of the socket listener (LIFE-009).
    pub listener_state: Option<String>,
    /// The last recovery event (LIFE-009).
    pub last_recovery_event: Option<String>,
}

impl ServerSnapshot {
    /// The application is waiting for the first status response.
    pub fn checking() -> Self {
        Self {
            state: ServerState::Checking,
            ..Self::default()
        }
    }

    /// The snapshot of a server that is not there.
    ///
    /// Use this after the application has confirmed that the surface is
    /// unreachable; use [`Self::checking`] before the first answer arrives.
    pub fn stopped() -> Self {
        Self {
            state: ServerState::Stopped,
            ..Self::default()
        }
    }
}

/// One root of the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootInfo {
    /// The name of the root.
    pub name: String,
    /// Whether the volume of the root is mounted.
    pub mounted: bool,
    /// The number of the objects of the root.
    pub objects: u64,
}

/// One stream that is running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferInfo {
    /// The name of the client that plays the stream.
    pub client: String,
    /// What is being played.
    pub title: String,
}
