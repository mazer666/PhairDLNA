//! The resolution of a range against a file.
//!
//! What: the byte range a file can actually deliver, and the resolution of
//!       the range the client asked for against the length of the file.
//! Why: the length of the file is the only thing that tells whether a range
//!       can be satisfied. A range that starts behind the end of the file
//!       cannot be satisfied and is answered with `416` (INT-082); a range
//!       that ends behind it is cut at the end, as RFC 9110 §14.2 asks.
//! Example: `resolve` turns `bytes=100-199` into the range 100 to 199 of a
//!       file of a thousand bytes.

use crate::http::range::{RangeRequest, RangeSpec};

/// A byte range of a representation, both ends included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    /// The first byte of the range.
    pub first: u64,
    /// The last byte of the range, included.
    pub last: u64,
}

impl ByteRange {
    /// Builds a range.
    pub fn new(first: u64, last: u64) -> Self {
        Self { first, last }
    }

    /// The number of bytes the range holds.
    pub fn length(&self) -> u64 {
        self.last.saturating_sub(self.first).saturating_add(1)
    }

    /// The range as the `Content-Range` header spells it.
    pub fn as_header_value(&self, size: u64) -> String {
        format!("bytes {}-{}/{}", self.first, self.last, size)
    }
}

/// Resolves a range request against the length of the representation.
///
/// A range that starts at or behind the end of the file cannot be satisfied
/// (INT-082); a range that ends behind it is cut at the end, as RFC 9110
/// §14.2 asks. A set of ranges is served as the first one that fits — never
/// as a `multipart/byteranges` body (INT-085).
pub fn resolve(request: &RangeRequest, size: u64) -> Option<ByteRange> {
    match request {
        RangeRequest::Whole | RangeRequest::Ignored => Some(whole(size)),
        RangeRequest::Unsatisfiable => None,
        RangeRequest::Single(spec) => resolve_spec(spec, size),
        RangeRequest::Many(specs) => specs.iter().find_map(|spec| resolve_spec(spec, size)),
    }
}

/// The whole representation as a range.
fn whole(size: u64) -> ByteRange {
    ByteRange::new(0, size.saturating_sub(1))
}

/// Resolves one range against the length of the representation.
fn resolve_spec(spec: &RangeSpec, size: u64) -> Option<ByteRange> {
    if size == 0 {
        return None;
    }
    match *spec {
        RangeSpec::Closed { first, last } => {
            if first >= size {
                return None;
            }
            Some(ByteRange::new(first, last.min(size - 1)))
        }
        RangeSpec::From { first } => {
            if first >= size {
                return None;
            }
            Some(ByteRange::new(first, size - 1))
        }
        RangeSpec::Suffix { length } => {
            if length > size {
                return Some(whole(size));
            }
            Some(ByteRange::new(size - length, size - 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve, ByteRange};
    use crate::http::range::{parse, RangeRequest};

    /// A range that fits the file is resolved as it was asked.
    #[test]
    fn a_range_that_fits_is_resolved() {
        let range = resolve(&parse("bytes=100-199"), 1000).expect("the range fits");
        assert_eq!(range, ByteRange::new(100, 199));
        assert_eq!(range.length(), 100);
    }

    /// An open range reaches the end of the file.
    #[test]
    fn an_open_range_reaches_the_end() {
        let range = resolve(&parse("bytes=500-"), 1000).expect("the range fits");
        assert_eq!(range, ByteRange::new(500, 999));
    }

    /// A suffix range reaches the end of the file.
    #[test]
    fn a_suffix_range_reaches_the_end() {
        let range = resolve(&parse("bytes=-100"), 1000).expect("the range fits");
        assert_eq!(range, ByteRange::new(900, 999));
    }

    /// A suffix longer than the file is the whole file.
    #[test]
    fn a_long_suffix_is_the_whole_file() {
        let range = resolve(&parse("bytes=-5000"), 1000).expect("the range fits");
        assert_eq!(range, ByteRange::new(0, 999));
    }

    /// A range that ends behind the file is cut at the end.
    #[test]
    fn a_range_behind_the_file_is_cut() {
        let range = resolve(&parse("bytes=900-5000"), 1000).expect("the range fits");
        assert_eq!(range, ByteRange::new(900, 999));
    }

    /// A range that starts behind the file cannot be satisfied.
    #[test]
    fn a_range_behind_the_file_is_not_satisfiable() {
        assert_eq!(resolve(&parse("bytes=1000-"), 1000), None);
        assert_eq!(resolve(&parse("bytes=5000-6000"), 1000), None);
    }

    /// A set of ranges is served as one range, never as a multipart body.
    #[test]
    fn a_set_of_ranges_is_served_as_one() {
        let range = resolve(&parse("bytes=0-1,5-6"), 1000).expect("a range fits");
        assert_eq!(range, ByteRange::new(0, 1));
    }

    /// A set whose ranges all fail is not satisfiable.
    #[test]
    fn a_set_of_impossible_ranges_is_not_satisfiable() {
        assert_eq!(resolve(&parse("bytes=2000-2001,3000-3001"), 1000), None);
    }

    /// A file of zero bytes satisfies no range at all.
    #[test]
    fn an_empty_file_satisfies_no_range() {
        assert_eq!(resolve(&parse("bytes=0-0"), 0), None);
        assert_eq!(resolve(&parse("bytes=0-"), 0), None);
    }

    /// The whole representation is answered without a range header.
    #[test]
    fn the_whole_representation_is_the_whole_file() {
        assert_eq!(
            resolve(&RangeRequest::Whole, 1000),
            Some(ByteRange::new(0, 999))
        );
        assert_eq!(
            resolve(&RangeRequest::Ignored, 1000),
            Some(ByteRange::new(0, 999))
        );
    }

    /// The range is written as the `Content-Range` header spells it.
    #[test]
    fn the_range_is_written_for_the_header() {
        assert_eq!(
            ByteRange::new(100, 199).as_header_value(1000),
            "bytes 100-199/1000"
        );
    }

    /// An offset beyond four gigabytes is not truncated (FUN-028).
    #[test]
    fn an_offset_beyond_four_gigabytes_is_kept() {
        let start = 5_000_000_000u64;
        let range =
            resolve(&parse(&format!("bytes={start}-")), start + 10).expect("the range fits");
        assert_eq!(range.first, start);
        assert_eq!(range.last, start + 9);
    }
}
