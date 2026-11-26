//! Stored `M-SEARCH` request replay and diagnostic response builder.
//!
//! What: parses raw M-SEARCH requests from disk/corpus and computes the byte-for-byte
//!       response datagrams the server would emit (`INT-103`).
//! Why: lets operators and diagnostic tools inspect how the server would respond
//!      to actual client discovery requests captured from real devices without
//!      requiring active UDP network communication (`F-07`, `INT-103`).

use std::path::Path;

use super::{
    build_response, location_url, matching_targets, parse_search, DeviceIdentity, Ignored,
    ServerIdentity,
};

/// A replayed search outcome holding the matching responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayResult {
    /// The search target (ST) from the replayed request.
    pub search_target: String,
    /// The number of matching responses generated.
    pub response_count: usize,
    /// The formatted response datagrams that would be transmitted.
    pub responses: Vec<String>,
}

/// A single file replay outcome.
pub type FileReplayOutcome = (String, Result<ReplayResult, Ignored>);

/// Replays a raw M-SEARCH datagram against the provided device and server identities.
///
/// Returns `Ok(ReplayResult)` if the request was valid and matched zero or more targets,
/// or `Err(Ignored)` if the datagram was ignored according to UPnP SSDP rules.
pub fn replay_datagram(
    datagram: &str,
    device: &DeviceIdentity,
    server: &ServerIdentity,
    address: &str,
    port: u16,
) -> Result<ReplayResult, Ignored> {
    let search = parse_search(datagram)?;
    let targets = matching_targets(device, &search.search_target);
    let location = location_url(address, port);

    let responses: Vec<String> = targets
        .iter()
        .map(|target| build_response(target, server, &location))
        .collect();

    Ok(ReplayResult {
        search_target: search.search_target,
        response_count: responses.len(),
        responses,
    })
}

/// Helper that reads a directory of requests into outcomes without deep nesting.
fn replay_directory(
    path: &Path,
    device: &DeviceIdentity,
    server: &ServerIdentity,
    address: &str,
    port: u16,
) -> Result<Vec<FileReplayOutcome>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("cannot read directory \"{}\": {e}", path.display()))?;

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&entry_path) else {
            continue;
        };
        let res = replay_datagram(&content, device, server, address, port);
        results.push((entry_path.display().to_string(), res));
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

/// Replays a file containing an M-SEARCH datagram or inspects a corpus directory.
///
/// If `path` is a file, replays its content. If it does not exist or has an empty
/// corpus, returns an error message or empty corpus notification (`INT-103`, `OPS-014`).
pub fn replay_path(
    path: &Path,
    device: &DeviceIdentity,
    server: &ServerIdentity,
    address: &str,
    port: u16,
) -> Result<Vec<FileReplayOutcome>, String> {
    if !path.exists() {
        return Err(format!("path \"{}\" does not exist", path.display()));
    }

    if path.is_file() {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read file \"{}\": {e}", path.display()))?;
        let res = replay_datagram(&content, device, server, address, port);
        return Ok(vec![(path.display().to_string(), res)]);
    }

    if path.is_dir() {
        return replay_directory(path, device, server, address, port);
    }

    Err(format!("invalid path type \"{}\"", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_device() -> DeviceIdentity {
        DeviceIdentity::new("12345678-1234-1234-1234-123456789abc", "Living Room TV")
    }

    fn test_server() -> ServerIdentity {
        ServerIdentity {
            friendly_name: "Living Room TV".to_owned(),
            product: "PhairDLNA".to_owned(),
            version: "0.1.0".to_owned(),
        }
    }

    #[test]
    fn replay_valid_all_search() {
        let raw = "M-SEARCH * HTTP/1.1\r\n\
                   HOST: 239.255.255.250:1900\r\n\
                   MAN: \"ssdp:discover\"\r\n\
                   MX: 3\r\n\
                   ST: ssdp:all\r\n\r\n";
        let res = replay_datagram(raw, &test_device(), &test_server(), "192.168.1.50", 8200)
            .expect("valid replay");
        assert_eq!(res.search_target, "ssdp:all");
        assert_eq!(res.response_count, 5);
        assert_eq!(res.responses.len(), 5);
        assert!(res.responses[0].starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(res.responses[0].contains("LOCATION: http://192.168.1.50:8200/description.xml\r\n"));
    }

    #[test]
    fn replay_unknown_target_gives_zero_responses() {
        let raw = "M-SEARCH * HTTP/1.1\r\n\
                   HOST: 239.255.255.250:1900\r\n\
                   MAN: \"ssdp:discover\"\r\n\
                   MX: 3\r\n\
                   ST: urn:schemas-upnp-org:device:Unknown:1\r\n\r\n";
        let res = replay_datagram(raw, &test_device(), &test_server(), "192.168.1.50", 8200)
            .expect("valid replay");
        assert_eq!(res.response_count, 0);
        assert!(res.responses.is_empty());
    }

    #[test]
    fn replay_invalid_datagram_fails() {
        let raw = "NOTIFY * HTTP/1.1\r\n\r\n";
        let err = replay_datagram(raw, &test_device(), &test_server(), "192.168.1.50", 8200)
            .expect_err("should fail");
        assert_eq!(err, Ignored::NotASearch);
    }
}
