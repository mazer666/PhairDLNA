//! The optional administration surface of PhairDLNA.
//!
//! What: the crate that will host the loopback-only administration API.
//! Why: the surface is a separate crate so that the server can be built
//!      *without* it — the headless server must never depend on the presence
//!      of an administration interface (`PRD-006`).
//! Example: none yet.
//!
//! **Status: no route exists.** The crate exists so that the dependency
//! boundary is real from the first commit; the specification is F-14, the
//! milestone is M2, and nothing in this crate serves a request yet. A future
//! reader must not mistake the empty crate for a finished feature.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// The name of the crate, for logs and for the status output.
pub const SURFACE_NAME: &str = "phairdlna-admin";

/// Where the administration token for a given configuration lives.
///
/// The token is never inside the configuration file: it is a separate file
/// with owner-only permissions, so that copying the configuration around does
/// not copy the credential with it (`CFG-014`).
pub fn token_path(config: &phairdlna_core::Config) -> std::path::PathBuf {
    std::path::PathBuf::from(&config.security.admin_token_file)
}

#[cfg(test)]
mod tests {
    use super::{token_path, SURFACE_NAME};

    #[test]
    fn surface_is_named() {
        assert_eq!(SURFACE_NAME, "phairdlna-admin");
    }

    #[test]
    fn token_path_comes_from_the_configuration() {
        let config = phairdlna_core::Config::default();
        let path = token_path(&config);
        assert!(path.to_string_lossy().contains("admin.token"));
    }
}
