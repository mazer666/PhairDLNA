//! Configuration and identity inputs for daemon startup.

use std::net::Ipv4Addr;
use std::path::PathBuf;

use phairdlna_core::config::Config;
use phairdlna_core::identity;
use phairdlna_core::upnp::DeviceInfo;

use super::super::{address, ServeOptions};

/// Reads and validates the configuration.
///
/// A configuration that cannot be used stops the daemon before any socket is
/// opened, and the user is told what is wrong (`CFG-002`, `F-13` § 9).
pub fn load_config(path: &std::path::Path) -> Result<Config, String> {
    Config::load(path).map_err(|error| {
        format!(
            "the configuration \"{}\" is not usable:\n{}",
            path.display(),
            error.message()
        )
    })
}

/// Loads the identity of the installation, creating it when it is missing.
pub fn load_identity(
    config: &Config,
    path: Option<&std::path::Path>,
) -> Result<identity::InstalledIdentity, String> {
    let path = match path {
        Some(path) => path.to_path_buf(),
        None => identity::identity_path(),
    };
    identity::load_or_create(&path, &config.server.device_name)
        .map_err(|error| format!("the device identity cannot be read: {error}"))
}

/// The identity strings of the device description.
pub fn device_info(config: &Config) -> DeviceInfo {
    DeviceInfo::new(config.server.device_name.clone())
}

/// The roots the media may not leave.
pub fn roots_of(config: &Config) -> Vec<PathBuf> {
    config
        .library
        .roots
        .iter()
        .map(|root| PathBuf::from(&root.path))
        .collect()
}

/// The address and the port of the server.
pub fn address_and_port(options: &ServeOptions, config: &Config) -> (Ipv4Addr, u16) {
    let host = options
        .host
        .unwrap_or_else(address::local_address_or_loopback);
    let port = options.http_port.unwrap_or(config.server.http_port);
    (host, port)
}
