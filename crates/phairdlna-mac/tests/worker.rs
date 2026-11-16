//! The acceptance criteria of the thread that talks to the surface.
//!
//! What: the tests that check the worker — that it asks the surface on
//!       another thread, that it answers with the state of the server, and
//!       that a surface that is not there becomes an answer and not a crash.
//! Why: the main thread of a menu bar application must never wait for the
//!      network; the worker is the part that does the waiting (`MB-002`).
//! Example: the worker answers with the state of the server, and the
//!          application takes that answer when the timer of the menu bar
//!          ticks.

use std::collections::VecDeque;
use std::time::Duration;

use phairdlna_mac::ipc::client::AdminClient;
use phairdlna_mac::ipc::http::{HttpRequest, HttpResponse, Transport, TransportError};
use phairdlna_mac::model::ServerState;
use phairdlna_mac::worker::{Answer, Command, Worker};

/// A transport that answers with recorded answers.
struct Recorded {
    /// The answers the transport gives, in order.
    answers: VecDeque<Result<HttpResponse, TransportError>>,
}

impl Recorded {
    /// A transport that gives the answers in order.
    fn with(answers: Vec<Result<HttpResponse, TransportError>>) -> Self {
        Self {
            answers: answers.into(),
        }
    }
}

impl Transport for Recorded {
    fn send(&mut self, _request: &HttpRequest) -> Result<HttpResponse, TransportError> {
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

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// The answer of a surface that runs.
fn answer_of_a_running_server() -> Result<HttpResponse, TransportError> {
    Ok(HttpResponse::new(
        200,
        Vec::new(),
        br#"{"active_streams": 2, "catalogue_objects": 9, "roots": []}"#.to_vec(),
    ))
}

/// The answer of the diagnostics of a surface that runs.
fn answer_of_the_diagnostics() -> Result<HttpResponse, TransportError> {
    Ok(HttpResponse::new(
        200,
        Vec::new(),
        br#"{"warnings": [], "transfers": [], "last_scan": null}"#.to_vec(),
    ))
}

/// The worker asks the surface when it starts and answers with its state.
#[test]
fn the_worker_answers_with_the_state_of_the_server() {
    let client = AdminClient::new(
        Recorded::with(vec![
            answer_of_a_running_server(),
            answer_of_the_diagnostics(),
        ]),
        "token",
    );
    let worker = Worker::spawn(client).expect("the worker starts");

    let answer = wait_for(&worker);

    let Answer::State(snapshot) = answer else {
        panic!("the answer must be the state of the server: {answer:?}");
    };
    assert_eq!(snapshot.state, ServerState::Running);
    assert_eq!(snapshot.active_streams, 2);
    assert_eq!(snapshot.catalogue_objects, 9);
}

/// The negative case: a surface that is not there is an answer and not a
/// crash.
#[test]
fn a_surface_that_is_not_there_is_an_answer() {
    let client = AdminClient::new(
        Recorded::with(vec![Err(TransportError::Connect {
            reason: "no route to host".to_owned(),
        })]),
        "token",
    );
    let worker = Worker::spawn(client).expect("the worker starts");

    let answer = wait_for(&worker);

    assert!(matches!(answer, Answer::Failed(_)), "{answer:?}");
}

/// The scan of the library is a command of the worker.
#[test]
fn the_scan_is_a_command_of_the_worker() {
    let client = AdminClient::new(
        Recorded::with(vec![
            answer_of_a_running_server(),
            answer_of_the_diagnostics(),
            answer_of_a_running_server(),
            Ok(HttpResponse::new(
                200,
                Vec::new(),
                br#"{"rescanned": true, "objects": 4, "duration_seconds": 1, "message": "ok"}"#
                    .to_vec(),
            )),
        ]),
        "token",
    );
    let worker = Worker::spawn(client).expect("the worker starts");
    let _ = wait_for(&worker);

    worker.ask(Command::Rescan);

    let answer = wait_for(&worker);
    let Answer::Scanned(report) = answer else {
        panic!("the answer must be the report of the scan: {answer:?}");
    };
    assert!(report.rescanned);
    assert_eq!(report.objects, 4);
}

/// Waits for the answer of the worker.
fn wait_for(worker: &Worker) -> Answer {
    for _ in 0..200 {
        if let Some(answer) = worker.take() {
            return answer;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the worker did not answer");
}
