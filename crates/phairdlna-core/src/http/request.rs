//! The request of the media layer.
//!
//! What: the method of a media request and the request itself.
//! Why: the request comes from the network and is therefore untrusted; it
//!       reaches the evaluator as a value, so that every rule can be tested
//!       without a socket (ARCHITECTURE §4.7).
//! Example: `MediaRequest::new` builds a request for a media URL.

use crate::http::headers::HeaderMap;

/// The method of a media request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    /// The client asks for the object.
    Get,
    /// The client asks for the headers of the object.
    Head,
    /// A method the media layer does not serve.
    Other(String),
}

impl Method {
    /// The method as it appears in the request line.
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Other(name) => name,
        }
    }
}

/// A request for a media object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaRequest {
    /// The method of the request.
    pub method: Method,
    /// The target of the request, the path with its query string.
    pub target: String,
    /// The headers of the request.
    pub headers: HeaderMap,
}

impl MediaRequest {
    /// Builds a request.
    pub fn new(method: Method, target: impl Into<String>, headers: HeaderMap) -> Self {
        Self {
            method,
            target: target.into(),
            headers,
        }
    }
}
