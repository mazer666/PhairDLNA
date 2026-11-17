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
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap as AxumHeaders, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use phairdlna_core::catalogue::ContentSource;
use phairdlna_core::http::headers::{GET_CONTENT_FEATURES, IF_RANGE, RANGE, TRANSFER_MODE};
use phairdlna_core::http::request::{MediaRequest, Method as CoreMethod};
use phairdlna_core::http::response::Body as CoreBody;
use phairdlna_core::media::cache_key;
use phairdlna_core::scheduler::Priority;
use tokio::sync::OwnedSemaphorePermit;

use futures_core::Stream;

use super::guard;
use super::http::ServerContext;
use super::resources::StreamToken;
use super::stream::{FileStream, PermittedStream};
use super::transfers::{TrackedStream, TransferHandle};

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
    let started = Instant::now();
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
    // The transfer is registered before the file is opened: the status of the
    // server must show a stream that is running, even when the file cannot be
    // read (`UX-012`).
    let transfer = context
        .transfers
        .start(&client_of(&headers), &title_of(&context, &id));
    // The transfer takes a place out of the resources of the daemon, so that
    // the scheduler knows that a film is playing (`RES-001`).
    let token = context.resources.stream_started();
    let response = media_answer(&answer, &context, &id, is_head, permit, transfer, token).await;
    // The time the server needed to prepare the answer is a hint about the
    // machine: the scheduler uses it to decide whether the background work
    // has to yield (`RES-003`).
    context
        .resources
        .note_latency(started.elapsed().as_millis() as u64);
    response
}

/// The name of the client that asks for the file.
///
/// The name comes from the header the client sends. A client that sends no
/// name is called *unknown*: the server does not guess.
fn client_of(headers: &AxumHeaders) -> String {
    headers
        .get("User-Agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

/// The title of the object the client asks for.
fn title_of(context: &ServerContext, id: &str) -> String {
    context
        .library
        .with_catalogue(|catalogue| catalogue.item_of(id))
        .map(|item| item.title)
        .unwrap_or_else(|| id.to_owned())
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
    artwork_body(&context, &id).await
}

/// The body of the answer for the artwork.
///
/// The lock of the catalogue is held only for the short moment in which the
/// path of the object and the cached picture are looked up. The picture
/// itself is produced without the lock, because the production reads the file
/// of the library and may take a while.
async fn artwork_body(context: &ServerContext, id: &str) -> Response {
    let (key, cached) = {
        let catalogue = context.library.catalogue();
        // The path is copied, so that the lock is not held while the picture
        // is produced.
        let Some(path) = catalogue.path_of(id).map(|path| path.to_path_buf()) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let key = cache_key(id, &path);
        let cached = context.pictures.cache().get(&key);
        (key, cached)
    };
    if let Some((mime_type, bytes)) = cached {
        return picture_response(&mime_type, bytes);
    }
    let picture = match produce_picture(context, id).await {
        Ok(picture) => picture,
        Err(error) => {
            context.logger.warn(
                "a picture could not be produced",
                &[("reason", &error.to_string())],
            );
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    let Some(picture) = picture else {
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

/// Produces the picture of an object.
///
/// The production reads the file of the library and encodes a thumbnail, so it
/// runs in the metadata class of the scheduler: a client waits a moment for a
/// cover, but not for the scan of the library (`RES-001`).
async fn produce_picture(
    context: &ServerContext,
    id: &str,
) -> Result<Option<phairdlna_core::media::Artwork>, super::resources::TaskError> {
    let library = context.library.clone();
    let target = id.to_owned();
    context
        .resources
        .run(Priority::Media, move || {
            library.with_catalogue(|catalogue| catalogue.picture_of(&target))
        })
        .await
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
    transfer: TransferHandle,
    token: StreamToken,
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
                Delivery {
                    path,
                    offset: 0,
                    length: answer.content_length,
                    id: id.to_owned(),
                    permit,
                    transfer,
                    token: Some(token),
                },
                context,
            )
            .await
        }
        CoreBody::Slice { offset, length } => {
            with_file(
                response,
                Delivery {
                    path,
                    offset,
                    length,
                    id: id.to_owned(),
                    permit,
                    transfer,
                    token: Some(token),
                },
                context,
            )
            .await
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

/// What the answer of a media request needs to deliver the file.
struct Delivery {
    /// The path of the file, when the catalogue knows it.
    path: Option<PathBuf>,
    /// The first byte of the answer.
    offset: u64,
    /// The number of bytes of the answer.
    length: u64,
    /// The identifier of the object that is played.
    id: String,
    /// The place of the transfer, when the transfer may run.
    permit: Option<OwnedSemaphorePermit>,
    /// The transfer that the answer carries.
    transfer: TransferHandle,
    /// The place of the transfer in the resources of the daemon.
    token: Option<StreamToken>,
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
    delivery: Delivery,
    context: &ServerContext,
) -> Response {
    let Delivery {
        path,
        offset,
        length,
        id,
        permit,
        transfer,
        token,
    } = delivery;
    let Some(path) = path else {
        return refused(response, context, &id);
    };
    let mut stream = match FileStream::open(path.clone(), offset).await {
        Ok(stream) => stream,
        Err(error) => {
            let path = path.display().to_string();
            context.logger.warn(
                "a file could not be opened",
                &[("id", &id), ("path", &path), ("reason", &error.to_string())],
            );
            return refused(response, context, &id);
        }
    };
    let size = stream.size();
    if offset >= size {
        // The client asks for bytes that the file does not have: the range
        // cannot be satisfied (`INT-082`).
        context.logger.warn(
            "a range is behind the end of the file",
            &[("id", &id), ("size", &size.to_string())],
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
    // The place of the transfer is given back when the answer has been sent,
    // and the transfer ends with it (`SEC-012`, `UX-012`).
    if let Some(permit) = permit {
        let stream = PermittedStream::new(stream, permit);
        let tracked = TrackedStream::new(stream, transfer);
        // The token of the stream ends the place of the transfer when the
        // answer has been sent (`RES-004`).
        let counted = match token {
            Some(token) => CountedStream::new(tracked, token),
            None => return response,
        };
        *response.body_mut() = Body::from_stream(counted);
    }
    response
}

/// A stream that tells the resources of the daemon when it ends.
///
/// The place of the transfer in the resources is given back with the stream,
/// so that a client that goes away leaves nothing behind (`RES-004`).
struct CountedStream<S> {
    /// The bytes of the answer.
    inner: TrackedStream<S>,
    /// The place of the transfer.
    _token: StreamToken,
}

impl<S> CountedStream<S> {
    /// Builds a stream that carries the place of the transfer.
    fn new(inner: TrackedStream<S>, token: StreamToken) -> Self {
        Self {
            inner,
            _token: token,
        }
    }
}

impl<S: Stream + Unpin> Stream for CountedStream<S> {
    type Item = S::Item;

    /// Reads the next piece of the answer.
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<S::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(context)
    }
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
