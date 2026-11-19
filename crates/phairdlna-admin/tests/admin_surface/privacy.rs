//! Acceptance tests for the privacy behavior of the administration surface.

use super::*;

/// The positive case: the diagnostics are answered.
#[tokio::test]
async fn the_diagnostics_are_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/diagnostics", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert!(
        answer.body.contains("the library could not be read"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"warnings\""), "{}", answer.body);
}

/// The positive case: the diagnostics do not name the files of the user.
#[tokio::test]
async fn the_diagnostics_hide_the_paths() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/diagnostics", &allowed(&token)).await;

    assert!(
        !answer.body.contains("/Users/me"),
        "the path is gone: {}",
        answer.body
    );
    assert!(answer.body.contains("<path:1>"), "{}", answer.body);
}

/// The positive case: the token file is readable by its owner alone.
#[cfg(unix)]
#[tokio::test]
async fn the_token_file_is_private() {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().join("admin.token");
    let (_server, _token, _address) = start(&path).await;

    let mode = std::fs::metadata(&path)
        .expect("the token file is there")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "the file belongs to its owner alone");
}
