//! The origin the device description is served from.
//!
//! What: the validated origin — scheme, host and port — that the URLs inside
//!       the description are built from.
//! Why: the URLs of the document are absolute, so that a client never has to
//!       guess which address of a multi-homed host it should use (`INT-022`).
//!       The origin is validated once, here, instead of at every place a URL
//!       is assembled.
//! Example: `BaseUrl::new("http://192.168.1.20:8200")` accepts the address of
//!       the interface that served the description.

use super::error::UpnpError;

/// The origin the description is served from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseUrl {
    origin: String,
}

impl BaseUrl {
    /// Builds the origin from a string such as `http://192.168.1.20:8200`.
    ///
    /// The origin must carry a scheme and a host and must not carry a path:
    /// the paths of the documents are fixed, and a base that already carried
    /// one would make the advertised URLs ambiguous.
    pub fn new(raw: &str) -> Result<Self, UpnpError> {
        let trimmed = raw.trim().trim_end_matches('/');
        if trimmed.is_empty() {
            return Err(UpnpError::EmptyBaseUrl);
        }
        let rest = trimmed
            .strip_prefix("http://")
            .or_else(|| trimmed.strip_prefix("https://"))
            .ok_or_else(|| UpnpError::InvalidBaseUrl {
                value: raw.to_owned(),
            })?;
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        if authority != rest || !has_host(authority) {
            return Err(UpnpError::InvalidBaseUrl {
                value: raw.to_owned(),
            });
        }
        Ok(Self {
            origin: trimmed.to_owned(),
        })
    }

    /// The origin as it is written in the document.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// The absolute URL of a path on this origin.
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }
}

/// Whether an authority part carries a usable host and port.
///
/// The authority must not carry a user name: the origin is written into the
/// document verbatim, and a password in it would be published to every
/// client on the network.
fn has_host(authority: &str) -> bool {
    if authority.is_empty() || authority.contains(char::is_whitespace) || authority.contains('@') {
        return false;
    }
    let (host, port) = split_host_and_port(authority);
    if host.is_empty() {
        return false;
    }
    match port {
        None => true,
        Some(port) => !port.is_empty() && port.chars().all(|digit| digit.is_ascii_digit()),
    }
}

/// Splits an authority into its host and its optional port.
fn split_host_and_port(authority: &str) -> (&str, Option<&str>) {
    match authority.strip_prefix('[') {
        // An IPv6 literal: the host ends at the closing bracket.
        Some(rest) => match rest.find(']') {
            Some(index) => (
                &authority[..index + 1],
                authority[index + 2..].strip_prefix(':'),
            ),
            None => ("", None),
        },
        None => match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::BaseUrl;
    use crate::upnp::UpnpError;

    /// A base URL that is not an absolute HTTP URL with a host is refused.
    #[test]
    fn an_invalid_base_url_is_refused() {
        assert_eq!(
            BaseUrl::new(""),
            Err(UpnpError::EmptyBaseUrl),
            "an empty base URL must be refused"
        );
        assert!(BaseUrl::new("192.168.1.20:8200").is_err());
        assert!(BaseUrl::new("http://").is_err());
        assert!(BaseUrl::new("http://host/some/path").is_err());
    }

    /// A base URL with an IPv6 literal is accepted.
    #[test]
    fn an_ipv6_base_url_is_accepted() {
        let base = BaseUrl::new("http://[fe80::1]:8200").expect("a valid base URL");
        assert_eq!(base.origin(), "http://[fe80::1]:8200");
    }

    /// A trailing slash does not change the origin, because the paths of the
    /// documents are appended to it.
    #[test]
    fn a_trailing_slash_is_ignored() {
        let base = BaseUrl::new("http://192.168.1.20:8200/").expect("a valid base URL");
        assert_eq!(base.origin(), "http://192.168.1.20:8200");
        assert_eq!(base.url("/ctl/CD"), "http://192.168.1.20:8200/ctl/CD");
    }

    /// An origin that carries a user name is refused, because the origin is
    /// published in the document.
    #[test]
    fn an_origin_with_credentials_is_refused() {
        assert!(BaseUrl::new("http://user:secret@192.168.1.20:8200").is_err());
    }

    /// An origin whose port is not a number is refused.
    #[test]
    fn an_origin_with_a_broken_port_is_refused() {
        assert!(BaseUrl::new("http://192.168.1.20:notaport").is_err());
        assert!(BaseUrl::new("http://192.168.1.20:").is_err());
    }
}
