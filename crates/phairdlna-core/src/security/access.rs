//! The clients that may talk to the server.
//!
//! What: the rule that decides whether a client address may browse the
//!       library and stream a file.
//! Why: the server lives on a network the user does not control. A user who
//!       shares the network with guests may want to keep them out of the
//!       library, and a setting that is written down but never obeyed is
//!       worse than no setting at all (`CFG-009`, `CFG-013`).
//! Example: the rule "192.168.1.0/24" admits every client of that network and
//!          refuses every other one.
//!
//! **The empty list admits everyone.** A server without an access-control list
//! is a server for the network it lives in, which is what the user asked for
//! when the list is empty.

use std::net::IpAddr;

/// The clients that may talk to the server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientAccess {
    /// The rules of the list.
    rules: Vec<Rule>,
}

/// One rule of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Rule {
    /// Every client is admitted.
    Any,
    /// One address is admitted.
    Address(IpAddr),
    /// One network is admitted.
    Network(IpAddr, u8),
}

impl ClientAccess {
    /// Builds the list of the clients that may talk to the server.
    ///
    /// An entry is an address (`192.168.1.20`), a network (`192.168.1.0/24`)
    /// or the word `any`. An entry that is none of these is an error: a rule
    /// that cannot be read would silently admit or silently refuse, and the
    /// user would not know which (`CFG-002`).
    pub fn new(entries: &[String]) -> Result<Self, String> {
        let mut rules = Vec::new();
        for entry in entries {
            rules.push(Rule::parse(entry)?);
        }
        Ok(Self { rules })
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Whether a client may talk to the server.
    ///
    /// An empty list admits every client: the user who writes no rule asks for
    /// no restriction.
    pub fn admits(&self, address: &IpAddr) -> bool {
        if self.rules.is_empty() {
            return true;
        }
        self.rules.iter().any(|rule| rule.admits(address))
    }
}

impl Rule {
    /// Reads one rule.
    fn parse(entry: &str) -> Result<Self, String> {
        let trimmed = entry.trim();
        if trimmed.eq_ignore_ascii_case("any") {
            return Ok(Self::Any);
        }
        if let Some((address, prefix)) = trimmed.split_once('/') {
            let address: IpAddr = address
                .trim()
                .parse()
                .map_err(|_| format!("\"{trimmed}\" is not an address with a prefix"))?;
            let prefix: u8 = prefix
                .trim()
                .parse()
                .map_err(|_| format!("\"{trimmed}\" does not name a number of bits"))?;
            if prefix > address_prefix(&address) {
                return Err(format!(
                    "\"{trimmed}\" names more bits than the address has"
                ));
            }
            return Ok(Self::Network(address, prefix));
        }
        let address: IpAddr = trimmed
            .parse()
            .map_err(|_| format!("\"{trimmed}\" is not an address"))?;
        Ok(Self::Address(address))
    }

    /// Whether the rule admits an address.
    fn admits(&self, address: &IpAddr) -> bool {
        match self {
            Self::Any => true,
            Self::Address(allowed) => allowed == address,
            Self::Network(network, prefix) => in_network(network, *prefix, address),
        }
    }
}

/// The number of bits of an address.
fn address_prefix(address: &IpAddr) -> u8 {
    match address {
        IpAddr::V4(_) => 32,
        IpAddr::V6(_) => 128,
    }
}

/// Whether an address lies in a network.
fn in_network(network: &IpAddr, prefix: u8, address: &IpAddr) -> bool {
    match (network, address) {
        (IpAddr::V4(network), IpAddr::V4(address)) => {
            let mask = mask_of_four(prefix);
            u32::from(*network) & mask == u32::from(*address) & mask
        }
        (IpAddr::V6(network), IpAddr::V6(address)) => {
            let mask = mask_of_six(prefix);
            u128::from(*network) & mask == u128::from(*address) & mask
        }
        // A rule of one family never admits an address of the other family.
        _ => false,
    }
}

/// The mask of a number of bits of an address of four bytes.
fn mask_of_four(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

/// The mask of a number of bits of an address of sixteen bytes.
fn mask_of_six(prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientAccess;
    use std::net::IpAddr;

    /// The list of the entries of the configuration.
    fn access(entries: &[&str]) -> ClientAccess {
        let entries: Vec<String> = entries.iter().map(|entry| (*entry).to_owned()).collect();
        ClientAccess::new(&entries).expect("the list is usable")
    }

    /// The empty list admits every client.
    #[test]
    fn the_empty_list_admits_every_client() {
        let list = access(&[]);
        let address: IpAddr = "192.168.1.20".parse().expect("an address");

        assert!(list.is_empty());
        assert!(list.admits(&address));
    }

    /// An address admits itself and nobody else.
    #[test]
    fn an_address_admits_itself() {
        let list = access(&["192.168.1.20"]);
        let allowed: IpAddr = "192.168.1.20".parse().expect("an address");
        let other: IpAddr = "192.168.1.21".parse().expect("an address");

        assert!(list.admits(&allowed));
        assert!(!list.admits(&other));
    }

    /// A network admits every address of it.
    #[test]
    fn a_network_admits_its_addresses() {
        let list = access(&["192.168.1.0/24"]);
        let inside: IpAddr = "192.168.1.99".parse().expect("an address");
        let outside: IpAddr = "192.168.2.1".parse().expect("an address");

        assert!(list.admits(&inside));
        assert!(!list.admits(&outside));
    }

    /// The word "any" admits every client.
    #[test]
    fn the_word_any_admits_every_client() {
        let list = access(&["any"]);
        let address: IpAddr = "203.0.113.7".parse().expect("an address");

        assert!(list.admits(&address));
    }

    /// The negative case: an entry that is not an address is an error.
    #[test]
    fn an_entry_that_is_not_an_address_is_an_error() {
        let entries = vec!["not an address".to_owned()];
        assert!(ClientAccess::new(&entries).is_err());
    }

    /// The negative case: a prefix that is longer than the address is an error.
    #[test]
    fn a_prefix_that_is_too_long_is_an_error() {
        let entries = vec!["192.168.1.0/33".to_owned()];
        assert!(ClientAccess::new(&entries).is_err());
    }

    /// The negative case: a rule of one family never admits the other.
    #[test]
    fn a_rule_of_one_family_never_admits_the_other() {
        let list = access(&["192.168.1.0/24"]);
        let address: IpAddr = "::1".parse().expect("an address");

        assert!(!list.admits(&address));
    }
}
