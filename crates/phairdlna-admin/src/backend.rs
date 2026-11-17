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

/// The facts about the server, in the model of the core.
///
/// The model lives in the core so that the command line, the menu bar
/// application and the surface show the same facts and cannot disagree about
/// them (`OPS-013`, `UX-011`).
pub use phairdlna_core::diagnostics::{Diagnostics, LogRecord, RootState, TransferInfo};

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
