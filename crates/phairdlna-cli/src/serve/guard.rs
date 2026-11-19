//! The guard of the daemon: the rules every request must pass.
//!
//! What: the check of the `Host` header, the check of the framing of the
//!       request, the deadline of a request and the bound of the transfers.
//! Why: the core owns the rules and the socket code owns the sockets
//!       (`ADR-0001`). A request that names another host, or that carries a
//!       body the server did not expect, must be refused **before** it reaches
//!       a handler, and the refusal must say nothing about the state of the
//!       server (`SEC-011`, `SEC-012`, `SEC-013`, `SEC-016`, `SEC-071`).
//! Example: a request that carries `Host: attacker.example` is refused with
//!       `421`, and the answer carries no path and no name.
//!
//! **What the refusals say.** Every refusal is the same sentence: the server
//! says that the request was refused and nothing else. A client that probes
//! the server learns that the server is there, and nothing about the file
//! system, the configuration or the versions of the software (`SEC-013`).

use std::sync::Arc;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use phairdlna_core::security::{
    check, values_of, ClientAccess, FramingError, HeaderPairs, HostPolicy, Limits, Redactor,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use super::limit::PeerAddress;

/// The name of the header that names the host of the request.
const HOST: &str = "Host";

/// The name of the header that names the length of the body.
const CONTENT_LENGTH: &str = "Content-Length";

/// The rules every request of the daemon must pass.
#[derive(Clone)]
pub struct Guard {
    /// The names the server answers to.
    hosts: HostPolicy,
    /// The clients that may talk to the server.
    access: ClientAccess,
    /// The bounds of the requests and of the transfers.
    limits: Limits,
    /// The pseudonyms that replace the private values of a log record.
    redactor: Arc<Redactor>,
    /// The transfers that may run at the same time.
    transfers: Arc<Semaphore>,
}

impl Guard {
    /// Builds the guard of a server.
    ///
    /// The rules come from the core, the names from the configuration, and the
    /// pseudonyms from the log: the guard and the log must name the same path
    /// with the same pseudonym (`SEC-094`).
    pub fn new(limits: Limits, hosts: HostPolicy, redactor: Arc<Redactor>) -> Self {
        Self::with_access(limits, hosts, ClientAccess::default(), redactor)
    }

    /// Builds the guard of a server that knows the clients that may talk to it.
    ///
    /// The list is the one of the configuration: an empty list admits every
    /// client, and a client that is not on the list is refused before it
    /// reaches a handler (`CFG-009`).
    pub fn with_access(
        limits: Limits,
        hosts: HostPolicy,
        access: ClientAccess,
        redactor: Arc<Redactor>,
    ) -> Self {
        let transfers = Arc::new(Semaphore::new(limits.transfers.max(1)));
        Self {
            hosts,
            access,
            limits,
            redactor,
            transfers,
        }
    }

    /// The bounds of the requests.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// The pseudonyms that replace the private values of a log record.
    pub fn redactor(&self) -> &Arc<Redactor> {
        &self.redactor
    }

    /// The permit of a transfer, when the bound of the transfers is not reached.
    ///
    /// The permit belongs to the transfer: it is given back when the answer
    /// has been sent, and not before (`SEC-012`).
    pub fn transfer_permit(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.transfers).try_acquire_owned().ok()
    }
}

/// Refuses the requests that must not reach a handler.
///
/// The order of the checks is the order of the cost: the name of the host is
/// read first, because it is the cheapest and the most important.
pub async fn enforce(State(guard): State<Arc<Guard>>, request: Request, next: Next) -> Response {
    if let Some(refusal) = refuse_client(&guard, &request) {
        return refusal;
    }
    if let Some(refusal) = refuse_host(&guard, &request) {
        return refusal;
    }
    if let Some(refusal) = refuse_framing(&guard, &request) {
        return refusal;
    }
    next.run(request).await
}

/// The refusal of a request of a client that may not talk to the server.
///
/// The address of the client comes from the socket, and not from a header: a
/// header is written by the client, and a client that is not allowed may write
/// whatever it likes (`CFG-009`).
fn refuse_client<B>(guard: &Guard, request: &Request<B>) -> Option<Response> {
    if guard.access.is_empty() {
        return None;
    }
    let address = peer_address(request)?;
    if guard.access.admits(&address.ip()) {
        return None;
    }
    Some(refused(StatusCode::FORBIDDEN))
}

/// The address of the client that sent the request.
fn peer_address<B>(request: &Request<B>) -> Option<std::net::SocketAddr> {
    request
        .extensions()
        .get::<ConnectInfo<PeerAddress>>()
        .map(|info| info.0 .0)
}

/// The refusal of a request that names a host this server does not answer to.
///
/// A request without a `Host` header is refused as well: it says nothing
/// about which server the client meant to reach (`SEC-071`).
fn refuse_host<B>(guard: &Guard, request: &axum::http::Request<B>) -> Option<Response> {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok());
    if guard.hosts.accepts(host) {
        return None;
    }
    Some(refused(StatusCode::MISDIRECTED_REQUEST))
}

/// The refusal of a request whose framing the server does not accept.
///
/// The request is refused when its line is too long, when it carries too many
/// headers, when its headers are too large, when its body is too large, or
/// when its headers name the length of the body in two ways (`SEC-011`,
/// `SEC-TH-12`, `SEC-TH-13`).
fn refuse_framing<B>(guard: &Guard, request: &axum::http::Request<B>) -> Option<Response> {
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

/// The size of the request line, in bytes.
///
/// The line is the method, the target and the version. The server reads the
/// line before it reads anything else, so its size is the first bound the
/// server keeps (`SEC-011`).
fn request_line_bytes<B>(request: &axum::http::Request<B>) -> usize {
    let method = request.method().as_str().len();
    let target = request
        .uri()
        .path_and_query()
        .map(|target| target.as_str().len())
        .unwrap_or(0);
    method + target + version_bytes(request.version()) + 2
}

/// The size of the version of a request, in bytes.
///
/// `HTTP/1.1` is eight characters; the versions of HTTP/2 and HTTP/3 are two.
fn version_bytes(version: axum::http::Version) -> usize {
    match version {
        axum::http::Version::HTTP_2 | axum::http::Version::HTTP_3 => 2,
        _ => 8,
    }
}

/// The size of the body the request names, in bytes.
///
/// The length is read from the headers, because the body has not been read
/// yet. A length that is not a number counts as no length at all: the check
/// of the framing refuses the request anyway.
fn body_bytes(headers: &HeaderPairs) -> usize {
    values_of(headers, CONTENT_LENGTH)
        .iter()
        .filter_map(|length| length.trim().parse::<usize>().ok())
        .max()
        .unwrap_or(0)
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

/// The answer of a transfer that the bound of the transfers refuses.
///
/// The answer names the bound and asks the client to come back later: a
/// client that is told to wait does wait (`SEC-012`).
pub fn too_many_transfers() -> Response {
    let mut response = refused(StatusCode::SERVICE_UNAVAILABLE);
    response.headers_mut().insert(
        axum::http::header::RETRY_AFTER,
        axum::http::HeaderValue::from_static("5"),
    );
    response
}

/// The reason of a refusal, for the log.
///
/// The reason is a word, not a sentence about the state of the server: the log
/// says what happened, and the answer says nothing at all (`SEC-013`).
pub fn reason_of(error: FramingError) -> &'static str {
    error.reason()
}

#[cfg(test)]
#[path = "guard/tests.rs"]
mod tests;
