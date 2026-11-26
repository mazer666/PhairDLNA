//! SSDP CLI inspection and replay commands.
//!
//! What: prints SSDP advertisement headers or replays stored M-SEARCH requests (`INT-103`).
//! Why: lets users and testing tools inspect SSDP behavior and responses deterministically.

use std::path::Path;

use phairdlna_core::config::Config;
use phairdlna_core::identity::InstalledIdentity;
use phairdlna_core::ssdp::{
    advertised_targets, build_advertisement, build_response, location_url, replay_path,
    FileReplayOutcome, ReplayResult, ServerIdentity,
};

use crate::{load_identity, resolve, Outcome};

/// Prints the SSDP headers the server would advertise and respond with.
pub fn headers(path: Option<&Path>) -> Outcome {
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
    Outcome::ok(ssdp_report(&identity, &config))
}

/// Formats the responses for one outcome.
fn append_outcome(
    output: &mut String,
    file: &str,
    outcome: Result<ReplayResult, phairdlna_core::ssdp::Ignored>,
) {
    output.push_str(&format!("=== replay: {file} ===\n"));
    match outcome {
        Ok(res) => {
            output.push_str(&format!(
                "Search Target: {}\nResponses: {}\n\n",
                res.search_target, res.response_count
            ));
            for resp in res.responses {
                output.push_str(&resp);
                if !resp.ends_with("\r\n\r\n") {
                    output.push_str("\r\n");
                }
            }
        }
        Err(ignored) => {
            output.push_str(&format!("Ignored: {ignored:?}\n\n"));
        }
    }
}

/// Formats the outcomes of all replayed requests.
fn format_results(results: Vec<FileReplayOutcome>) -> Outcome {
    if results.is_empty() {
        return Outcome::ok("no stored requests found in replay corpus\n".to_owned());
    }

    let mut output = String::new();
    for entry in &results {
        append_outcome(&mut output, &entry.0, entry.1.clone());
    }
    Outcome::ok(output)
}

/// Replays a stored M-SEARCH request or directory of requests against the configuration.
pub fn replay(corpus_path: &Path, config_path: Option<&Path>) -> Outcome {
    let cfg_path = resolve(config_path);
    let config = match Config::load(&cfg_path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                cfg_path.display(),
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

    let address = "192.168.0.10";
    let port = config.server.http_port;

    match replay_path(corpus_path, &device, &server, address, port) {
        Ok(results) => format_results(results),
        Err(err) => Outcome::failure(format!("replay failed: {err}\n")),
    }
}

/// The text of the SSDP report.
fn ssdp_report(identity: &InstalledIdentity, config: &Config) -> String {
    let device = identity.as_device_identity();
    let server = ServerIdentity {
        friendly_name: config.server.device_name.clone(),
        product: phairdlna_core::identity::PRODUCT_NAME.to_owned(),
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
    };
    let location = location_url("192.168.0.10", config.server.http_port);
    let mut report = String::new();
    report.push_str(&format!(
        "Device identity: {}\nDevice name:      {}\n\n",
        device.uuid(),
        identity.friendly_name
    ));
    report.push_str("=== advertisement (one per advertised target) ===\n\n");
    for target in advertised_targets(&device) {
        report.push_str(&build_advertisement(&target, &server, &location, true));
    }
    report.push_str("=== example answer to 'M-SEARCH * HTTP/1.1' with ST: ssdp:all ===\n\n");
    let target = advertised_targets(&device)
        .into_iter()
        .next()
        .unwrap_or_else(|| unreachable!("the root device is always advertised"));
    report.push_str(&build_response(&target, &server, &location));
    report.push_str("=== notes ===\n\n");
    report.push_str(
        "The LOCATION above uses a placeholder address. The server replaces it with the\n\
         address of the interface the request arrived on, never a wildcard or a loopback\n\
         address. Run 'ssdp headers' on the machine that serves the library.\n",
    );
    report
}
