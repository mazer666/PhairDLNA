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

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap as AxumHeaders, HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::{any, get, post};
use axum::Router;
use phairdlna_core::upnp::DeviceDescription;
use phairdlna_core::upnp::Dispatcher;
use phairdlna_core::upnp::ServiceSpec;

use super::guard::{self, Guard};
use super::library::Library;
use super::log::Logger;
use super::media::MediaLibrary;
use super::media_route::{artwork, media};

/// What the handlers of the daemon share.
pub struct ServerContext {
    /// The library of the daemon: the catalogue and the rules that keep it fresh.
    pub library: Library,
    /// The media of the library, and the roots they may not leave.
    pub media: MediaLibrary,
    /// The description of the device, with the base URL of this server.
    pub description: DeviceDescription,
    /// The base URL the documents are built with.
    pub base_url: String,
    /// Where the daemon writes what it has to say.
    pub logger: Logger,
    /// The rules every request must pass.
    pub guard: Arc<Guard>,
}

/// The content type of an XML answer.
const XML: &str = "text/xml; charset=\"utf-8\"";

/// Builds the routes of the daemon.
///
/// The service documents and the control endpoints are served at the paths the
/// device description advertises, so that a client that follows the
/// description finds them (`INT-021`, `INT-022`). The paths other servers use
/// are served as well, because a client may hard-code them.
///
/// **The rules come first.** The guard reads the request before a handler
/// sees it, and the body of a request is bounded before it is read: a request
/// that names another host, or that is too large, never reaches the library
/// (`SEC-011`, `SEC-071`).
pub fn router(context: Arc<ServerContext>) -> Router {
    let content = ServiceSpec::content_directory();
    let manager = ServiceSpec::connection_manager();
    let limits = context.guard.limits();
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
        .layer(DefaultBodyLimit::max(limits.body_bytes))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&context.guard),
            guard::enforce,
        ))
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
    let catalogue = context.library.catalogue();
    let dispatcher = Dispatcher::new(&*catalogue, &context.base_url);
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

/// Builds an answer that carries an XML document.
fn xml_response(document: String) -> Response {
    let mut response = Response::new(Body::from(document));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(XML),
    );
    response
}
