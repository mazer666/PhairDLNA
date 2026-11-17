//! The reader of the JSON answers of the administration surface.
//!
//! What: the names of the reader of JSON that the core holds.
//! Why: the reader lives in the core, so that the command line and this
//!      application read the answers of the surface in the same way and cannot
//!      disagree about what an answer means (`SEC-063`).
//! Example: the application reads the number of the streams of the surface
//!          with `value.as_u64()`.

pub use phairdlna_core::json::{Json, JsonError};
