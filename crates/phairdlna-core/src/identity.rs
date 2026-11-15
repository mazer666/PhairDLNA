//! The persistent identity of the server.
//!
//! What: the unique device name the server announces, and where it is kept.
//! Why: the unique device name must be **stable across restarts** — a client that
//!       remembers the device must find it again (INT-014). It is also the
//!       prefix of every `USN`, so it must never be invented per request.
//! Example: the identity file holds one UUID; the second start reads it.

use std::path::{Path, PathBuf};

use crate::config::user_config_dir;
use crate::ssdp::DeviceIdentity;

/// The name of the file that holds the identity.
pub const IDENTITY_FILE_NAME: &str = "identity";

/// The product name a client sees in the `SERVER` header.
pub const PRODUCT_NAME: &str = "PhairDLNA";

/// The identity of this installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledIdentity {
    /// The unique device name, without the `uuid:` prefix.
    pub udn: String,
    /// The name a client shows.
    pub friendly_name: String,
}

impl InstalledIdentity {
    /// The identity as the discovery layer uses it.
    pub fn as_device_identity(&self) -> DeviceIdentity {
        DeviceIdentity::new(self.udn.clone(), self.friendly_name.clone())
    }
}

/// The path of the identity file.
pub fn identity_path() -> PathBuf {
    user_config_dir().join(IDENTITY_FILE_NAME)
}

/// Reads the identity, creating it if it does not exist yet.
///
/// Creating the file is a deliberate act: the first start writes a new unique
/// device name, and every later start reads the same one back. The file is
/// written with owner-only permissions, because a device identity that another
/// local user could rewrite would let them impersonate this server.
pub fn load_or_create(path: &Path, friendly_name: &str) -> std::io::Result<InstalledIdentity> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let udn = text.trim().to_owned();
            if udn.is_empty() {
                write_new(path, friendly_name)
            } else {
                Ok(InstalledIdentity {
                    udn,
                    friendly_name: friendly_name.to_owned(),
                })
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_new(path, friendly_name)
        }
        Err(error) => Err(error),
    }
}

/// Writes a new identity.
fn write_new(path: &Path, friendly_name: &str) -> std::io::Result<InstalledIdentity> {
    let identity = InstalledIdentity {
        udn: uuid::Uuid::new_v4().to_string(),
        friendly_name: friendly_name.to_owned(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", identity.udn))?;
    restrict_to_owner(path)?;
    Ok(identity)
}

/// Restricts a file to its owner.
///
/// A device identity that another local user could rewrite would let them
/// impersonate this server on the network.
fn restrict_to_owner(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(path, permissions)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_or_create, InstalledIdentity};
    use crate::ssdp::DeviceIdentity;

    /// The first start creates the identity, the second start reads it back.
    #[test]
    fn the_identity_survives_a_restart() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let path = directory.path().join("identity");

        let first = load_or_create(&path, "Living room").expect("the identity should be created");
        let second = load_or_create(&path, "Living room").expect("the identity should be read");

        assert_eq!(first, second);
        assert_eq!(first.udn.len(), 36, "a UUID has 36 characters");
    }

    /// Two installations get different identities.
    #[test]
    fn two_installations_differ() {
        let first = tempfile::tempdir().expect("a temporary directory");
        let second = tempfile::tempdir().expect("a temporary directory");
        let one = load_or_create(&first.path().join("identity"), "a").expect("created");
        let two = load_or_create(&second.path().join("identity"), "b").expect("created");
        assert_ne!(one.udn, two.udn);
    }

    /// The identity file is readable only by its owner.
    #[cfg(unix)]
    #[test]
    fn the_identity_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().expect("a temporary directory");
        let path = directory.path().join("identity");
        load_or_create(&path, "Living room").expect("the identity should be created");
        let mode = std::fs::metadata(&path)
            .expect("the file should exist")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    /// The identity converts into the form the discovery layer uses.
    #[test]
    fn the_identity_carries_the_uuid_prefix() {
        let installed = InstalledIdentity {
            udn: "11111111-2222-3333-4444-555555555555".to_owned(),
            friendly_name: "Living room".to_owned(),
        };
        let device: DeviceIdentity = installed.as_device_identity();
        assert_eq!(device.uuid(), "uuid:11111111-2222-3333-4444-555555555555");
    }
}
