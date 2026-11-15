//! The constants of the discovery protocol.
//!
//! What: the group, the port, the cache lifetime and the header names.
//! Why: these are reserved by IANA and by the UPnP Device Architecture. They are
//!      written down once, so that a change cannot half-apply.
//! Example: `MULTICAST_GROUP` is `239.255.255.250`, the group SSDP is defined on.

/// The IPv4 multicast group SSDP uses.
pub const MULTICAST_GROUP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(239, 255, 255, 250);

/// The IPv6 link-local multicast group SSDP uses.
pub const MULTICAST_GROUP_V6: std::net::Ipv6Addr =
    std::net::Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x000c);

/// The port SSDP uses.
pub const PORT: u16 = 1900;

/// How long a client may cache an answer, in seconds.
///
/// The UPnP Device Architecture asks for at least 1800 s.
pub const CACHE_CONTROL_MAX_AGE: u32 = 1800;

/// How often the server re-advertises, in seconds.
///
/// A third of the cache lifetime is the conservative choice; the research records
/// `max-age/4` as the usual recommendation. 450 s satisfies both.
pub const ADVERTISEMENT_INTERVAL_SECONDS: u32 = CACHE_CONTROL_MAX_AGE / 4;

/// The time-to-live of an advertisement.
///
/// Two hops is enough for a home network and keeps the datagram off the internet.
pub const MULTICAST_TTL: u8 = 2;

/// The largest `MX` the server honours, in seconds.
pub const MAX_MX_SECONDS: u32 = 5;

/// The smallest `MX` the server accepts, in seconds.
pub const MIN_MX_SECONDS: u32 = 1;

/// The method that searches for devices.
pub const METHOD_SEARCH: &str = "M-SEARCH";

/// The method that announces a device.
pub const METHOD_NOTIFY: &str = "NOTIFY";

/// The `MAN` header value a search must carry.
pub const MAN_DISCOVER: &str = "\"ssdp:discover\"";

/// The `NT` value of the root device.
pub const NT_ROOT_DEVICE: &str = "upnp:rootdevice";

/// The `NT` value that asks for everything.
pub const NT_ALL: &str = "ssdp:all";

/// The `NTS` value of a live announcement.
pub const NTS_ALIVE: &str = "ssdp:alive";

/// The `NTS` value of a withdrawal.
pub const NTS_BYEBYE: &str = "ssdp:byebye";

/// The device type of a UPnP AV media server.
pub const DEVICE_TYPE_MEDIA_SERVER: &str = "urn:schemas-upnp-org:device:MediaServer:1";

/// The type of the ContentDirectory service.
pub const SERVICE_TYPE_CONTENT_DIRECTORY: &str = "urn:schemas-upnp-org:service:ContentDirectory:1";

/// The type of the ConnectionManager service.
pub const SERVICE_TYPE_CONNECTION_MANAGER: &str =
    "urn:schemas-upnp-org:service:ConnectionManager:1";

/// The path of the device description document.
pub const DESCRIPTION_PATH: &str = "/description.xml";
