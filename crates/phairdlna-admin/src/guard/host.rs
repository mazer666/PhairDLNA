//! Host-header validation for the administration surface.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;

use super::refused;

const HOST: &str = "Host";

/// The refusal of a request that names a host this surface does not answer to.
///
/// Only the loopback names are answered to. A name the attacker controls is
/// what the "DNS rebinding" attack needs (`SEC-TH-23`, `SEC-071`).
pub(super) fn refuse_host<B>(request: &Request<B>) -> Option<Response> {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok());
    if is_loopback_host(host) {
        return None;
    }
    Some(refused(StatusCode::MISDIRECTED_REQUEST))
}

/// Whether a `Host` header names the loopback interface.
///
/// The name `localhost` and every address of the loopback interface are
/// accepted; everything else is refused. A request without a `Host` header is
/// refused as well: it says nothing about which server the client meant to
/// reach.
fn is_loopback_host(header: Option<&str>) -> bool {
    let Some(header) = header else {
        return false;
    };
    let host = host_of(header);
    if host.is_empty() {
        return false;
    }
    if let Ok(address) = host.parse::<std::net::IpAddr>() {
        return address.is_loopback();
    }
    host == "localhost"
}

/// The host part of a `Host` header, in lower case and without the port.
fn host_of(header: &str) -> String {
    let trimmed = header.trim();
    let without_port = match trimmed.rfind(':') {
        // A colon that belongs to an IPv6 literal is not a port separator.
        Some(position) if !trimmed[position..].contains(']') => &trimmed[..position],
        _ => trimmed,
    };
    without_port
        .trim_matches(|character| character == '[' || character == ']')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{host_of, is_loopback_host};

    /// The loopback names are the names of this machine.
    #[test]
    fn the_loopback_names_are_accepted() {
        assert!(is_loopback_host(Some("localhost")));
        assert!(is_loopback_host(Some("localhost:8210")));
        assert!(is_loopback_host(Some("127.0.0.1")));
        assert!(is_loopback_host(Some("127.0.0.1:8210")));
        assert!(is_loopback_host(Some("[::1]")));
        assert!(is_loopback_host(Some("[::1]:8210")));
    }

    /// The negative case: a name the attacker controls is refused.
    #[test]
    fn a_name_the_attacker_controls_is_refused() {
        assert!(!is_loopback_host(Some("attacker.example")));
        assert!(!is_loopback_host(Some("attacker.example:8210")));
        assert!(!is_loopback_host(Some("192.168.1.20")));
        assert!(!is_loopback_host(Some("phairdlna.local")));
    }

    /// The negative case: a request without a host is refused.
    #[test]
    fn a_request_without_a_host_is_refused() {
        assert!(!is_loopback_host(None));
        assert!(!is_loopback_host(Some("")));
    }

    /// The host is read without its port and in lower case.
    #[test]
    fn the_host_is_read_without_its_port() {
        assert_eq!(host_of("LocalHost:8210"), "localhost");
        assert_eq!(host_of("[::1]:8210"), "::1");
    }
}
