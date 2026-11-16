//! The local administration surface of PhairDLNA.
//!
//! What: the HTTP surface that tells the owner of the machine what the server
//!       is doing — the state of the server, the scan of the library and the
//!       diagnostics — and that lets the library be scanned again.
//! Why: the server runs on a machine the user owns, and the user needs to see
//!       what it does without reading a log file. The surface is on the
//!       loopback interface, behind a token, and it is a surface of its own:
//!       a page of the internet must not be able to reach it (`OPS-020`,
//!       `F-14`).
//! Example: a request that carries the token and the custom header
//!          `X-Phair-Admin: 1` reads `GET /api/status` on
//!          `http://127.0.0.1:8210`.
//!
//! **The defences of the surface.** The surface answers only to a `Host` header
//! that names the loopback interface, so that a name an attacker controls
//! cannot reach it. It answers only to a request that carries the token, so
//! that another program of the machine cannot use it. It answers only to a
//! request that carries the custom header `X-Phair-Admin`, which a form in a
//! browser cannot send. It never answers a request of another origin, and it
//! never answers a question about the permissions it would need (`SEC-071`,
//! `SEC-TH-23`).
//!
//! **The crate.** The crate holds the surface and nothing else. What it knows
//! about the server comes through the trait [`backend::AdminBackend`], so that
//! the surface stays a surface and the daemon stays the daemon (`ADR-0001`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod backend;
mod guard;
mod json;
mod routes;
pub mod server;
pub mod token;

pub use backend::{
    AdminBackend, Diagnostics, LogRecord, RescanOutcome, RootState, Status, TransferInfo,
};
pub use guard::AdminGuard;
pub use json::Json;
pub use server::{AdminError, AdminOptions, AdminServer};
pub use token::{AdminToken, TokenError};

/// The name of the custom header every request must carry.
///
/// The header is the answer to the "cross-site request forgery" question: a
/// form in a browser cannot send a header the user did not choose, and a page
/// of another origin cannot send one without asking first — and the surface
/// answers no such question (`OPS-020`).
pub const ADMIN_HEADER: &str = "X-Phair-Admin";

/// The value the custom header must carry.
pub const ADMIN_HEADER_VALUE: &str = "1";

/// The name of the header that carries the token of the "cross-site request
/// forgery" defence.
///
/// The value is bound to the session and must be sent with every request that
/// changes something (`OPS-023`).
pub const ADMIN_CSRF_HEADER: &str = "X-Phair-Admin-Csrf";
