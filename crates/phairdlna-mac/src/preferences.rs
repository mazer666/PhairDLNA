//! The preferences of the application.
//!
//! What: the file that holds the preference of the view — the mode of the
//!       presentation.
//! Why: the mode is a preference of the *application* and not a setting of
//!      the server. It therefore lives in a file of its own, beside the
//!      configuration of the server and not inside it, so that switching the
//!      mode changes no setting of the server and interrupts no transfer
//!      (`UX-015`, `MB-004`).
//! Example: the file holds the word `advanced`, and the menu shows the detail
//!          of the server the next time the application starts.
//!
//! **The file belongs to the user.** It lies in the directory of the
//! configuration of the project and holds one word. A file that cannot be
//! read is not an error: the application starts in the short view, because a
//! missing preference must never keep the application from running.

use std::path::{Path, PathBuf};

use crate::model::{PresentationMode, TextScale};

/// The name of the file of the preference of the view.
const FILE_NAME: &str = "presentation.mode";

/// The preferences of the application.
#[derive(Clone, Debug)]
pub struct Preferences {
    /// The file that holds the mode of the presentation.
    path: PathBuf,
}

impl Preferences {
    /// The preferences that live in the directory of the project.
    pub fn of_the_user() -> Self {
        Self {
            path: phairdlna_core::config::user_config_dir().join(FILE_NAME),
        }
    }

    /// The preferences that live in a known file.
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The file that holds the preference.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The mode of the presentation.
    ///
    /// The answer is the short view when the file is not there or when it
    /// holds something that is not a mode: the application starts and shows
    /// the state of the server.
    pub fn mode(&self) -> PresentationMode {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return PresentationMode::Simple;
        };
        PresentationMode::of(&text).unwrap_or_default()
    }

    /// Writes the mode of the presentation.
    ///
    /// The answer is an error when the file cannot be written; the mode of
    /// the running application changes anyway, and the next start reads the
    /// old mode.
    pub fn set_mode(&self, mode: PresentationMode) -> Result<(), PreferencesError> {
        self.write_value(&self.path, mode.as_str())
    }

    /// The text size of the native application, stored beside the view mode.
    pub fn text_scale(&self) -> TextScale {
        let path = self.path.with_file_name("text.scale");
        let Ok(text) = std::fs::read_to_string(path) else {
            return TextScale::default();
        };
        TextScale::of(&text)
    }

    /// Writes the app-owned text size without changing server configuration.
    pub fn set_text_scale(&self, scale: TextScale) -> Result<(), PreferencesError> {
        let path = self.path.with_file_name("text.scale");
        self.write_value(&path, scale.as_str())
    }

    fn write_value(&self, path: &Path, value: &str) -> Result<(), PreferencesError> {
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(|error| PreferencesError {
                reason: error.to_string(),
            })?;
            make_private(folder);
        }
        std::fs::write(path, value).map_err(|error| PreferencesError {
            reason: error.to_string(),
        })?;
        make_private(path);
        Ok(())
    }
}

/// Makes a file or a folder private to its owner.
///
/// The preference is no secret, but a file that only its owner may read is
/// the rule of the folder of the configuration (`SEC-071`).
#[cfg(unix)]
fn make_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    let mode = metadata.permissions().mode();
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o700));
}

/// The rule of the permissions exists only on the systems that have them.
#[cfg(not(unix))]
fn make_private(_path: &Path) {}

/// Why a preference could not be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferencesError {
    /// What the file system said.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::Preferences;
    use crate::model::PresentationMode;

    /// The mode of the presentation is written and read back.
    #[test]
    fn the_mode_is_written_and_read_back() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let preferences = Preferences::at(folder.path().join("presentation.mode"));

        assert_eq!(preferences.mode(), PresentationMode::Simple);
        preferences
            .set_mode(PresentationMode::Advanced)
            .expect("the preference is written");
        assert_eq!(preferences.mode(), PresentationMode::Advanced);
    }

    /// The negative case: a file that holds something else is the short view.
    #[test]
    fn a_file_that_holds_something_else_is_the_short_view() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let path = folder.path().join("presentation.mode");
        std::fs::write(&path, "nonsense").expect("the file is written");

        let preferences = Preferences::at(path);

        assert_eq!(preferences.mode(), PresentationMode::Simple);
    }
}
