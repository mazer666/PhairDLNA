//! The acceptance criteria of the control of the job of the server.
//!
//! What: the cases that prove which command of `launchctl` the control runs,
//!       what a refusal of the service manager looks like, and that the
//!       control asks for no privilege.
//! Why: the control of a job touches the system of the user, and a command
//!      that is wrong stops the server of somebody else (`PLT-010`, `OPS-015`).
//! Example: starting the job runs `/bin/launchctl start org.phairdlna.server`.
//!
//! The module under test exists on macOS alone, and so does this file.

#![cfg(target_os = "macos")]

use std::sync::{Arc, Mutex};

use phairdlna_mac::platform::service_control::{
    Action, Outcome, ServiceControl, ServiceError, ServiceRunner, SystemRunner, LABEL, LAUNCHCTL,
};

/// What the double of the tests answers.
#[derive(Clone, Debug)]
enum Answer {
    /// The program answered with this result.
    With(Outcome),
    /// The program could not be run at all.
    NotRun,
}

/// One call that the runner made: the program and its arguments.
type Call = (String, Vec<String>);

/// The calls that were made, shared between the runner and the test.
type Calls = Arc<Mutex<Vec<Call>>>;

/// The runner that remembers the calls and answers with a fixed result.
///
/// The log of the calls is shared, so that the test can read it after the
/// control has taken the runner.
#[derive(Clone, Debug)]
struct RecordingRunner {
    /// The calls that were made.
    calls: Calls,
    /// The answer that every call receives.
    answer: Answer,
}

impl RecordingRunner {
    /// A runner that answers with an answer.
    fn answering(answer: Answer) -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            answer,
        }
    }

    /// The calls that were made, in the order they were made.
    fn calls(&self) -> Vec<Call> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl ServiceRunner for RecordingRunner {
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<Outcome> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((program.to_owned(), arguments.to_vec()));
        match &self.answer {
            Answer::With(outcome) => Ok(outcome.clone()),
            Answer::NotRun => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "the program is not there",
            )),
        }
    }
}

/// The positive case: starting the job asks `launchctl` to start it.
#[test]
fn the_start_command_asks_launchctl_to_start_the_job() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::new(runner.clone());

    control.start().expect("the job is started");

    let calls = runner.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].0, LAUNCHCTL);
    assert_eq!(calls[0].1, vec!["start".to_owned(), LABEL.to_owned()]);
}

/// The positive case: stopping the job asks `launchctl` to stop it.
#[test]
fn the_stop_command_asks_launchctl_to_stop_the_job() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::new(runner.clone());

    control.stop().expect("the job is stopped");

    let calls = runner.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].1, vec!["stop".to_owned(), LABEL.to_owned()]);
}

/// The positive case: the program is named by its absolute path, so that no
/// path of the user can hold another program.
#[test]
fn the_program_is_named_by_its_absolute_path() {
    assert!(LAUNCHCTL.starts_with('/'), "{LAUNCHCTL}");
    assert!(std::path::Path::new(LAUNCHCTL).is_absolute(), "{LAUNCHCTL}");
}

/// The positive case: no command asks for a privilege, and no argument of the
/// program is an option of it.
#[test]
fn no_command_asks_for_a_privilege() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::new(runner.clone());

    control.start().expect("the job is started");
    control.stop().expect("the job is stopped");

    for (program, arguments) in runner.calls() {
        assert!(!program.contains("sudo"), "{program}");
        assert_eq!(arguments.len(), 2, "{arguments:?}");
        for argument in arguments {
            assert!(!argument.starts_with('-'), "{argument}");
        }
    }
}

/// The positive case: the label of the job is the label of the product.
#[test]
fn the_label_of_the_job_is_the_label_of_the_product() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::new(runner);

    assert_eq!(control.label(), "org.phairdlna.server");
    assert_eq!(Action::Start.verb(), "start");
    assert_eq!(Action::Stop.verb(), "stop");
}

/// The negative case: a `launchctl` that refuses the command is an error that
/// carries the words of the system.
#[test]
fn a_refusal_of_launchctl_is_an_error_with_its_words() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::failure(
        113,
        "could not find service org.phairdlna.server",
    )));
    let control = ServiceControl::new(runner);

    let error = control.start().expect_err("the job is not started");

    assert!(error.reason().contains("could not find service"), "{error}");
    assert!(matches!(error, ServiceError::Failed { .. }), "{error:?}");
}

/// The negative case: a `launchctl` that says nothing is still an error.
#[test]
fn a_launchctl_that_says_nothing_is_an_error() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::failure(1, "")));
    let control = ServiceControl::new(runner);

    let error = control.stop().expect_err("the job is not stopped");

    assert!(error.reason().contains("said nothing"), "{error}");
}

/// The negative case: a program that cannot be run is an error and not a
/// panic.
#[test]
fn a_program_that_cannot_be_run_is_an_error() {
    let runner = RecordingRunner::answering(Answer::NotRun);
    let control = ServiceControl::new(runner);

    let error = control.start().expect_err("the job is not started");

    assert!(error.reason().contains("cannot be run"), "{error}");
}

/// The negative case: a label that is not the label of a job is refused
/// before any command is run.
#[test]
fn a_label_that_is_not_a_label_is_refused() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::with_label("org.phairdlna.server; rm -rf /", runner.clone());

    let error = control.expect_err("the label is refused");

    assert!(matches!(error, ServiceError::Refused { .. }), "{error:?}");
    assert!(error.reason().contains("is not the label"), "{error}");
    assert!(runner.calls().is_empty(), "no command may be run");
}

/// The negative case: an empty label is refused as well.
#[test]
fn an_empty_label_is_refused() {
    let runner = RecordingRunner::answering(Answer::With(Outcome::success()));
    let control = ServiceControl::with_label("", runner.clone());

    let error = control.expect_err("the label is refused");

    assert!(matches!(error, ServiceError::Refused { .. }), "{error:?}");
    assert!(runner.calls().is_empty(), "no command may be run");
}

/// The positive case: the runner that runs the program for real runs a
/// program that exists.
#[test]
fn the_real_runner_runs_a_program() {
    let runner = SystemRunner;
    let outcome = runner
        .run("/bin/echo", &["hello".to_owned()])
        .expect("the program runs");

    assert!(outcome.is_success());
    assert_eq!(outcome.stdout.trim(), "hello");
}

/// The negative case: the runner that runs the program for real reports a
/// program that does not exist as an error.
#[test]
fn the_real_runner_reports_a_program_that_does_not_exist() {
    let runner = SystemRunner;
    let error = runner
        .run("/nowhere/phairdlna-launchctl", &[])
        .expect_err("the program cannot be run");

    assert!(error.to_string().contains("cannot be run"), "{error}");
}
