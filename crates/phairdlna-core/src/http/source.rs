//! The media the server serves.
//!
//! What: the metadata of a media object, the kind that decides the transfer
//!       mode, and the trait through which the server finds its objects.
//! Why: the HTTP layer must be testable without a disk, so it reads the
//!       bytes through a `MediaSource` (ARCHITECTURE §4.7). The trait is the
//!       boundary: the HTTP layer knows what an object is, never where the
//!       file lives.
//! Example: `MediaObject::transfer_mode` answers the mode the server uses for
//!       an image.

use crate::http::headers::TransferMode;

/// The kind of a media object.
///
/// The kind decides the transfer mode the server answers with, and the modes
/// it accepts from the client (INT-096).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    /// Sound.
    Audio,
    /// Moving pictures.
    Video,
    /// A still picture.
    Image,
    /// A file of a kind the server does not describe further.
    Other,
}

impl ContentKind {
    /// The mode the server uses for this kind.
    ///
    /// Audio and video are streamed, an image is delivered interactively, and
    /// a file of an unknown kind is streamed like the others (standards §5.2).
    pub fn transfer_mode(&self) -> TransferMode {
        match self {
            ContentKind::Image => TransferMode::Interactive,
            ContentKind::Audio | ContentKind::Video | ContentKind::Other => TransferMode::Streaming,
        }
    }

    /// Whether the server serves this content in the given mode.
    ///
    /// An image is delivered interactively and in no other way; a renderer
    /// that asks to stream an image gets `406`, as the reference
    /// implementation does (INT-096, standards §5.2).
    pub fn supports(&self, mode: TransferMode) -> bool {
        match self {
            ContentKind::Image => mode == TransferMode::Interactive,
            _ => matches!(mode, TransferMode::Streaming | TransferMode::Background),
        }
    }
}

/// The metadata of a media object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaObject {
    /// The opaque identifier of the object.
    pub id: String,
    /// The length of the file in bytes.
    pub length: u64,
    /// The MIME type the server sends, the one of the `res` element (INT-087).
    pub mime_type: String,
    /// The kind of the content.
    pub kind: ContentKind,
    /// The DLNA features of the content.
    ///
    /// The value is the fourth field of the `protocolInfo` of the object
    /// (F-05), so that a client that compares the two sees no difference.
    pub content_features: String,
    /// The validator of the object, used by `If-Range` (INT-093).
    pub validator: Option<String>,
}

impl MediaObject {
    /// Builds an object of the given length and type.
    pub fn new(
        id: impl Into<String>,
        length: u64,
        mime_type: impl Into<String>,
        kind: ContentKind,
    ) -> Self {
        Self {
            id: id.into(),
            length,
            mime_type: mime_type.into(),
            kind,
            content_features: String::new(),
            validator: None,
        }
    }

    /// The mode the server uses for this object.
    pub fn transfer_mode(&self) -> TransferMode {
        self.kind.transfer_mode()
    }

    /// Whether the server serves this object in the given mode.
    pub fn supports(&self, mode: TransferMode) -> bool {
        self.kind.supports(mode)
    }
}

/// Where the server finds the media it serves.
///
/// The implementation owns the catalogue and the roots; the HTTP layer only
/// asks for the metadata of an object (INT-090, INT-098).
pub trait MediaSource {
    /// The metadata of the object with the given identifier.
    ///
    /// `None` when the server does not have the object. The answer never
    /// carries a path (SEC-013).
    fn metadata(&self, id: &str) -> Option<MediaObject>;
}

#[cfg(test)]
mod tests {
    use super::{ContentKind, MediaObject, MediaSource};
    use crate::http::headers::TransferMode;

    /// An image is delivered interactively and in no other way.
    #[test]
    fn an_image_is_interactive() {
        assert_eq!(
            ContentKind::Image.transfer_mode(),
            TransferMode::Interactive
        );
        assert!(ContentKind::Image.supports(TransferMode::Interactive));
        assert!(!ContentKind::Image.supports(TransferMode::Streaming));
    }

    /// Sound and pictures are streamed.
    #[test]
    fn sound_and_pictures_are_streamed() {
        assert_eq!(ContentKind::Audio.transfer_mode(), TransferMode::Streaming);
        assert_eq!(ContentKind::Video.transfer_mode(), TransferMode::Streaming);
        assert!(ContentKind::Video.supports(TransferMode::Streaming));
        assert!(ContentKind::Video.supports(TransferMode::Background));
    }

    /// The object knows its mode.
    #[test]
    fn the_object_knows_its_mode() {
        let object = MediaObject::new("a3f1", 1000, "image/jpeg", ContentKind::Image);
        assert_eq!(object.transfer_mode(), TransferMode::Interactive);
        assert!(!object.supports(TransferMode::Streaming));
    }

    /// A source that knows nothing answers nothing.
    #[test]
    fn an_empty_source_answers_nothing() {
        struct Empty;
        impl MediaSource for Empty {
            fn metadata(&self, _id: &str) -> Option<MediaObject> {
                None
            }
        }
        assert!(Empty.metadata("a3f1").is_none());
    }
}
