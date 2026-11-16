//! The names the server answers to.
//!
//! What: the rule that decides whether the `Host` header of a request names
//!       this server, and the set of names it accepts.
//! Why: a browser on the same network can be pointed at the server through a
//!       name the attacker controls — the "DNS rebinding" attack. The browser
//!       believes it is talking to the attacker's page, and the server believes
//!       it is talking to a client on the local network (`SEC-TH-23`). The
//!       `Host` header is what tells the two apart, so the server reads it
//!       before it answers (`SEC-071`).
//! Example: a request that carries `Host: attacker.example` is refused,
//!       because a name that is not one of ours is not the server the client
//!       meant to reach.
//!
//! **What is accepted.** A host that is an **IP literal** is always accepted:
//! a name is what an attacker can point somewhere else, an address is not.
//! A host that is a **name** is accepted only when the user configured it.
//! The rule therefore costs a client that connects by address nothing, and it
//! stops the attack that the rule exists for.

use std::net::IpAddr;

/// The names a server answers to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostPolicy {
    /// The names the user configured, without a port and in lower case.
    allowed: Vec<String>,
}

impl HostPolicy {
    /// Builds a policy that accepts the given names.
    ///
    /// A name is read without its port and in lower case, because a host name
    /// is not case sensitive (`RFC 9110` § 4.2.3).
    pub fn new(names: impl IntoIterator<Item = String>) -> Self {
        Self {
            allowed: names
                .into_iter()
                .map(|name| normalise(&name))
                .filter(|name| !name.is_empty())
                .collect(),
        }
    }

    /// Whether a `Host` header names this server.
    ///
    /// A header that names nothing is refused: a request without a `Host`
    /// header says nothing about which server the client meant to reach.
    pub fn accepts(&self, header: Option<&str>) -> bool {
        let Some(header) = header else {
            return false;
        };
        let host = host_of(header);
        if host.is_empty() {
            return false;
        }
        if host.parse::<IpAddr>().is_ok() {
            return true;
        }
        self.allowed.contains(&host)
    }

    /// The names the policy accepts, in lower case and without a port.
    pub fn allowed(&self) -> &[String] {
        &self.allowed
    }
}

/// The host part of a `Host` header.
///
/// The port is dropped, the brackets of an IPv6 literal are kept, and the name
/// is read in lower case.
fn host_of(header: &str) -> String {
    let trimmed = header.trim();
    let without_port = match trimmed.rfind(':') {
        // A colon that belongs to an IPv6 literal is not a port separator.
        Some(position) if !trimmed[position..].contains(']') => &trimmed[..position],
        _ => trimmed,
    };
    normalise(without_port)
}

/// Reads a host name in the form the policy stores it.
fn normalise(name: &str) -> String {
    name.trim()
        .trim_matches(|c| c == '[' || c == ']')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::HostPolicy;

    /// A policy with no name accepts nothing but addresses.
    #[test]
    fn an_address_is_always_accepted() {
        let policy = HostPolicy::new(Vec::new());

        assert!(policy.accepts(Some("192.168.1.20:8200")));
        assert!(policy.accepts(Some("127.0.0.1:8200")));
        assert!(policy.accepts(Some("[::1]:8200")));
    }

    /// A name that was not configured is refused.
    #[test]
    fn an_unknown_name_is_refused() {
        let policy = HostPolicy::new(Vec::new());

        assert!(!policy.accepts(Some("attacker.example")));
        assert!(!policy.accepts(Some("attacker.example:8200")));
    }

    /// A name the user configured is accepted.
    #[test]
    fn a_configured_name_is_accepted() {
        let policy = HostPolicy::new(vec!["phairdlna.local".to_owned()]);

        assert!(policy.accepts(Some("phairdlna.local:8200")));
        assert!(policy.accepts(Some("PhairDLNA.local")));
    }

    /// A header that names nothing is refused.
    #[test]
    fn a_missing_host_is_refused() {
        let policy = HostPolicy::new(vec!["phairdlna.local".to_owned()]);

        assert!(!policy.accepts(None));
        assert!(!policy.accepts(Some("")));
        assert!(!policy.accepts(Some("   ")));
    }

    /// A header that carries only a port names nothing.
    #[test]
    fn a_port_alone_names_nothing() {
        let policy = HostPolicy::new(Vec::new());

        assert!(!policy.accepts(Some(":8200")));
    }

    /// The names are read without their port and in lower case.
    #[test]
    fn the_names_are_read_in_their_canonical_form() {
        let policy = HostPolicy::new(vec!["Living-Room.local".to_owned()]);

        assert_eq!(policy.allowed(), ["living-room.local"]);
    }
}
