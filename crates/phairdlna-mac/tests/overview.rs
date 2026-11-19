//! The acceptance criteria of the primary overview's state-dependent content.
//!
//! What: verify that every state maps to honest, localized text before AppKit displays it.
//! Why: stale or invented values mislead ordinary users (`AC-20`, `AC-22`).
//! Example: a refused response never appears as a zero-item library.

use phairdlna_mac::model::{RootInfo, ServerSnapshot, ServerState, TransferInfo};
use phairdlna_mac::overview::overview_content;
use phairdlna_mac::text::Strings;

fn strings() -> Strings {
    Strings::load("en-US").expect("the language is shipped")
}

fn strings_for(locale: &str) -> Strings {
    Strings::load(locale).expect("the language is shipped")
}

#[test]
fn the_first_view_says_it_is_checking_in_every_section() {
    let content = overview_content(&strings(), &ServerSnapshot::checking());

    assert_eq!(content.state, strings().text("overview.state.checking"));
    assert_eq!(content.library, strings().text("overview.library_checking"));
    assert_eq!(content.folders, strings().text("overview.folders_checking"));
    assert_eq!(
        content.connection,
        strings().text("overview.connection_checking")
    );
    assert_eq!(
        content.activity,
        strings().text("overview.activity_checking")
    );
}

#[test]
fn stopped_and_refused_views_hide_stale_counts_folders_and_activity() {
    for state in [ServerState::Stopped, ServerState::Refused] {
        let snapshot = ServerSnapshot {
            state,
            catalogue_objects: 42,
            active_streams: 3,
            roots: vec![RootInfo {
                name: "stale folder".to_owned(),
                mounted: true,
                objects: 42,
            }],
            transfers: vec![TransferInfo {
                client: "stale client".to_owned(),
                title: "stale title".to_owned(),
            }],
            ..ServerSnapshot::default()
        };

        let content = overview_content(&strings(), &snapshot);
        assert_eq!(
            content.library,
            strings().text("overview.library_unavailable")
        );
        assert_eq!(
            content.folders,
            strings().text("overview.folders_unavailable")
        );
        assert_eq!(
            content.activity,
            strings().text("overview.activity_unavailable")
        );
        assert!(!content.library.contains("42"));
        assert!(!content.folders.contains("stale"));
        assert!(!content.activity.contains("stale"));
    }
}

#[test]
fn a_running_view_lists_every_folder_and_marks_a_missing_volume() {
    let snapshot = ServerSnapshot {
        state: ServerState::Running,
        catalogue_objects: 9,
        roots: vec![
            RootInfo {
                name: "Music".to_owned(),
                mounted: true,
                objects: 8,
            },
            RootInfo {
                name: "Films".to_owned(),
                mounted: false,
                objects: 1,
            },
        ],
        ..ServerSnapshot::default()
    };

    let content = overview_content(&strings(), &snapshot);
    assert!(content.library.contains('9'));
    assert!(content.folders.contains("Music"));
    assert!(content.folders.contains("Films"));
    assert_eq!(
        content.folders.lines().nth(1),
        Some(
            strings()
                .format("menu.status.root_missing", &[("name", "Films")])
                .as_str()
        )
    );
}

#[test]
fn a_scanning_view_keeps_scan_state_and_activity_visible() {
    let snapshot = ServerSnapshot {
        state: ServerState::Scanning,
        ..ServerSnapshot::default()
    };

    let content = overview_content(&strings(), &snapshot);
    assert_eq!(content.state, strings().text("overview.state.scanning"));
    assert!(content
        .activity
        .contains(strings().text("overview.activity_scanning")));
}

/// The absent device name gives a concrete recovery action in German.
#[test]
fn missing_tv_name_points_to_server_setup() {
    let strings = strings_for("de-DE");
    let content = overview_content(
        &strings,
        &ServerSnapshot {
            state: ServerState::Running,
            ..ServerSnapshot::default()
        },
    );

    assert_eq!(
        content.connection,
        strings.text("overview.connection_unavailable")
    );
    assert!(content.connection.contains("Servereinrichtung"));
    assert!(content.connection.contains("erneut"));
}

/// When discovery is unavailable, the guidance names the server and its address.
#[test]
fn unavailable_tv_discovery_keeps_the_server_address_visible() {
    let strings = strings();
    let snapshot = ServerSnapshot {
        state: ServerState::Running,
        device_name: "PhairDLNA Wohnzimmer".to_owned(),
        device_address: Some("192.168.1.42".to_owned()),
        discovery_available: Some(false),
        ..ServerSnapshot::default()
    };

    let content = overview_content(&strings, &snapshot);
    let expected = strings.format(
        "overview.connection_discovery_unavailable",
        &[
            ("name", "PhairDLNA Wohnzimmer"),
            ("address", "192.168.1.42"),
        ],
    );
    assert_eq!(content.connection, expected);
}
