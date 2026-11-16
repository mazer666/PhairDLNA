use super::{run_in, ServiceCommand, ServiceEnvironment};
use crate::service::layout::Platform;
use crate::service::runner::{CommandOutcome, RecordingRunner};
use std::path::{Path, PathBuf};

/// An environment of a machine that does not exist.
fn an_environment(home: &std::path::Path) -> ServiceEnvironment {
    ServiceEnvironment {
        platform: Platform::MacOs,
        home: home.to_path_buf(),
        program: std::env::current_exe().expect("the running program"),
        config: home.join("config.toml"),
    }
}

/// The file that describes the service of the user.
fn the_unit_path(home: &Path) -> PathBuf {
    home.join("Library")
        .join("LaunchAgents")
        .join("org.phairdlna.server.plist")
}

/// Writes a file of the service that is already there.
fn an_installed_service(home: &Path) -> PathBuf {
    let path = the_unit_path(home);
    std::fs::create_dir_all(path.parent().expect("the folder")).expect("the folder is made");
    std::fs::write(&path, "old").expect("the file is written");
    path
}

/// The positive case: the status of a service that is not installed says
/// that there is nothing to do.
#[test]
fn a_service_that_is_not_installed_says_so() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::answering(CommandOutcome::failure(113, "not found"));
    let outcome = run_in(
        &ServiceCommand::Status { system: false },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 3);
    assert!(
        outcome.stdout.contains("not installed"),
        "{}",
        outcome.stdout
    );
}

/// The positive case: the dry run says what would be written and writes
/// nothing.
#[test]
fn the_dry_run_writes_nothing() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: false,
            dry_run: true,
            system: false,
            config: None,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(
        outcome.stdout.contains("would be written"),
        "{}",
        outcome.stdout
    );
    assert!(
        !directory.path().join("Library").exists(),
        "nothing was written: {:?}",
        directory.path()
    );
}

/// The positive case: the install writes the file of the service and
/// registers it with the manager of the platform.
#[test]
fn the_install_writes_the_file_and_registers_the_service() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: false,
            dry_run: false,
            system: false,
            config: None,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    let path = the_unit_path(directory.path());
    assert!(path.exists(), "the file of the service is there: {path:?}");
    let file = std::fs::read_to_string(&path).expect("the file is read");
    assert!(file.contains("org.phairdlna.server"), "{file}");
    let call = runner.last_command_line().expect("the manager was asked");
    assert!(call.starts_with("launchctl load"), "{call}");
}

/// The negative case: a file that is already there is not replaced
/// without the word of the user.
#[test]
fn a_file_that_is_already_there_is_not_replaced() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = an_installed_service(directory.path());
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: false,
            dry_run: false,
            system: false,
            config: None,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 1, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("--force"), "{}", outcome.stdout);
    assert_eq!(
        std::fs::read_to_string(&path).expect("the file is read"),
        "old",
        "the file of the user is untouched"
    );
    assert!(runner.calls().is_empty(), "the manager was not asked");
}

/// The positive case: the force replaces the file of the service.
#[test]
fn the_force_replaces_the_file_of_the_service() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = an_installed_service(directory.path());
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: true,
            dry_run: false,
            system: false,
            config: None,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    let file = std::fs::read_to_string(&path).expect("the file is read");
    assert!(file.contains("org.phairdlna.server"), "{file}");
}

/// The negative case: a configuration file that is not an absolute path is
/// refused, because a service file with a relative path starts a server
/// that reads the wrong file.
#[test]
fn a_relative_configuration_is_refused() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: false,
            dry_run: true,
            system: false,
            config: Some(PathBuf::from("config.toml")),
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 1, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("absolute"), "{}", outcome.stdout);
}

/// The positive case: the uninstall removes the file and the service.
#[test]
fn the_uninstall_removes_the_file_and_the_service() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = an_installed_service(directory.path());
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Uninstall {
            force: false,
            system: false,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(!path.exists(), "the file of the service is gone");
    let call = runner.last_command_line().expect("the manager was asked");
    assert!(call.starts_with("launchctl unload"), "{call}");
}

/// The negative case: the uninstall of a service that is not there says
/// that there is nothing to do.
#[test]
fn the_uninstall_of_a_service_that_is_not_there_says_so() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Uninstall {
            force: false,
            system: false,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 3, "{}", outcome.stdout);
    assert!(
        outcome.stdout.contains("nothing to remove"),
        "{}",
        outcome.stdout
    );
    assert!(runner.calls().is_empty(), "the manager was not asked");
}

/// The positive case: the status says that the service runs, and names
/// the process of the server.
#[test]
fn the_status_says_that_the_service_runs() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    an_installed_service(directory.path());
    let runner = RecordingRunner::answering(CommandOutcome::success(
        "PID\tStatus\tLabel\n4321\t0\torg.phairdlna.server\n",
    ));
    let outcome = run_in(
        &ServiceCommand::Status { system: false },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("running"), "{}", outcome.stdout);
    assert!(outcome.stdout.contains("4321"), "{}", outcome.stdout);
}

/// The positive case: the start and the stop ask the manager of the
/// platform.
#[test]
fn the_start_and_the_stop_ask_the_manager() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    an_installed_service(directory.path());
    let runner = RecordingRunner::new();

    let started = run_in(
        &ServiceCommand::Start { system: false },
        &runner,
        &an_environment(directory.path()),
    );
    assert_eq!(started.exit_code, 0, "{}", started.stdout);
    let call = runner.last_command_line().expect("the manager was asked");
    assert_eq!(call, "launchctl start org.phairdlna.server");

    let stopped = run_in(
        &ServiceCommand::Stop { system: false },
        &runner,
        &an_environment(directory.path()),
    );
    assert_eq!(stopped.exit_code, 0, "{}", stopped.stdout);
    let call = runner.last_command_line().expect("the manager was asked");
    assert_eq!(call, "launchctl stop org.phairdlna.server");
}

/// The boundary case: the system scope writes into the folder of the
/// machine, and the dry run says so without touching it.
#[test]
fn the_system_scope_names_the_folder_of_the_machine() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let runner = RecordingRunner::new();
    let outcome = run_in(
        &ServiceCommand::Install {
            force: false,
            dry_run: true,
            system: true,
            config: None,
        },
        &runner,
        &an_environment(directory.path()),
    );

    assert_eq!(outcome.exit_code, 0, "{}", outcome.stdout);
    assert!(
        outcome.stdout.contains("/Library/LaunchDaemons"),
        "{}",
        outcome.stdout
    );
    assert!(
        !directory.path().join("Library").exists(),
        "nothing was written: {:?}",
        directory.path()
    );
}
