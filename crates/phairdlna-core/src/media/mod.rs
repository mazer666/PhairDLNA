//! The pictures of the library.
//!
//! What: the discovery of the picture that belongs to a media file — the cover
//!      that is embedded in an audio file, the image that lies next to it, and
//!      the thumbnail of a photo — and the cache that keeps the pictures that
//!      were produced.
//! Why: a picture is the first thing a control point asks for, and it is also
//!      the first thing a damaged file can break. The module therefore never
//!      trusts a file: every read is bounded, every failure is a missing
//!      picture and never a panic (`F-10`, `SEC-067`, `UX-004`).
//! Example: `ArtworkFinder::source_of` tells where the picture of a file comes
//!       from, and `ArtworkFinder::picture_of` produces it.
//!
//! **The order of the search.** A sidecar wins over an embedded cover, and a
//! media file that is a picture itself is its own thumbnail (`F-10` § 3).

pub mod artwork;
pub mod cache;
pub mod id3;
pub mod mp4;
pub mod sidecar;
pub mod thumbnail;

pub use artwork::{cache_key, Artwork, ArtworkFinder, ArtworkSource, PictureLimits, RootBoundary};
pub use cache::PictureCache;
pub use sidecar::mime_type_of;
