//! The acceptance criteria of the menu of the application.
//!
//! What: the tests that check the entries of the menu — that the state of the
//!       server is a line of text, that the actions are possible only when
//!       they can do something, and that a warning of the server stays
//!       visible.
//! Why: the menu is the one view of the application; a menu that shows
//!      something else than the state of the server misleads the user
//!      (`UX-007`, `UX-016`, `UX-018`).
//! Example: a running server with two streams shows the line "Server running
//!          — 2 active streams" and the actions of the menu.

use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::menu::{accessibility_value, entries, status_line, MenuEntry};
use phairdlna_mac::model::{PresentationMode, RootInfo, ServerSnapshot, ServerState, TransferInfo};
use phairdlna_mac::text::Strings;

/// The texts of the language of the tests.
fn texts() -> Strings {
    Strings::load("en-US").expect("the language is shipped")
}

/// The lines of text of the menu.
fn lines_of(menu: &[MenuEntry]) -> Vec<String> {
    menu.iter()
        .filter_map(|entry| match entry {
            MenuEntry::Status { title } => Some(title.clone()),
            _ => None,
        })
        .collect()
}

/// The action of the menu, with the state that says whether it may be chosen.
fn action_of(menu: &[MenuEntry], action: MenuAction) -> Option<(String, bool)> {
    menu.iter().find_map(|entry| match entry {
        MenuEntry::Action {
            title,
            action: chosen,
            enabled,
            ..
        } if *chosen == action => Some((title.clone(), *enabled)),
        _ => None,
    })
}

/// A running server shows its streams and explains why service controls are absent.
#[test]
fn a_running_server_shows_its_streams() {
    let snapshot = ServerSnapshot {
        state: ServerState::Running,
        active_streams: 2,
        catalogue_objects: 128,
        roots: vec![RootInfo {
            name: "music".to_owned(),
            mounted: true,
            objects: 128,
        }],
        transfers: vec![TransferInfo {
            client: "the television".to_owned(),
            title: "a film".to_owned(),
        }],
        ..ServerSnapshot::default()
    };

    let menu = entries(&snapshot, &texts(), None, PresentationMode::Simple);
    let lines = lines_of(&menu);

    assert!(status_line(&snapshot, &texts()).contains('2'));
    assert!(lines.iter().any(|line| line.contains("2 active streams")));
    assert!(lines.iter().any(|line| line.contains("music")));
    assert!(lines.iter().any(|line| line.contains("the television")));
    assert!(lines
        .iter()
        .any(|line| { line == texts().text("menu.status.server_controls_unavailable") }));
    assert!(action_of(&menu, MenuAction::RescanLibrary).is_some());
    assert!(action_of(&menu, MenuAction::OpenMediaFolders).is_some());
}

/// The negative case: a server that is not there says so and offers less.
#[test]
fn a_stopped_server_says_so_and_offers_less() {
    let menu = entries(
        &ServerSnapshot::stopped(),
        &texts(),
        None,
        PresentationMode::Simple,
    );

    assert_eq!(
        status_line(&ServerSnapshot::stopped(), &texts()),
        texts().text("menu.status.stopped")
    );
    let (title, enabled) =
        action_of(&menu, MenuAction::RescanLibrary).expect("the action is there");
    assert!(!enabled, "the scan needs a server: {title}");
    let (title, enabled) =
        action_of(&menu, MenuAction::OpenMediaFolders).expect("the action is there");
    assert!(!enabled, "there is no folder to open: {title}");
}

/// The first refresh offers no server control until the app knows the state.
#[test]
fn a_checking_server_reports_its_state() {
    let snapshot = ServerSnapshot::checking();

    assert_eq!(
        status_line(&snapshot, &texts()),
        texts().text("menu.status.checking")
    );
}

/// A server that refuses the app is running, so the menu must not offer start.
#[test]
fn a_refusing_server_shows_recovery_actions_without_service_controls() {
    let snapshot = ServerSnapshot {
        state: ServerState::Refused,
        ..ServerSnapshot::default()
    };
    let menu = entries(&snapshot, &texts(), None, PresentationMode::Simple);

    assert_eq!(
        status_line(&snapshot, &texts()),
        texts().text("menu.status.refused")
    );
    assert!(lines_of(&menu)
        .iter()
        .any(|line| line == texts().text("menu.status.server_controls_unavailable")));
    assert!(action_of(&menu, MenuAction::RescanLibrary).is_some_and(|(_, enabled)| !enabled));
    assert!(action_of(&menu, MenuAction::AddMediaFolders).is_some());
    assert!(action_of(&menu, MenuAction::OpenPreferences).is_some());
}

/// The warning of the server is a line of the menu, in every state.
#[test]
fn a_warning_of_the_server_stays_visible() {
    let warning = "the firewall may block the connections".to_owned();
    let running = ServerSnapshot {
        state: ServerState::Running,
        warnings: vec![warning.clone()],
        ..ServerSnapshot::default()
    };
    let stopped = ServerSnapshot {
        state: ServerState::Stopped,
        warnings: vec![warning.clone()],
        ..ServerSnapshot::default()
    };

    let menu = entries(&running, &texts(), None, PresentationMode::Simple);
    let lines = lines_of(&menu);

    assert!(lines.contains(&warning));
    assert!(
        lines_of(&entries(&stopped, &texts(), None, PresentationMode::Simple)).contains(&warning)
    );
}

/// The state of the server is a text for the screen reader, not only a
/// picture.
#[test]
fn the_state_is_a_text_for_the_screen_reader() {
    let running = ServerSnapshot {
        state: ServerState::Running,
        active_streams: 3,
        ..ServerSnapshot::default()
    };

    let value = accessibility_value(&running, &texts());

    assert!(
        value.contains('3'),
        "the value must hold the number: {value}"
    );
    assert_ne!(
        value,
        accessibility_value(&ServerSnapshot::stopped(), &texts())
    );
}

/// The message of the platform is a line of the menu.
#[test]
fn a_message_of_the_platform_is_a_line_of_the_menu() {
    let menu = entries(
        &ServerSnapshot::stopped(),
        &texts(),
        Some("the server service is not installed"),
        PresentationMode::Simple,
    );

    let lines = lines_of(&menu);

    assert!(lines.iter().any(|line| line.contains("not installed")));
}

/// The menu holds every action of the application, and the actions are the
/// ones of the crate.
#[test]
fn the_menu_holds_supported_actions_but_no_unapproved_service_controls() {
    let stopped_menu = entries(
        &ServerSnapshot::stopped(),
        &texts(),
        None,
        PresentationMode::Simple,
    );
    let running_menu = entries(
        &ServerSnapshot {
            state: ServerState::Running,
            ..ServerSnapshot::default()
        },
        &texts(),
        None,
        PresentationMode::Simple,
    );

    assert!(lines_of(&stopped_menu)
        .iter()
        .any(|line| line == texts().text("menu.status.server_controls_unavailable")));
    assert!(lines_of(&running_menu)
        .iter()
        .any(|line| line == texts().text("menu.status.server_controls_unavailable")));

    let actions = [
        MenuAction::OpenMediaFolders,
        MenuAction::RescanLibrary,
        MenuAction::OpenPreferences,
        MenuAction::ExportDiagnostics,
        MenuAction::Quit,
    ];

    for action in actions {
        let (title, _) = action_of(&running_menu, action).expect("the action is in the menu");
        assert!(!title.is_empty(), "the action must have a text");
    }
}
