//! The targets the server advertises, and the matching of a search to them.
//!
//! What: the set of things a client may search for, and the rule that decides
//!       which of them the server answers.
//! Why: the UPnP Device Architecture fixes this table; getting it wrong means
//!       a client finds the device but cannot browse it (`INT-002`, `INT-005`).
//! Example: a search for `ssdp:all` produces five answers — the root device, the
//!       device itself, its unique identifier and its two services.

use super::constants::{
    DEVICE_TYPE_MEDIA_SERVER, NT_ALL, NT_ROOT_DEVICE, SERVICE_TYPE_CONNECTION_MANAGER,
    SERVICE_TYPE_CONTENT_DIRECTORY,
};

/// The identity of the server, as it appears in a `USN`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// The unique device name, without the `uuid:` prefix.
    pub udn: String,
    /// The name a client shows.
    pub friendly_name: String,
}

impl DeviceIdentity {
    /// Builds an identity from the unique device name.
    pub fn new(udn: impl Into<String>, friendly_name: impl Into<String>) -> Self {
        Self {
            udn: udn.into(),
            friendly_name: friendly_name.into(),
        }
    }

    /// The unique device name with the `uuid:` prefix.
    pub fn uuid(&self) -> String {
        format!("uuid:{}", self.udn)
    }
}

/// One thing the server advertises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The `NT` value: what this target is.
    pub nt: String,
    /// The `USN` value: the identity of this target.
    pub usn: String,
}

/// The targets the server advertises, in the order they are answered.
pub fn advertised_targets(identity: &DeviceIdentity) -> Vec<Target> {
    let uuid = identity.uuid();
    vec![
        Target {
            nt: NT_ROOT_DEVICE.to_owned(),
            usn: format!("{uuid}::{NT_ROOT_DEVICE}"),
        },
        Target {
            nt: DEVICE_TYPE_MEDIA_SERVER.to_owned(),
            usn: format!("{uuid}::{DEVICE_TYPE_MEDIA_SERVER}"),
        },
        Target {
            nt: uuid.clone(),
            usn: uuid.clone(),
        },
        Target {
            nt: SERVICE_TYPE_CONTENT_DIRECTORY.to_owned(),
            usn: format!("{uuid}::{SERVICE_TYPE_CONTENT_DIRECTORY}"),
        },
        Target {
            nt: SERVICE_TYPE_CONNECTION_MANAGER.to_owned(),
            usn: format!("{uuid}::{SERVICE_TYPE_CONNECTION_MANAGER}"),
        },
    ]
}

/// The targets that answer a search for `search_target`.
///
/// An empty result means the search is for something the server does not
/// advertise, and the request is ignored (`INT-002`).
pub fn matching_targets(identity: &DeviceIdentity, search_target: &str) -> Vec<Target> {
    let targets = advertised_targets(identity);
    if search_target == NT_ALL {
        return targets;
    }
    targets
        .into_iter()
        .filter(|target| target.nt == search_target)
        .collect()
}

/// Whether the server advertises the searched target at all.
pub fn is_advertised(identity: &DeviceIdentity, search_target: &str) -> bool {
    !matching_targets(identity, search_target).is_empty()
}

#[cfg(test)]
mod tests {
    use super::{advertised_targets, is_advertised, matching_targets, DeviceIdentity};

    /// A fixed identity, so that the tests do not depend on randomness.
    fn identity() -> DeviceIdentity {
        DeviceIdentity::new("11111111-2222-3333-4444-555555555555", "Living room")
    }

    /// The five targets the device architecture expects for a root device with
    /// two services.
    #[test]
    fn five_targets_are_advertised() {
        assert_eq!(advertised_targets(&identity()).len(), 5);
    }

    /// A search for everything answers with everything.
    #[test]
    fn a_search_for_all_answers_with_all() {
        assert_eq!(matching_targets(&identity(), "ssdp:all").len(), 5);
    }

    /// A search for the root device answers with the root device.
    #[test]
    fn a_search_for_the_root_device_answers_once() {
        let matches = matching_targets(&identity(), "upnp:rootdevice");
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].usn,
            "uuid:11111111-2222-3333-4444-555555555555::upnp:rootdevice"
        );
    }

    /// A search for something the server does not advertise is ignored.
    #[test]
    fn an_unknown_target_is_ignored() {
        assert!(matching_targets(
            &identity(),
            "urn:schemas-upnp-org:device:InternetGatewayDevice:1"
        )
        .is_empty());
        assert!(!is_advertised(
            &identity(),
            "urn:schemas-upnp-org:device:InternetGatewayDevice:1"
        ));
    }

    /// The bare unique device name is a target of its own.
    #[test]
    fn the_bare_uuid_is_a_target() {
        let matches = matching_targets(&identity(), "uuid:11111111-2222-3333-4444-555555555555");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].usn, "uuid:11111111-2222-3333-4444-555555555555");
    }

    /// Every service type the server implements is advertised.
    #[test]
    fn both_services_are_advertised() {
        assert!(is_advertised(
            &identity(),
            "urn:schemas-upnp-org:service:ContentDirectory:1"
        ));
        assert!(is_advertised(
            &identity(),
            "urn:schemas-upnp-org:service:ConnectionManager:1"
        ));
    }
}
