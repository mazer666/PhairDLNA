//! The build script of the core: it writes the source revision into the binary.
//!
//! What: reads the revision of the working tree with `git` and hands it to the
//!       compiler as `PHAIRDLNA_SOURCE_REVISION`.
//! Why: `OPS-017` requires the server to report the exact source revision it
//!      was built from. A revision that is read at *run* time can disagree with
//!      the binary that is running; a revision that is read at *build* time
//!      cannot (`ADR-0010`).
//! Example: a build of the tree at `a1b2c3d4e5f6` reports that revision in
//!          the status output.
//!
//! **When the revision is not known.** A build from an archive without a
//! repository, and a build without `git` on the machine, produce the revision
//! `unknown`. The status output then says so plainly instead of pretending.

use std::process::Command;

/// The revision that is written when the tree does not tell one.
const UNKNOWN: &str = "unknown";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    watch_the_repository();
    println!("cargo:rustc-env=PHAIRDLNA_SOURCE_REVISION={}", revision());
}

/// Asks cargo to rebuild when the repository moves.
///
/// The files that carry the revision are few and their names are known: the
/// head of the repository, the reference it points at, and the packed list of
/// references. A file that does not exist is not watched — a repository that
/// uses packed references only never writes the loose one.
fn watch_the_repository() {
    for path in [
        "../../.git/HEAD",
        "../../.git/refs/heads/main",
        "../../.git/packed-refs",
    ] {
        if std::path::Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}

/// The revision of the working tree, or `unknown`.
fn revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output();
    let Ok(output) = output else {
        return UNKNOWN.to_owned();
    };
    if !output.status.success() {
        return UNKNOWN.to_owned();
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if text.is_empty() {
        return UNKNOWN.to_owned();
    }
    text
}
