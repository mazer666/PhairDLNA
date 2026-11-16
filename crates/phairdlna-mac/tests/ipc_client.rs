//! The acceptance criteria of the client of the administration surface.
//!
//! What: the tests that check the client — that every request carries the
//!       token and the custom header, that the answers of the surface become
//!       snapshots, and that a refusal of the surface becomes the right
//!       error.
//! Why: the application is a client of the surface and knows nothing else of
//!      the server; a client that sends the wrong request shows the user
//!      nothing at all (`MB-005`, `SEC-071`).
//! Example: an answer that carries `"active_streams": 2` becomes a snapshot
//!          whose `active_streams` is two.

use std::collections::VecDeque;

use phairdlna_mac::ipc::client::{AdminClient, ClientError};
use phairdlna_mac::ipc::http::{HttpRequest, HttpResponse, Transport, TransportError};

/// A transport that answers with recorded answers and records the requests.
struct Recorded {
    /// The answers the transport gives, in order.
    answers: VecDeque<Result<HttpResponse, TransportError>>,
    /// The requests the transport received.
    requests: Vec<HttpRequest>,
}

impl Recorded {
    /// A transport that gives the answers in order.
    fn with(answers: Vec<Result<HttpResponse, TransportError>>) -> Self {
        Self {
            answers: answers.into(),
            requests: Vec::new(),
        }
    }

    /// A transport that gives one answer with a body.
    fn answering(body: &str) -> Self {
        Self::with(vec![Ok(HttpResponse::new(
            200,
            vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body.as_bytes().to_vec(),
        ))])
    }
}

impl Transport for Recorded {
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        self.requests.push(request.clone());
        self.answers.pop_front().unwrap_or_else(|| {
            Err(TransportError::Malformed {
                reason: "no recorded answer".to_owned(),
            })
        })
    }

    fn address(&self) -> String {
        "127.0.0.1:8210".to_owned()
    }

    fn host(&self) -> String {
        "127.0.0.1:8210".to_owned()
    }
}

/// The header of a request, out of the request.
fn header_of(request: &HttpRequest, name: &str) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(header, _)| header.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

/// Every request carries the token, the custom header and the host.
#[test]
fn every_request_carries_the_token_and_the_header() {
    let transport = Recorded::answering(r#"{"active_streams": 0}"#);
    let mut client = AdminClient::new(transport, "the-token");
    client.status().expect("the surface answers");

    let request = &client.transport().requests[0];
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/api/status");
    assert_eq!(
        header_of(request, "Authorization").as_deref(),
        Some("Bearer the-token")
    );
    assert_eq!(header_of(request, "X-Phair-Admin").as_deref(), Some("1"));
    assert_eq!(
        header_of(request, "Host").as_deref(),
        Some("127.0.0.1:8210")
    );
}

/// The answer of the surface becomes a snapshot.
#[test]
fn the_answer_of_the_surface_becomes_a_snapshot() {
    let body = r#"{
        "version": "0.1.0",
        "uptime_seconds": 42,
        "active_streams": 2,
        "catalogue_objects": 128,
        "roots": [
            {"name": "music", "path": "…/music", "mounted": true, "objects": 128}
        ]
    }"#;
    let mut client = AdminClient::new(Recorded::answering(body), "the-token");

    let status = client.status().expect("the surface answers");

    assert_eq!(status.uptime_seconds, 42);
    assert_eq!(status.active_streams, 2);
    assert_eq!(status.catalogue_objects, 128);
    assert_eq!(status.roots.len(), 1);
    assert_eq!(status.roots[0].name, "music");
    assert!(status.roots[0].mounted);
    assert_eq!(status.roots[0].objects, 128);
}

/// The scan carries the token of the "cross-site request forgery" defence.
#[test]
fn the_scan_carries_the_token_of_the_defence() {
    let transport = Recorded::with(vec![
        Ok(HttpResponse::new(
            200,
            vec![(
                "x-phairdlna-admin-csrf".to_owned(),
                "the-csrf-token".to_owned(),
            )],
            b"{}".to_vec(),
        )),
        Ok(HttpResponse::new(
            200,
            Vec::new(),
            br#"{"rescanned": true, "objects": 7, "duration_seconds": 2, "message": "ok"}"#
                .to_vec(),
        )),
    ]);
    let mut client = AdminClient::new(transport, "the-token");

    let report = client.rescan().expect("the surface answers");

    assert!(report.rescanned);
    assert_eq!(report.objects, 7);
    assert_eq!(report.duration_seconds, 2);
    let request = &client.transport().requests[1];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/api/catalogue/rescan");
    assert_eq!(
        header_of(request, "X-Phair-Admin-Csrf").as_deref(),
        Some("the-csrf-token")
    );
}

/// The negative case: a token that the surface refuses is an error of its own.
#[test]
fn a_refused_token_is_an_error_of_its_own() {
    let transport = Recorded::with(vec![Ok(HttpResponse::new(401, Vec::new(), Vec::new()))]);
    let mut client = AdminClient::new(transport, "the-token");

    let error = client.status().expect_err("the token is refused");

    assert_eq!(error, ClientError::Unauthorised);
}

/// The negative case: a request that the surface refuses names its number.
#[test]
fn a_refused_request_names_its_number() {
    let transport = Recorded::with(vec![Ok(HttpResponse::new(403, Vec::new(), Vec::new()))]);
    let mut client = AdminClient::new(transport, "the-token");

    let error = client.status().expect_err("the request is refused");

    assert_eq!(error, ClientError::Refused { status: 403 });
}

/// The negative case: an answer that is not json is refused.
#[test]
fn an_answer_that_is_not_json_is_refused() {
    let mut client = AdminClient::new(Recorded::answering("not json"), "the-token");

    let error = client.status().expect_err("the answer is refused");

    assert!(matches!(error, ClientError::Malformed { .. }), "{error:?}");
}

/// The negative case: a surface that is not there is a failure of the network.
#[test]
fn a_surface_that_is_not_there_is_a_failure_of_the_network() {
    let transport = Recorded::with(vec![Err(TransportError::Connect {
        reason: "no route to host".to_owned(),
    })]);
    let mut client = AdminClient::new(transport, "the-token");

    let error = client.status().expect_err("the surface is not there");

    assert!(matches!(error, ClientError::Transport(_)), "{error:?}");
}

/// The token of the installation is read out of its file.
///
/// The negative case: a file that another program of the machine may read is
/// refused, because a token that another program can read is no token.
#[test]
fn the_token_is_read_out_of_its_file() {
    use phairdlna_mac::ipc::client::read_token;

    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().join("admin.token");

    std::fs::write(&path, "the-token\n").expect("the file is written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("the permissions are set");
    }
    let token = read_token(&path).expect("the token is read");
    assert_eq!(token, "the-token");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("the permissions are set");
        assert!(read_token(&path).is_err(), "the file is refused");
    }
}
