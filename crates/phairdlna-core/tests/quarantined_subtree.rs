//! Acceptance test for quarantined subtree on permission denial (PLT-012, PLT-017, AC-09).
//!
//! What: verifies that when a subtree inside a media root produces a permission error,
//!       the unreadable subtree is quarantined and reported in catalogue diagnostics,
//!       while the rest of the media root is successfully scanned and indexed.
//! Why: PLT-012 mandates treating permission errors as quarantined subtrees requiring consent
//!      without aborting the whole root scan or crashing the server.

use phairdlna_core::catalogue::Catalogue;
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use std::fs::{create_dir, write, Permissions};
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn permission_denied_subtree_is_quarantined_and_rest_is_indexed() {
    let dir = tempdir().expect("tempdir");
    let root_path = dir.path().join("MediaRoot");
    create_dir(&root_path).expect("create root");

    // 1. Readable top-level media file
    let readable_file = root_path.join("accessible_track.mp3");
    write(&readable_file, b"ID3\x03\0\0\0\0\0\0dummy").expect("write readable file");

    // 2. Readable sibling folder with media
    let allowed_subdir = root_path.join("AllowedSubdir");
    create_dir(&allowed_subdir).expect("create allowed subdir");
    write(
        allowed_subdir.join("nested_track.mp3"),
        b"ID3\x03\0\0\0\0\0\0nested",
    )
    .expect("write nested track");

    // 3. Unreadable subtree (permission 000)
    let denied_subdir = root_path.join("ProtectedSubdir");
    create_dir(&denied_subdir).expect("create denied subdir");
    write(
        denied_subdir.join("secret_track.mp3"),
        b"ID3\x03\0\0\0\0\0\0secret",
    )
    .expect("write secret track");

    // Remove read/execute permissions from denied_subdir to simulate macOS/Linux permission denial
    let original_perms = std::fs::metadata(&denied_subdir)
        .expect("metadata")
        .permissions();
    std::fs::set_permissions(&denied_subdir, Permissions::from_mode(0o000))
        .expect("set permission 000");

    let root = MediaRoot {
        name: "TestRoot".to_owned(),
        path: root_path.to_string_lossy().to_string(),
        exclude: Vec::new(),
        symlink_policy: None,
    };

    let catalogue = Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem);

    // Restore permissions so tempdir cleanup succeeds
    let _ = std::fs::set_permissions(&denied_subdir, original_perms);

    // Verify:
    // a) The root continues indexing and registers readable tracks (PLT-012)
    let root_summary = &catalogue.roots()[0];
    assert!(root_summary.present, "media root must be marked present");
    assert!(
        root_summary.entries >= 2,
        "must index accessible tracks despite subtree error (got {})",
        root_summary.entries
    );

    // b) The inaccessible subtree is recorded in diagnostics as quarantined/unreadable (PLT-012, PLT-017)
    let diagnostics = catalogue.diagnostics();
    let has_quarantine_diagnostic = diagnostics.iter().any(|d| {
        d.contains("ProtectedSubdir")
            && (d.contains("cannot be read") || d.contains("Permission denied"))
    });
    assert!(
        has_quarantine_diagnostic,
        "diagnostics must report the quarantined subtree: {diagnostics:?}"
    );
}
