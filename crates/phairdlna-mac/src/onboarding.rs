//! Pure decisions for the first-run application flow.
//!
//! What: classify loaded media roots and select any needed overview recovery notice.
//! Why: configuration state must not be inferred from AppKit dialog behavior.
//! Example: an unreadable configuration becomes `ConfigurationUnavailable`.

use std::path::PathBuf;

/// The action the desktop layer takes after reading configured media folders.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnboardingDecision {
    /// Configuration could not be loaded; show recovery guidance without a modal prompt.
    ConfigurationUnavailable,
    /// At least one folder is configured; no first-run recovery notice is needed.
    ConfiguredWithRoots,
    /// Configuration is valid but has no folders; explain that app-based setup is unavailable.
    ValidConfigurationWithoutRoots,
}

impl OnboardingDecision {
    /// Classifies the configuration result without retaining raw error details.
    ///
    /// The UI supplies localized recovery guidance for an error. This function
    /// deliberately does not turn an error into defaults or a success state.
    pub fn from_media_roots<E>(roots: Result<Vec<PathBuf>, E>) -> Self {
        match roots {
            Err(_) => Self::ConfigurationUnavailable,
            Ok(roots) if roots.is_empty() => Self::ValidConfigurationWithoutRoots,
            Ok(_) => Self::ConfiguredWithRoots,
        }
    }

    /// The localized overview notice for a configuration state that needs recovery.
    pub fn notice_key(self) -> Option<&'static str> {
        match self {
            Self::ConfigurationUnavailable => Some("error.configuration_unreadable"),
            Self::ValidConfigurationWithoutRoots => Some("notice.folder_changes_unavailable"),
            Self::ConfiguredWithRoots => None,
        }
    }
}
