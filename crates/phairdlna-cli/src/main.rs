//! Command line entry point of PhairDLNA.
//!
//! What: prints a short status banner and exits.
//! Why: the DLNA server is not implemented yet (requirements phase). The binary
//!      exists so that the workspace, the build and the test gates are real from
//!      the first commit — an empty stub would make those gates meaningless.
//! Example: `cargo run -p phairdlna-cli` prints the banner below.
//!
//! This binary must never start a network listener until the corresponding
//! feature specification is approved and implemented.

fn main() {
    println!(
        "{} — project scaffold (DLNA server not yet implemented)",
        phairdlna_core::PROJECT_NAME
    );
    println!("See docs/01-product/TECHNICAL-REQUIREMENTS.md and docs/07-project/ROADMAP.md.");
}
