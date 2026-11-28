//! The answers of the administration surface.
//!
//! What: the local API routes for server state, library scans, diagnostics and
//!       settings explanations, plus the JSON they answer with.
//! Why: the surface is small on purpose. It answers what a user of the server
//!       asks on the machine the server runs on, and nothing else
//!       (`OPS-020`, `F-14`).
//! Example: `GET /api/status` answers how long the server has been running and
//!          how many streams are running.

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use phairdlna_core::security::Redactor;

use crate::backend::{AdminBackend, Diagnostics, RescanOutcome};
use crate::guard::{self, AdminGuard};
use crate::json::Json;
mod status;
mod timeout;
use timeout::apply_request_timeout;

/// The largest body the surface reads. The surface has no body to read.
const BODY_BYTES: usize = 4096;

/// What the handlers of the surface share.
pub struct AdminState {
    /// The server the surface asks for its answers.
    backend: Arc<dyn AdminBackend>,
    /// The rules every request must pass.
    guard: Arc<AdminGuard>,
    /// The pseudonyms that replace the private values of an answer.
    ///
    /// The surface redacts what it renders, even when the server behind it
    /// forgot to: an answer that carries a path of the user is a leak
    /// (`SEC-094`).
    redactor: Arc<Redactor>,
}

impl AdminState {
    /// Builds the state of the surface.
    pub fn new(backend: Arc<dyn AdminBackend>, guard: Arc<AdminGuard>) -> Self {
        Self {
            backend,
            guard,
            redactor: Arc::new(Redactor::new()),
        }
    }
}

/// Builds the routes of the surface.
///
/// **The rules come first.** The guard reads the request before a handler sees
/// it, and the body of a request is bounded before it is read: a request that
/// names another host, or that carries no token, never reaches the server
/// (`SEC-071`, `OPS-020`).
pub fn router(state: Arc<AdminState>) -> Router {
    let router = Router::new()
        .route("/api/status", get(status))
        .route("/api/catalogue/rescan", post(rescan))
        .route("/api/diagnostics", get(diagnostics))
        .route("/api/config/explain", get(config_explain))
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(BODY_BYTES));
    apply_request_timeout(router, state.guard.limits().read_timeout)
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state.guard),
            guard::enforce,
        ))
        .with_state(state)
}

/// Answers the explanation and documentation of settings.
async fn config_explain(
    State(state): State<Arc<AdminState>>,
    request: axum::extract::Request,
) -> Response {
    let expert = request
        .uri()
        .query()
        .map(|q| q.contains("expert=1") || q.contains("expert=true"))
        .unwrap_or(false);
    let settings = phairdlna_core::config::schema::registry();
    let filtered: Vec<Json> = settings
        .into_iter()
        .filter(|s| expert || s.tier == phairdlna_core::config::schema::SettingTier::Standard)
        .map(|s| {
            Json::map(vec![
                ("key".to_owned(), Json::text(s.key)),
                ("label".to_owned(), Json::text(s.label)),
                ("tier".to_owned(), Json::text(s.tier.as_str())),
                ("summary".to_owned(), Json::text(s.summary)),
                ("tooltip".to_owned(), Json::text(s.tooltip)),
                ("default_value".to_owned(), Json::text(s.default_value)),
            ])
        })
        .collect();
    json_response(Json::list(filtered), &state.guard)
}

/// Answers the state of the server.
async fn status(State(state): State<Arc<AdminState>>) -> Response {
    let status = state.backend.status();
    json_response(
        status::json_of_status(&status, &state.redactor),
        &state.guard,
    )
}

/// Answers the scan of the library.
async fn rescan(State(state): State<Arc<AdminState>>) -> Response {
    let outcome = state.backend.rescan();
    json_response(json_of_rescan(&outcome), &state.guard)
}

/// Answers what the server has to say about itself.
async fn diagnostics(State(state): State<Arc<AdminState>>) -> Response {
    let diagnostics = state.backend.diagnostics();
    json_response(
        json_of_diagnostics(&diagnostics, &state.redactor),
        &state.guard,
    )
}

/// The answer to a request the surface does not know.
async fn not_found(method: Method) -> Response {
    if method == Method::OPTIONS {
        // The surface answers no question of another origin, and it never
        // answers a question about the permissions it would need.
        return refused(StatusCode::METHOD_NOT_ALLOWED);
    }
    refused(StatusCode::NOT_FOUND)
}

/// The scan of the library, as JSON.
fn json_of_rescan(outcome: &RescanOutcome) -> Json {
    Json::map(vec![
        ("rescanned".to_owned(), Json::flag(outcome.rescanned)),
        ("objects".to_owned(), Json::uint(outcome.objects as u64)),
        (
            "duration_seconds".to_owned(),
            Json::uint(outcome.duration_seconds),
        ),
        ("message".to_owned(), Json::text(outcome.message.clone())),
    ])
}

/// The network and recovery fields of the diagnostics (LIFE-009).
fn diagnostics_network_fields(d: &Diagnostics, redactor: &Redactor) -> Vec<(String, Json)> {
    vec![
        (
            "effective_interface".to_owned(),
            d.effective_interface
                .as_ref()
                .map(|i| Json::text(i.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "effective_address".to_owned(),
            d.effective_address
                .as_ref()
                .map(|a| Json::text(redactor.value(a)))
                .unwrap_or(Json::Null),
        ),
        (
            "listener_state".to_owned(),
            d.listener_state
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "last_recovery_event".to_owned(),
            d.last_recovery_event
                .as_ref()
                .map(|e| Json::text(redactor.value(e)))
                .unwrap_or(Json::Null),
        ),
    ]
}

/// The diagnostics of the server, as JSON.
fn json_of_diagnostics(diagnostics: &Diagnostics, redactor: &Redactor) -> Json {
    let mut fields = core_diagnostics_fields(diagnostics, redactor);
    let warnings = diagnostics
        .warnings
        .iter()
        .map(|w| Json::text(w.clone()))
        .collect();
    fields.push(("warnings".to_owned(), Json::list(warnings)));
    fields.push((
        "resources".to_owned(),
        diagnostics
            .resources
            .as_ref()
            .map(|res| Json::text(res.clone()))
            .unwrap_or(Json::Null),
    ));
    fields.push((
        "timeline".to_owned(),
        diagnostics
            .timeline
            .as_ref()
            .map(|tl| Json::text(tl.clone()))
            .unwrap_or(Json::Null),
    ));
    fields.push((
        "last_scan".to_owned(),
        diagnostics
            .last_scan
            .as_ref()
            .map(|s| Json::text(s.clone()))
            .unwrap_or(Json::Null),
    ));
    fields.extend(diagnostics_network_fields(diagnostics, redactor));
    Json::map(fields)
}

/// The core status fields of the diagnostics.
fn core_diagnostics_fields(d: &Diagnostics, redactor: &Redactor) -> Vec<(String, Json)> {
    vec![
        ("version".to_owned(), Json::text(d.version.clone())),
        (
            "source_revision".to_owned(),
            Json::text(d.source_revision.clone()),
        ),
        ("uptime_seconds".to_owned(), Json::uint(d.uptime_seconds)),
        (
            "catalogue_objects".to_owned(),
            Json::uint(d.catalogue_objects as u64),
        ),
        (
            "resident_bytes".to_owned(),
            Json::maybe_uint(d.resident_bytes),
        ),
        (
            "system_update_id".to_owned(),
            Json::uint(u64::from(d.system_update_id)),
        ),
        (
            "roots".to_owned(),
            status::json_of_roots(&d.roots, redactor),
        ),
        (
            "transfers".to_owned(),
            Json::list(d.transfers.iter().map(json_of_transfer).collect()),
        ),
        ("records".to_owned(), json_of_records(&d.records, redactor)),
    ]
}

/// The records of the log, as JSON.
fn json_of_records(records: &[crate::backend::LogRecord], redactor: &Redactor) -> Json {
    Json::list(
        records
            .iter()
            .map(|record| json_of_record(record, redactor))
            .collect(),
    )
}

/// One stream that is running, as JSON.
fn json_of_transfer(transfer: &crate::backend::TransferInfo) -> Json {
    Json::map(vec![
        ("client".to_owned(), Json::text(transfer.client.clone())),
        ("address".to_owned(), Json::text(transfer.address.clone())),
        ("title".to_owned(), Json::text(transfer.title.clone())),
        (
            "running_seconds".to_owned(),
            Json::uint(transfer.running_seconds),
        ),
    ])
}

/// One record of the log, as JSON.
fn json_of_record(record: &crate::backend::LogRecord, redactor: &Redactor) -> Json {
    Json::map(vec![
        ("time".to_owned(), Json::text(record.time.clone())),
        ("level".to_owned(), Json::text(record.level.clone())),
        ("component".to_owned(), Json::text(record.component.clone())),
        (
            "message".to_owned(),
            Json::text(redactor.value(&record.message)),
        ),
        (
            "fields".to_owned(),
            Json::map(
                record
                    .fields
                    .iter()
                    .map(|(name, value)| (name.clone(), Json::text(redactor.value(value))))
                    .collect(),
            ),
        ),
    ])
}

/// The answer that carries a JSON document.
///
/// The answer carries the token of the "cross-site request forgery" defence:
/// a client reads it and sends it back with the request that changes
/// something. A client of another origin cannot read it, because the surface
/// answers no question of another origin (`OPS-023`).
fn json_response(value: Json, guard: &AdminGuard) -> Response {
    let mut response = value.to_string().into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json; charset=\"utf-8\""),
    );
    if let Ok(value) = HeaderValue::from_str(&guard.csrf_token()) {
        response.headers_mut().insert(
            axum::http::HeaderName::from_static("x-phairdlna-admin-csrf"),
            value,
        );
    }
    response
}

/// The answer of a request the surface does not answer.
fn refused(status: StatusCode) -> Response {
    let mut response = status.into_response();
    *response.body_mut() = axum::body::Body::from("The request was refused.");
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; charset=\"utf-8\""),
    );
    response
}
