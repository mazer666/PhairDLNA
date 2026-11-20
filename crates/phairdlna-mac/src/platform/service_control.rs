//! The control of the job of the server in `launchd`.
//!
//! What: starts and stops the job `org.phairdlna.server` with `launchctl`.
//! Why: the application is a front end for the server and not the server
//!      itself; the server keeps serving when the application is closed
//!      (`MB-001`), and the service manager of the system is the one that
//!      decides whether a job runs (`PLT-010`).
//! Example: `ServiceControl::system().start()` asks `launchd` to start the
//!          job of the server.
//!
//! **The scope of the job.** The job is the one of the user: its label names
//! no domain, so the command acts in the domain of the session of the user. A
//! job of the machine would be named `system/<label>` and would need a
//! privilege that the product does not ask for (`OPS-015`, `PLT-018`).
//!
//! **The gate.** These controls are not wired into the user interface yet: the
//! identity of the service and its access to the folders of the user are still
//! open (`OD-14`, `OD-15`). The module is the mechanism, and the mechanism is
//! built so that the decision can be taken without touching it again.
//!
//! **The thread.** The commands wait for `launchctl`, so they belong to the
//! worker thread and not to the main thread (`MB-002`).

use std::process::Command;

/// The label of the job of the server.
///
/// The label is the one the service of the command line writes into its
/// property list, so that both ways of the product speak to the same job.
pub const LABEL: &str = "org.phairdlna.server";

/// The program that manages the jobs of the system.
///
/// The path is absolute on purpose: the application is started by the system
/// of the windows and knows no search path of its own, and a program that is
/// looked up in a path could be one that the user placed there.
pub const LAUNCHCTL: &str = "/bin/launchctl";

/// What is asked of the service manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The job is started.
    Start,
    /// The job is stopped.
    Stop,
}

impl Action {
    /// The word of the command of `launchctl` that carries out the action.
    pub fn verb(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
        }
    }
}

/// What a program answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The exit code of the program, when it ran.
    pub code: Option<i32>,
    /// What the program wrote to its standard output.
    pub stdout: String,
    /// What the program wrote to its standard error.
    pub stderr: String,
}

impl Outcome {
    /// The answer of a program that succeeded.
    pub fn success() -> Self {
        Self {
            code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// The answer of a program that failed.
    pub fn failure(code: i32, stderr: &str) -> Self {
        Self {
            code: Some(code),
            stdout: String::new(),
            stderr: stderr.to_owned(),
        }
    }

    /// Whether the program succeeded.
    pub fn is_success(&self) -> bool {
        self.code == Some(0)
    }
}

/// A program that is run.
pub trait ServiceRunner {
    /// Runs a program with arguments and waits for it to end.
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<Outcome>;
}

/// The runner that runs the program for real.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemRunner;

impl ServiceRunner for SystemRunner {
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<Outcome> {
        let output = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| {
                std::io::Error::new(
                    error.kind(),
                    format!("the program {program} cannot be run: {error}"),
                )
            })?;
        Ok(Outcome {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Why the job was not started or not stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceError {
    /// The request was refused before any command was run.
    Refused {
        /// What was wrong with the request.
        reason: String,
    },
    /// The service manager refused the command.
    Failed {
        /// What the system said.
        reason: String,
    },
}

impl ServiceError {
    /// What was wrong, in the words of the system.
    pub fn reason(&self) -> &str {
        match self {
            Self::Refused { reason } | Self::Failed { reason } => reason,
        }
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.reason())
    }
}

impl std::error::Error for ServiceError {}

/// The control of the job of the server.
#[derive(Debug)]
pub struct ServiceControl<R> {
    /// The label of the job.
    label: String,
    /// The runner that asks the program of the system.
    runner: R,
}

impl<R: ServiceRunner> ServiceControl<R> {
    /// The control of the job of the server.
    pub fn new(runner: R) -> Self {
        Self {
            label: LABEL.to_owned(),
            runner,
        }
    }

    /// The control of a job with a given label.
    ///
    /// The label is refused when it is not the label of a job: a label with a
    /// space or a path separator is never the label of our job, and a command
    /// that names one is a command that was built from something this project
    /// does not control.
    pub fn with_label(label: &str, runner: R) -> Result<Self, ServiceError> {
        if !is_a_label(label) {
            return Err(ServiceError::Refused {
                reason: format!("the label {label} is not the label of a job of launchd"),
            });
        }
        Ok(Self {
            label: label.to_owned(),
            runner,
        })
    }

    /// The label of the job.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Starts the job.
    pub fn start(&self) -> Result<(), ServiceError> {
        self.perform(Action::Start)
    }

    /// Stops the job.
    pub fn stop(&self) -> Result<(), ServiceError> {
        self.perform(Action::Stop)
    }

    /// Asks the service manager to do something.
    ///
    /// The command names the program by its absolute path and passes the label
    /// as an argument of the program, so that no shell ever sees the label.
    fn perform(&self, action: Action) -> Result<(), ServiceError> {
        let arguments = vec![action.verb().to_owned(), self.label.clone()];
        let outcome =
            self.runner
                .run(LAUNCHCTL, &arguments)
                .map_err(|error| ServiceError::Failed {
                    reason: format!(
                        "the command {} cannot be run: {error}",
                        command_line(LAUNCHCTL, &arguments)
                    ),
                })?;
        if outcome.is_success() {
            return Ok(());
        }
        Err(ServiceError::Failed {
            reason: the_words_of_launchctl(&outcome, &arguments),
        })
    }
}

impl ServiceControl<SystemRunner> {
    /// The control of the job of the server, with the runner that runs the
    /// program of the system for real.
    pub fn system() -> Self {
        Self {
            label: LABEL.to_owned(),
            runner: SystemRunner,
        }
    }
}

/// The words of `launchctl` about a command that failed.
///
/// The words of the system are kept: the user should see what the system said
/// and not a sentence of ours.
fn the_words_of_launchctl(outcome: &Outcome, arguments: &[String]) -> String {
    let reason = if outcome.stderr.trim().is_empty() {
        outcome.stdout.trim()
    } else {
        outcome.stderr.trim()
    };
    if reason.is_empty() {
        return format!(
            "the command {} failed and said nothing",
            command_line(LAUNCHCTL, arguments)
        );
    }
    format!(
        "the command {} failed: {reason}",
        command_line(LAUNCHCTL, arguments)
    )
}

/// The command line of a program, as a person would type it.
fn command_line(program: &str, arguments: &[String]) -> String {
    let mut line = program.to_owned();
    for argument in arguments {
        line.push(' ');
        line.push_str(argument);
    }
    line
}

/// Whether a text is the label of a job of `launchd`.
///
/// A label of this project is a name in reverse domain order: letters, digits,
/// dots, dashes and underscores. Anything else is refused.
fn is_a_label(label: &str) -> bool {
    !label.is_empty()
        && label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character))
}
