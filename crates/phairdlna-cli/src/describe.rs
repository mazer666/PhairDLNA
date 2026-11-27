//! The describe command.
//!
//! What: prints the device description document and the SSDP headers it would
//!       advertise and answer with (`OPS-005`, `INT-015`).
//! Why: lets users and diagnostic tools inspect UPnP device description and SSDP
//!      headers offline without starting a daemon or opening a network socket.

use std::path::Path;

use phairdlna_core::config::Config;
use phairdlna_core::ssdp::{
    advertised_targets, build_advertisement, build_response, location_url, ServerIdentity,
};
use phairdlna_core::upnp::{BaseUrl, DeviceDescription, DeviceInfo};

use crate::{load_identity, resolve, Outcome, BASE_URL};

/// Prints the device description and the SSDP headers.
pub fn describe(path: Option<&Path>) -> Outcome {
    let path = resolve(path);
    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };
    let identity = match load_identity(&config) {
        Ok(identity) => identity,
        Err(error) => {
            return Outcome::failure(format!("cannot read the device identity: {error}\n"))
        }
    };

    let device = identity.as_device_identity();
    let server = ServerIdentity {
        friendly_name: config.server.device_name.clone(),
        product: phairdlna_core::identity::PRODUCT_NAME.to_owned(),
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
    };

    let base_url = match BaseUrl::new(BASE_URL) {
        Ok(url) => url,
        Err(error) => return Outcome::failure(format!("cannot create base URL: {error}\n")),
    };

    let dev_info = DeviceInfo::new(config.server.device_name.clone());
    let description = match DeviceDescription::new(&base_url, &device.uuid(), dev_info) {
        Ok(desc) => desc,
        Err(error) => {
            return Outcome::failure(format!("cannot build device description: {error}\n"))
        }
    };

    let location = location_url("192.168.0.10", config.server.http_port);
    let mut out = String::new();

    out.push_str("=== device description (UPnP XML) ===\n\n");
    out.push_str(&description.to_xml());
    out.push_str("\n\n=== SSDP advertisement header block (NOTIFY) ===\n\n");
    if let Some(target) = advertised_targets(&device).into_iter().next() {
        out.push_str(&build_advertisement(&target, &server, &location, true));
    }
    out.push_str("=== SSDP response header block (M-SEARCH 200 OK) ===\n\n");
    if let Some(target) = advertised_targets(&device).into_iter().next() {
        out.push_str(&build_response(&target, &server, &location));
    }

    Outcome::ok(out)
}
