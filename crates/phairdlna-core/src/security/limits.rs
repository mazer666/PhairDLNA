//! The bounds the server keeps inside.
//!
//! What: the size, count and time limits that every request and every
//!       transfer must stay within.
//! Why: a server that answers every request it receives, however large, is a
//!       server that one host on the network can take down (`SEC-011`,
//!       `SEC-012`, `SEC-016`, `SEC-060`, `SEC-061`, `SEC-063`). The limits
//!       are configurable within a safe range, and the defaults are the ones an
//!       ordinary machine serves well (`F-16` § 8).
//! Example: `Limits::default()` is the ceiling of a normal server, and
//!       `limits.accepts_body(4096)` says whether a body of that size is read.

use std::time::Duration;

use crate::config::LimitsSection;

/// The bounds the server keeps inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The largest request line the server reads, in bytes.
    pub request_line_bytes: usize,
    /// The largest header block the server reads, in bytes.
    pub header_block_bytes: usize,
    /// The largest number of headers the server reads.
    pub header_count: usize,
    /// The largest body the server reads, in bytes.
    pub body_bytes: usize,
    /// The largest number of connections that may be open at the same time.
    pub connections: usize,
    /// The largest number of transfers that may run at the same time.
    pub transfers: usize,
    /// The largest DIDL-Lite document the server writes, in bytes.
    pub didl_bytes: usize,
    /// How long a connection may stay idle before it is closed.
    pub read_timeout: Duration,
    /// How long the server may take to write an answer.
    pub write_timeout: Duration,
}

impl Default for Limits {
    /// The defaults of a server on an ordinary machine.
    ///
    /// The request line is short because a DLNA request is one line of a few
    /// hundred bytes; the header block is generous because a client may carry
    /// a `User-Agent` and a few DLNA headers; the body is the size of the
    /// largest SOAP envelope the XML parser accepts (`SEC-010`).
    fn default() -> Self {
        Self {
            request_line_bytes: 8 * 1024,
            header_block_bytes: 32 * 1024,
            header_count: 64,
            body_bytes: 64 * 1024,
            connections: 64,
            transfers: 10,
            didl_bytes: 2 * 1024 * 1024,
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
        }
    }
}

impl Limits {
    /// Reads the bounds out of the configuration.
    ///
    /// The configuration is the only place a bound is written down, so that a
    /// user who tightens a bound tightens it everywhere (`CFG-004`).
    pub fn from_section(section: &LimitsSection) -> Self {
        Self {
            request_line_bytes: section.max_request_line_bytes,
            header_block_bytes: section.max_header_block_bytes,
            header_count: section.max_header_count,
            body_bytes: section.max_body_bytes,
            connections: section.max_concurrent_connections,
            transfers: section.max_concurrent_streams,
            didl_bytes: section.max_didl_bytes,
            read_timeout: Duration::from_secs(section.read_timeout_seconds),
            write_timeout: Duration::from_secs(section.write_timeout_seconds),
        }
    }

    /// Whether a request line of the given size is read.
    pub fn accepts_request_line(&self, bytes: usize) -> bool {
        bytes <= self.request_line_bytes
    }

    /// Whether a header block of the given size is read.
    pub fn accepts_header_block(&self, bytes: usize) -> bool {
        bytes <= self.header_block_bytes
    }

    /// Whether the given number of headers is read.
    pub fn accepts_header_count(&self, count: usize) -> bool {
        count <= self.header_count
    }

    /// Whether a body of the given size is read.
    pub fn accepts_body(&self, bytes: usize) -> bool {
        bytes <= self.body_bytes
    }

    /// Whether a DIDL-Lite document of the given size is written.
    pub fn accepts_didl(&self, bytes: usize) -> bool {
        bytes <= self.didl_bytes
    }
}

/// The size of a header block, in bytes.
///
/// Every header costs its name, a colon, a space, its value and the two
/// characters that end the line (`SEC-011`).
pub fn header_block_size(headers: &[(String, String)]) -> usize {
    headers
        .iter()
        .map(|(name, value)| name.len() + value.len() + 4)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{header_block_size, Limits};
    use crate::config::LimitsSection;

    /// The defaults are the ones the documentation names.
    #[test]
    fn the_defaults_are_the_documented_ones() {
        let limits = Limits::default();
        assert_eq!(limits.request_line_bytes, 8 * 1024);
        assert_eq!(limits.body_bytes, 64 * 1024);
        assert_eq!(limits.transfers, 10);
    }

    /// A body inside the bound is read, and one outside it is not.
    #[test]
    fn a_body_inside_the_bound_is_read() {
        let limits = Limits::default();
        assert!(limits.accepts_body(4096));
        assert!(!limits.accepts_body(limits.body_bytes + 1));
    }

    /// The bounds are read out of the configuration.
    #[test]
    fn the_bounds_come_from_the_configuration() {
        let section = LimitsSection {
            max_request_line_bytes: 100,
            max_header_block_bytes: 200,
            max_header_count: 3,
            max_body_bytes: 300,
            max_concurrent_connections: 4,
            max_didl_bytes: 500,
            read_timeout_seconds: 5,
            write_timeout_seconds: 6,
            ..LimitsSection::default()
        };
        let limits = Limits::from_section(&section);
        assert_eq!(limits.request_line_bytes, 100);
        assert_eq!(limits.header_count, 3);
        assert_eq!(limits.connections, 4);
        assert_eq!(limits.didl_bytes, 500);
        assert_eq!(limits.read_timeout.as_secs(), 5);
        assert_eq!(limits.write_timeout.as_secs(), 6);
    }

    /// The size of a header block counts every header.
    #[test]
    fn the_header_block_is_measured() {
        let headers = vec![
            ("Range".to_owned(), "bytes=0-1".to_owned()),
            ("Host".to_owned(), "192.168.1.20:8200".to_owned()),
        ];
        let expected = headers
            .iter()
            .map(|(name, value)| name.len() + value.len() + 4)
            .sum::<usize>();

        assert_eq!(header_block_size(&headers), expected);
        assert_eq!(header_block_size(&[]), 0);
    }
}
