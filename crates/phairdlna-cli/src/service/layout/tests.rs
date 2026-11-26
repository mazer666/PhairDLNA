use super::{LayoutRequest, Platform, Scope, ServiceLayout};
use std::path::PathBuf;

/// A program that exists, for the tests.
fn a_program() -> PathBuf {
    std::env::current_exe().expect("the running program")
}

/// A request for a layout on macOS.
fn a_request(home: &std::path::Path) -> LayoutRequest {
    LayoutRequest {
        platform: Platform::MacOs,
        scope: Scope::User,
        home: home.to_path_buf(),
        program: a_program(),
        config: home.join("config.toml"),
    }
}

/// The positive case: the server is started with the configuration it
/// should read.
#[test]
fn the_server_is_started_with_its_configuration() {
    let home = PathBuf::from("/Users/tester");
    let layout = ServiceLayout::build(&a_request(&home)).expect("a layout");

    assert_eq!(layout.arguments[0], "serve");
    assert_eq!(layout.arguments[1], "--config");
    assert!(layout.arguments[2].ends_with("config.toml"), "{layout:?}");
}

/// The negative case: a program that does not exist stops the layout.
#[test]
fn a_program_that_does_not_exist_is_refused() {
    let home = PathBuf::from("/Users/tester");
    let mut request = a_request(&home);
    request.program = PathBuf::from("/nowhere/phairdlna");
    let error = ServiceLayout::build(&request).expect_err("the layout is refused");

    assert!(error.contains("does not exist"), "{error}");
}

/// The negative case: a relative program is refused.
#[test]
fn a_relative_program_is_refused() {
    let home = PathBuf::from("/Users/tester");
    let mut request = a_request(&home);
    request.program = PathBuf::from("phairdlna");
    let error = ServiceLayout::build(&request).expect_err("the layout is refused");

    assert!(error.contains("absolute"), "{error}");
}

/// The negative case: a job file outside the folder of the platform is
/// refused, so that the command cannot write a file where it should not.
#[test]
fn a_job_file_in_the_wrong_place_is_refused() {
    let home = PathBuf::from("/Users/tester");
    let mut layout = ServiceLayout::build(&a_request(&home)).expect("a layout");
    layout.unit_path = PathBuf::from("/tmp/org.phairdlna.server.plist");

    let error = layout.validate().expect_err("the layout is refused");
    assert!(error.contains("must lie in"), "{error}");
}

/// The negative case: a log file that is the job file is refused.
#[test]
fn a_log_file_that_is_the_job_file_is_refused() {
    let home = PathBuf::from("/Users/tester");
    let mut layout = ServiceLayout::build(&a_request(&home)).expect("a layout");
    layout.stdout_path = layout.unit_path.clone();

    let error = layout.validate().expect_err("the layout is refused");
    assert!(error.contains("must not be the file"), "{error}");
}
