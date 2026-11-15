//! The action table of the ContentDirectory service.
//!
//! What: the actions and the state variables of `ContentDirectory:1`, as the
//!       service description has to list them (`INT-023`).
//! Why: a control point parses the service description before it calls
//!       anything. An action that is missing there produces a failure that
//!       looks like a client bug, and an action that is listed but not
//!       implemented produces a failure that looks like a server bug. The
//!       table below therefore lists exactly the actions the server
//!       implements, and the write actions of the specification are absent
//!       (`INT-040`). The dispatcher answers the actions of this table and
//!       nothing else, so the two cannot drift apart.
//! Example: `Browse` takes six arguments and returns four.

use super::scpd::{ActionSpec, ArgumentSpec, Direction, StateVariableSpec};

/// The actions of `ContentDirectory:1`.
pub static ACTIONS: [ActionSpec; 5] = [
    ActionSpec {
        name: "GetSearchCapabilities",
        optional: false,
        arguments: &[ArgumentSpec {
            name: "SearchCaps",
            direction: Direction::Out,
            related_state_variable: "A_ARG_TYPE_SearchCaps",
        }],
    },
    ActionSpec {
        name: "GetSortCapabilities",
        optional: false,
        arguments: &[ArgumentSpec {
            name: "SortCaps",
            direction: Direction::Out,
            related_state_variable: "A_ARG_TYPE_SortCaps",
        }],
    },
    ActionSpec {
        name: "GetSystemUpdateID",
        optional: false,
        arguments: &[ArgumentSpec {
            name: "Id",
            direction: Direction::Out,
            related_state_variable: "A_ARG_TYPE_Id",
        }],
    },
    ActionSpec {
        name: "Browse",
        optional: false,
        arguments: &[
            ArgumentSpec {
                name: "ObjectID",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ObjectID",
            },
            ArgumentSpec {
                name: "BrowseFlag",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_BrowseFlag",
            },
            ArgumentSpec {
                name: "Filter",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Filter",
            },
            ArgumentSpec {
                name: "StartingIndex",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Index",
            },
            ArgumentSpec {
                name: "RequestedCount",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Count",
            },
            ArgumentSpec {
                name: "SortCriteria",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_SortCriteria",
            },
            ArgumentSpec {
                name: "Result",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_Result",
            },
            ArgumentSpec {
                name: "NumberReturned",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_NumberReturned",
            },
            ArgumentSpec {
                name: "TotalMatches",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_TotalMatches",
            },
            ArgumentSpec {
                name: "UpdateID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_UpdateID",
            },
        ],
    },
    ActionSpec {
        name: "Search",
        optional: false,
        arguments: &[
            ArgumentSpec {
                name: "ContainerID",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_ContainerID",
            },
            ArgumentSpec {
                name: "SearchCriteria",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_SearchCriteria",
            },
            ArgumentSpec {
                name: "Filter",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Filter",
            },
            ArgumentSpec {
                name: "StartingIndex",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Index",
            },
            ArgumentSpec {
                name: "RequestedCount",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_Count",
            },
            ArgumentSpec {
                name: "SortCriteria",
                direction: Direction::In,
                related_state_variable: "A_ARG_TYPE_SortCriteria",
            },
            ArgumentSpec {
                name: "Result",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_Result",
            },
            ArgumentSpec {
                name: "NumberReturned",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_NumberReturned",
            },
            ArgumentSpec {
                name: "TotalMatches",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_TotalMatches",
            },
            ArgumentSpec {
                name: "UpdateID",
                direction: Direction::Out,
                related_state_variable: "A_ARG_TYPE_UpdateID",
            },
        ],
    },
];

/// The state variables of `ContentDirectory:1`.
pub static STATE_VARIABLES: [StateVariableSpec; 18] = [
    StateVariableSpec {
        name: "SearchCapabilities",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "SortCapabilities",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "SystemUpdateID",
        data_type: "ui4",
        send_events: true,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_SearchCaps",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_SortCaps",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Id",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ObjectID",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_BrowseFlag",
        data_type: "string",
        send_events: false,
        allowed_values: &["BrowseMetadata", "BrowseDirectChildren"],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Filter",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Index",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Count",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_SortCriteria",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_SearchCriteria",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_ContainerID",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_Result",
        data_type: "string",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_NumberReturned",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_TotalMatches",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
    StateVariableSpec {
        name: "A_ARG_TYPE_UpdateID",
        data_type: "ui4",
        send_events: false,
        allowed_values: &[],
    },
];
