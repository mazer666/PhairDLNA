//! Plain-language content for the native server overview.
//!
//! What: turns a server snapshot into the five texts shown by the main window.
//! Why: the same state rules need automated coverage without opening AppKit windows.
//! Example: a stopped server gets unavailable library, folder and activity text.

use crate::actions::MenuAction;
use crate::model::{ServerSnapshot, ServerState};
use crate::text::Strings;

/// What the control that starts and stops the server offers for a state.
///
/// The rule lives here and not in the window, so that the state of the
/// control can be checked without a window (`MB-004`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerToggle {
    /// The action the control carries out.
    pub action: MenuAction,
    /// The key of the text the control shows.
    pub title_key: &'static str,
    /// Whether the control may be chosen.
    pub enabled: bool,
}

/// The control that starts or stops the server, for the state of the server.
///
/// A server that is checked cannot be started or stopped yet, so the control
/// is disabled until the application knows the state.
pub fn server_toggle(state: ServerState) -> ServerToggle {
    match state {
        ServerState::Running | ServerState::Scanning => ServerToggle {
            action: MenuAction::StopServer,
            title_key: "menu.action.stop_server",
            enabled: true,
        },
        ServerState::Stopped | ServerState::Refused => ServerToggle {
            action: MenuAction::StartServer,
            title_key: "menu.action.start_server",
            enabled: true,
        },
        ServerState::Checking => ServerToggle {
            action: MenuAction::StartServer,
            title_key: "menu.action.start_server",
            enabled: false,
        },
    }
}

/// The localized text displayed by the server overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewContent {
    /// The server state as a sentence.
    pub state: String,
    /// A media-item count or an explanation that it is unavailable.
    pub library: String,
    /// The shared-folder list or an appropriate empty/checking message.
    pub folders: String,
    /// Connection and discovery guidance.
    pub connection: String,
    /// Current scan or playback activity.
    pub activity: String,
}

/// Formats every state-dependent text shown in the primary overview.
pub fn overview_content(strings: &Strings, snapshot: &ServerSnapshot) -> OverviewContent {
    OverviewContent {
        state: state_text(strings, snapshot.state),
        library: library_text(strings, snapshot),
        folders: folder_text(strings, snapshot),
        connection: connection_text(strings, snapshot),
        activity: activity_text(strings, snapshot),
    }
}

fn state_text(strings: &Strings, state: ServerState) -> String {
    let key = match state {
        ServerState::Checking => "overview.state.checking",
        ServerState::Running => "overview.state.running",
        ServerState::Scanning => "overview.state.scanning",
        ServerState::Stopped => "overview.state.stopped",
        ServerState::Refused => "overview.state.refused",
    };
    strings.text(key).to_owned()
}

fn library_text(strings: &Strings, snapshot: &ServerSnapshot) -> String {
    match snapshot.state {
        ServerState::Checking => strings.text("overview.library_checking").to_owned(),
        ServerState::Running | ServerState::Scanning => {
            let count = snapshot.catalogue_objects.to_string();
            strings.format("overview.library_count", &[("count", &count)])
        }
        ServerState::Stopped | ServerState::Refused => {
            strings.text("overview.library_unavailable").to_owned()
        }
    }
}

fn folder_text(strings: &Strings, snapshot: &ServerSnapshot) -> String {
    if snapshot.state == ServerState::Checking {
        return strings.text("overview.folders_checking").to_owned();
    }
    if matches!(snapshot.state, ServerState::Stopped | ServerState::Refused) {
        return strings.text("overview.folders_unavailable").to_owned();
    }
    if snapshot.roots.is_empty() {
        return strings.text("overview.no_folders").to_owned();
    }
    snapshot
        .roots
        .iter()
        .map(|root| {
            if root.mounted {
                root.name.clone()
            } else {
                strings.format("menu.status.root_missing", &[("name", &root.name)])
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn connection_text(strings: &Strings, snapshot: &ServerSnapshot) -> String {
    if snapshot.state == ServerState::Checking {
        return strings.text("overview.connection_checking").to_owned();
    }
    let name = snapshot.device_name.trim();
    if name.is_empty() {
        return strings.text("overview.connection_unavailable").to_owned();
    }
    match (
        snapshot.device_address.as_deref(),
        snapshot.discovery_available,
    ) {
        (Some(address), Some(true)) => strings.format(
            "overview.connection_help",
            &[("name", name), ("address", address)],
        ),
        (Some(address), Some(false)) => strings.format(
            "overview.connection_discovery_unavailable",
            &[("name", name), ("address", address)],
        ),
        (Some(address), None) => strings.format(
            "overview.connection_discovery_unknown",
            &[("name", name), ("address", address)],
        ),
        (None, _) => strings.format("overview.connection_local_only", &[("name", name)]),
    }
}

fn activity_text(strings: &Strings, snapshot: &ServerSnapshot) -> String {
    if snapshot.state == ServerState::Checking {
        return strings.text("overview.activity_checking").to_owned();
    }
    if matches!(snapshot.state, ServerState::Stopped | ServerState::Refused) {
        return strings.text("overview.activity_unavailable").to_owned();
    }
    let mut lines = Vec::new();
    if snapshot.state == ServerState::Scanning {
        lines.push(strings.text("overview.activity_scanning").to_owned());
    }
    let shown_transfers = snapshot
        .transfers
        .iter()
        .take(2usize.saturating_sub(lines.len()))
        .map(|transfer| {
            strings.format(
                "overview.activity_transfer",
                &[("client", &transfer.client), ("title", &transfer.title)],
            )
        })
        .collect::<Vec<_>>();
    lines.extend(shown_transfers);
    let displayed_transfers = lines.len() - usize::from(snapshot.state == ServerState::Scanning);
    if snapshot.transfers.len() > displayed_transfers {
        let remaining = (snapshot.transfers.len() - displayed_transfers).to_string();
        lines.push(strings.format("overview.activity_more", &[("count", &remaining)]));
    }
    if lines.is_empty() && snapshot.active_streams > 0 {
        let count = snapshot.active_streams.to_string();
        lines.push(strings.format("overview.activity_stream_count", &[("count", &count)]));
    }
    if lines.is_empty() {
        lines.push(strings.text("overview.activity_empty").to_owned());
    }
    lines.join("\n")
}
