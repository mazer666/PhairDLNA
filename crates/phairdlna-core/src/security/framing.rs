//! The framing of a request, and the rules that keep it unambiguous.
//!
//! What: the checks that decide whether the headers of a request agree with
//!       each other, and whether the request stays inside the bounds.
//! Why: a request whose headers disagree about where it ends is a request
//!       that a proxy in front of the server and the server itself read
//!       differently. That difference is a request smuggling, and the headers
//!       that cause it are ordinary text (`SEC-TH-12`, `SEC-TH-13`,
//!       `SEC-011`).
//! Example: `check` refuses a request that carries both a `Content-Length`
//!       and a `Transfer-Encoding`.
//!
//! **What is refused.** A request that names two lengths, a request that
//! names a length the server did not receive, a request whose headers are
//! too many or too large, and a request whose line is too long. Every refusal
//! is the same answer: the request is bad, and the server says nothing about
//! its own state.

use super::limits::{header_block_size, Limits};
use crate::http::headers::HeaderMap;

/// The header that names the length of a body.
const CONTENT_LENGTH: &str = "Content-Length";

/// The header that names the way the body is framed.
const TRANSFER_ENCODING: &str = "Transfer-Encoding";

/// The header that names the target of the request.
const HOST: &str = "Host";

/// Why a request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramingError {
    /// The request line is longer than the bound.
    RequestLineTooLong,
    /// The header block is longer than the bound.
    HeaderBlockTooLong,
    /// The request carries more headers than the bound.
    TooManyHeaders,
    /// The body is longer than the bound.
    BodyTooLarge,
    /// The request names the length of its body twice, in two ways.
    ConflictingLength,
    /// The request names a length that is not a number.
    MalformedLength,
}

impl FramingError {
    /// The sentence that says why the request was refused.
    pub fn reason(self) -> &'static str {
        match self {
            FramingError::RequestLineTooLong => "the request line is too long",
            FramingError::HeaderBlockTooLong => "the headers are too large",
            FramingError::TooManyHeaders => "the request carries too many headers",
            FramingError::BodyTooLarge => "the body is too large",
            FramingError::ConflictingLength => {
                "the request names the length of its body in two ways"
            }
            FramingError::MalformedLength => "the request names a length that is not a number",
        }
    }
}

/// The headers of a request, as a list of pairs.
///
/// The list is what the checks read, so that the core stays free of the type
/// of the HTTP library (`ADR-0001`).
pub type HeaderPairs = Vec<(String, String)>;

/// The checks a request must pass.
///
/// The order is fixed — the cheap checks first — so that a request that
/// breaks several rules always produces the same answer.
pub fn check(
    request_line_bytes: usize,
    headers: &[(String, String)],
    body_bytes: usize,
    limits: &Limits,
) -> Result<(), FramingError> {
    if !limits.accepts_request_line(request_line_bytes) {
        return Err(FramingError::RequestLineTooLong);
    }
    if !limits.accepts_header_count(headers.len()) {
        return Err(FramingError::TooManyHeaders);
    }
    if !limits.accepts_header_block(header_block_size(headers)) {
        return Err(FramingError::HeaderBlockTooLong);
    }
    if !limits.accepts_body(body_bytes) {
        return Err(FramingError::BodyTooLarge);
    }
    check_length(headers)
}

/// The checks on the length of the body.
///
/// A request carries the length of its body in one of two ways, never in both
/// (`RFC 9112` § 6.1), and the length it names is a number. A length that is
/// named twice with two different values is refused as well: the two ends of
/// the connection would not read the same body (`SEC-TH-13`).
fn check_length(headers: &[(String, String)]) -> Result<(), FramingError> {
    let lengths = values_of(headers, CONTENT_LENGTH);
    let encoding = value_of(headers, TRANSFER_ENCODING);

    if !lengths.is_empty() && encoding.is_some() {
        return Err(FramingError::ConflictingLength);
    }
    if lengths.len() > 1 && lengths.iter().any(|length| *length != lengths[0]) {
        return Err(FramingError::ConflictingLength);
    }
    for length in &lengths {
        if !length.trim().parse::<u64>().is_ok() {
            return Err(FramingError::MalformedLength);
        }
    }
    Ok(())
}

/// The value of a header, without regard to case.
fn value_of<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

/// Every value of a header, without regard to case.
///
/// A header may be sent more than once; the checks read all of them, because
/// a second value is exactly what a smuggled request carries.
pub fn values_of<'a>(headers: &'a [(String, String)], name: &str) -> Vec<&'a str> {
    headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
        .collect()
}

/// The value of the `Host` header of a request, if it carries one.
pub fn host_of(headers: &HeaderMap) -> Option<&str> {
    headers.get(HOST)
}

#[cfg(test)]
mod tests {
    use super::{check, FramingError, HeaderPairs};
    use crate::security::Limits;

    /// A request with one length is fine.
    #[test]
    fn one_length_is_fine() {
        let headers: HeaderPairs = vec![("Content-Length".to_owned(), "5".to_owned())];

        assert!(check(100, &headers, 5, &Limits::default()).is_ok());
    }

    /// A request with two lengths is refused.
    #[test]
    fn two_lengths_are_refused() {
        let headers: HeaderPairs = vec![
            ("Content-Length".to_owned(), "5".to_owned()),
            ("Transfer-Encoding".to_owned(), "chunked".to_owned()),
        ];

        assert_eq!(
            check(100, &headers, 0, &Limits::default()),
            Err(FramingError::ConflictingLength)
        );
    }

    /// A length that is not a number is refused.
    #[test]
    fn a_malformed_length_is_refused() {
        let headers: HeaderPairs = vec![("Content-Length".to_owned(), "5, 5".to_owned())];

        assert_eq!(
            check(100, &headers, 0, &Limits::default()),
            Err(FramingError::MalformedLength)
        );
    }

    /// The same length twice is not a smuggling attempt.
    #[test]
    fn the_same_length_twice_is_accepted() {
        let headers: HeaderPairs = vec![
            ("Content-Length".to_owned(), "5".to_owned()),
            ("Content-Length".to_owned(), "5".to_owned()),
        ];

        assert!(check(100, &headers, 5, &Limits::default()).is_ok());
    }

    /// Two different lengths are refused: the two ends of the connection would
    /// not read the same body.
    #[test]
    fn two_different_lengths_are_refused() {
        let headers: HeaderPairs = vec![
            ("Content-Length".to_owned(), "5".to_owned()),
            ("Content-Length".to_owned(), "6".to_owned()),
        ];

        assert_eq!(
            check(100, &headers, 5, &Limits::default()),
            Err(FramingError::ConflictingLength)
        );
    }

    /// The names of the headers are read without regard to case.
    #[test]
    fn the_names_are_read_without_regard_to_case() {
        let headers: HeaderPairs = vec![
            ("content-length".to_owned(), "5".to_owned()),
            ("transfer-encoding".to_owned(), "chunked".to_owned()),
        ];

        assert_eq!(
            check(100, &headers, 0, &Limits::default()),
            Err(FramingError::ConflictingLength)
        );
    }

    /// A request line that is too long is refused.
    #[test]
    fn a_long_request_line_is_refused() {
        let limits = Limits::default();

        assert_eq!(
            check(limits.request_line_bytes + 1, &[], 0, &limits),
            Err(FramingError::RequestLineTooLong)
        );
    }

    /// A body that is too large is refused.
    #[test]
    fn a_large_body_is_refused() {
        let limits = Limits::default();

        assert_eq!(
            check(100, &[], limits.body_bytes + 1, &limits),
            Err(FramingError::BodyTooLarge)
        );
    }
}
