//! The start of the daemon.
//!
//! What: the steps that bring the daemon up — the configuration, the identity,
//!       the catalogue, the sockets.
//! Why: the order of these steps is a rule, not a preference: a broken
//!       configuration must be refused before a socket is opened, so that a
//!       failed start leaves nothing behind (`CFG-002`, `F-13` § 9).
//! Example: `start_http` opens the listener and hands it to the server task.

mod configuration;
mod discovery;
mod http_server;
mod socket;
mod startup;

pub use configuration::{address_and_port, device_info, load_config, load_identity, roots_of};
pub use discovery::start_ssdp;
pub use http_server::{start_http, HttpServer};
pub use socket::{open_the_socket, start_capped_http};
pub use startup::start;
