//! What the administration surface tells about the server.
//!
//! What: the data the surface answers with, and the trait that brings that
//!       data in.
//! Why: the surface does not know the server. It knows how to ask for the
//!       state of the server, how to answer it as JSON, and how to refuse a
//!       request. Everything it knows about the daemon comes through the
//!       trait, so that the surface stays a surface and not a second daemon
//!       (`ADR-0001`).
//! Example: `status()` answers how long the server has been running, how many
//!          streams are running, and what the roots of the library look like.

/// What the server is doing right now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    /// How long the server has been running, in seconds.
    pub uptime_seconds: u64,
    /// The version of the server.
    pub version: String,
    /// The source revision the server was built from.
    pub source_revision: String,
    /// How many streams are running.
    pub active_streams: usize,
    /// How many objects the catalogue holds.
    pub catalogue_objects: usize,
    /// The bytes the process occupies, when the platform can say.
    pub resident_bytes: Option<u64>,
    /// The bytes the pictures of the library occupy.
    pub cache_bytes: u64,
    /// The identifier the server tells its clients that it has changed.
    pub system_update_id: u32,
    /// The roots of the library.
    pub roots: Vec<RootState>,
}

/// One root of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootState {
    /// The name of the root.
    pub name: String,
    /// The path of the root, with the private parts already replaced.
    pub path: String,
    /// Whether the volume of the root is there.
    pub mounted: bool,
    /// How many objects the root holds.
    pub objects: usize,
}

/// One stream that is running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferInfo {
    /// The name of the client that plays the stream.
    pub client: String,
    /// The address of the client, with the private parts already replaced.
    pub address: String,
    /// What is being played.
    pub title: String,
    /// How long the stream has been running, in seconds.
    pub running_seconds: u64,
}

/// One record of the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    /// When the record was written.
    pub time: String,
    /// How serious the record is.
    pub level: String,
    /// Which part of the server wrote the record.
    pub component: String,
    /// What the record says.
    pub message: String,
    /// The fields of the record, with the private parts already replaced.
    pub fields: Vec<(String, String)>,
}

/// What the server has to say about itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    /// The version of the server.
    pub version: String,
    /// The source revision the server was built from.
    pub source_revision: String,
    /// How long the server has been running, in seconds.
    pub uptime_seconds: u64,
    /// How many objects the catalogue holds.
    pub catalogue_objects: usize,
    /// The bytes the process occupies, when the platform can say.
    pub resident_bytes: Option<u64>,
    /// The identifier the server tells its clients that it has changed.
    pub system_update_id: u32,
    /// The roots of the library.
    pub roots: Vec<RootState>,
    /// The streams that are running.
    pub transfers: Vec<TransferInfo>,
    /// The records of the log that the server keeps.
    pub records: Vec<LogRecord>,
    /// The warnings of the server, as short sentences.
    pub warnings: Vec<String>,
    /// What the last scan of the library found.
    pub last_scan: Option<String>,
}

/// What a scan of the library found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RescanOutcome {
    /// Whether the library was scanned again.
    pub rescanned: bool,
    /// How many objects the catalogue holds after the scan.
    pub objects: usize,
    /// How long the scan took, in seconds.
    pub duration_seconds: u64,
    /// What happened, in one sentence.
    pub message: String,
}

/// The data the administration surface asks the server for.
///
/// The trait is the whole contact between the surface and the daemon. A
/// server that does not want the surface simply does not implement it, and a
/// surface that is asked for something the server cannot do gets an answer
/// that says so.
pub trait AdminBackend: Send + Sync + 'static {
    /// The state of the server.
    fn status(&self) -> Status;

    /// What the server has to say about itself.
    fn diagnostics(&self) -> Diagnostics;

    /// Scans the library again.
    fn rescan(&self) -> RescanOutcome;

    /// Notes that a request was refused, so that the log tells it.
    ///
    /// The reason is a word, not a sentence about the state of the server.
    fn note_refusal(&self, reason: &str);
}
