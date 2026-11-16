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

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use phairdlna_core::security::{check, values_of, HeaderPairs, Limits};

use crate::token::AdminToken;

/// The name of the header that names the host of the request.
const HOST: &str = "Host";

/// The name of the header that names the length of the body.
const CONTENT_LENGTH: &str = "Content-Length";

/// The name of the header that carries the token.
const AUTHORIZATION: &str = "Authorization";

/// The name of the header that a form in a browser cannot send.
const ADMIN_HEADER: &str = "X-Phair-Admin";

/// The value the header must carry.
const ADMIN_HEADER_VALUE: &str = "1";

/// The name of the header that carries the origin of the request.
const ORIGIN: &str = "Origin";

/// The name of the header that tells how the request was fetched.
const SEC_FETCH_SITE: &str = "Sec-Fetch-Site";

/// The name of the header that carries the "cross-site request forgery" token.
const CSRF_HEADER: &str = "X-Phair-Admin-Csrf";

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
    match tokio::time::timeout(guard.limits.read_timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => refused(StatusCode::REQUEST_TIMEOUT),
    }
}

/// The refusal of a request that names a host this surface does not answer to.
///
/// Only the loopback names are answered to. A name the attacker controls is
/// what the "DNS rebinding" attack needs (`SEC-TH-23`, `SEC-071`).
fn refuse_host<B>(request: &Request<B>) -> Option<Response> {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok());
    if is_loopback_host(host) {
        return None;
    }
    Some(refused(StatusCode::MISDIRECTED_REQUEST))
}

/// Whether a `Host` header names the loopback interface.
///
/// The name `localhost` and every address of the loopback interface are
/// accepted; everything else is refused. A request without a `Host` header is
/// refused as well: it says nothing about which server the client meant to
/// reach.
fn is_loopback_host(header: Option<&str>) -> bool {
    let Some(header) = header else {
        return false;
    };
    let host = host_of(header);
    if host.is_empty() {
        return false;
    }
    if let Ok(address) = host.parse::<std::net::IpAddr>() {
        return address.is_loopback();
    }
    host == "localhost"
}

/// The host part of a `Host` header, in lower case and without the port.
fn host_of(header: &str) -> String {
    let trimmed = header.trim();
    let without_port = match trimmed.rfind(':') {
        // A colon that belongs to an IPv6 literal is not a port separator.
        Some(position) if !trimmed[position..].contains(']') => &trimmed[..position],
        _ => trimmed,
    };
    without_port
        .trim_matches(|character| character == '[' || character == ']')
        .to_lowercase()
}

/// The refusal of a request whose framing the server does not accept.
fn refuse_framing<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
    let headers = header_pairs(request.headers());
    let body = body_bytes(&headers);
    match check(request_line_bytes(request), &headers, body, guard.limits()) {
        Ok(()) => None,
        Err(_) => Some(refused(StatusCode::BAD_REQUEST)),
    }
}

/// The headers of a request, as the pairs the core reads.
fn header_pairs(headers: &HeaderMap) -> HeaderPairs {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_owned(),
                value.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// The size of the body the request names, in bytes.
fn body_bytes(headers: &HeaderPairs) -> usize {
    values_of(headers, CONTENT_LENGTH)
        .iter()
        .filter_map(|length| length.trim().parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}

/// The size of the request line, in bytes.
fn request_line_bytes<B>(request: &Request<B>) -> usize {
    let method = request.method().as_str().len();
    let target = request
        .uri()
        .path_and_query()
        .map(|target| target.as_str().len())
        .unwrap_or(0);
    method + target + version_bytes(request.version()) + 2
}

/// The size of the version of a request, in bytes.
fn version_bytes(version: axum::http::Version) -> usize {
    match version {
        axum::http::Version::HTTP_2 | axum::http::Version::HTTP_3 => 2,
        _ => 8,
    }
}

/// The refusal of a request that comes from another origin.
///
/// A browser that sends a request from another page sends the origin of that
/// page. The surface answers only the requests of its own origin (`OPS-020`).
fn refuse_origin<B>(request: &Request<B>) -> Option<Response> {
    let origin = header_of(request, ORIGIN)?;
    if origin == "null" || !origin.starts_with("http://") {
        return Some(refused(StatusCode::FORBIDDEN));
    }
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let expected = format!("http://{host}");
    if origin.eq_ignore_ascii_case(&expected) {
        return None;
    }
    Some(refused(StatusCode::FORBIDDEN))
}

/// The refusal of a request that a browser fetched for another page.
///
/// The header tells where the request came from. A request that was fetched for
/// another site, or by a frame that is not the site itself, is refused
/// (`OPS-020`).
fn refuse_fetch_site<B>(request: &Request<B>) -> Option<Response> {
    let site = header_of(request, SEC_FETCH_SITE)?;
    if site == "same-origin" || site == "none" {
        return None;
    }
    Some(refused(StatusCode::FORBIDDEN))
}

/// The refusal of a request that does not carry the token.
fn refuse_token<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
    let Some(value) = header_of(request, AUTHORIZATION) else {
        return Some(unauthorised());
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Some(unauthorised());
    };
    if guard.token.matches(token) {
        return None;
    }
    Some(unauthorised())
}

/// The refusal of a request that does not carry the custom header.
///
/// A form in a browser cannot send a header the user did not choose, and a
/// page on another origin cannot send one without asking first — and the
/// surface answers no such question (`OPS-020`).
fn refuse_custom_header<B>(request: &Request<B>) -> Option<Response> {
    match header_of(request, ADMIN_HEADER) {
        Some(value) if value.trim() == ADMIN_HEADER_VALUE => None,
        _ => Some(refused(StatusCode::FORBIDDEN)),
    }
}

/// The refusal of a request that changes something without the token of the
/// "cross-site request forgery" defence.
///
/// A request that only reads needs no such token: it changes nothing. A
/// request that changes something carries the value that belongs to the
/// session, and a page of another origin cannot know it (`OPS-023`).
fn refuse_csrf<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
    if !changes_something(request.method()) {
        return None;
    }
    let Some(value) = header_of(request, CSRF_HEADER) else {
        return Some(refused(StatusCode::FORBIDDEN));
    };
    if guard.csrf_token() == value.trim() {
        return None;
    }
    Some(refused(StatusCode::FORBIDDEN))
}

/// Whether a method changes something on the server.
fn changes_something(method: &axum::http::Method) -> bool {
    matches!(
        *method,
        axum::http::Method::POST
            | axum::http::Method::PUT
            | axum::http::Method::PATCH
            | axum::http::Method::DELETE
    )
}

/// The value of a header of the request, when it is a text.
fn header_of<B>(request: &Request<B>, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_owned())
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

#[cfg(test)]
mod tests {
    use super::{host_of, is_loopback_host};

    /// The loopback names are the names of this machine.
    #[test]
    fn the_loopback_names_are_accepted() {
        assert!(is_loopback_host(Some("localhost")));
        assert!(is_loopback_host(Some("localhost:8210")));
        assert!(is_loopback_host(Some("127.0.0.1")));
        assert!(is_loopback_host(Some("127.0.0.1:8210")));
        assert!(is_loopback_host(Some("[::1]")));
        assert!(is_loopback_host(Some("[::1]:8210")));
    }

    /// The negative case: a name the attacker controls is refused.
    #[test]
    fn a_name_the_attacker_controls_is_refused() {
        assert!(!is_loopback_host(Some("attacker.example")));
        assert!(!is_loopback_host(Some("attacker.example:8210")));
        assert!(!is_loopback_host(Some("192.168.1.20")));
        assert!(!is_loopback_host(Some("phairdlna.local")));
    }

    /// The negative case: a request without a host is refused.
    #[test]
    fn a_request_without_a_host_is_refused() {
        assert!(!is_loopback_host(None));
        assert!(!is_loopback_host(Some("")));
    }

    /// The host is read without its port and in lower case.
    #[test]
    fn the_host_is_read_without_its_port() {
        assert_eq!(host_of("LocalHost:8210"), "localhost");
        assert_eq!(host_of("[::1]:8210"), "::1");
    }
}
