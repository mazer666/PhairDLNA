//! The optional native macOS application of PhairDLNA.
//!
//! What: the menu bar application that controls the headless server.
//! Why: the server is the product; the application is a front end for it. The
//!      two are separate crates so that the server can run without any
//!      graphical process (`PRD-006`).
//! Example: none yet.
//!
//! **Status: no window exists.** The crate exists so that the dependency
//! boundary is real from the first commit; the specification is F-15, the
//! milestone is M2, and this binary does not draw anything yet. A future
//! reader must not mistake the empty binary for a finished application.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

fn main() {
    println!(
        "{} — the macOS application is not implemented yet (see docs/03-features/F-15-macos-native-experience.md)",
        phairdlna_core::PROJECT_NAME
    );
}
