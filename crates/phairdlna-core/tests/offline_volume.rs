//! Offline volume handling and non-destructive availability reporting (REL-002, AC-12).
//!
//! What: verifies that an offline media root is marked unavailable without modifying configuration
//!       or destroying un-mounted root definitions.
//! Why: network mounts and external drives may disconnect temporarily; definitions must be preserved.

use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

#[test]
fn offline_media_root_reported_unavailable_and_preserved() {
    let dir = tempdir().expect("tempdir");
    let missing_path = dir.path().join("external_offline_drive");

    let root = MediaRoot {
        name: "External Drive".to_owned(),
        path: missing_path.to_string_lossy().to_string(),
        exclude: Vec::new(),
        symlink_policy: None,
    };

    let roots = vec![root.clone()];
    let catalogue =
        phairdlna_core::catalogue::Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);

    // The root was offline / unmounted, but catalogue is created cleanly
    let root_summaries = catalogue.roots();
    assert_eq!(root_summaries.len(), 1);
    assert_eq!(root_summaries[0].name, "External Drive");
    assert!(
        !root_summaries[0].present,
        "offline volume must be marked not present"
    );

    // Configuration retains the root definition without deletion (REL-002)
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].path, missing_path.to_string_lossy().to_string());
}
