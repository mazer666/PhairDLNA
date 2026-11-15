//! The service descriptions as a control point sees them.
//!
//! What: the acceptance criteria of the feature specification F-02 that
//!       concern the SCPD documents of the two advertised services.
//! Why: a control point parses a service description before it calls
//!       anything, so an action that is missing there, or a state variable
//!       without a data type, is a defect a client cannot work around.
//! Example: `cargo test --package phairdlna-core --test scpd`.

use phairdlna_core::upnp::{Direction, ServiceSpec};

mod support;

use support::assert_well_formed;

/// The service description of the ContentDirectory service.
fn content_directory() -> String {
    ServiceSpec::content_directory().to_xml()
}

/// The service description of the ConnectionManager service.
fn connection_manager() -> String {
    ServiceSpec::connection_manager().to_xml()
}

/// The state table of a service description.
fn state_table(xml: &str) -> &str {
    let start = xml
        .find("<serviceStateTable>")
        .expect("the service description has a state table");
    &xml[start..]
}

/// The block of one state variable, so that its data type can be checked.
fn state_variable(xml: &str, name: &str) -> String {
    let table = state_table(xml);
    let marker = format!("<name>{name}</name>");
    let position = table
        .find(&marker)
        .unwrap_or_else(|| panic!("the state variable `{name}` is missing"));
    let start = table[..position]
        .rfind("<stateVariable")
        .expect("the state variable is in a block");
    let end = table[position..]
        .find("</stateVariable>")
        .map(|end| position + end)
        .unwrap_or(table.len());
    table[start..end].to_owned()
}

/// The ContentDirectory service lists every action it implements with its
/// arguments and their direction.
#[test]
fn the_content_directory_lists_every_action_with_its_arguments() {
    let xml = content_directory();
    for action in [
        "GetSearchCapabilities",
        "GetSortCapabilities",
        "GetSystemUpdateID",
        "Browse",
        "Search",
    ] {
        assert!(
            xml.contains(&format!("<name>{action}</name>")),
            "{action} is missing"
        );
    }
    assert!(xml.contains("<name>ObjectID</name>"));
    assert!(xml.contains("<direction>in</direction>"));
    assert!(xml.contains("<direction>out</direction>"));
    assert!(xml.contains("<relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>"));
}

/// The ContentDirectory service lists no write action, because the server does
/// not implement one.
#[test]
fn the_content_directory_lists_no_write_action() {
    let xml = content_directory();
    for absent in [
        "CreateObject",
        "DestroyObject",
        "UpdateObject",
        "MoveObject",
        "ImportResource",
        "ExportResource",
        "DeleteResource",
    ] {
        assert!(
            !xml.contains(&format!("<name>{absent}</name>")),
            "{absent} must be absent"
        );
    }
}

/// Every state variable declares its data type, and the type is the one the
/// specification names for that variable.
#[test]
fn every_state_variable_declares_its_data_type() {
    for service in [
        ServiceSpec::content_directory(),
        ServiceSpec::connection_manager(),
    ] {
        let xml = service.to_xml();
        for variable in service.state_variables() {
            let block = state_variable(&xml, variable.name);
            assert!(
                block.contains(&format!("<dataType>{}</dataType>", variable.data_type)),
                "{} must be of type {}, but the document says: {}",
                variable.name,
                variable.data_type,
                block
            );
        }
    }
}

/// The counters of the ContentDirectory service are unsigned, and the
/// identifiers of the ConnectionManager service are signed. A service that
/// mixed them up would send a negative number to a client that expects an
/// unsigned one.
#[test]
fn the_counter_and_identifier_types_are_not_mixed_up() {
    let xml = content_directory();
    assert!(state_variable(&xml, "SystemUpdateID").contains("<dataType>ui4</dataType>"));
    assert!(state_variable(&xml, "A_ARG_TYPE_Index").contains("<dataType>ui4</dataType>"));
    assert!(state_variable(&xml, "A_ARG_TYPE_Count").contains("<dataType>ui4</dataType>"));

    let xml = connection_manager();
    assert!(state_variable(&xml, "A_ARG_TYPE_ConnectionID").contains("<dataType>i4</dataType>"));
    assert!(state_variable(&xml, "A_ARG_TYPE_AVTransportID").contains("<dataType>i4</dataType>"));
    assert!(state_variable(&xml, "A_ARG_TYPE_RcsID").contains("<dataType>i4</dataType>"));
}

/// A variable with a fixed value set declares every allowed value.
#[test]
fn a_variable_with_a_fixed_value_set_declares_its_values() {
    let xml = content_directory();
    assert!(xml.contains("<allowedValue>BrowseMetadata</allowedValue>"));
    assert!(xml.contains("<allowedValue>BrowseDirectChildren</allowedValue>"));

    let xml = connection_manager();
    assert!(xml.contains("<allowedValue>Input</allowedValue>"));
    assert!(xml.contains("<allowedValue>Output</allowedValue>"));
    assert!(xml.contains("<allowedValue>ContentFormatMismatch</allowedValue>"));
}

/// The ConnectionManager service lists the actions of the specification.
#[test]
fn the_connection_manager_lists_its_actions() {
    let xml = connection_manager();
    for action in [
        "GetProtocolInfo",
        "PrepareForConnection",
        "ConnectionComplete",
        "GetCurrentConnectionIDs",
        "GetCurrentConnectionInfo",
    ] {
        assert!(
            xml.contains(&format!("<name>{action}</name>")),
            "{action} is missing"
        );
    }
    assert!(xml.contains("<name>Source</name>"));
    assert!(xml.contains("<name>Sink</name>"));
}

/// The evented variables are marked as such.
#[test]
fn the_evented_variables_are_marked() {
    let xml = content_directory();
    assert!(xml.contains("<stateVariable sendEvents=\"yes\">"));
    assert!(xml.contains("<stateVariable sendEvents=\"no\">"));
}

/// Both service descriptions carry no DOCTYPE and declare their encoding.
#[test]
fn the_service_descriptions_carry_no_doctype() {
    for xml in [content_directory(), connection_manager()] {
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#));
        assert!(!xml.contains("<!DOCTYPE"));
    }
}

/// Both service descriptions are well-formed documents.
#[test]
fn the_service_descriptions_are_well_formed() {
    for xml in [content_directory(), connection_manager()] {
        assert_well_formed(&xml);
    }
}

/// The service descriptions use the namespace of the service architecture and
/// carry the elements the specification names.
#[test]
fn the_service_descriptions_use_the_service_namespace() {
    for xml in [content_directory(), connection_manager()] {
        assert!(xml.contains(r#"<scpd xmlns="urn:schemas-upnp-org:service-1-0">"#));
        for (element, opening) in [
            ("specVersion", "<specVersion>"),
            ("actionList", "<actionList>"),
            ("action", "<action>"),
            ("argumentList", "<argumentList>"),
            ("argument", "<argument>"),
            ("direction", "<direction>in</direction>"),
            ("relatedStateVariable", "<relatedStateVariable>"),
            ("serviceStateTable", "<serviceStateTable>"),
            ("stateVariable", "<stateVariable "),
        ] {
            assert!(xml.contains(opening), "{element} is missing");
        }
    }
}

/// A variable with a fixed value set declares every allowed value in a list
/// of its own.
#[test]
fn the_allowed_values_are_listed() {
    let xml = content_directory();
    assert!(xml.contains("<allowedValueList>"));
    assert!(xml.contains("</allowedValueList>"));
    let xml = connection_manager();
    assert!(xml.contains("<allowedValueList>"));
    assert!(xml.contains("</allowedValueList>"));
}

/// An action of the service can be looked up by its name, which is what the
/// SOAP dispatcher needs to tell `401` from `402`.
#[test]
fn an_action_can_be_looked_up_by_its_name() {
    let service = ServiceSpec::content_directory();
    let browse = service.action("Browse").expect("Browse is a known action");
    assert_eq!(browse.arguments.len(), 10);
    assert!(browse
        .arguments
        .iter()
        .any(|argument| { argument.name == "ObjectID" && argument.direction == Direction::In }));
    assert!(service.action("NoSuchAction").is_none());
}

/// The service descriptions stay small.
#[test]
fn the_service_descriptions_stay_small() {
    for xml in [content_directory(), connection_manager()] {
        assert!(
            xml.len() < 64 * 1024,
            "a service description is {} bytes, which is too large",
            xml.len()
        );
    }
}
