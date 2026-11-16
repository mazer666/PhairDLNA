//! The rules of the administration surface.
//!
//! What: the rules that decide whether the administration surface of a
//!       configuration may be started at all.
//! Why: the surface is meant for the machine the server runs on. A
//!      configuration that would put it on the network has to be refused, and
//!      the refusal has to name the setting that has to change (`CFG-002`,
//!      `CFG-005`, `OPS-020`).
//! Example: a configuration that binds the surface to `192.168.1.20` is
//!      refused with the sentence "field \"security.admin_bind_addresses[0]\"
//!      is not a loopback address; set \"security.admin_allow_non_loopback\" to
//!      true to allow it".

use std::net::IpAddr;

use super::super::error::ConfigError;
use super::super::sections::SecuritySection;

/// Checks the settings of the administration surface.
pub fn validate_admin(security: &SecuritySection) -> Result<(), ConfigError> {
    if security.admin_enabled && security.admin_port == 0 {
        return Err(ConfigError::field(
            "security.admin_port",
            "field \"security.admin_port\" is 0 while the administration surface is enabled; \
             expected a TCP port between 1 and 65535",
        ));
    }
    for (index, address) in security.admin_bind_addresses.iter().enumerate() {
        validate_bind_address(index, address, security.admin_allow_non_loopback)?;
    }
    Ok(())
}

/// Checks one address the surface wants to bind to.
///
/// The address must be an address and not a name: a name is what an attacker
/// can point somewhere else. An address that is not a loopback address is
/// refused unless the user has said, in the configuration, that the surface may
/// leave the machine (`SEC-071`, `OPS-020`).
fn validate_bind_address(
    index: usize,
    address: &str,
    allow_non_loopback: bool,
) -> Result<(), ConfigError> {
    let field = format!("security.admin_bind_addresses[{index}]");
    let trimmed = address.trim();
    let parsed = trimmed.parse::<IpAddr>().map_err(|_| {
        ConfigError::field(
            field.clone(),
            format!(
                "field \"{field}\" is \"{trimmed}\"; expected an IP address such as 127.0.0.1 \
                 or ::1, and not a host name"
            ),
        )
    })?;
    if !parsed.is_loopback() && !allow_non_loopback {
        return Err(ConfigError::field(
            field.clone(),
            format!(
                "field \"{field}\" is \"{trimmed}\", which is not a loopback address; the \
                 administration surface is only reachable from this machine. Set \
                 \"security.admin_allow_non_loopback\" to true to allow it anyway"
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_admin;
    use crate::config::SecuritySection;

    /// The settings of a surface that is switched off.
    fn section() -> SecuritySection {
        SecuritySection {
            admin_enabled: true,
            admin_port: 8210,
            ..SecuritySection::default()
        }
    }

    /// The loopback addresses are accepted.
    #[test]
    fn the_loopback_addresses_are_accepted() {
        let security = SecuritySection {
            admin_bind_addresses: vec!["127.0.0.1".to_owned(), "::1".to_owned()],
            ..section()
        };
        assert!(validate_admin(&security).is_ok());
    }

    /// The negative case: an address on the network is refused.
    #[test]
    fn a_non_loopback_address_is_refused() {
        let security = SecuritySection {
            admin_bind_addresses: vec!["192.168.1.20".to_owned()],
            ..section()
        };
        let error = validate_admin(&security).expect_err("the address is refused");
        assert!(error.message().contains("admin_allow_non_loopback"));
    }

    /// The negative case: a name is refused, a name is what an attacker moves.
    #[test]
    fn a_name_is_refused() {
        let security = SecuritySection {
            admin_bind_addresses: vec!["phairdlna.local".to_owned()],
            ..section()
        };
        assert!(validate_admin(&security).is_err());
    }

    /// The user may override the refusal, and then the address is accepted.
    #[test]
    fn the_user_may_override_the_refusal() {
        let security = SecuritySection {
            admin_bind_addresses: vec!["192.168.1.20".to_owned()],
            admin_allow_non_loopback: true,
            ..section()
        };
        assert!(validate_admin(&security).is_ok());
    }

    /// The port of the surface must be a port.
    #[test]
    fn the_port_of_the_surface_must_be_a_port() {
        let security = SecuritySection {
            admin_port: 0,
            ..section()
        };
        assert!(validate_admin(&security).is_err());
    }
}
