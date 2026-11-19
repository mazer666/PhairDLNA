//! The token of the administration surface.
//!
//! What: the secret that a request to the administration surface must carry,
//!       and the file that holds it.
//! Why: the surface is on the loopback interface, and the loopback interface
//!       is shared with every program of the machine. The token is what tells
//!       the server that the request comes from the owner of the server and
//!       not from another program of the same machine (`OPS-020`, `SEC-071`).
//! Example: a request that carries `Authorization: Bearer <token>` passes the
//!          check; a request without it is refused with `401`.
//!
//! **The file.** The token is written to a file the owner alone may read. The
//! file is created with the permissions `600`; a file that is readable by
//! others is refused, because a token that another program can read is no
//! token at all.

use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

mod error;
pub use error::TokenError;

/// The permissions the token file must have: the owner reads and writes, and
/// nobody else may do anything with it.
const PRIVATE_FILE: u32 = 0o600;

/// The permissions the folder of the token file must have.
const PRIVATE_FOLDER: u32 = 0o700;

/// The context the "cross-site request forgery" token is derived in.
///
/// The context is a constant of this program: the value is only good for the
/// administration surface, and not for anything else.
const CSRF_CONTEXT: &[u8] = b"phairdlna-admin-csrf-v1";

/// The number of characters of the token.
///
/// The token is two random identifiers of 128 bits each, written as hexadecimal
/// characters. 256 bits of randomness cannot be guessed in the lifetime of a
/// machine.
const TOKEN_CHARACTERS: usize = 64;

/// The secret of the administration surface.
#[derive(Clone, PartialEq, Eq)]
pub struct AdminToken {
    /// The secret itself, as hexadecimal characters.
    secret: String,
}

impl AdminToken {
    /// Builds a token out of fresh randomness.
    pub fn generate() -> Self {
        let first = uuid::Uuid::new_v4().simple();
        let second = uuid::Uuid::new_v4().simple();
        let secret = format!("{first}{second}");
        Self { secret }
    }

    /// Reads a token out of a text.
    ///
    /// A text that is too short is refused: a short secret is a secret that is
    /// found by trying.
    pub fn parse(text: &str) -> Result<Self, TokenError> {
        let secret = text.trim().to_owned();
        if secret.len() != TOKEN_CHARACTERS {
            return Err(TokenError::Malformed);
        }
        if !secret
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        {
            return Err(TokenError::Malformed);
        }
        Ok(Self { secret })
    }

    /// The token as a text, for the file and for the header.
    pub fn as_str(&self) -> &str {
        &self.secret
    }

    /// The token of the "cross-site request forgery" defence.
    ///
    /// The value is bound to the session: it is derived from the token of the
    /// installation, so a client that knows the token knows the value, and a
    /// client that does not know it cannot compute it. A page of another origin
    /// cannot read the value, because the surface answers no question of
    /// another origin and carries no header that would let it (`OPS-023`).
    pub fn csrf_token(&self) -> String {
        let digest = blake3::keyed_hash(&self.secret_bytes(), CSRF_CONTEXT);
        digest.to_hex().to_string()
    }

    /// The bytes of the secret, read out of its hexadecimal form.
    ///
    /// The secret is 32 bytes of randomness, written as 64 hexadecimal
    /// characters.
    fn secret_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let high = hex_value(self.secret.as_bytes()[index * 2]);
            let low = hex_value(self.secret.as_bytes()[index * 2 + 1]);
            *byte = high * 16 + low;
        }
        bytes
    }

    /// Whether a token that a client presented is the right one.
    ///
    /// The comparison reads every character of both tokens, whatever the
    /// answer is: a comparison that stops at the first difference tells an
    /// attacker how much of the token he guessed right.
    pub fn matches(&self, presented: &str) -> bool {
        constant_time_eq(self.secret.as_bytes(), presented.trim().as_bytes())
    }
}

impl fmt::Debug for AdminToken {
    /// The token never appears in a log or in a panic message.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AdminToken(<secret>)")
    }
}

/// The value of a hexadecimal character.
fn hex_value(character: u8) -> u8 {
    match character {
        b'0'..=b'9' => character - b'0',
        b'a'..=b'f' => character - b'a' + 10,
        b'A'..=b'F' => character - b'A' + 10,
        _ => 0,
    }
}

/// Compares two texts without telling how much of them is equal.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let length = left.len().max(right.len());
    for index in 0..length {
        let one = left.get(index).copied().unwrap_or_default();
        let other = right.get(index).copied().unwrap_or_default();
        difference |= usize::from(one ^ other);
    }
    difference == 0
}

/// Reads the token of the surface, and writes it when it is not there yet.
///
/// The file is created with the permissions `600`, and the folder above it
/// with `700`. A file that exists but is readable by others is refused: the
/// server does not start with a secret that another program of the machine can
/// read.
pub fn load_or_create(path: &Path) -> Result<AdminToken, TokenError> {
    if path.exists() {
        return load(path);
    }
    create(path)
}

/// Reads a token out of a file that exists.
fn load(path: &Path) -> Result<AdminToken, TokenError> {
    check_private(path)?;
    let text = std::fs::read_to_string(path).map_err(|error| TokenError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    AdminToken::parse(&text)
}

/// Writes a new token into a file that does not exist yet.
fn create(path: &Path) -> Result<AdminToken, TokenError> {
    let folder = parent_of(path);
    prepare_folder(&folder)?;
    let token = AdminToken::generate();
    let mut file = create_private(path).map_err(|reason| TokenError::Unwritable {
        path: path.to_path_buf(),
        reason,
    })?;
    file.write_all(token.as_str().as_bytes())
        .map_err(|error| TokenError::Unwritable {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    Ok(token)
}

/// Creates a file that only the owner may read.
#[cfg(unix)]
fn create_private(path: &Path) -> Result<std::fs::File, String> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE)
        .open(path)
        .map_err(|error| error.to_string())
}

/// Creates a file. The systems without file permissions get the file as it is.
#[cfg(not(unix))]
fn create_private(path: &Path) -> Result<std::fs::File, String> {
    let _ = PRIVATE_FILE;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())
}

/// The folder that holds the token file.
fn parent_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Makes sure the folder of the token file exists and is private.
fn prepare_folder(folder: &Path) -> Result<(), TokenError> {
    if !folder.exists() {
        std::fs::create_dir_all(folder).map_err(|error| TokenError::Unwritable {
            path: folder.to_path_buf(),
            reason: error.to_string(),
        })?;
    }
    set_private(folder, PRIVATE_FOLDER).map_err(|error| TokenError::Unwritable {
        path: folder.to_path_buf(),
        reason: error,
    })
}

/// Refuses a token file that another program of the machine may read.
#[cfg(unix)]
fn check_private(path: &Path) -> Result<(), TokenError> {
    let metadata = std::fs::metadata(path).map_err(|error| TokenError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let mode = std::os::unix::fs::PermissionsExt::mode(&metadata.permissions());
    if mode & 0o077 != 0 {
        return Err(TokenError::NotPrivate {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

/// The rule of the private file only exists on the systems that have file
/// permissions; elsewhere the token file is written and read as it is.
#[cfg(not(unix))]
fn check_private(_path: &Path) -> Result<(), TokenError> {
    Ok(())
}

/// Gives a file or a folder the permissions `600` respectively `700`.
#[cfg(unix)]
fn set_private(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

/// The rule of the private file only exists on the systems that have file
/// permissions.
#[cfg(not(unix))]
fn set_private(path: &Path, mode: u32) -> Result<(), String> {
    let _ = (path, mode);
    Ok(())
}

#[cfg(test)]
#[path = "token/tests.rs"]
mod tests;
