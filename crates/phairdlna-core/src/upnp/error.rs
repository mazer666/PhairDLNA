//! The error type of the UPnP module.
//!
//! What: the failures the description generator can report before it writes a
//!       single byte of XML.
//! Why: the server must refuse to start with a message that names the missing
//!       field (`CFG-002`, `UX-004`) instead of advertising a description that
//!       a client will reject. A typed error makes that message possible.
//! Example: `DeviceDescription::new` returns `Err(UpnpError::MissingField)` when
//! the configuration left the model name empty.

use thiserror::Error;

/// Every failure the UPnP module can report.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UpnpError {
    /// The base URL of the description is empty.
    #[error("the base URL of the device description is empty")]
    EmptyBaseUrl,

    /// The base URL is not an absolute HTTP URL with a host.
    #[error("the base URL `{value}` is not an absolute HTTP URL with a host")]
    InvalidBaseUrl {
        /// The value that was rejected.
        value: String,
    },

    /// A field of the device description is required but empty.
    #[error("the field `{field}` of the device description is required but empty")]
    MissingField {
        /// The name of the field, as it appears in the configuration.
        field: &'static str,
    },

    /// The unique device name is not a UUID.
    #[error("the unique device name `{value}` is not a UUID")]
    InvalidUdn {
        /// The value that was rejected.
        value: String,
    },
}
