#![allow(dead_code)]
//! Fixtures for the tests of the media delivery.
//!
//! What: a media source that holds its objects in memory, and the helpers
//!       that build the requests and read the answers.
//! Why: the HTTP layer must be testable without a disk and without a socket
//!       (ARCHITECTURE §4.7). The fixture therefore holds the bytes of the
//!       file, so that a test can compare what the server promised to send
//!       with what the client really receives.
//! Example: `TestMedia::with_file("a3f1", "audio/mpeg", ContentKind::Audio, 1000)`
//!       builds a source with one object of a thousand bytes.

use phairdlna_core::http::{
    Body, ContentKind, HeaderMap, MediaObject, MediaRequest, MediaResponse, MediaSource, Method,
};

/// The bytes of a file that the fixture holds.
#[derive(Debug, Clone)]
struct Entry {
    object: MediaObject,
    /// The bytes of the file, or `None` for a file whose bytes the fixture
    /// does not hold.
    bytes: Option<Vec<u8>>,
}

/// A media source that holds its objects in memory.
#[derive(Debug, Clone, Default)]
pub struct TestMedia {
    entries: Vec<Entry>,
}

impl TestMedia {
    /// Builds an empty source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file of the given length.
    ///
    /// The bytes are the offsets of the file, so that a test can tell which
    /// byte it received.
    pub fn with_file(
        &mut self,
        id: &str,
        mime_type: &str,
        kind: ContentKind,
        length: u64,
    ) -> &mut Self {
        let bytes = (0..length).map(|index| (index % 251) as u8).collect();
        self.entries.push(Entry {
            object: MediaObject::new(id, length, mime_type, kind),
            bytes: Some(bytes),
        });
        self
    }

    /// Adds a file of the given length whose bytes the fixture does not hold.
    ///
    /// The fixture cannot hold five gigabytes of bytes, so a file beyond the
    /// memory of a test is recorded with its length alone; the offsets of
    /// such a file are still answered exactly (FUN-028).
    pub fn with_sparse_file(
        &mut self,
        id: &str,
        mime_type: &str,
        kind: ContentKind,
        length: u64,
    ) -> &mut Self {
        self.entries.push(Entry {
            object: MediaObject::new(id, length, mime_type, kind),
            bytes: None,
        });
        self
    }

    /// Adds a file with the given DLNA features and validator.
    pub fn with_features(
        &mut self,
        id: &str,
        mime_type: &str,
        kind: ContentKind,
        length: u64,
        features: &str,
        validator: Option<&str>,
    ) -> &mut Self {
        self.with_file(id, mime_type, kind, length);
        let entry = self.entries.last_mut().expect("the entry was just pushed");
        entry.object.content_features = features.to_owned();
        entry.object.validator = validator.map(|value| value.to_owned());
        self
    }

    /// The bytes of a file.
    pub fn bytes_of(&self, id: &str) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|entry| entry.object.id == id)
            .and_then(|entry| entry.bytes.as_deref())
    }

    /// The bytes an answer carries.
    ///
    /// The answer names the slice of the file to read, and this helper reads
    /// exactly that slice — so a test compares what the server promised with
    /// what the client receives.
    pub fn body_of(&self, id: &str, response: &MediaResponse) -> Vec<u8> {
        match response.body {
            Body::Whole => self.bytes_of(id).unwrap_or_default().to_vec(),
            Body::Slice { offset, length } => {
                let bytes = self.bytes_of(id).unwrap_or_default();
                let start = (offset as usize).min(bytes.len());
                let end = (offset as usize + length as usize).min(bytes.len());
                bytes[start..end].to_vec()
            }
            Body::Text(text) => text.as_bytes().to_vec(),
            Body::None => Vec::new(),
        }
    }
}

impl MediaSource for TestMedia {
    fn metadata(&self, id: &str) -> Option<MediaObject> {
        self.entries
            .iter()
            .find(|entry| entry.object.id == id)
            .map(|entry| entry.object.clone())
    }
}

/// A request with the given method and target.
pub fn request(method: Method, target: &str) -> MediaRequest {
    MediaRequest::new(method, target, HeaderMap::new())
}

/// A request with one header.
pub fn request_with(method: Method, target: &str, name: &str, value: &str) -> MediaRequest {
    let mut headers = HeaderMap::new();
    headers.set(name, value);
    MediaRequest::new(method, target, headers)
}
