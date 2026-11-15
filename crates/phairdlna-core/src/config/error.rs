//! The error type of the configuration module.
//!
//! What: a failure that names the field, the file and what was expected.
//! Why: `CFG-002` requires an error that a user can act on; "invalid
//!      configuration" is not actionable.
//! Example: `field "server.http_port" is 70000; expected a TCP port between 1 and 65535`.

/// A configuration failure, with the context a user needs to fix it.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ConfigError {
    /// The full, actionable sentence shown to the user.
    message: String,
    /// The configuration field the failure belongs to, when there is one.
    field: Option<String>,
}

impl ConfigError {
    /// Builds an error that names a field.
    pub fn field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Builds an error about the document as a whole.
    pub fn document(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    /// The configuration field the failure belongs to, if any.
    pub fn field_name(&self) -> Option<&str> {
        self.field.as_deref()
    }

    /// The actionable message.
    pub fn message(&self) -> &str {
        &self.message
    }
}
