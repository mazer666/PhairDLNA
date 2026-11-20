//! Menu actions handled by the application model.
//!
//! What: maps user actions to app state changes or requests for the platform.
//! Why: action behavior stays separate from snapshot accessors and menu rendering.
//! Example: opening Preferences returns a request that the AppKit layer carries out.

use crate::actions::MenuAction;
use crate::app::{ActionOutcome, App, DesktopRequest};
use crate::model::{ServerState, TextScale};

impl App {
    /// Carries out an action of the menu.
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
            MenuAction::ExportDiagnostics => {
                ActionOutcome::Request(DesktopRequest::ExportDiagnostics)
            }
            MenuAction::TogglePresentation => self.switch_view(),
            MenuAction::TextSize100 => self.set_text_scale(TextScale::Percent100),
            MenuAction::TextSize150 => self.set_text_scale(TextScale::Percent150),
            MenuAction::TextSize200 => self.set_text_scale(TextScale::Percent200),
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

    fn set_text_scale(&mut self, scale: TextScale) -> ActionOutcome {
        self.text_scale = scale;
        if self.preferences.set_text_scale(scale).is_err() {
            self.notice = Some(self.strings.text("error.preference_failed").to_owned());
        }
        ActionOutcome::Nothing
    }
}
