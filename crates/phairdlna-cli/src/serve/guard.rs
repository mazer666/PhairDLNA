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

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use phairdlna_core::security::{
    check, values_of, FramingError, HeaderPairs, HostPolicy, Limits, Redactor,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// The name of the header that names the host of the request.
const HOST: &str = "Host";

/// The name of the header that names the length of the body.
const CONTENT_LENGTH: &str = "Content-Length";

/// The rules every request of the daemon must pass.
#[derive(Clone)]
pub struct Guard {
    /// The names the server answers to.
    hosts: HostPolicy,
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
        let transfers = Arc::new(Semaphore::new(limits.transfers.max(1)));
        Self {
            hosts,
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
    if let Some(refusal) = refuse_host(&guard, &request) {
        return refusal;
    }
    if let Some(refusal) = refuse_framing(&guard, &request) {
        return refusal;
    }
    match tokio::time::timeout(guard.limits.read_timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => refused(StatusCode::REQUEST_TIMEOUT),
    }
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
mod tests {
    use super::{body_bytes, request_line_bytes, Guard, HeaderPairs};
    use axum::http::Request as HttpRequest;
    use phairdlna_core::security::{HostPolicy, Limits, Redactor};
    use std::sync::Arc;

    /// The guard of a server that answers to one name.
    fn guard() -> Guard {
        Guard::new(
            Limits::default(),
            HostPolicy::new(vec!["media.local".to_owned()]),
            Arc::new(Redactor::new()),
        )
    }

    /// A request without a host is refused.
    #[test]
    fn a_request_without_a_host_is_refused() {
        let request = HttpRequest::builder()
            .uri("/")
            .body(())
            .expect("a valid request");
        assert!(!guard().hosts.accepts(None));
        assert!(!request.headers().contains_key("Host"));
    }

    /// The size of the request line counts the method, the target and the
    /// version.
    #[test]
    fn the_request_line_is_measured() {
        let request = HttpRequest::builder()
            .method("GET")
            .uri("/description.xml")
            .body(())
            .expect("a valid request");
        assert_eq!(
            request_line_bytes(&request),
            "GET".len() + "/description.xml".len() + 8 + 2
        );
    }

    /// The size of the body is read from the headers.
    #[test]
    fn the_body_is_measured_from_the_headers() {
        let headers: HeaderPairs = vec![("Content-Length".to_owned(), "42".to_owned())];
        assert_eq!(body_bytes(&headers), 42);
    }

    /// A length that is not a number is no length.
    #[test]
    fn a_malformed_length_is_no_length() {
        let headers: HeaderPairs = vec![("Content-Length".to_owned(), "abc".to_owned())];
        assert_eq!(body_bytes(&headers), 0);
    }

    /// The transfers are bounded: the permit of the bound is the last one.
    #[test]
    fn the_transfers_are_bounded() {
        let limits = Limits {
            transfers: 2,
            ..Default::default()
        };
        let guard = Guard::new(limits, HostPolicy::default(), Arc::new(Redactor::new()));

        let first = guard.transfer_permit();
        let second = guard.transfer_permit();
        let third = guard.transfer_permit();

        assert!(first.is_some());
        assert!(second.is_some());
        assert!(third.is_none(), "the third transfer is refused");
    }

    /// A permit that is given back lets the next transfer in.
    #[test]
    fn a_permit_that_is_given_back_frees_a_place() {
        let limits = Limits {
            transfers: 1,
            ..Default::default()
        };
        let guard = Guard::new(limits, HostPolicy::default(), Arc::new(Redactor::new()));

        let first = guard.transfer_permit();
        assert!(guard.transfer_permit().is_none());
        drop(first);
        assert!(guard.transfer_permit().is_some());
    }
}
