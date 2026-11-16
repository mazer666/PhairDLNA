//! The entries of the menu.
//!
//! What: the model of the menu — the lines of text and the actions.
//! Why: the menu is the one view of the application, and its content must be
//!      decided by logic that can be tested without a window (`F-15`).
//! Example: a running server with two streams produces a status line and the
//!      six actions of the menu.
//!
//! **The state is always a text.** Every state of the server is a line of
//! text and never a colour or a picture alone, so that the menu can be read
//! by a screen reader and by a user whose display cannot show the picture
//! (`UX-007`, `UX-018`, `MB-006`).

use crate::actions::MenuAction;
use crate::model::{PresentationMode, ServerSnapshot, ServerState};
use crate::text::Strings;

/// One entry of the menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    /// A line of text that cannot be chosen.
    Status {
        /// The text of the line.
        title: String,
    },
    /// A separator between two groups of entries.
    Separator,
    /// An action the user can choose.
    Action {
        /// The text of the entry.
        title: String,
        /// What the entry does.
        action: crate::actions::MenuAction,
        /// Whether the entry can be chosen.
        enabled: bool,
    },
}

/// The entries of the menu for a state of the server.
///
/// The notice is the line that tells the user what happened last; it is
/// `None` when nothing happened since the last refresh.
///
/// **The two views.** The short view and the detailed view show the same
/// state of the server and the same warnings; the detailed view adds the
/// detail of the server. A warning of the server stays visible in the short
/// view, because a warning is a property of the state and not of the view
/// (`UX-015`, `UX-016`).
pub fn entries(
    snapshot: &ServerSnapshot,
    strings: &Strings,
    notice: Option<&str>,
    mode: PresentationMode,
) -> Vec<MenuEntry> {
    let mut menu = vec![
        MenuEntry::Status {
            title: strings.text("menu.status.header").to_owned(),
        },
        MenuEntry::Status {
            title: status_line(snapshot, strings),
        },
    ];
    if let Some(notice) = notice {
        menu.push(MenuEntry::Status {
            title: notice.to_owned(),
        });
    }
    menu.extend(root_entries(snapshot, strings));
    menu.extend(transfer_entries(snapshot, strings));
    menu.extend(warning_entries(snapshot));
    if mode == PresentationMode::Advanced {
        menu.extend(detail_entries(snapshot, strings));
    }
    menu.push(MenuEntry::Separator);
    menu.extend(action_entries(snapshot, strings, mode));
    menu
}

/// The lines of the menu that tell the detail of the server.
///
/// The lines belong to the detailed view only; the short view shows the
/// state, the folders and the actions (`UX-015`).
fn detail_entries(snapshot: &ServerSnapshot, strings: &Strings) -> Vec<MenuEntry> {
    let objects = snapshot.catalogue_objects.to_string();
    let seconds = snapshot.uptime_seconds.to_string();
    let mut lines = vec![
        MenuEntry::Status {
            title: strings.format("menu.status.objects", &[("objects", &objects)]),
        },
        MenuEntry::Status {
            title: strings.format("menu.status.uptime", &[("seconds", &seconds)]),
        },
    ];
    if let Some(when) = &snapshot.last_scan {
        lines.push(MenuEntry::Status {
            title: strings.format("menu.status.last_scan", &[("when", when)]),
        });
    }
    lines
}

/// The line of the menu that tells the state of the server.
pub fn status_line(snapshot: &ServerSnapshot, strings: &Strings) -> String {
    match snapshot.state {
        ServerState::Running => {
            let streams = snapshot.active_streams.to_string();
            strings.format("menu.status.running", &[("streams", &streams)])
        }
        ServerState::Scanning => strings.text("menu.status.scanning").to_owned(),
        ServerState::Stopped => strings.text("menu.status.stopped").to_owned(),
        ServerState::Refused => strings.text("menu.status.refused").to_owned(),
    }
}

/// The value the screen reader reads for the state of the server.
pub fn accessibility_value(snapshot: &ServerSnapshot, strings: &Strings) -> String {
    match snapshot.state {
        ServerState::Running => {
            let streams = snapshot.active_streams.to_string();
            strings.format("menu.accessibility.running", &[("streams", &streams)])
        }
        ServerState::Scanning => strings.text("menu.accessibility.scanning").to_owned(),
        ServerState::Stopped => strings.text("menu.accessibility.stopped").to_owned(),
        ServerState::Refused => strings.text("menu.accessibility.refused").to_owned(),
    }
}

/// The lines of the menu that tell the shared folders.
fn root_entries(snapshot: &ServerSnapshot, strings: &Strings) -> Vec<MenuEntry> {
    if snapshot.roots.is_empty() {
        return vec![MenuEntry::Status {
            title: strings.text("menu.status.no_roots").to_owned(),
        }];
    }
    snapshot
        .roots
        .iter()
        .map(|root| {
            let name = root.name.as_str();
            let title = if root.mounted {
                let objects = root.objects.to_string();
                strings.format("menu.status.root", &[("name", name), ("objects", &objects)])
            } else {
                strings.format("menu.status.root_missing", &[("name", name)])
            };
            MenuEntry::Status { title }
        })
        .collect()
}

/// The lines of the menu that tell what is being played.
fn transfer_entries(snapshot: &ServerSnapshot, strings: &Strings) -> Vec<MenuEntry> {
    snapshot
        .transfers
        .iter()
        .map(|transfer| {
            let client = transfer.client.as_str();
            let title = transfer.title.as_str();
            MenuEntry::Status {
                title: strings.format(
                    "menu.status.transfer",
                    &[("client", client), ("title", title)],
                ),
            }
        })
        .collect()
}

/// The lines of the menu that tell the warnings of the server.
///
/// The warnings are a property of the state of the server and not of the
/// view, so that a warning of the advanced mode stays visible in the simple
/// mode (`UX-016`).
fn warning_entries(snapshot: &ServerSnapshot) -> Vec<MenuEntry> {
    snapshot
        .warnings
        .iter()
        .map(|warning| MenuEntry::Status {
            title: warning.clone(),
        })
        .collect()
}

/// The actions of the menu, with the state that says whether they may be
/// chosen.
fn action_entries(
    snapshot: &ServerSnapshot,
    strings: &Strings,
    mode: PresentationMode,
) -> Vec<MenuEntry> {
    let serving = matches!(snapshot.state, ServerState::Running | ServerState::Scanning);
    vec![
        action(
            strings,
            "menu.action.open_admin",
            MenuAction::OpenAdmin,
            serving,
        ),
        action(
            strings,
            "menu.action.open_folders",
            MenuAction::OpenMediaFolders,
            !snapshot.roots.is_empty(),
        ),
        action(
            strings,
            "menu.action.rescan",
            MenuAction::RescanLibrary,
            serving,
        ),
        action(
            strings,
            "menu.action.preferences",
            MenuAction::OpenPreferences,
            true,
        ),
        action(strings, "menu.action.quit", MenuAction::Quit, true),
        action(
            strings,
            mode_key(mode),
            MenuAction::TogglePresentation,
            true,
        ),
    ]
}

/// The key of the text of the action that switches the view.
///
/// The text says what the action does and not which mode is on: the user
/// reads "show the details" and knows what happens.
fn mode_key(mode: PresentationMode) -> &'static str {
    match mode {
        PresentationMode::Simple => "menu.action.mode_advanced",
        PresentationMode::Advanced => "menu.action.mode_simple",
    }
}

/// One action of the menu.
fn action(strings: &Strings, key: &str, action: MenuAction, enabled: bool) -> MenuEntry {
    MenuEntry::Action {
        title: strings.text(key).to_owned(),
        action,
        enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::{entries, status_line, MenuEntry};
    use crate::actions::MenuAction;
    use crate::model::PresentationMode;
    use crate::model::{RootInfo, ServerSnapshot, ServerState};
    use crate::text::Strings;

    /// The texts of the language of the tests.
    fn texts() -> Strings {
        Strings::load("en-US").expect("the language is shipped")
    }

    /// A running server shows the number of the streams.
    #[test]
    fn a_running_server_shows_its_streams() {
        let snapshot = ServerSnapshot {
            state: ServerState::Running,
            active_streams: 2,
            ..ServerSnapshot::default()
        };
        let line = status_line(&snapshot, &texts());
        assert!(line.contains('2'), "the line must hold the number: {line}");
    }

    /// The negative case: a server that is not there says so.
    #[test]
    fn a_stopped_server_says_so() {
        let snapshot = ServerSnapshot::stopped();
        let line = status_line(&snapshot, &texts());
        assert_eq!(line, texts().text("menu.status.stopped"));
    }

    /// The menu holds the actions, and the action of the server is only
    /// possible when the server runs.
    #[test]
    fn the_actions_depend_on_the_state() {
        let stopped = entries(
            &ServerSnapshot::stopped(),
            &texts(),
            None,
            PresentationMode::Simple,
        );
        let serving = entries(
            &ServerSnapshot {
                state: ServerState::Running,
                roots: vec![RootInfo {
                    name: "music".to_owned(),
                    mounted: true,
                    objects: 3,
                }],
                ..ServerSnapshot::default()
            },
            &texts(),
            None,
            PresentationMode::Simple,
        );
        assert!(!enabled_of(&stopped, MenuAction::RescanLibrary));
        assert!(enabled_of(&serving, MenuAction::RescanLibrary));
        assert!(enabled_of(&serving, MenuAction::OpenMediaFolders));
    }

    /// The warning of the server is a line of the menu.
    #[test]
    fn a_warning_is_a_line_of_the_menu() {
        let snapshot = ServerSnapshot {
            state: ServerState::Running,
            warnings: vec!["the firewall may block the connections".to_owned()],
            ..ServerSnapshot::default()
        };
        let menu = entries(&snapshot, &texts(), None, PresentationMode::Simple);
        let texts: Vec<&str> = menu
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Status { title } => Some(title.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"the firewall may block the connections"));
    }

    /// Whether an action of the menu may be chosen.
    fn enabled_of(menu: &[MenuEntry], action: MenuAction) -> bool {
        menu.iter().any(|entry| match entry {
            MenuEntry::Action {
                action: chosen,
                enabled,
                ..
            } => *chosen == action && *enabled,
            _ => false,
        })
    }
}
