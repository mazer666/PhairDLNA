//! The commands the service manager is asked to run.
//!
//! What: the trait that runs a program, the runner that runs it for real, and
//!       the runner that only remembers — the double the tests use.
//! Why: a service command must be able to be tested without touching the
//!      machine of the user; the double records what *would* have been run
//!      (`F-13` § 11).
//! Example: `RecordingRunner` answers every call with the same result and
//!      remembers the calls, so that a test can say "the service was started".

use std::sync::Mutex;

/// What a program answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    /// The exit code of the program, when it ran.
    pub code: Option<i32>,
    /// What the program wrote to its standard output.
    pub stdout: String,
    /// What the program wrote to its standard error.
    pub stderr: String,
}

impl CommandOutcome {
    /// The outcome of a program that succeeded.
    pub fn success(stdout: &str) -> Self {
        Self {
            code: Some(0),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    /// The outcome of a program that failed.
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
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<CommandOutcome>;
}

/// The runner that runs the program for real.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemRunner;

impl ServiceRunner for SystemRunner {
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<CommandOutcome> {
        let output = std::process::Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| {
                std::io::Error::new(
                    error.kind(),
                    format!("the program \"{program}\" cannot be run: {error}"),
                )
            })?;
        Ok(CommandOutcome {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// One call that a runner made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The program that was run.
    pub program: String,
    /// The arguments the program was run with.
    pub arguments: Vec<String>,
}

/// The runner that only remembers.
///
/// The runner is the double of the tests: it runs nothing and answers the
/// same way every time, so that a test can prove what the service command
/// *would* do without touching the machine of the user.
#[derive(Debug)]
pub struct RecordingRunner {
    /// The calls that were made.
    calls: Mutex<Vec<Call>>,
    /// The answer that every call receives.
    answer: Mutex<CommandOutcome>,
}

impl RecordingRunner {
    /// Builds a runner that answers with a successful result.
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            answer: Mutex::new(CommandOutcome::success("")),
        }
    }

    /// Builds a runner that answers with a given result.
    pub fn answering(answer: CommandOutcome) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            answer: Mutex::new(answer),
        }
    }

    /// The calls that were made, in the order they were made.
    pub fn calls(&self) -> Vec<Call> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// The command line of the last call.
    pub fn last_call(&self) -> Option<Call> {
        self.calls().pop()
    }

    /// The whole command line of the last call, as a person would type it.
    pub fn last_command_line(&self) -> Option<String> {
        let call = self.last_call()?;
        let mut line = call.program;
        for argument in &call.arguments {
            line.push(' ');
            line.push_str(argument);
        }
        Some(line)
    }
}

impl Default for RecordingRunner {
    /// The runner of the tests answers with a successful result and remembers
    /// every call.
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRunner for RecordingRunner {
    fn run(&self, program: &str, arguments: &[String]) -> std::io::Result<CommandOutcome> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Call {
                program: program.to_owned(),
                arguments: arguments.to_vec(),
            });
        Ok(self
            .answer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandOutcome, RecordingRunner, ServiceRunner, SystemRunner};

    /// The positive case: the runner remembers what it was asked to run.
    #[test]
    fn the_runner_remembers_the_calls() {
        let runner = RecordingRunner::new();
        runner
            .run("launchctl", &["list".to_owned()])
            .expect("the call is answered");

        let call = runner.last_call().expect("a call was made");
        assert_eq!(call.program, "launchctl");
        assert_eq!(call.arguments, vec!["list".to_owned()]);
    }

    /// The positive case: the runner answers with the result it was given.
    #[test]
    fn the_runner_answers_with_the_given_result() {
        let runner = RecordingRunner::answering(CommandOutcome::failure(113, "not found"));
        let outcome = runner.run("launchctl", &[]).expect("the call is answered");

        assert!(!outcome.is_success());
        assert_eq!(outcome.stderr, "not found");
    }

    /// The positive case: the real runner runs a program that exists.
    #[test]
    fn the_real_runner_runs_a_program() {
        let runner = SystemRunner;
        let outcome = runner
            .run("/bin/sh", &["-c".to_owned(), "echo hello".to_owned()])
            .expect("the program runs");

        assert!(outcome.is_success());
        assert_eq!(outcome.stdout.trim(), "hello");
    }

    /// The negative case: a program that does not exist is an error, not a
    /// panic.
    #[test]
    fn a_program_that_does_not_exist_is_an_error() {
        let runner = SystemRunner;
        let error = runner
            .run("/nowhere/phairdlna-launchctl", &[])
            .expect_err("the program cannot be run");

        assert!(error.to_string().contains("cannot be run"), "{error}");
    }
}
