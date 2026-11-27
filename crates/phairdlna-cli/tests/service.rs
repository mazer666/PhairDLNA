//! Tests of the `service` commands of the command line.
//!
//! What: proves that the five commands of the service — install, uninstall,
//!       status, start and stop — are offered, that they answer in the words of
//!       a person, and that they use the exit codes of the standards.
//! Why: the service commands are how the server runs without a person in front
//!       of it; a command that is not there, or that lies about what it did, is
//!       a defect the user meets the first time they install the server
//!       (`OPS-009`, `OPS-014`).
//! Example: `the_status_of_a_service_that_is_not_installed` runs the binary
//!       with a home directory of its own and reads the answer of the command.
//!
//! **The machine of the user is never touched.** Every test sets `HOME` to a
//! directory of its own, and the only command that writes a file is run with
//! `--dry-run`: the tests never ask the service manager of the machine to do
//! anything.

use std::process::Command;

/// The answer of a command of the command line.
struct Answer {
    /// The exit code of the process.
    code: i32,
    /// What the command wrote to its standard output.
    stdout: String,
    /// What the command wrote to its standard error.
    stderr: String,
}

/// Runs a command of the command line in a home directory of the test.
fn run_in(home: &std::path::Path, arguments: &[&str]) -> Answer {
    let output = Command::new(env!("CARGO_BIN_EXE_phairdlna-cli"))
        .args(arguments)
        .env("HOME", home)
        .output()
        .expect("the command runs");
    Answer {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// The positive case: the status of a service that is not installed says that
/// there is nothing to do, and the exit code says the same.
#[test]
fn the_status_of_a_service_that_is_not_installed_says_so() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["service", "status"]);

    assert_eq!(answer.code, 3, "{}", answer.stdout);
    assert!(answer.stdout.contains("not installed"), "{}", answer.stdout);
}

/// The positive case: the dry run of the install says what would be written
/// and writes nothing.
#[test]
fn the_dry_run_of_the_install_writes_nothing() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["service", "install", "--dry-run"]);

    assert_eq!(answer.code, 0, "{}", answer.stdout);
    assert!(
        answer.stdout.contains("would be written"),
        "{}",
        answer.stdout
    );
    let expected_service_marker = if cfg!(target_os = "linux") {
        "phairdlna.service"
    } else {
        "org.phairdlna.server"
    };
    assert!(
        answer.stdout.contains(expected_service_marker),
        "{}",
        answer.stdout
    );
    assert!(
        !directory.path().join("Library").exists(),
        "nothing was written: {:?}",
        directory.path()
    );
}

/// The positive case: the usage text names the five commands of the service.
#[test]
fn the_usage_names_the_commands_of_the_service() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["--help"]);

    for command in ["install", "uninstall", "status", "start", "stop"] {
        assert!(
            answer.stderr.contains(&format!("service {command}")),
            "the usage names the command: {}",
            answer.stderr
        );
    }
}

/// The negative case: the service command without a subcommand says what it
/// needs, and the exit code says that the arguments were wrong.
#[test]
fn the_service_command_without_a_subcommand_is_refused() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["service"]);

    assert_eq!(answer.code, 2, "{}", answer.stderr);
    assert!(
        answer.stderr.contains("service needs a subcommand"),
        "{}",
        answer.stderr
    );
}

/// The negative case: a subcommand that does not exist is refused.
#[test]
fn a_subcommand_that_does_not_exist_is_refused() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["service", "restart"]);

    assert_eq!(answer.code, 2, "{}", answer.stderr);
    assert!(
        answer.stderr.contains("unknown service command"),
        "{}",
        answer.stderr
    );
}

/// The negative case: the options of the service are refused in a command
/// that is not one, so that the user is not left believing that something
/// happened that did not happen.
#[test]
fn the_options_of_the_service_are_refused_elsewhere() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let answer = run_in(directory.path(), &["config", "path", "--force"]);

    assert_eq!(answer.code, 2, "{}", answer.stderr);
    assert!(
        answer
            .stderr
            .contains("--force belongs to the service commands"),
        "{}",
        answer.stderr
    );
}
