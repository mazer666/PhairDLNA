# Getting started (for beginners)

PhairDLNA is currently a Rust workspace scaffold, **not a working DLNA server**.

1. Install a supported Rust toolchain using https://rustup.rs/.
2. Clone the repository and open a terminal in its root.
3. Run `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
4. Run `cargo run -p phairdlna-cli` to see the scaffold message.
5. Read [WHERE_TO_CHANGE](../../WHERE_TO_CHANGE.md) before modifying anything.
6. Read [project standards](../00-governance/PROJECT-STANDARDS.md) and [feature template](../03-features/FEATURE-TEMPLATE.md).

These commands are instructions, not evidence they have been run. Add platform-specific prerequisites as implementation evolves.
