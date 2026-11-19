//! Selectors implemented by the AppKit action target.
//!
//! What: maps each application action to its Objective-C selector.
//! Why: the mapping is kept separate so the runtime bridge stays reviewable.
//! Example: `OpenAdmin` maps to the selector handled by the target class.

use objc2::runtime::Sel;
use objc2::sel;

use crate::actions::MenuAction;

/// The selector AppKit sends for a menu or native control action.
pub(super) fn selector_of(action: MenuAction) -> Sel {
    match action {
        MenuAction::OpenAdmin => sel!(phairOpenAdmin:),
        MenuAction::OpenMediaFolders => sel!(phairOpenFolders:),
        MenuAction::RescanLibrary => sel!(phairRescan:),
        MenuAction::OpenPreferences => sel!(phairPreferences:),
        MenuAction::AddMediaFolders => sel!(phairAddFolders:),
        MenuAction::ExportDiagnostics => sel!(phairExportDiagnostics:),
        MenuAction::Quit => sel!(phairQuit:),
        MenuAction::TogglePresentation => sel!(phairTogglePresentation:),
        MenuAction::TextSize100 => sel!(phairTextSize100:),
        MenuAction::TextSize150 => sel!(phairTextSize150:),
        MenuAction::TextSize200 => sel!(phairTextSize200:),
    }
}
