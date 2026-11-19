//! Build the independent groups of menu entries.
//!
//! What: format folders, playback and actions for the menu.
//! Why: keeping each group small makes the user-facing state rules readable.
//! Example: a checking snapshot shows a loading line instead of an empty list.

use crate::actions::MenuAction;
use crate::model::{PresentationMode, ServerSnapshot, ServerState};
use crate::text::Strings;

use super::MenuEntry;

/// The lines of the menu that tell the shared folders.
pub(super) fn root_entries(snapshot: &ServerSnapshot, strings: &Strings) -> Vec<MenuEntry> {
    if snapshot.state == ServerState::Checking {
        return vec![MenuEntry::Status {
            title: strings.text("menu.status.roots_checking").to_owned(),
        }];
    }
    if matches!(snapshot.state, ServerState::Stopped | ServerState::Refused) {
        return Vec::new();
    }
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
pub(super) fn transfer_entries(snapshot: &ServerSnapshot, strings: &Strings) -> Vec<MenuEntry> {
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

/// The actions of the menu, with the state that says whether they may be
/// chosen.
pub(super) fn action_entries(
    snapshot: &ServerSnapshot,
    strings: &Strings,
    mode: PresentationMode,
) -> Vec<MenuEntry> {
    let serving = matches!(snapshot.state, ServerState::Running | ServerState::Scanning);
    let mut actions = Vec::new();
    actions.extend([
        action(
            strings,
            "menu.action.open_folders",
            MenuAction::OpenMediaFolders,
            !snapshot.roots.is_empty(),
        ),
        action(
            strings,
            "menu.action.add_folders",
            MenuAction::AddMediaFolders,
            false,
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
        action(
            strings,
            "menu.action.export_diagnostics",
            MenuAction::ExportDiagnostics,
            true,
        ),
        action(strings, "menu.action.quit", MenuAction::Quit, true),
        action(
            strings,
            mode_key(mode),
            MenuAction::TogglePresentation,
            true,
        ),
    ]);
    actions
}

/// The key of the text of the action that switches the view.
fn mode_key(mode: PresentationMode) -> &'static str {
    match mode {
        PresentationMode::Simple => "menu.action.mode_advanced",
        PresentationMode::Advanced => "menu.action.mode_simple",
    }
}

/// One action of the menu.
fn action(strings: &Strings, key: &str, action: MenuAction, enabled: bool) -> MenuEntry {
    let tooltip_key = match action {
        MenuAction::OpenMediaFolders => Some("menu.tooltip.open_folders"),
        MenuAction::AddMediaFolders => Some("menu.tooltip.add_folders"),
        MenuAction::RescanLibrary => Some("menu.tooltip.rescan"),
        MenuAction::OpenPreferences => Some("menu.tooltip.preferences"),
        MenuAction::ExportDiagnostics => Some("menu.tooltip.export_diagnostics"),
        MenuAction::Quit => Some("menu.tooltip.quit"),
        _ => None,
    };
    let tooltip = tooltip_key.map(|key| strings.text(key).to_owned());
    MenuEntry::Action {
        title: strings.text(key).to_owned(),
        action,
        enabled,
        tooltip,
    }
}
