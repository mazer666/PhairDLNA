//! Framing, browser-origin and credential checks for each request.

use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use phairdlna_core::security::{check, values_of, HeaderPairs};

use super::{refused, unauthorised, AdminGuard};

/// The name of the header that names the request host.
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

/// The refusal of a request whose framing the server does not accept.
pub(super) fn refuse_framing<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
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
pub(super) fn refuse_origin<B>(request: &Request<B>) -> Option<Response> {
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
pub(super) fn refuse_fetch_site<B>(request: &Request<B>) -> Option<Response> {
    let site = header_of(request, SEC_FETCH_SITE)?;
    if site == "same-origin" || site == "none" {
        return None;
    }
    Some(refused(StatusCode::FORBIDDEN))
}

/// The refusal of a request that does not carry the token.
pub(super) fn refuse_token<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
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
pub(super) fn refuse_custom_header<B>(request: &Request<B>) -> Option<Response> {
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
pub(super) fn refuse_csrf<B>(guard: &AdminGuard, request: &Request<B>) -> Option<Response> {
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
