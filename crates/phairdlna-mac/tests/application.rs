//! The acceptance criteria of the application.
//!
//! What: the tests that check the wiring — that the answer of the surface
//!       becomes the state of the application, that a failure says what went
//!       wrong, and that the actions of the menu ask the system for the right
//!       thing.
//! Why: the application holds no state of the server of its own; a state
//!      that the application keeps after the server is gone is a lie the user
//!      reads (`MB-001`, `MB-005`).
//! Example: an answer that says that the server runs becomes the state
//!          "running", and a failure of the network becomes the state
//!          "stopped" with a message that says why.

use std::path::PathBuf;

use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::app::{ActionOutcome, App, DesktopRequest};
use phairdlna_mac::ipc::client::ClientError;
use phairdlna_mac::model::{ServerSnapshot, ServerState};
use phairdlna_mac::text::Strings;

/// The texts of the language of the tests.
fn texts() -> Strings {
    Strings::load("en-US").expect("the language is shipped")
}

/// The application of the tests.
fn application() -> App {
    App::new(
        texts(),
        PathBuf::from("/nowhere/config.toml"),
        "http://127.0.0.1:8210".to_owned(),
    )
}

/// The state of a server that runs.
fn running() -> ServerSnapshot {
    ServerSnapshot {
        state: ServerState::Running,
        active_streams: 1,
        catalogue_objects: 10,
        ..ServerSnapshot::default()
    }
}

/// The answer of the surface replaces the state of the application.
#[test]
fn the_answer_of_the_surface_replaces_the_state() {
    let mut app = application();
    assert_eq!(app.snapshot().state, ServerState::Stopped);

    let outcome = app.take_state(running());

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Changed);
    assert_eq!(app.snapshot().state, ServerState::Running);
    assert_eq!(app.snapshot().active_streams, 1);
}

/// The same answer twice changes nothing the second time.
#[test]
fn the_same_answer_twice_changes_nothing() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.take_state(running());

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Unchanged);
}

/// The negative case: a surface that is not there is a stopped server and a
/// message that says why.
#[test]
fn a_surface_that_is_not_there_is_a_stopped_server() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.take_failure(&ClientError::Transport(
        phairdlna_mac::ipc::http::TransportError::Connect {
            reason: "no route to host".to_owned(),
        },
    ));

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Changed);
    assert_eq!(app.snapshot().state, ServerState::Stopped);
    let lines = lines_of(&app);
    assert!(
        lines.iter().any(|line| line.contains("cannot be reached")),
        "the menu must say why: {lines:?}"
    );
}

/// The negative case: a surface that refuses the requests is not a surface
/// that is gone.
#[test]
fn a_surface_that_refuses_is_not_a_surface_that_is_gone() {
    let mut app = application();

    app.take_failure(&ClientError::Refused { status: 403 });

    assert_eq!(app.snapshot().state, ServerState::Refused);
    let lines = lines_of(&app);
    assert!(
        lines.iter().any(|line| line.contains("403")),
        "the menu must name the number: {lines:?}"
    );
}

/// The scan of the library leaves a message that says what it found.
#[test]
fn a_scan_leaves_a_message() {
    let mut app = application();
    app.take_state(running());

    app.take_rescan(&phairdlna_mac::ipc::snapshots::RescanReport {
        rescanned: true,
        objects: 12,
        duration_seconds: 1,
        message: "ok".to_owned(),
    });

    let lines = lines_of(&app);
    assert!(lines.iter().any(|line| line.contains('1')));
    assert!(lines.iter().any(|line| line.contains("12")));
}

/// The action that opens the administration page asks for the address of the
/// surface.
#[test]
fn the_action_that_opens_the_page_asks_for_the_address() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenAdmin);

    assert_eq!(
        outcome,
        ActionOutcome::Request(DesktopRequest::OpenUrl("http://127.0.0.1:8210".to_owned()))
    );
}

/// The action that opens the configuration asks for the file of the
/// configuration.
#[test]
fn the_action_that_opens_the_configuration_asks_for_the_file() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenPreferences);

    assert_eq!(
        outcome,
        ActionOutcome::Request(DesktopRequest::OpenPath(PathBuf::from(
            "/nowhere/config.toml"
        )))
    );
}

/// The negative case: a configuration that is not there means that there is
/// no folder to open, and the menu says so.
#[test]
fn a_configuration_that_is_not_there_means_no_folder() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenMediaFolders);

    assert_eq!(outcome, ActionOutcome::Nothing);
    let lines = lines_of(&app);
    assert!(
        lines.iter().any(|line| line.contains("No media folder")),
        "the menu must say so: {lines:?}"
    );
}

/// The scan is a command for the worker and not a request of the menu.
#[test]
fn the_scan_is_a_command_for_the_worker() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.perform(MenuAction::RescanLibrary);

    assert_eq!(outcome, ActionOutcome::Nothing);
    assert!(app.is_scanning());
    assert_eq!(app.snapshot().state, ServerState::Scanning);
}

/// The lines of text of the menu of the application.
fn lines_of(app: &App) -> Vec<String> {
    app.entries()
        .into_iter()
        .filter_map(|entry| match entry {
            phairdlna_mac::menu::MenuEntry::Status { title } => Some(title),
            _ => None,
        })
        .collect()
}
