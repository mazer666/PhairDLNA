//! The HTTP server of the daemon.
//!
//! What: the routes the daemon serves — the device description, the two
//!       service descriptions, the two control endpoints, the media and the
//!       artwork.
//! Why: the daemon is what a client talks to. The routes are the contract of
//!       the specification (`F-02`, `F-03`, `F-04`, `F-06`), and every one of
//!       them is answered by the core, so that the rules live in one place and
//!       not in the socket code (`INT-090`).
//! Example: `GET /description.xml` answers the document a client reads to learn
//!       what the server is.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap as AxumHeaders, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::Router;
use phairdlna_core::catalogue::Catalogue;
use phairdlna_core::http::headers::{GET_CONTENT_FEATURES, IF_RANGE, RANGE, TRANSFER_MODE};
use phairdlna_core::http::request::{MediaRequest, Method as CoreMethod};
use phairdlna_core::http::response::Body as CoreBody;
use phairdlna_core::upnp::DeviceDescription;
use phairdlna_core::upnp::Dispatcher;
use phairdlna_core::upnp::ServiceSpec;

use super::log::Logger;
use super::media::MediaLibrary;
use super::stream::FileStream;

/// What the handlers of the daemon share.
pub struct ServerContext {
    /// The catalogue of the library.
    pub catalogue: Arc<Catalogue>,
    /// The media of the library, and the roots they may not leave.
    pub library: MediaLibrary,
    /// The description of the device, with the base URL of this server.
    pub description: DeviceDescription,
    /// The base URL the documents are built with.
    pub base_url: String,
    /// Where the daemon writes what it has to say.
    pub logger: Logger,
}

/// The content type of an XML answer.
const XML: &str = "text/xml; charset=\"utf-8\"";

/// Builds the routes of the daemon.
///
/// The service documents and the control endpoints are served at the paths the
/// device description advertises, so that a client that follows the
/// description finds them (`INT-021`, `INT-022`). The paths other servers use
/// are served as well, because a client may hard-code them.
pub fn router(context: Arc<ServerContext>) -> Router {
    let content = ServiceSpec::content_directory();
    let manager = ServiceSpec::connection_manager();
    Router::new()
        .route("/description.xml", get(description))
        .route(content.definition().scpd_path, get(content_directory_scpd))
        .route(manager.definition().scpd_path, get(connection_manager_scpd))
        .route(
            content.definition().control_path,
            post(content_directory_control),
        )
        .route(
            manager.definition().control_path,
            post(connection_manager_control),
        )
        .route("/ContentDirectory/scpd.xml", get(content_directory_scpd))
        .route("/ConnectionManager/scpd.xml", get(connection_manager_scpd))
        .route("/ContentDirectory/control", post(content_directory_control))
        .route(
            "/ConnectionManager/control",
            post(connection_manager_control),
        )
        .route(content.definition().event_path, any(event_subscription))
        .route(manager.definition().event_path, any(event_subscription))
        .route("/media/{id}", get(media).head(media))
        .route("/art/{id}", get(artwork))
        .with_state(context)
}

/// Answers the description of the device.
async fn description(State(context): State<Arc<ServerContext>>) -> Response {
    xml_response(context.description.to_xml())
}

/// Answers the description of the ContentDirectory service.
async fn content_directory_scpd() -> Response {
    xml_response(ServiceSpec::content_directory().to_xml())
}

/// Answers the description of the ConnectionManager service.
async fn connection_manager_scpd() -> Response {
    xml_response(ServiceSpec::connection_manager().to_xml())
}

/// Answers a control request of the ContentDirectory service.
async fn content_directory_control(
    State(context): State<Arc<ServerContext>>,
    headers: AxumHeaders,
    body: String,
) -> Response {
    control_response(&context, &headers, &body, ServiceSpec::content_directory())
}

/// Answers a control request of the ConnectionManager service.
async fn connection_manager_control(
    State(context): State<Arc<ServerContext>>,
    headers: AxumHeaders,
    body: String,
) -> Response {
    control_response(&context, &headers, &body, ServiceSpec::connection_manager())
}

/// Answers a control request with the SOAP dispatcher of the core.
///
/// The path decides which service the client thinks it is talking to. A
/// request for another service is refused here, so that a service never
/// answers on the address of another one.
fn control_response(
    context: &ServerContext,
    headers: &AxumHeaders,
    body: &str,
    service: ServiceSpec,
) -> Response {
    let action = headers
        .get("SOAPACTION")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim_matches('"'));
    if !request_belongs_to(body, &service) {
        context.logger.warn(
            "a control request was sent to the wrong service",
            &[("service", service.definition().service_type)],
        );
        return fault_response(StatusCode::BAD_REQUEST);
    }
    let dispatcher = Dispatcher::new(context.catalogue.as_ref(), &context.base_url);
    let answer = dispatcher.dispatch(body, action);
    let status = StatusCode::from_u16(answer.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = Response::new(Body::from(answer.body));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(XML),
    );
    response
}

/// Whether the request carries the service it is sent to.
///
/// The envelope names the service in its namespace; a request of another
/// service does not belong to this address.
fn request_belongs_to(body: &str, service: &ServiceSpec) -> bool {
    match phairdlna_core::upnp::parse_request(body) {
        Ok(request) => request.service_type == service.definition().service_type,
        // A body the parser refuses is refused by the dispatcher, which knows
        // how to answer with a fault.
        Err(_) => true,
    }
}

/// Answers a request for an event subscription.
///
/// The eventing is not implemented (it is the work of M2). The address is
/// answered, so that a client that follows the description finds a real
/// answer instead of a `404`, and the answer says plainly that the feature is
/// not there yet.
async fn event_subscription() -> Response {
    let mut response = Response::new(Body::from("The event subscription is not implemented yet."));
    *response.status_mut() = StatusCode::NOT_IMPLEMENTED;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=\"utf-8\""),
    );
    response
}

/// Builds an answer that carries a fault of the protocol.
fn fault_response(status: StatusCode) -> Response {
    let body = phairdlna_core::upnp::fault_body(500);
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(XML),
    );
    response
}

/// Answers a request for a media file.
async fn media(
    State(context): State<Arc<ServerContext>>,
    Path(id): Path<String>,
    method: Method,
    headers: AxumHeaders,
) -> Response {
    let request = media_request(&id, &method, &headers);
    let answer = phairdlna_core::http::delivery::evaluate(&request, &context.library);
    media_answer(&answer, &context, &id, method == Method::HEAD).await
}

/// Answers a request for artwork.
///
/// The catalogue has no artwork yet (F-11 is the work of M2), so the answer is
/// `404` — and never a placeholder that a client would cache (`INT-065`).
async fn artwork(Path(_id): Path<String>) -> Response {
    StatusCode::NOT_FOUND.into_response()
}

/// Builds the request the delivery layer of the core reads.
fn media_request(id: &str, method: &Method, headers: &AxumHeaders) -> MediaRequest {
    let core_method = match *method {
        Method::HEAD => CoreMethod::Head,
        _ => CoreMethod::Get,
    };
    let mut request = MediaRequest::new(core_method, format!("/media/{id}"), Default::default());
    for name in [RANGE, IF_RANGE, TRANSFER_MODE, GET_CONTENT_FEATURES] {
        if let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) {
            request.headers.set(name, value);
        }
    }
    request
}

/// Turns the answer of the delivery layer into an HTTP answer.
///
/// The file is resolved once, for every method: a file that left the root is
/// refused the same way for a `GET` and for a `HEAD` (`INT-098`).
async fn media_answer(
    answer: &phairdlna_core::http::response::MediaResponse,
    context: &ServerContext,
    id: &str,
    is_head: bool,
) -> Response {
    let mut response = Response::new(Body::empty());
    *response.status_mut() =
        StatusCode::from_u16(answer.status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    for (name, value) in answer.headers.iter() {
        if let (Ok(name), Ok(value)) = (
            axum::http::HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            response.headers_mut().insert(name, value);
        }
    }
    let path = context.library.path_of(id);
    response = match answer.body {
        CoreBody::Whole => with_file(response, path, 0, answer.content_length, context, id).await,
        CoreBody::Slice { offset, length } => {
            with_file(response, path, offset, length, context, id).await
        }
        CoreBody::Text(text) => {
            if !is_head {
                *response.body_mut() = Body::from(text);
            }
            response
        }
        CoreBody::None => match path {
            // The answer is a `HEAD`: the file is inside a root, and the
            // length of the answer is the one of the catalogue.
            Some(_) => response,
            None => refused(response, context, id),
        },
    };
    response
}

/// Puts the bytes of a file into the answer.
///
/// The length of the catalogue may be older than the file on the disk: a file
/// may have grown, or shrunk, since the scan. The answer then carries the
/// length the file really has, and its `Content-Range` describes the bytes
/// that are really sent (`INT-081`, `INT-088`).
async fn with_file(
    mut response: Response,
    path: Option<PathBuf>,
    offset: u64,
    length: u64,
    context: &ServerContext,
    id: &str,
) -> Response {
    let Some(path) = path else {
        return refused(response, context, id);
    };
    let mut stream = match FileStream::open(path, offset).await {
        Ok(stream) => stream,
        Err(error) => {
            context.logger.warn(
                "a file could not be opened",
                &[("id", id), ("reason", &error.to_string())],
            );
            return refused(response, context, id);
        }
    };
    let size = stream.size();
    if offset >= size {
        // The client asks for bytes that the file does not have: the range
        // cannot be satisfied (`INT-082`).
        context.logger.warn(
            "a range is behind the end of the file",
            &[("id", id), ("size", &size.to_string())],
        );
        return unsatisfiable(response, size);
    }
    let served = length.min(size - offset);
    stream.limit(served);
    set_header(
        &mut response,
        axum::http::header::CONTENT_LENGTH,
        &served.to_string(),
    );
    if response.status() == StatusCode::PARTIAL_CONTENT {
        set_header(
            &mut response,
            axum::http::header::CONTENT_RANGE,
            &format!("bytes {}-{}/{}", offset, offset + served - 1, size),
        );
    }
    *response.body_mut() = Body::from_stream(stream);
    response
}

/// The answer for a file that may not be read.
fn refused(mut response: Response, context: &ServerContext, id: &str) -> Response {
    context.logger.warn("a file was refused", &[("id", id)]);
    *response.status_mut() = StatusCode::NOT_FOUND;
    // The answer of an error carries no promise of a stream (`INT-089`).
    for name in [
        axum::http::header::CONTENT_TYPE,
        axum::http::header::ACCEPT_RANGES,
    ] {
        response.headers_mut().remove(name);
    }
    response.headers_mut().remove(TRANSFER_MODE);
    response.headers_mut().remove("transfermode.dlna.org");
    response.headers_mut().remove("realtimeinfo.dlna.org");
    response
        .headers_mut()
        .remove(axum::http::header::CONTENT_LENGTH);
    *response.body_mut() = Body::empty();
    response
}

/// The answer for a range that the file cannot satisfy.
fn unsatisfiable(mut response: Response, size: u64) -> Response {
    *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
    set_header(
        &mut response,
        axum::http::header::CONTENT_RANGE,
        &format!("bytes */{size}"),
    );
    *response.body_mut() = Body::empty();
    response
}

/// Sets a header of an answer.
fn set_header(response: &mut Response, name: axum::http::HeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response.headers_mut().insert(name, value);
    }
}

/// Builds an answer that carries an XML document.
fn xml_response(document: String) -> Response {
    let mut response = Response::new(Body::from(document));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(XML),
    );
    response
}
