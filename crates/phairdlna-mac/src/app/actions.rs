//! Menu actions handled by the application model.
//!
//! What: maps user actions to app state changes or requests for the platform.
//! Why: action behavior stays separate from snapshot accessors and menu rendering.
//! Example: opening Preferences returns a request that the AppKit layer carries out.

use crate::actions::MenuAction;
use crate::app::{ActionOutcome, App, DesktopRequest};
use crate::model::ServerState;

impl App {
    /// Carries out an action of the menu.
    ///
    /// The two actions of the overview that carry values — saving the settings
    /// and removing a folder — are carried out by the platform layer, because
    /// only it knows what the user typed into the controls of the window. The
    /// application answers them with [`ActionOutcome::Nothing`] and waits for
    /// the platform to call [`crate::app::App::save_settings`] and
    /// [`crate::app::App::remove_media_root`].
    pub fn perform(&mut self, action: MenuAction) -> ActionOutcome {
        match action {
            MenuAction::OpenAdmin => ActionOutcome::Request(DesktopRequest::ShowMainWindow),
            MenuAction::OpenMediaFolders => self.open_media_folders(),
            MenuAction::RescanLibrary => self.start_rescan(),
            MenuAction::OpenPreferences => ActionOutcome::Request(DesktopRequest::ShowPreferences),
            MenuAction::AddMediaFolders => {
                if self.can_add_media_roots() {
                    ActionOutcome::Request(DesktopRequest::PickMediaFolders)
                } else {
                    self.notice = Some(
                        self.strings
                            .text("notice.folder_changes_unavailable")
                            .to_owned(),
                    );
                    ActionOutcome::Nothing
                }
            }
            MenuAction::StartServer => ActionOutcome::Request(DesktopRequest::StartServer),
            MenuAction::StopServer => ActionOutcome::Request(DesktopRequest::StopServer),
            MenuAction::ExportDiagnostics => {
                ActionOutcome::Request(DesktopRequest::ExportDiagnostics)
            }
            MenuAction::TogglePresentation => self.switch_view(),
            MenuAction::SaveSettings => ActionOutcome::Nothing,
            MenuAction::RemoveMediaFolder => ActionOutcome::Nothing,
            MenuAction::Quit => ActionOutcome::Request(DesktopRequest::Quit),
        }
    }

    fn start_rescan(&mut self) -> ActionOutcome {
        if matches!(
            self.snapshot.state,
            ServerState::Running | ServerState::Scanning
        ) {
            self.scanning = true;
            self.snapshot.state = ServerState::Scanning;
        }
        ActionOutcome::Nothing
    }
}
