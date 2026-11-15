//! The error type of the UPnP module.
//!
//! What: the failures the description generator and the SOAP dispatcher can
//!       report.
//! Why: the server must refuse to start with a message that names the missing
//!       field (`CFG-002`, `UX-004`) instead of advertising a description that
//!       a client will reject. A typed error makes that message possible, and
//!       it lets the dispatcher turn every failure into the right UPnP fault
//!       without a string comparison.
//! Example: `DeviceDescription::new` returns `Err(UpnpError::MissingField)` when
//!       the configuration left the model name empty.

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

    /// The request is larger than the server accepts.
    #[error("the request is larger than the accepted {limit} bytes")]
    RequestTooLarge {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The request is nested more deeply than the server accepts.
    #[error("the request is nested deeper than the accepted {limit} levels")]
    RequestTooDeep {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The request carries more elements than the server accepts.
    #[error("the request carries more than the accepted {limit} elements")]
    TooManyNodes {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The text of an element is longer than the server accepts.
    #[error("the text of an element is longer than the accepted {limit} bytes")]
    TextTooLong {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The request is not a well-formed SOAP envelope.
    #[error("the request is not a well-formed SOAP envelope: {reason}")]
    MalformedRequest {
        /// What is wrong with the request.
        reason: &'static str,
    },

    /// The request names an entity that XML 1.0 does not define.
    #[error("the request names the entity `{entity}`, which XML 1.0 does not define")]
    UnknownEntity {
        /// The entity that was named.
        entity: String,
    },

    /// The request addresses a service the server does not have.
    #[error("the request addresses the unknown service `{service_type}`")]
    UnknownService {
        /// The service type the request named.
        service_type: String,
    },

    /// The request addresses an action the service does not have.
    #[error("the request addresses the unknown action `{action}`")]
    UnknownAction {
        /// The action the request named.
        action: String,
    },

    /// An argument of the action is missing or malformed.
    #[error("the argument `{argument}` is missing or malformed")]
    InvalidArgument {
        /// The argument that is wrong.
        argument: String,
    },

    /// The object the request names does not exist.
    #[error("the object `{object_id}` does not exist")]
    NoSuchObject {
        /// The object identifier the request named.
        object_id: String,
    },

    /// The service could not carry the action out.
    #[error("the action failed: {reason}")]
    ActionFailed {
        /// Why the action failed.
        reason: String,
    },

    /// The action is marked optional in the service description, and the
    /// server does not carry it out.
    #[error("the action `{action}` is optional and the server does not implement it")]
    OptionalActionNotImplemented {
        /// The action the request named.
        action: String,
    },

    /// The total number of matches could not be computed and nothing was
    /// returned.
    #[error("the total number of matches could not be computed")]
    CannotComputeTotalMatches,
}

impl UpnpError {
    /// The UPnP error code this failure carries, if a client may see it.
    ///
    /// The failures that prevent the server from describing itself have no
    /// code: they are startup errors, not answers to a client.
    pub fn fault_code(&self) -> Option<u16> {
        match self {
            Self::UnknownService { .. } | Self::UnknownAction { .. } => Some(401),
            Self::InvalidArgument { .. }
            | Self::MalformedRequest { .. }
            | Self::UnknownEntity { .. }
            | Self::RequestTooLarge { .. }
            | Self::RequestTooDeep { .. }
            | Self::TooManyNodes { .. }
            | Self::TextTooLong { .. } => Some(402),
            Self::ActionFailed { .. } => Some(501),
            Self::OptionalActionNotImplemented { .. } => Some(600),
            Self::NoSuchObject { .. } => Some(701),
            Self::CannotComputeTotalMatches => Some(720),
            _ => None,
        }
    }
}

/// The description that belongs to a UPnP error code.
///
/// The descriptions are the ones the specification names, because a control
/// point shows them to the user.
pub fn fault_description(code: u16) -> &'static str {
    match code {
        401 => "Invalid Action",
        402 => "Invalid Args",
        501 => "Action Failed",
        600 => "Optional Action Not Implemented",
        701 => "No Such Object",
        720 => "Cannot compute TotalMatches",
        _ => "Action Failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{fault_description, UpnpError};

    /// Every failure a client may see carries the code the specification names.
    #[test]
    fn the_fault_codes_are_the_specified_ones() {
        assert_eq!(
            UpnpError::UnknownAction {
                action: "NoSuchAction".to_owned()
            }
            .fault_code(),
            Some(401)
        );
        assert_eq!(
            UpnpError::InvalidArgument {
                argument: "BrowseFlag".to_owned()
            }
            .fault_code(),
            Some(402)
        );
        assert_eq!(
            UpnpError::ActionFailed {
                reason: "no".to_owned()
            }
            .fault_code(),
            Some(501)
        );
        assert_eq!(
            UpnpError::NoSuchObject {
                object_id: "42".to_owned()
            }
            .fault_code(),
            Some(701)
        );
        assert_eq!(UpnpError::CannotComputeTotalMatches.fault_code(), Some(720));
    }

    /// A failure that stops the server from starting has no client code.
    #[test]
    fn a_startup_failure_has_no_fault_code() {
        assert_eq!(UpnpError::EmptyBaseUrl.fault_code(), None);
        assert_eq!(
            UpnpError::MissingField {
                field: "model_name"
            }
            .fault_code(),
            None
        );
    }

    /// The descriptions are the ones the specification names.
    #[test]
    fn the_descriptions_are_the_specified_ones() {
        assert_eq!(fault_description(401), "Invalid Action");
        assert_eq!(fault_description(402), "Invalid Args");
        assert_eq!(fault_description(501), "Action Failed");
        assert_eq!(fault_description(600), "Optional Action Not Implemented");
        assert_eq!(fault_description(701), "No Such Object");
    }
}
