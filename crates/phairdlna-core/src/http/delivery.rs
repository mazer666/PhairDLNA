//! The evaluator of a media request.
//!
//! What: the function that turns a request into an answer.
//! Why: the evaluator is pure. It reads no socket and no file, it only
//!       decides what the answer is: the status, the headers and the slice
//!       of the file the caller has to read. That makes every rule of the
//!       media layer testable without a network and without a disk
//!       (ARCHITECTURE §4.7), and it keeps the rules in one place.
//! Example: `evaluate` answers a `GET` with a range as `206` and a slice of
//!       the file to read.

use crate::http::headers::{
    TransferMode, ETAG, GET_CONTENT_FEATURES, GET_CONTENT_FEATURES_VALUE, IF_RANGE, RANGE,
    TRANSFER_MODE,
};
use crate::http::path::{self, Target};
use crate::http::range::RangeRequest;
use crate::http::request::{MediaRequest, Method};
use crate::http::response::{
    error, media_response, method_not_allowed, unsatisfiable, Body, MediaResponse, Status,
};
use crate::http::source::{MediaObject, MediaSource};

/// Answers a media request.
///
/// The answer is the status, the headers and the slice of the file the caller
/// has to read; the caller reads the bytes and writes them to the socket.
pub fn evaluate(request: &MediaRequest, source: &dyn MediaSource) -> MediaResponse {
    let identifier = match path::from_target(&request.target) {
        Target::Media(identifier) => identifier,
        Target::Malformed => return error(Status::BadRequest, request),
        Target::NotMedia => return error(Status::NotFound, request),
    };
    if !matches!(request.method, Method::Get | Method::Head) {
        return method_not_allowed(request);
    }
    // A request that carries a header the server cannot read is refused
    // before the catalogue is touched: the request itself is broken
    // (INT-097).
    if !content_features_request_is_valid(request) {
        return error(Status::BadRequest, request);
    }
    let Some(object) = source.metadata(identifier.as_str()) else {
        return error(Status::NotFound, request);
    };
    if !transfer_mode_is_served(request, &object) {
        return error(Status::NotAcceptable, request);
    }
    let range = requested_range(request, &object);
    serve(request, &object, range)
}

/// Builds the answer of a media request.
///
/// A range that the file can satisfy is answered with `206` and the slice of
/// the file to read; a range it cannot satisfy is answered with `416`
/// (INT-082).
fn serve(request: &MediaRequest, object: &MediaObject, range: RangeRequest) -> MediaResponse {
    let is_head = matches!(request.method, Method::Head);
    match &range {
        // No range, or a range the server ignores: the whole representation.
        RangeRequest::Whole | RangeRequest::Ignored => media_response(
            request,
            object,
            Status::Ok,
            object.length,
            Body::Whole,
            is_head,
            None,
        ),
        _ => match crate::http::resolver::resolve(&range, object.length) {
            Some(resolved) => {
                let length = resolved.length();
                media_response(
                    request,
                    object,
                    Status::PartialContent,
                    length,
                    Body::Slice {
                        offset: resolved.first,
                        length,
                    },
                    is_head,
                    Some(resolved.as_header_value(object.length)),
                )
            }
            None => unsatisfiable(request, object),
        },
    }
}

/// The range the client asks for, if the request asks for one.
///
/// A range is only read on a `GET` (INT-092), and only when the condition of
/// an `If-Range` holds (INT-093).
fn requested_range(request: &MediaRequest, object: &MediaObject) -> RangeRequest {
    if !matches!(request.method, Method::Get) {
        return RangeRequest::Whole;
    }
    let Some(header) = request.headers.get(RANGE) else {
        return RangeRequest::Whole;
    };
    if !if_range_holds(request, object) {
        return RangeRequest::Whole;
    }
    crate::http::range::parse(header)
}

/// Whether the condition of an `If-Range` holds.
///
/// An entity tag has to match the validator of the object. A date cannot be
/// compared, because the server does not keep the modification time of the
/// file, so the condition does not hold and the range is ignored (INT-093).
fn if_range_holds(request: &MediaRequest, object: &MediaObject) -> bool {
    let Some(condition) = request.headers.get(IF_RANGE) else {
        return true;
    };
    let condition = condition.trim();
    if !(condition.starts_with('"') || condition.starts_with("W/")) {
        return false;
    }
    match (&object.validator, request.headers.get(ETAG)) {
        (Some(validator), Some(etag)) => validator == etag.trim(),
        _ => false,
    }
}

/// Whether the request for the DLNA content features is valid.
///
/// The server answers `1` and refuses every other value (INT-097).
fn content_features_request_is_valid(request: &MediaRequest) -> bool {
    match request.headers.get(GET_CONTENT_FEATURES) {
        Some(value) => value.trim() == GET_CONTENT_FEATURES_VALUE,
        None => true,
    }
}

/// Whether the server serves the content in the transfer mode the client
/// asks for.
///
/// A client that asks for a mode the server does not offer for that content
/// gets `406` — and so does a client that names a mode that does not exist
/// (INT-096, standards §5.2).
fn transfer_mode_is_served(request: &MediaRequest, object: &MediaObject) -> bool {
    let Some(requested) = request.headers.get(TRANSFER_MODE) else {
        return true;
    };
    match TransferMode::parse(requested) {
        Some(mode) => object.supports(mode),
        None => false,
    }
}
