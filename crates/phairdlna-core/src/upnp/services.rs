//! The services the server advertises.
//!
//! What: the two services a media server must offer, the identifiers they are
//!       known by, and the paths their documents and endpoints live at.
//! Why: `MediaServer:1` mandates a ContentDirectory and a ConnectionManager
//!       service, and the advertised paths must be the paths the HTTP router
//!       serves. Writing both down once keeps the description and the router
//!       from drifting apart (`INT-021`, `INT-029`, `INT-044`).
//! Example: the ContentDirectory service is advertised with the service
//!       identifier `urn:upnp-org:serviceId:ContentDirectory`.

use crate::ssdp::{SERVICE_TYPE_CONNECTION_MANAGER, SERVICE_TYPE_CONTENT_DIRECTORY};

/// The identifier of the ContentDirectory service.
pub const SERVICE_ID_CONTENT_DIRECTORY: &str = "urn:upnp-org:serviceId:ContentDirectory";

/// The identifier of the ConnectionManager service.
///
/// The specification template writes this identifier as `ConnectorManager`,
/// which is an evident typo; every shipped implementation and the annex of the
/// specification use `ConnectionManager` (`INT-044`).
pub const SERVICE_ID_CONNECTION_MANAGER: &str = "urn:upnp-org:serviceId:ConnectionManager";

/// One advertised service and where its documents live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceDefinition {
    /// The service type, for example `urn:schemas-upnp-org:service:ContentDirectory:1`.
    pub service_type: &'static str,
    /// The service identifier, for example `urn:upnp-org:serviceId:ContentDirectory`.
    pub service_id: &'static str,
    /// The path of the service description document.
    pub scpd_path: &'static str,
    /// The path the control requests are sent to.
    pub control_path: &'static str,
    /// The path the event subscriptions are sent to.
    pub event_path: &'static str,
}

/// The ContentDirectory service.
pub const CONTENT_DIRECTORY: ServiceDefinition = ServiceDefinition {
    service_type: SERVICE_TYPE_CONTENT_DIRECTORY,
    service_id: SERVICE_ID_CONTENT_DIRECTORY,
    scpd_path: "/ContentDirectory.xml",
    control_path: "/ctl/CD",
    event_path: "/evt/CD",
};

/// The ConnectionManager service.
///
/// It is advertised even when no transcoding is available, because a client
/// uses it to learn the transfer protocols the server speaks (`INT-029`).
pub const CONNECTION_MANAGER: ServiceDefinition = ServiceDefinition {
    service_type: SERVICE_TYPE_CONNECTION_MANAGER,
    service_id: SERVICE_ID_CONNECTION_MANAGER,
    scpd_path: "/ConnectionManager.xml",
    control_path: "/ctl/CM",
    event_path: "/evt/CM",
};

/// The services of the server, in the order they are advertised.
pub fn advertised_services() -> [ServiceDefinition; 2] {
    [CONTENT_DIRECTORY, CONNECTION_MANAGER]
}

#[cfg(test)]
mod tests {
    use super::{
        advertised_services, CONNECTION_MANAGER, CONTENT_DIRECTORY, SERVICE_ID_CONNECTION_MANAGER,
        SERVICE_ID_CONTENT_DIRECTORY,
    };

    /// Both mandatory services are advertised, and no third one.
    #[test]
    fn exactly_two_services_are_advertised() {
        let services = advertised_services();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0], CONTENT_DIRECTORY);
        assert_eq!(services[1], CONNECTION_MANAGER);
    }

    /// The service identifiers are the spellings every implementation uses.
    #[test]
    fn the_service_identifiers_are_the_standard_ones() {
        assert_eq!(
            CONTENT_DIRECTORY.service_id,
            "urn:upnp-org:serviceId:ContentDirectory"
        );
        assert_eq!(
            CONNECTION_MANAGER.service_id,
            "urn:upnp-org:serviceId:ConnectionManager"
        );
        assert_eq!(SERVICE_ID_CONTENT_DIRECTORY, CONTENT_DIRECTORY.service_id);
        assert_eq!(SERVICE_ID_CONNECTION_MANAGER, CONNECTION_MANAGER.service_id);
    }

    /// Every advertised path is a root-relative path that starts with a slash.
    #[test]
    fn every_advertised_path_is_root_relative() {
        for service in advertised_services() {
            for path in [service.scpd_path, service.control_path, service.event_path] {
                assert!(path.starts_with('/'), "{path} is not root-relative");
            }
        }
    }
}
