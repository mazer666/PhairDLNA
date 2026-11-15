//! The ConnectionManager service of the description.
//!
//! What: the actions and the state variables of `ConnectionManager:1`, as the
//!       service description has to list them (`INT-023`).
//! Why: the service is mandatory on a media server even when the server only
//!       pushes files over HTTP, because a client reads the transfer
//!       protocols from it before it plays anything (`INT-029`). Listing the
//!       actions the server really implements keeps the description honest
//!       (`INT-053`).
//! Example: `GetProtocolInfo` returns the `Source` and the `Sink` protocol
//!       lists.

use super::scpd::{ActionSpec, ArgumentSpec, Direction, StateVariableSpec};

/// The actions of `ConnectionManager:1`.
pub static ACTIONS: [ActionSpec; 5] = [
    ActionSpec {
        name: "GetProtocolInfo",
        arguments: &[
            ArgumentSpec {
                name: "Source",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_Source",
            },
            ArgumentSpec {
                name: "Sink",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_Sink",
            },
        ],
    },
    ActionSpec {
        name: "PrepareForConnection",
        arguments: &[
            ArgumentSpec {
                name: "RemoteProtocolInfo",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ProtocolInfo",
            },
            ArgumentSpec {
                name: "PeerConnectionManager",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ConnectionManager",
            },
            ArgumentSpec {
                name: "PeerConnectionID",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ConnectionID",
            },
            ArgumentSpec {
                name: "Direction",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Direction",
            },
            ArgumentSpec {
                name: "ConnectionID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_ConnectionID",
            },
            ArgumentSpec {
                name: "AVTransportID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_AVTransportID",
            },
            ArgumentSpec {
                name: "RcsID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_RcsID",
            },
        ],
    },
    ActionSpec {
        name: "ConnectionComplete",
        arguments: &[ArgumentSpec {
            name: "ConnectionID",
            direction: Direction::In,
            related_state_variable: "A_ARG_TYPE_ConnectionID",
        }],
    },
    ActionSpec {
        name: "GetCurrentConnectionIDs",
        arguments: &[ArgumentSpec {
            name: "ConnectionIDs",
            direction: Direction::Out,
            related_state_variable: "A_ARG_TYPE_CurrentConnectionIDs",
        }],
    },
    ActionSpec {
        name: "GetCurrentConnectionInfo",
        arguments: &[
            ArgumentSpec {
                name: "ConnectionID",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ConnectionID",
            },
            ArgumentSpec {
                name: "RcsID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_RcsID",
            },
            ArgumentSpec {
                name: "AVTransportID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_AVTransportID",
            },
            ArgumentSpec {
                name: "ProtocolInfo",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_ProtocolInfo",
            },
            ArgumentSpec {
                name: "PeerConnectionManager",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_ConnectionManager",
            },
            ArgumentSpec {
                name: "PeerConnectionID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_ConnectionID",
            },
            ArgumentSpec {
                name: "Direction",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_Direction",
            },
            ArgumentSpec {
                name: "Status",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_ConnectionStatus",
            },
        ],
    },
];

/// The state variables of `ConnectionManager:1`.
pub static STATE_VARIABLES: [StateVariableSpec; 13] = [
    StateVariableSpec {
        name: "SourceProtocolInfo",
        data_type: "string",
        send_events: true,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "SinkProtocolInfo",
        data_type: "string",
        send_events: true,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "CurrentConnectionIDs",
        data_type: "string",
        send_events: true,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ProtocolInfo",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ConnectionManager",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ConnectionID",
        data_type: "i4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Direction",
        data_type: "string",
        send_events: false,
        allowed_values: &["Input", "Output"],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ConnectionStatus",
        data_type: "string",
        send_events: false,
        allowed_values: &[
            "OK",
            "ContentFormatMismatch",
            "InsufficientBandwidth",
            "UnreliableChannel",
            "Unknown",
        ],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_AVTransportID",
        data_type: "i4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_RcsID",
        data_type: "i4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Source",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Sink",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_CurrentConnectionIDs",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
];
