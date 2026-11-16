use super::{commands_for, read_the_status, ServiceManager};
use crate::service::layout::{LayoutRequest, Platform, Scope, ServiceLayout};
use crate::service::runner::{CommandOutcome, RecordingRunner};
use std::path::PathBuf;

/// The program of the service, for the tests.
///
/// The program of a service must exist: a job file that names a program
/// that is not there starts a server that never answers.
fn a_program() -> PathBuf {
    std::env::current_exe().expect("the running program")
}

/// A layout on macOS, for the tests.
fn a_macos_layout() -> ServiceLayout {
    let request = LayoutRequest {
        platform: Platform::MacOs,
        scope: Scope::User,
        home: PathBuf::from("/Users/tester"),
        program: a_program(),
        config: PathBuf::from("/Users/tester/.config/phairdlna/config.toml"),
    };
    ServiceLayout::build(&request).expect("a layout")
}

/// A layout on Linux, for the tests.
fn a_linux_layout() -> ServiceLayout {
    let request = LayoutRequest {
        platform: Platform::Linux,
        scope: Scope::User,
        home: PathBuf::from("/home/tester"),
        program: a_program(),
        config: PathBuf::from("/home/tester/.config/phairdlna/config.toml"),
    };
    ServiceLayout::build(&request).expect("a layout")
}

/// The positive case: the commands of macOS are the commands of `launchctl`.
#[test]
fn the_macos_commands_are_the_commands_of_launchctl() {
    let layout = a_macos_layout();
    let commands = commands_for(&layout);

    assert_eq!(commands.program, "launchctl");
    assert_eq!(commands.load[0], "load");
    assert!(commands.load[2].ends_with("org.phairdlna.server.plist"));
    assert_eq!(
        commands.start,
        vec!["start".to_owned(), "org.phairdlna.server".to_owned()]
    );
}

/// The positive case: the commands of Linux are the commands of
/// `systemctl`, of the user.
#[test]
fn the_linux_commands_are_the_commands_of_systemctl() {
    let layout = a_linux_layout();
    let commands = commands_for(&layout);

    assert_eq!(commands.program, "systemctl");
    assert_eq!(commands.load, vec!["--user", "enable", "phairdlna.service"]);
    assert_eq!(
        commands.status,
        vec!["--user", "is-active", "phairdlna.service"]
    );
}

/// The negative case: a file that is already there is not replaced without
/// the word of the user.
#[test]
fn a_file_that_is_already_there_is_not_replaced() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let mut layout = a_macos_layout();
    layout.unit_path = directory.path().join("org.phairdlna.server.plist");
    std::fs::write(&layout.unit_path, "old").expect("the file is written");

    let runner = RecordingRunner::new();
    let manager = ServiceManager::new(&layout, &runner);
    let error = manager
        .write_unit_file(false)
        .expect_err("the file is kept");

    assert!(error.contains("--force"), "{error}");
    assert_eq!(
        std::fs::read_to_string(&layout.unit_path).expect("the file is read"),
        "old"
    );
}

/// The positive case: the file is written with the permissions of the
/// standards.
#[test]
fn the_file_is_written_with_the_right_permissions() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let mut layout = a_macos_layout();
    layout.unit_path = directory.path().join("org.phairdlna.server.plist");

    let runner = RecordingRunner::new();
    let manager = ServiceManager::new(&layout, &runner);
    manager.write_unit_file(false).expect("the file is written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&layout.unit_path)
            .expect("the file is there")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o644, "the mode of the file: {mode:o}");
    }
}

/// The negative case: a manager that says the service is not there is
/// reported as such.
#[test]
fn a_service_that_is_not_there_is_reported() {
    let runner = RecordingRunner::answering(CommandOutcome::failure(113, "not found"));
    let layout = a_macos_layout();
    let manager = ServiceManager::new(&layout, &runner);

    let status = manager.status().expect("the status is read");
    assert!(!status.loaded);
    assert!(!status.running);
    assert_eq!(status.pid, None);
}

/// The positive case: a running service is reported with its number.
#[test]
fn a_running_service_is_reported() {
    let runner = RecordingRunner::answering(CommandOutcome::success("active"));
    let layout = a_linux_layout();
    let manager = ServiceManager::new(&layout, &runner);

    let status = manager.status().expect("the status is read");
    assert!(status.loaded);
    assert!(status.running);
}

/// The answer of the manager is read.
#[test]
fn the_answer_of_the_manager_is_read() {
    let active = read_the_status(Platform::Linux, &CommandOutcome::success("active"));
    assert_eq!((active.0, active.1), (true, true));

    let inactive = read_the_status(Platform::Linux, &CommandOutcome::success("inactive"));
    assert_eq!((inactive.0, inactive.1), (true, false));

    let missing = read_the_status(Platform::Linux, &CommandOutcome::failure(113, "not found"));
    assert_eq!((missing.0, missing.1), (false, false));
}

/// The answer of `launchctl` is read: the second line carries the number.
#[test]
fn the_answer_of_launchctl_is_read() {
    let answer = CommandOutcome::success("PID\tStatus\tLabel\n1234\t-\torg.phairdlna.server");
    let (loaded, running, _) = read_the_status(Platform::MacOs, &answer);

    assert!(loaded);
    assert!(running);
}
