//! Error categories returned by the local administration client.
//!
//! What: translate transport and document failures into user-facing categories.
//! Why: presentation text stays localized in the application resources.
//! Example: a refused token maps to `Unauthorised`.

use crate::ipc::http::{TransportError, TransportKind};
use crate::ipc::json::JsonError;

/// Why the surface could not be asked.
///
/// The error carries the *kind* of the failure and the words of the system,
/// and no sentence: the sentence that the user reads comes from the resource
/// file of the language of the user (`UX-020`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientError {
    /// The surface cannot be reached.
    Unreachable {
        /// What the system said.
        reason: String,
    },
    /// The surface cannot be read.
    Unreadable {
        /// What the system said.
        reason: String,
    },
    /// The answer is not an answer of the protocol.
    Invalid {
        /// What was wrong with it.
        reason: String,
    },
    /// The answer is not the answer the client expects.
    Unexpected {
        /// What was wrong with it.
        reason: String,
    },
    /// The token of the installation was refused.
    Unauthorised,
    /// The surface refused the request.
    Refused {
        /// The number of the refusal.
        status: u16,
    },
    /// The token of the installation could not be read.
    Token {
        /// What the system said.
        reason: String,
    },
    /// The file of the token is not there: the server was never started.
    NoToken,
    /// The application is busy and the request was dropped.
    Busy,
}

impl ClientError {
    /// What was wrong, in the words of the system.
    ///
    /// The answer is an empty text when the failure has no words of its own.
    pub fn reason(&self) -> &str {
        match self {
            Self::Unreachable { reason }
            | Self::Unreadable { reason }
            | Self::Invalid { reason }
            | Self::Unexpected { reason }
            | Self::Token { reason } => reason,
            Self::Unauthorised | Self::Refused { .. } | Self::NoToken | Self::Busy => "",
        }
    }
}

impl From<TransportError> for ClientError {
    fn from(error: TransportError) -> Self {
        let reason = error.reason().to_owned();
        match error.kind() {
            TransportKind::Unreachable => Self::Unreachable { reason },
            TransportKind::Unreadable => Self::Unreadable { reason },
            TransportKind::Invalid => Self::Invalid { reason },
        }
    }
}

impl From<JsonError> for ClientError {
    fn from(error: JsonError) -> Self {
        Self::Unexpected {
            reason: error.to_string(),
        }
    }
}
