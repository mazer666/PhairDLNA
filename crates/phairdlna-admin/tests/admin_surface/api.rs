//! Acceptance tests for the api behavior of the administration surface.

use super::*;

/// The positive case: the status of the server is answered.
#[tokio::test]
async fn the_status_is_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(
        answer.header("content-type"),
        Some("application/json; charset=\"utf-8\"")
    );
    assert!(
        answer.body.contains("\"uptime_seconds\":42"),
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("\"active_streams\":1"),
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("\"system_update_id\":3"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"Films\""), "{}", answer.body);
}

/// The positive case: the scan of the library is answered.
#[tokio::test]
async fn the_rescan_is_answered() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;
    let csrf = csrf_token(address, &token).await;

    let mut headers = allowed(&token);
    headers.push(header("X-Phair-Admin-Csrf", &csrf));
    let answer = request(address, "POST", "/api/catalogue/rescan", &headers).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert!(
        answer.body.contains("\"rescanned\":true"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("\"objects\":7"), "{}", answer.body);
}

/// The negative case: a request that changes something without the token of
/// the "cross-site request forgery" defence is refused.
#[tokio::test]
async fn a_change_without_the_csrf_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "POST", "/api/catalogue/rescan", &allowed(&token)).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request that changes something with a wrong token of
/// the "cross-site request forgery" defence is refused.
#[tokio::test]
async fn a_change_with_a_wrong_csrf_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("X-Phair-Admin-Csrf", "0000"));
    let answer = request(address, "POST", "/api/catalogue/rescan", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}
