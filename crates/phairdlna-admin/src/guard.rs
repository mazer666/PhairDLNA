//! The rules every request of the administration surface must pass.
//!
//! What: the check of the `Host` header, the check of the framing, the check
//!       of the origin, the check of the token and the check of the custom
//!       header that a form in a browser cannot send.
//! Why: the surface runs on the loopback interface, and the loopback
//!       interface is shared with every program of the machine. A browser on
//!       the machine may be pointed at the surface by a page the user opened,
//!       and the surface has to tell that page from the owner of the server
//!       (`OPS-020`, `SEC-071`, `SEC-TH-23`).
//! Example: a request that carries `Host: attacker.example` is refused with
//!          `421`, a request without the token is refused with `401`, and a
//!          request without the custom header is refused with `403`.
//!
//! **The order of the checks.** The name of the host is read first, because it
//! is the cheapest and the most important. The origin and the fetch metadata
//! come next, because they are what a browser adds. The token comes after
//! them, because a request that is not for us need not be authenticated. The
//! custom header comes last, because it is the rule that a form in a browser
//! cannot satisfy.
//!
//! **No cross-origin answers.** The surface never answers a request with a
//! header that says that another origin may read the answer. A browser that
//! asks the surface from another page therefore learns nothing, and the
//! request of the other page is refused before it reaches a handler.

use std::sync::Arc;

mod host;
use host::refuse_host;
mod request_checks;
use request_checks::{
    refuse_csrf, refuse_custom_header, refuse_fetch_site, refuse_framing, refuse_origin,
    refuse_token,
};

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use phairdlna_core::security::Limits;

use crate::token::AdminToken;

/// The rules every request of the administration surface must pass.
#[derive(Clone)]
pub struct AdminGuard {
    /// The secret the request must carry.
    token: Arc<AdminToken>,
    /// The bounds of the requests.
    limits: Limits,
}

impl AdminGuard {
    /// Builds the rules of the surface.
    pub fn new(token: AdminToken, limits: Limits) -> Self {
        Self {
            token: Arc::new(token),
            limits,
        }
    }

    /// The bounds of the requests.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// The token of the "cross-site request forgery" defence.
    ///
    /// The value is bound to the session: the same token of the installation
    /// always yields the same value (`OPS-023`).
    pub fn csrf_token(&self) -> String {
        self.token.csrf_token()
    }
}

/// Refuses the requests that must not reach a handler.
pub async fn enforce(
    State(guard): State<Arc<AdminGuard>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(refusal) = refuse_host(&request) {
        return refusal;
    }
    if let Some(refusal) = refuse_framing(&guard, &request) {
        return refusal;
    }
    if let Some(refusal) = refuse_origin(&request) {
        return refusal;
    }
    if let Some(refusal) = refuse_fetch_site(&request) {
        return refusal;
    }
    if let Some(refusal) = refuse_token(&guard, &request) {
        return refusal;
    }
    if let Some(refusal) = refuse_custom_header(&request) {
        return refusal;
    }
    if let Some(refusal) = refuse_csrf(&guard, &request) {
        return refusal;
    }
    next.run(request).await
}

/// The answer of a refusal.
///
/// The answer is the same for every refusal, and it says nothing about the
/// server beyond the fact that it is there (`SEC-013`).
fn refused(status: StatusCode) -> Response {
    let mut response = status.into_response();
    *response.body_mut() = axum::body::Body::from("The request was refused.");
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; charset=\"utf-8\""),
    );
    response
}

/// The answer of a request without the right token.
fn unauthorised() -> Response {
    let mut response = refused(StatusCode::UNAUTHORIZED);
    response.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_static("Bearer realm=\"phairdlna-admin\""),
    );
    response
}
