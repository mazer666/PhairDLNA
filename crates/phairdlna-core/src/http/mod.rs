//! The delivery of media over HTTP.
//!
//! What: the `Range` header, the headers of a media request and of a media
//!       response, the source the server reads its objects from, the
//!       containment of a media URL, and the evaluator that turns a request
//!       into an answer.
//! Why: everything the user watches travels over this module. The rules are
//!       pure — the evaluator reads no socket and no file, it decides the
//!       status, the headers and the slice of the file to read — so every
//!       rule can be tested without a network and without a disk
//!       (ARCHITECTURE §4.7). The media URL is the trust boundary: a request
//!       may never make the server read outside a root (INT-098, SEC-001).
//! Example:
//!
//! ```
//! use phairdlna_core::http::{evaluate, Body, HeaderMap, MediaRequest, Method};
//!
//! let request = MediaRequest::new(Method::Get, "/media/a3f1", HeaderMap::new());
//! let response = evaluate(&request, &EmptySource);
//! assert_eq!(response.status.code(), 404);
//!
//! # struct EmptySource;
//! # impl phairdlna_core::http::MediaSource for EmptySource {
//! #     fn metadata(&self, _id: &str) -> Option<phairdlna_core::http::MediaObject> { None }
//! # }
//! ```

pub mod delivery;
pub mod headers;
pub mod path;
pub mod range;
pub mod request;
pub mod resolver;
pub mod response;
pub mod source;
pub mod transfer;

pub use delivery::evaluate;
pub use headers::{
    HeaderMap, TransferMode, ACCEPT_RANGES, ACCEPT_RANGES_VALUE, ALLOW, CONNECTION,
    CONTENT_FEATURES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, GET_CONTENT_FEATURES,
    GET_CONTENT_FEATURES_VALUE, IF_RANGE, RANGE, REAL_TIME_INFO, REAL_TIME_INFO_VALUE,
    TRANSFER_MODE,
};
pub use path::{is_inside_roots, MediaId, Target, MEDIA_PATH};
pub use range::{parse as parse_range, RangeRequest, RangeSpec};
pub use request::{MediaRequest, Method};
pub use resolver::{resolve as resolve_range, ByteRange};
pub use response::{Body, MediaResponse, Status};
pub use source::{ContentKind, MediaObject, MediaSource};
pub use transfer::TransferState;
