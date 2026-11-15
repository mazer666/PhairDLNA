//! The discovery protocol.
//!
//! What: the half of SSDP that does not need a socket — parsing a search,
//!       deciding which advertised targets answer it, and writing the bytes of
//!       a reply or an advertisement.
//! Why: the socket is the easy part to get wrong and the hard part to test. The
//!       decisions — *should this datagram be answered at all?*, *which of the
//!       five targets answers it?*, *what exactly goes on the wire?* — are pure
//!       and are tested here.
//! Example:
//!
//! ```
//! use phairdlna_core::ssdp::{parse_search, Ignored};
//!
//! let datagram = "M-SEARCH * HTTP/1.1\r\n\
//!     MAN: \"ssdp:discover\"\r\nMX: 3\r\nST: ssdp:all\r\n\r\n";
//! let search = parse_search(datagram).expect("a valid search");
//! assert_eq!(search.mx, 3);
//! assert_eq!(parse_search("NOTIFY * HTTP/1.1\r\n\r\n"), Err(Ignored::NotASearch));
//! ```

mod constants;
mod request;
mod targets;

pub use constants::{
    ADVERTISEMENT_INTERVAL_SECONDS, CACHE_CONTROL_MAX_AGE, DESCRIPTION_PATH,
    DEVICE_TYPE_MEDIA_SERVER, MAN_DISCOVER, MAX_MX_SECONDS, METHOD_NOTIFY, METHOD_SEARCH,
    MIN_MX_SECONDS, MULTICAST_GROUP, MULTICAST_GROUP_V6, MULTICAST_TTL, NTS_ALIVE, NTS_BYEBYE,
    NT_ALL, NT_ROOT_DEVICE, PORT, SERVICE_TYPE_CONNECTION_MANAGER, SERVICE_TYPE_CONTENT_DIRECTORY,
};
pub use request::{
    build_advertisement, build_response, format_http_date, location_url, parse_search,
    reply_delay_millis, Ignored, Search, ServerIdentity,
};
pub use targets::{advertised_targets, is_advertised, matching_targets, DeviceIdentity, Target};
