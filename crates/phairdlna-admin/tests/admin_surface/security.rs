//! Acceptance tests for the security behavior of the administration surface.

use super::*;

/// The negative case: a request without a token is refused.
#[tokio::test]
async fn a_request_without_a_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, _token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[header("X-Phair-Admin", "1")],
    )
    .await;

    assert_eq!(answer.status, 401, "{}", answer.body);
}

/// The negative case: a request with a wrong token is refused.
#[tokio::test]
async fn a_request_with_a_wrong_token_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, _token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[
            header("Authorization", "Bearer 0000"),
            header("X-Phair-Admin", "1"),
        ],
    )
    .await;

    assert_eq!(answer.status, 401, "{}", answer.body);
}

/// The negative case: a request that names another host is refused.
///
/// This is the attack the rule exists for: a name the attacker controls points
/// the browser of the user at the machine of the user (`SEC-TH-23`).
#[tokio::test]
async fn a_request_that_names_another_host_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut stream = TcpStream::connect(address)
        .await
        .expect("the surface accepts");
    let text = format!(
        "GET /api/status HTTP/1.1\r\nHost: attacker.example\r\nAuthorization: Bearer {token}\r\n\
         X-Phair-Admin: 1\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(text.as_bytes())
        .await
        .expect("the request is sent");
    let mut raw = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut raw))
        .await
        .expect("the answer comes in time")
        .expect("the answer is read");

    let answer = parse(&raw);
    assert_eq!(answer.status, 421, "{}", answer.body);
}

/// The negative case: a request without the custom header is refused.
///
/// A form in a browser cannot send the header (`OPS-020`).
#[tokio::test]
async fn a_request_without_the_custom_header_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(
        address,
        "GET",
        "/api/status",
        &[header("Authorization", &format!("Bearer {token}"))],
    )
    .await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request from another origin is refused.
#[tokio::test]
async fn a_request_from_another_origin_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("Origin", "http://attacker.example"));
    let answer = request(address, "GET", "/api/status", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: a request that a browser fetched for another page is
/// refused.
#[tokio::test]
async fn a_request_for_another_page_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let mut headers = allowed(&token);
    headers.push(header("Sec-Fetch-Site", "cross-site"));
    let answer = request(address, "GET", "/api/status", &headers).await;

    assert_eq!(answer.status, 403, "{}", answer.body);
}

/// The negative case: the answer never tells another origin that it may read
/// it.
#[tokio::test]
async fn the_answer_never_carries_a_cross_origin_header() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(answer.header("access-control-allow-origin"), None);
    assert_eq!(answer.header("access-control-allow-credentials"), None);
}

/// The negative case: a path the surface does not know is refused.
#[tokio::test]
async fn an_unknown_path_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "GET", "/api/secret", &allowed(&token)).await;

    assert_eq!(answer.status, 404, "{}", answer.body);
}

/// The negative case: the surface answers no question about permissions.
#[tokio::test]
async fn a_preflight_question_is_refused() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let (_server, token, address) = start(&folder.path().join("admin.token")).await;

    let answer = request(address, "OPTIONS", "/api/status", &allowed(&token)).await;

    assert_eq!(answer.status, 405, "{}", answer.body);
    assert_eq!(answer.header("access-control-allow-methods"), None);
}
