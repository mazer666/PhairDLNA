//! The media routes of the daemon.
//!
//! What: the two routes that deliver a media file — the file itself and the
//!       artwork that belongs to it.
//! Why: the delivery of a media file is the part of the server that touches the
//!       disk, and it has its own rules: a transfer takes a place out of the
//!       bound of the transfers, a range that the file cannot satisfy is
//!       refused, and a file that left its root is never opened (`SEC-012`,
//!       `SEC-001`, `INT-082`).
//! Example: `media` answers a request for a byte range of a file of the
//!       library.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap as AxumHeaders, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use phairdlna_core::http::headers::{GET_CONTENT_FEATURES, IF_RANGE, RANGE, TRANSFER_MODE};
use phairdlna_core::http::request::{MediaRequest, Method as CoreMethod};
use phairdlna_core::http::response::Body as CoreBody;
use phairdlna_core::media::cache_key;
use tokio::sync::OwnedSemaphorePermit;

use super::guard;
use super::http::ServerContext;
use super::stream::{FileStream, PermittedStream};

/// Answers a request for a media file.
///
/// A transfer takes a place out of the bound of the transfers, and gives it
/// back when the answer has been sent. A request that arrives when the bound
/// is reached is refused, and the client is told to come back later
/// (`SEC-012`).
pub(super) async fn media(
    State(context): State<Arc<ServerContext>>,
    Path(id): Path<String>,
    method: Method,
    headers: AxumHeaders,
) -> Response {
    let request = media_request(&id, &method, &headers);
    let answer = phairdlna_core::http::delivery::evaluate(&request, &context.media);
    let is_head = method == Method::HEAD;
    let permit = if is_head {
        None
    } else {
        context.guard.transfer_permit()
    };
    if !is_head && permit.is_none() {
        context.logger.warn(
            "a transfer was refused: too many transfers are running",
            &[],
        );
        return guard::too_many_transfers();
    }
    media_answer(&answer, &context, &id, is_head, permit).await
}

/// Answers a request for artwork.
///
/// The picture of an object comes from the library: the image that lies next to
/// the file, the cover that is embedded in it, or the thumbnail of a photo. A
/// file that has no picture gets a truthful `404` and never a placeholder that
/// a client would keep (`INT-065`, `F-10` § 6).
///
/// The picture is kept in the disposable cache, so that a client that asks
/// again is answered without reading the library again (`F-10` § 5).
pub(super) async fn artwork(
    State(context): State<Arc<ServerContext>>,
    Path(id): Path<String>,
) -> Response {
    let catalogue = context.library.catalogue();
    let Some(path) = catalogue.path_of(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let key = cache_key(&id, path);
    if let Some((mime_type, bytes)) = context.pictures.cache().get(&key) {
        return picture_response(&mime_type, bytes);
    }
    let Some(picture) = catalogue.picture_of(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if let Err(error) = context
        .pictures
        .cache()
        .put(&key, &picture.mime_type, &picture.bytes)
    {
        context.logger.warn(
            "a picture could not be kept in the cache",
            &[("reason", &error.to_string())],
        );
    }
    picture_response(&picture.mime_type, picture.bytes)
}

/// Builds the answer that carries a picture.
fn picture_response(mime_type: &str, bytes: Vec<u8>) -> Response {
    let mut response = Response::new(Body::from(bytes));
    if let Ok(value) = HeaderValue::from_str(mime_type) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_TYPE, value);
    }
    response
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
    permit: Option<OwnedSemaphorePermit>,
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
    let path = context.media.path_of(id);
    response = match answer.body {
        CoreBody::Whole => {
            with_file(
                response,
                path,
                0,
                answer.content_length,
                context,
                id,
                permit,
            )
            .await
        }
        CoreBody::Slice { offset, length } => {
            with_file(response, path, offset, length, context, id, permit).await
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
///
/// The path of the file is written to the log only as a pseudonym: the log
/// must not become a list of the files of the user (`SEC-094`).
async fn with_file(
    mut response: Response,
    path: Option<PathBuf>,
    offset: u64,
    length: u64,
    context: &ServerContext,
    id: &str,
    permit: Option<OwnedSemaphorePermit>,
) -> Response {
    let Some(path) = path else {
        return refused(response, context, id);
    };
    let mut stream = match FileStream::open(path.clone(), offset).await {
        Ok(stream) => stream,
        Err(error) => {
            let path = path.display().to_string();
            context.logger.warn(
                "a file could not be opened",
                &[("id", id), ("path", &path), ("reason", &error.to_string())],
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
    // The place of the transfer is given back when the answer has been sent.
    if let Some(permit) = permit {
        *response.body_mut() = Body::from_stream(PermittedStream::new(stream, permit));
    }
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
