//! The validation of a configuration document.
//!
//! What: the rules that decide whether a configuration may be used, and the
//!       messages that say what to change when it may not.
//! Why: `CFG-002` requires the failure to name the field, the file and the
//!      accepted values — a rule that fails silently is a rule that fails late.
//! Example: `server.http_port` of `0` is rejected with the sentence
//!      "field \"server.http_port\" is 0; expected a TCP port between 1 and 65535".

use super::defaults::Config;
use super::error::ConfigError;
use super::sections::{MediaRoot, NetworkSection, SecuritySection, ServerSection};

/// Checks the whole document and returns the first problem it finds.
///
/// The checks run in a fixed order — identity, then listener, then network,
/// then library, then limits, then access — so that the same broken file
/// always produces the same message. A user who fixes one problem should not
/// meet a *different* first problem next time.
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    validate_server(&config.server)?;
    validate_network(&config.network)?;
    validate_library(&config.library.roots)?;
    validate_limits(config)?;
    validate_security(&config.security)?;
    Ok(())
}

/// The identity and the listener.
fn validate_server(server: &ServerSection) -> Result<(), ConfigError> {
    if server.device_name.trim().is_empty() {
        return Err(ConfigError::field(
            "server.device_name",
            "field \"server.device_name\" is empty; expected a name a client can read, \
             for example \"Living room\"",
        ));
    }
    if server.http_port == 0 {
        return Err(ConfigError::field(
            "server.http_port",
            "field \"server.http_port\" is 0; expected a TCP port between 1 and 65535",
        ));
    }
    Ok(())
}

/// The interface selection.
fn validate_network(network: &NetworkSection) -> Result<(), ConfigError> {
    for (index, interface) in network.interfaces.iter().enumerate() {
        if interface.trim().is_empty() {
            return Err(ConfigError::field(
                format!("network.interfaces[{index}]"),
                format!(
                    "field \"network.interfaces[{index}]\" is empty; expected the name of an \
                     interface, for example \"en0\""
                ),
            ));
        }
    }
    Ok(())
}

/// The shared folders.
fn validate_library(roots: &[MediaRoot]) -> Result<(), ConfigError> {
    for (index, root) in roots.iter().enumerate() {
        validate_root(index, root)?;
    }
    Ok(())
}

/// One shared folder.
fn validate_root(index: usize, root: &MediaRoot) -> Result<(), ConfigError> {
    let field = format!("library.roots[{index}]");
    if root.name.trim().is_empty() {
        return Err(ConfigError::field(
            format!("{field}.name"),
            format!(
                "field \"{field}.name\" is empty; expected a name a client can read, \
                 for example \"Films\""
            ),
        ));
    }
    if root.path.trim().is_empty() {
        return Err(ConfigError::field(
            format!("{field}.path"),
            format!(
                "field \"{field}.path\" is empty; expected an absolute path, \
                 for example \"/Users/me/Movies\""
            ),
        ));
    }
    Ok(())
}

/// The resource ceilings.
fn validate_limits(config: &Config) -> Result<(), ConfigError> {
    let limits = &config.limits;
    if limits.max_concurrent_streams == 0 {
        return Err(ConfigError::field(
            "limits.max_concurrent_streams",
            "field \"limits.max_concurrent_streams\" is 0; expected at least 1, because a \
             server that serves no stream serves nothing",
        ));
    }
    if limits.index_workers == 0 {
        return Err(ConfigError::field(
            "limits.index_workers",
            "field \"limits.index_workers\" is 0; expected at least 1",
        ));
    }
    if limits.metadata_timeout_seconds == 0 {
        return Err(ConfigError::field(
            "limits.metadata_timeout_seconds",
            "field \"limits.metadata_timeout_seconds\" is 0; expected at least 1 second, \
             because a file that may take forever is a file that is never finished",
        ));
    }
    Ok(())
}

/// The access rules.
fn validate_security(security: &SecuritySection) -> Result<(), ConfigError> {
    if security.admin_enabled && security.admin_token_file.trim().is_empty() {
        return Err(ConfigError::field(
            "security.admin_token_file",
            "field \"security.admin_token_file\" is empty while the administration surface is \
             enabled; expected the path of the file holding the token",
        ));
    }
    Ok(())
}
