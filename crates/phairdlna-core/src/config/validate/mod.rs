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

mod library;
use super::sections::{NetworkSection, SecuritySection, ServerSection};

use self::library::validate_library;

/// Checks the whole document and returns the first problem it finds.
///
/// The checks run in a fixed order — identity, then listener, then network,
/// then library, then limits, then access — so that the same broken file
/// always produces the same message. A user who fixes one problem should not
/// meet a *different* first problem next time.
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    validate_server(&config.server)?;
    validate_network(&config.network)?;
    validate_library(&config.library)?;
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
    if server.ssdp_port == 0 {
        return Err(ConfigError::field(
            "server.ssdp_port",
            "field \"server.ssdp_port\" is 0; expected a UDP port between 1 and 65535",
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
    validate_protocol_limits(limits)
}

/// Validates protocol size and timeout ceilings.
fn validate_protocol_limits(limits: &crate::config::LimitsSection) -> Result<(), ConfigError> {
    validate_positive(
        "limits.max_request_line_bytes",
        limits.max_request_line_bytes,
        "the request line of a client",
    )?;
    validate_positive(
        "limits.max_header_block_bytes",
        limits.max_header_block_bytes,
        "the headers of a client",
    )?;
    validate_positive(
        "limits.max_header_count",
        limits.max_header_count,
        "the number of headers of a client",
    )?;
    validate_positive(
        "limits.max_body_bytes",
        limits.max_body_bytes,
        "the body of a client",
    )?;
    validate_positive(
        "limits.max_concurrent_connections",
        limits.max_concurrent_connections,
        "the number of connections",
    )?;
    validate_positive(
        "limits.max_didl_bytes",
        limits.max_didl_bytes,
        "the size of a DIDL-Lite document",
    )?;
    validate_positive(
        "limits.read_timeout_seconds",
        limits.read_timeout_seconds as usize,
        "how long a connection may stay idle",
    )?;
    validate_positive(
        "limits.write_timeout_seconds",
        limits.write_timeout_seconds as usize,
        "how long an answer may take to be written",
    )
}

/// Rejects a ceiling of zero.
///
/// A ceiling of zero is not a tight bound, it is a server that answers nothing
/// — and a user who writes it deserves to be told so (`CFG-002`).
fn validate_positive(field: &str, value: usize, what: &str) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::field(
            field,
            format!(
                "field \"{field}\" is 0; expected at least 1, because a limit of 0 on {what} \
                 means the server answers nothing"
            ),
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
    for (index, name) in security.allowed_host_names.iter().enumerate() {
        validate_host_name(index, name)?;
    }
    Ok(())
}

/// The names the server answers to must be names, and nothing else.
///
/// A name with a port in it, or with a separator in it, is a mistake in the
/// configuration: the server would never accept it, and the user would wait
/// for an answer that never comes (`SEC-071`).
fn validate_host_name(index: usize, name: &str) -> Result<(), ConfigError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::field(
            "security.allowed_host_names",
            format!(
                "field \"security.allowed_host_names[{index}]\" is empty; expected the name of a \
                 host the server answers to"
            ),
        ));
    }
    if trimmed.contains(':') || trimmed.contains('/') || trimmed.contains(' ') {
        return Err(ConfigError::field(
            "security.allowed_host_names",
            format!(
                "field \"security.allowed_host_names[{index}]\" is \"{trimmed}\"; expected a host \
                 name without a port, a separator or a space"
            ),
        ));
    }
    Ok(())
}
