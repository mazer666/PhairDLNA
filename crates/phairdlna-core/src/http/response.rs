//! The answer of the media layer.
//!
//! What: the status of an answer, its body and the headers that describe it.
//! Why: the answer is what the television sees. The body names the slice of
//!       the file to read, and the headers say what the client received
//!       (INT-080, INT-081, INT-084, INT-088).
//! Example: `Body::Slice` names the offset and the length to read.

use crate::http::headers::{
    HeaderMap, ACCEPT_RANGES, ACCEPT_RANGES_VALUE, ALLOW, CONNECTION, CONTENT_FEATURES,
    CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, REAL_TIME_INFO, REAL_TIME_INFO_VALUE,
    TRANSFER_MODE,
};
use crate::http::request::{MediaRequest, Method};
use crate::http::source::MediaObject;

/// The status of a media response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The whole representation.
    Ok,
    /// A part of the representation.
    PartialContent,
    /// The request line or the header block is broken.
    BadRequest,
    /// The server does not have the object.
    NotFound,
    /// The method is not served on a media URL.
    MethodNotAllowed,
    /// The range cannot be satisfied.
    RangeNotSatisfiable,
    /// The client asks for a transfer mode the server does not offer.
    NotAcceptable,
}

impl Status {
    /// The number of the status.
    pub fn code(&self) -> u16 {
        match self {
            Status::Ok => 200,
            Status::PartialContent => 206,
            Status::BadRequest => 400,
            Status::NotFound => 404,
            Status::MethodNotAllowed => 405,
            Status::RangeNotSatisfiable => 416,
            Status::NotAcceptable => 406,
        }
    }

    /// The reason of the status.
    pub fn reason(&self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::PartialContent => "Partial Content",
            Status::BadRequest => "Bad Request",
            Status::NotFound => "Not Found",
            Status::MethodNotAllowed => "Method Not Allowed",
            Status::RangeNotSatisfiable => "Range Not Satisfiable",
            Status::NotAcceptable => "Not Acceptable",
        }
    }
}

/// The body of a media response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    /// The whole file, from its beginning.
    Whole,
    /// A slice of the file.
    Slice {
        /// The offset of the first byte to read.
        offset: u64,
        /// The number of bytes to read.
        length: u64,
    },
    /// A fixed text, used for the error answers.
    Text(&'static str),
    /// No body at all, as the answer to a `HEAD`.
    None,
}

/// The answer of the media layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaResponse {
    /// The status of the answer.
    pub status: Status,
    /// The headers of the answer.
    pub headers: HeaderMap,
    /// The body of the answer.
    pub body: Body,
    /// The number of bytes the body has, zero for an empty body.
    pub content_length: u64,
}

/// Builds the answer of a media request that carries the whole representation.
pub fn media_response(
    request: &MediaRequest,
    object: &MediaObject,
    status: Status,
    length: u64,
    body: Body,
    is_head: bool,
    content_range: Option<String>,
) -> MediaResponse {
    let mut headers = HeaderMap::new();
    headers.set(CONTENT_TYPE, content_type_of(object));
    headers.set(CONTENT_LENGTH, length.to_string());
    headers.set(ACCEPT_RANGES, ACCEPT_RANGES_VALUE);
    if let Some(content_range) = content_range {
        headers.set(CONTENT_RANGE, content_range);
    }
    headers.set(TRANSFER_MODE, object.transfer_mode().as_str());
    if !object.content_features.is_empty() {
        headers.set(CONTENT_FEATURES, object.content_features.clone());
    }
    headers.set(REAL_TIME_INFO, REAL_TIME_INFO_VALUE);
    if wants_close(request) {
        headers.set(CONNECTION, "close");
    }
    let body = if is_head { Body::None } else { body };
    MediaResponse {
        status,
        headers,
        body,
        content_length: length,
    }
}

/// The type the answer carries.
///
/// The type is the one the catalogue read — the extension of the file,
/// refined by the sniffed content (F-10). A file whose type is unknown is
/// sent as `application/octet-stream` (INT-087).
pub fn content_type_of(object: &MediaObject) -> &str {
    if object.mime_type.is_empty() {
        "application/octet-stream"
    } else {
        &object.mime_type
    }
}

/// Builds the answer of a request whose range cannot be satisfied.
pub fn unsatisfiable(request: &MediaRequest, object: &MediaObject) -> MediaResponse {
    let mut headers = HeaderMap::new();
    headers.set(CONTENT_RANGE, format!("bytes */{}", object.length));
    headers.set(CONTENT_LENGTH, "0");
    if wants_close(request) {
        headers.set(CONNECTION, "close");
    }
    MediaResponse {
        status: Status::RangeNotSatisfiable,
        headers,
        body: Body::None,
        content_length: 0,
    }
}

/// Builds the answer of a request the server cannot carry out.
///
/// The body is a fixed text and never carries a path (SEC-013).
pub fn error(status: Status, request: &MediaRequest) -> MediaResponse {
    let text = match status {
        Status::BadRequest => "Bad request",
        Status::NotFound => "Not found",
        Status::RangeNotSatisfiable => "Range not satisfiable",
        Status::NotAcceptable => "Not acceptable",
        Status::MethodNotAllowed => "Method not allowed",
        Status::Ok | Status::PartialContent => "",
    };
    let mut headers = HeaderMap::new();
    headers.set(CONTENT_LENGTH, text.len().to_string());
    if wants_close(request) {
        headers.set(CONNECTION, "close");
    }
    // A `HEAD` answer carries the headers of the `GET` answer and no body
    // (INT-086).
    let body = if matches!(request.method, Method::Head) {
        Body::None
    } else {
        Body::Text(text)
    };
    MediaResponse {
        status,
        headers,
        body,
        content_length: text.len() as u64,
    }
}

/// Builds the answer of a request with a method the media layer does not
/// serve.
pub fn method_not_allowed(request: &MediaRequest) -> MediaResponse {
    let mut response = error(Status::MethodNotAllowed, request);
    response.headers.set(ALLOW, "GET, HEAD");
    response
}

/// Whether the client asked for the connection to be closed.
pub fn wants_close(request: &MediaRequest) -> bool {
    request
        .headers
        .get(CONNECTION)
        .map(|value| value.trim().eq_ignore_ascii_case("close"))
        .unwrap_or(false)
}
