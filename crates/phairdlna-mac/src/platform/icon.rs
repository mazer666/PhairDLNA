//! The icon of the status item.
//!
//! What: the small picture in the menu bar that shows what the server does.
//! Why: the icon must be readable in the light appearance and in the dark
//!      appearance of the menu bar, and it must never be the only sign of the
//!      state: the state is always a line of text as well (`MB-006`).
//! Example: a running server shows the symbol of a radio antenna, a stopped
//!          server the same symbol with a slash.
//!
//! **The template image.** The picture is a *template* image: macOS draws it
//! with the colour of the menu bar, so the same picture is right in the light
//! appearance and in the dark appearance. The system symbols of macOS are
//! drawn for that purpose.

use objc2::rc::Retained;
use objc2_app_kit::NSImage;
use objc2_foundation::NSString;

use crate::actions::MenuAction;
use crate::model::ServerState;

/// The name of the system symbol of a state.
///
/// The symbols are the ones of macOS itself, so that the icon looks like the
/// rest of the system.
fn symbol_of(state: &ServerState) -> &'static str {
    match state {
        ServerState::Checking => "hourglass",
        ServerState::Running => "antenna.radiowaves.left.and.right",
        ServerState::Scanning => "arrow.triangle.2.circlepath",
        ServerState::Stopped => "antenna.radiowaves.left.and.right.slash",
        ServerState::Refused => "exclamationmark.triangle",
    }
}

/// The name of the system symbol of a menu action.
fn action_symbol_of(action: &MenuAction) -> &'static str {
    match action {
        MenuAction::OpenAdmin => "macwindow",
        MenuAction::OpenMediaFolders => "folder",
        MenuAction::RescanLibrary => "arrow.clockwise",
        MenuAction::OpenPreferences => "gearshape",
        MenuAction::AddMediaFolders => "folder.badge.plus",
        MenuAction::StartServer => "play.fill",
        MenuAction::StopServer => "stop.fill",
        MenuAction::ExportDiagnostics => "square.and.arrow.up",
        MenuAction::TogglePresentation => "slider.horizontal.3",
        MenuAction::TextSize100 | MenuAction::TextSize150 | MenuAction::TextSize200 => {
            "textformat.size"
        }
        MenuAction::Quit => "power",
    }
}

/// The picture of a state, as a template image.
///
/// The answer is `None` when macOS does not know the symbol; the caller then
/// keeps the picture it has, because a missing symbol must not empty the
/// menu bar.
pub fn image_of(state: &ServerState) -> Option<Retained<NSImage>> {
    let name = NSString::from_str(symbol_of(state));
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(&name, None)?;
    // A template image follows the appearance of the menu bar.
    image.setTemplate(true);
    Some(image)
}

/// The picture of an action, as a template image.
pub fn action_image_of(action: &MenuAction) -> Option<Retained<NSImage>> {
    let name = NSString::from_str(action_symbol_of(action));
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(&name, None)?;
    image.setTemplate(true);
    Some(image)
}
