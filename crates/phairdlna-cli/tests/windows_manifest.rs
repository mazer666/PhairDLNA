//! Windows executable manifest and long-path documentation acceptance tests (PLT-015, F-22 AC-11).
//!
//! What: verifies that the Windows application manifest exists and explicitly declares
//!       `longPathAware` support, and verifies that documentation explains the requirement
//!       for system-wide `LongPathsEnabled`.
//! Why: PLT-015 requires Windows installations to declare long-path support in the executable
//!      manifest and document the registry prerequisite so paths exceeding MAX_PATH (260 chars)
//!      are handled properly without truncation.

use std::path::Path;

#[test]
fn the_windows_manifest_declares_long_path_awareness() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("phairdlna.manifest");

    assert!(
        manifest_path.exists(),
        "Windows executable manifest must exist at resources/phairdlna.manifest (PLT-015)"
    );

    let content = std::fs::read_to_string(&manifest_path).expect("read manifest content");

    assert!(
        content.contains("<longPathAware"),
        "manifest must declare <longPathAware> element (PLT-015)"
    );
    assert!(
        content.contains(">true</longPathAware>"),
        "longPathAware element must be set to true"
    );
    assert!(
        content.contains("http://schemas.microsoft.com/SMI/2016/WindowsSettings"),
        "longPathAware must use the correct WindowsSettings namespace"
    );
}

#[test]
fn the_documentation_declares_windows_long_paths_enabled_requirement() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("docs/03-features/F-08-media-roots-and-filesystem.md"))
        .expect("doc path resolution");

    assert!(
        doc_path.exists(),
        "F-08 filesystem documentation must exist"
    );

    let content = std::fs::read_to_string(&doc_path).expect("read documentation");

    assert!(
        content.contains("LongPathsEnabled"),
        "documentation must name LongPathsEnabled registry requirement (PLT-015)"
    );
}
