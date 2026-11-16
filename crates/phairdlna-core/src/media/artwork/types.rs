//! The data of the pictures of a library.
//!
//! What: the bounds of the picture pipeline, the roots it looks inside, and
//!       the ways a picture can be found.
//! Why: the bounds are the security boundary of the pipeline and the roots are
//!       its containment rule; both are data, and both are named once
//!       (`SEC-003`, `SEC-067`).

use std::path::PathBuf;

use crate::config::SymlinkPolicy;

/// The bounds of the picture pipeline.
///
/// Every read of a file and every allocation of a picture is bounded by one of
/// these numbers, so that a file that lies about its size cannot make the
/// server use an unbounded amount of memory (`SEC-067`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PictureLimits {
    /// The largest embedded picture that is read, in bytes.
    pub max_embedded_bytes: u64,
    /// The largest sidecar image that is read, in bytes.
    pub max_sidecar_bytes: u64,
    /// The largest edge of a picture that is decoded, in pixels.
    pub max_dimension: u32,
    /// The largest number of pixels a picture may have.
    pub max_pixels: u64,
    /// The largest edge of a thumbnail, in pixels.
    pub thumbnail_edge: u32,
}

impl Default for PictureLimits {
    /// The limits of the first release: generous enough for a cover of a
    /// normal album, small enough to stay inside the memory of a small
    /// machine.
    fn default() -> Self {
        Self {
            max_embedded_bytes: 8 * 1024 * 1024,
            max_sidecar_bytes: 16 * 1024 * 1024,
            max_dimension: 8192,
            max_pixels: 40_000_000,
            thumbnail_edge: 512,
        }
    }
}

/// One root of the library, as the picture pipeline sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootBoundary {
    /// The canonical path of the root.
    ///
    /// The path is canonical, because a sidecar is only inside the library
    /// when its target is inside the root after the links are resolved
    /// (`SEC-003`).
    pub path: PathBuf,
    /// How the symbolic links of this root are treated.
    pub symlink_policy: SymlinkPolicy,
}

/// Where the picture of a media file comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkSource {
    /// An image that lies next to the media file.
    Sidecar {
        /// The path of the image.
        path: PathBuf,
        /// The type of the image.
        mime_type: String,
    },
    /// A picture that is embedded in the media file.
    Embedded {
        /// The type of the picture.
        mime_type: String,
    },
    /// The media file is a picture, and its thumbnail is produced from it.
    Thumbnail {
        /// The type of the produced thumbnail.
        mime_type: String,
    },
}

/// A picture and its type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artwork {
    /// The type of the picture.
    pub mime_type: String,
    /// The picture itself.
    pub bytes: Vec<u8>,
}
