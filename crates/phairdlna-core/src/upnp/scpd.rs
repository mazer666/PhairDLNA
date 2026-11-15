//! The service description (SCPD) of a UPnP service.
//!
//! What: the document that tells a control point which actions a service
//!       offers, which arguments each action takes, and which state variables
//!       the service exposes (`INT-023`).
//! Why: a control point reads the service description before it calls
//!       anything. The action table below is the single source of truth for
//!       the service: the document is generated from it, and the SOAP
//!       dispatcher of a later work package looks its actions up in the very
//!       same table, so the two cannot drift apart.
//! Example: `ServiceSpec::content_directory().to_xml()` produces the document
//!       a client fetches at `/ContentDirectory.xml`.

use super::connection_manager;
use super::content_directory;
use super::services::{ServiceDefinition, CONNECTION_MANAGER, CONTENT_DIRECTORY};
use super::xml_writer::XmlWriter;

/// The namespace of a service description document.
pub const SCPD_NAMESPACE: &str = "urn:schemas-upnp-org:service-1-0";

/// The direction of an argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// The client sends the argument to the server.
    In,
    /// The server sends the argument back to the client.
    Out,
}

impl Direction {
    /// The spelling of the direction in the document.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

/// One argument of an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgumentSpec {
    /// The name of the argument.
    pub name: &'static str,
    /// Whether the client sends it or the server returns it.
    pub direction: Direction,
    /// The state variable that describes the argument.
    pub related_state_variable: &'static str,
}

/// One action of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionSpec {
    /// The name of the action.
    pub name: &'static str,
    /// The arguments of the action.
    pub arguments: &'static [ArgumentSpec],
}

/// One state variable of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateVariableSpec {
    /// The name of the state variable.
    pub name: &'static str,
    /// The data type, for example `string` or `ui4`.
    pub data_type: &'static str,
    /// Whether the variable is sent when it changes.
    pub send_events: bool,
    /// The values the variable may take, if the type has a fixed value set.
    pub allowed_values: &'static [&'static str],
}

/// The full description of one service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceSpec {
    definition: ServiceDefinition,
    actions: &'static [ActionSpec],
    state_variables: &'static [StateVariableSpec],
}

impl ServiceSpec {
    /// The description of the `ContentDirectory:1` service.
    pub fn content_directory() -> Self {
        Self {
            definition: CONTENT_DIRECTORY,
            actions: &content_directory::ACTIONS,
            state_variables: &content_directory::STATE_VARIABLES,
        }
    }

    /// The description of the `ConnectionManager:1` service.
    pub fn connection_manager() -> Self {
        Self {
            definition: CONNECTION_MANAGER,
            actions: &connection_manager::ACTIONS,
            state_variables: &connection_manager::STATE_VARIABLES,
        }
    }

    /// The identity and the paths of the service.
    pub fn definition(&self) -> ServiceDefinition {
        self.definition
    }

    /// The actions of the service.
    pub fn actions(&self) -> &'static [ActionSpec] {
        self.actions
    }

    /// The state variables of the service.
    pub fn state_variables(&self) -> &'static [StateVariableSpec] {
        self.state_variables
    }

    /// The action with the given name, if the service has it.
    ///
    /// The SOAP dispatcher uses this lookup to tell an unknown action
    /// (`401 Invalid Action`) from a known one (`402 Invalid Args`).
    pub fn action(&self, name: &str) -> Option<&ActionSpec> {
        self.actions.iter().find(|action| action.name == name)
    }

    /// The service description as an XML document.
    pub fn to_xml(&self) -> String {
        let mut writer = XmlWriter::new();
        writer.declaration();
        writer.open("scpd", &[("xmlns", SCPD_NAMESPACE)]);
        writer.open("specVersion", &[]);
        writer.text("major", "1");
        writer.text("minor", "0");
        writer.close("specVersion");
        self.write_actions(&mut writer);
        self.write_state_variables(&mut writer);
        writer.close("scpd");
        writer.finish()
    }

    /// Writes the action list.
    fn write_actions(&self, writer: &mut XmlWriter) {
        writer.open("actionList", &[]);
        for action in self.actions {
            writer.open("action", &[]);
            writer.text("name", action.name);
            writer.open("argumentList", &[]);
            for argument in action.arguments {
                writer.open("argument", &[]);
                writer.text("name", argument.name);
                writer.text("direction", argument.direction.as_str());
                writer.text("relatedStateVariable", argument.related_state_variable);
                writer.close("argument");
            }
            writer.close("argumentList");
            writer.close("action");
        }
        writer.close("actionList");
    }

    /// Writes the service state table.
    fn write_state_variables(&self, writer: &mut XmlWriter) {
        writer.open("serviceStateTable", &[]);
        for variable in self.state_variables {
            let events = if variable.send_events { "yes" } else { "no" };
            writer.open("stateVariable", &[("sendEvents", events)]);
            writer.text("name", variable.name);
            writer.text("dataType", variable.data_type);
            if !variable.allowed_values.is_empty() {
                writer.open("allowedValueList", &[]);
                for value in variable.allowed_values {
                    writer.text("allowedValue", value);
                }
                writer.close("allowedValueList");
            }
            writer.close("stateVariable");
        }
        writer.close("serviceStateTable");
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, ServiceSpec};

    /// The ContentDirectory service lists the five actions it implements.
    #[test]
    fn the_content_directory_lists_its_actions() {
        let service = ServiceSpec::content_directory();
        let names: Vec<&str> = service.actions().iter().map(|action| action.name).collect();
        assert_eq!(
            names,
            vec![
                "GetSearchCapabilities",
                "GetSortCapabilities",
                "GetSystemUpdateID",
                "Browse",
                "Search",
            ]
        );
    }

    /// The write actions of the specification are absent, because the server
    /// does not implement them (`INT-040`).
    #[test]
    fn the_write_actions_are_absent() {
        let service = ServiceSpec::content_directory();
        for absent in ["CreateObject", "DestroyObject", "UpdateObject"] {
            assert!(service.action(absent).is_none(), "{absent} must be absent");
        }
    }

    /// An action that the service does not have is not found.
    #[test]
    fn an_unknown_action_is_not_found() {
        let service = ServiceSpec::content_directory();
        assert!(service.action("NoSuchAction").is_none());
    }

    /// Every argument of an action points at a state variable the service
    /// declares, and carries its direction.
    #[test]
    fn every_argument_points_at_a_declared_state_variable() {
        for service in [
            ServiceSpec::content_directory(),
            ServiceSpec::connection_manager(),
        ] {
            let declared: Vec<&str> = service
                .state_variables()
                .iter()
                .map(|variable| variable.name)
                .collect();
            for action in service.actions() {
                for argument in action.arguments {
                    assert!(
                        declared.contains(&argument.related_state_variable),
                        "{} refers to an undeclared variable",
                        argument.name
                    );
                    assert!(matches!(argument.direction, Direction::In | Direction::Out));
                }
            }
        }
    }

    /// The service description carries no DOCTYPE.
    #[test]
    fn the_service_description_carries_no_doctype() {
        for service in [
            ServiceSpec::content_directory(),
            ServiceSpec::connection_manager(),
        ] {
            let xml = service.to_xml();
            assert!(!xml.contains("<!DOCTYPE"));
            assert!(xml.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#));
        }
    }
}
