//! What the entries of the menu do.
//!
//! What: the actions of the menu bar application.
//! Why: the actions are the part of the application that changes something;
//!      they are kept apart from the drawing so that they can be tested
//!      without a window (`MB-004`).
//! Example: the action `RescanLibrary` asks the administration surface to read
//!          the library again.

/// An action of the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    /// Open the address of the administration surface in the browser.
    OpenAdmin,
    /// Open the media folders in the file manager.
    OpenMediaFolders,
    /// Read the library again.
    RescanLibrary,
    /// Open the configuration file.
    OpenPreferences,
    /// Export the diagnostics report for support.
    ExportDiagnostics,
    /// Switch between the short view and the detailed view.
    TogglePresentation,
    /// Leave the application; the server keeps running.
    Quit,
}

impl MenuAction {
    /// The number that stands for this action.
    ///
    /// The number travels from the Objective-C bridge to the application and
    /// back. The number of the tick of the timer is `-1`, and the numbers of
    /// the actions start at one, so that the two can never be mixed up.
    pub fn tag(self) -> i64 {
        match self {
            Self::OpenAdmin => 1,
            Self::OpenMediaFolders => 2,
            Self::RescanLibrary => 3,
            Self::OpenPreferences => 4,
            Self::Quit => 5,
            Self::TogglePresentation => 6,
            Self::ExportDiagnostics => 7,
        }
    }

    /// The action that a number stands for.
    ///
    /// The answer is `None` for a number that belongs to no action: a message
    /// of the system that the application does not know is ignored and not a
    /// crash.
    pub fn from_tag(tag: i64) -> Option<MenuAction> {
        match tag {
            1 => Some(Self::OpenAdmin),
            2 => Some(Self::OpenMediaFolders),
            3 => Some(Self::RescanLibrary),
            4 => Some(Self::OpenPreferences),
            5 => Some(Self::Quit),
            6 => Some(Self::TogglePresentation),
            7 => Some(Self::ExportDiagnostics),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MenuAction;

    /// Every action has its own number, and every number finds its action.
    #[test]
    fn every_action_has_its_own_number() {
        let actions = [
            MenuAction::OpenAdmin,
            MenuAction::OpenMediaFolders,
            MenuAction::RescanLibrary,
            MenuAction::OpenPreferences,
            MenuAction::Quit,
            MenuAction::TogglePresentation,
            MenuAction::ExportDiagnostics,
        ];
        let mut tags: Vec<i64> = actions.iter().map(|action| action.tag()).collect();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), actions.len(), "the numbers must be different");
        for action in actions {
            assert_eq!(MenuAction::from_tag(action.tag()), Some(action));
        }
    }

    /// The negative case: a number that belongs to no action is nothing.
    #[test]
    fn an_unknown_number_is_nothing() {
        assert_eq!(MenuAction::from_tag(0), None);
        assert_eq!(MenuAction::from_tag(99), None);
    }
}
