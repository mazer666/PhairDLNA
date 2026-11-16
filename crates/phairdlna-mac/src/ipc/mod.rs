//! The client of the administration surface.
//!
//! What: the code that talks to the local administration surface of the
//!       server — the request that is built, the answer that is read and the
//!       snapshot that the menu bar shows.
//! Why: the menu bar application holds no state of the server of its own. It
//!       asks the surface what the server does and shows the answer, so that
//!       the application and the server can never disagree (`MB-001`,
//!       `MB-005`).
//! Example: a client that knows the port of the surface and the token of the
//!          installation asks `GET /api/status` and reads the number of the
//!          streams that are running.
//!
//! **The surface is on the loopback interface.** Every request of this module
//! names `127.0.0.1` and carries the token of the installation. The module
//! never opens a connection to another host, and it never reads the token out
//! of a place other than the file the owner of the machine may read
//! (`SEC-071`, `SEC-094`).
//!
//! **Three parts.** [`http`] carries the bytes, [`json`] reads the answers
//! and [`snapshots`] turns the answers into what the application understands;
//! [`client`] puts the three together.

pub mod client;
pub mod http;
pub mod json;
pub mod snapshots;

pub use client::{read_token, AdminClient, ClientError};
pub use http::{HttpRequest, HttpResponse, TcpTransport, Transport, TransportError};
pub use json::{Json, JsonError};
pub use snapshots::{
    DiagnosticsSnapshot, RescanReport, RootSnapshot, StatusSnapshot, TransferSnapshot,
};
