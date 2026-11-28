//! Unauthenticated AGPL Corresponding Source offer handler (`SEC-070`, `OPS-016`).
//!
//! What: serves exact build provenance, source repository details, dependency
//!       tree (`Cargo.lock`), and instructions to obtain the exact source.
//! Why: AGPL-3.0 §13 requires offering the corresponding source to all users
//!      interacting remotely with the server over a network at no charge.

use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;

/// The embedded build-time Git commit hash.
pub const BUILD_REVISION: &str = env!("CARGO_PKG_VERSION");

/// The canonical source code repository URL under AGPL-3.0.
pub const REPOSITORY_URL: &str = "https://github.com/mazer666/PhairDLNA";

/// Answers the unauthenticated corresponding source information.
pub async fn source_offer() -> Response {
    let cargo_lock = include_str!("../../../../Cargo.lock");
    let body = format!(
        "PhairDLNA Corresponding Source Offer (AGPL-3.0-or-later, §13)\n\n\
        Product: PhairDLNA\n\
        Version: {}\n\
        License: AGPL-3.0-or-later\n\
        Repository: {}\n\
        Exact Source Archive: {}/archive/refs/tags/v{}.tar.gz\n\n\
        Build Recipe:\n\
        1. Install Rust toolchain (MSRV 1.85.0+): https://rustup.rs\n\
        2. Clone or extract the corresponding source archive\n\
        3. Execute: cargo build --release\n\n\
        --- Complete Dependency Tree (Cargo.lock) ---\n\
        {}\n",
        BUILD_REVISION, REPOSITORY_URL, REPOSITORY_URL, BUILD_REVISION, cargo_lock
    );

    let mut response = Response::new(Body::from(body));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response
}
