//! The portable filesystem adapter of the core.
//!
//! What: a `FileSystem` implementation that answers from the standard library,
//!       so that the catalogue can scan a directory on a platform that has no
//!       adapter of its own yet.
//! Why: the core must be able to read a directory before the platform layer of
//!       a distribution is finished, and it must do so **honestly**: where the
//!       platform offers no fact, the adapter says what it assumed and why
//!       (ADR-0003, `PLT-003`). A guessed fact that is written down is a fact
//!       that can be replaced; a guessed fact that is hidden is a bug waiting
//!       for a client to find it.
//! Example: `LocalFileSystem.volume_of` answers the constant volume identity
//!       `local`, because no adapter has yet reported a volume that survives a
//!       reboot — and an object identifier must survive a reboot (`INT-062`).
//!
//! **The two approximations of this adapter.**
//!
//! - The **volume identity** is the constant `local`. The consequence is that
//!   the object identifiers of a library do not change when the volume is
//!   remounted or the library is moved to another machine, which is what
//!   `INT-062` asks for. The cost is that two roots on two different volumes
//!   share a volume identity; the roots are told apart by their own key
//!   instead (see `catalogue::identifier`).
//! - The **case sensitivity** is a property of the platform, not of the
//!   volume: a Unix volume is assumed to be case-sensitive and a Windows volume
//!   case-insensitive. A macOS volume may be either, and the adapter says
//!   "sensitive", which is the safe direction: folding names that the volume
//!   keeps apart would merge two files into one, while keeping names apart
//!   that the volume treats as one only splits a file in two (`PLT-005`).
//!
//! **The third answer of this adapter** is the fingerprint of a path: the
//! device the path lives on. It is not a volume identity and it does not
//! pretend to be one — it says only that *something* at that path is the same
//! as it was, or that it is not.

use std::path::Path;

use super::path::{
    display_name, identity_of_with, CaseSensitivity, FileId, MediaPath, VolumeFingerprint, VolumeId,
};
use super::traits::{FileSystem, PlatformError, PlatformResult};

/// The volume identity the portable adapter reports.
///
/// It is a constant, and it is deliberately so: an identifier that changed
/// every time a volume was remounted would break every client that cached it
/// (`INT-062`).
pub const LOCAL_VOLUME: &str = "local";

/// The filesystem of the platform the server runs on.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFileSystem;

impl LocalFileSystem {
    /// The case sensitivity of the platform.
    ///
    /// A Unix filesystem distinguishes `Film.mkv` from `film.mkv` unless it was
    /// created otherwise; a Windows filesystem does not.
    fn platform_case_sensitivity() -> CaseSensitivity {
        if cfg!(windows) {
            CaseSensitivity::Insensitive
        } else {
            CaseSensitivity::Sensitive
        }
    }
}

impl FileSystem for LocalFileSystem {
    fn media_path(&self, path: &Path) -> PlatformResult<MediaPath> {
        let sensitivity = Self::platform_case_sensitivity();
        Ok(MediaPath::new(
            path.to_path_buf(),
            identity_of_with(path, sensitivity),
            VolumeId::new(LOCAL_VOLUME),
            display_name(path),
        ))
    }

    fn volume_of(&self, _path: &Path) -> PlatformResult<VolumeId> {
        Ok(VolumeId::new(LOCAL_VOLUME))
    }

    fn case_sensitivity(&self, _volume: &VolumeId) -> CaseSensitivity {
        Self::platform_case_sensitivity()
    }

    fn file_identity(&self, path: &Path) -> PlatformResult<FileId> {
        let metadata =
            std::fs::metadata(path).map_err(|source| PlatformError::io("stat a file", source))?;
        Ok(FileId {
            volume: VolumeId::new(LOCAL_VOLUME),
            device: device_of(&metadata),
            inode: inode_of(&metadata),
        })
    }

    /// The device the path lives on.
    ///
    /// The fingerprint is what tells the catalogue that the drive behind a root
    /// is no longer the drive it saw last time: when a volume is unmounted,
    /// its place on the disk is taken by a folder of the system volume, and the
    /// device of the path changes (`FUN-007`, `FUN-017`).
    ///
    /// The consequence is written down here: a drive that is plugged in again
    /// may report another device number, and the catalogue then reads the root
    /// again and says so. Whether it is the same volume is a question only a
    /// platform with a real volume identity can answer (`PLT-005`, `F-13`).
    fn volume_fingerprint(&self, path: &Path) -> PlatformResult<VolumeFingerprint> {
        #[cfg(unix)]
        {
            let metadata = std::fs::metadata(path)
                .map_err(|source| PlatformError::io("stat a path", source))?;
            Ok(VolumeFingerprint::new(format!(
                "dev:{}",
                device_of(&metadata)
            )))
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            Err(PlatformError::unsupported("the volume of a path"))
        }
    }
}

/// The device the file lives on.
#[cfg(unix)]
fn device_of(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.dev()
}

/// The device the file lives on.
#[cfg(not(unix))]
fn device_of(_metadata: &std::fs::Metadata) -> u64 {
    0
}

/// The number the platform gives the file within its device.
#[cfg(unix)]
fn inode_of(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.ino()
}

/// The number the platform gives the file within its device.
#[cfg(not(unix))]
fn inode_of(_metadata: &std::fs::Metadata) -> u64 {
    0
}

#[cfg(test)]
mod tests {
    use super::{LocalFileSystem, LOCAL_VOLUME};
    use crate::platform::path::CaseSensitivity;
    use crate::platform::traits::FileSystem;
    use std::path::Path;

    /// The volume identity is the same for every path, and it is a constant.
    #[test]
    fn the_volume_identity_is_a_constant() {
        let filesystem = LocalFileSystem;
        let first = filesystem
            .volume_of(Path::new("/tmp"))
            .expect("the adapter answers");
        let second = filesystem
            .volume_of(Path::new("/var"))
            .expect("the adapter answers");
        assert_eq!(first, second);
        assert_eq!(first.as_str(), LOCAL_VOLUME);
    }

    /// The case sensitivity of the platform is the one the platform has.
    #[test]
    fn the_case_sensitivity_is_the_one_of_the_platform() {
        let filesystem = LocalFileSystem;
        let sensitivity = filesystem.case_sensitivity(
            &LocalFileSystem
                .volume_of(Path::new("/tmp"))
                .expect("the adapter answers"),
        );
        assert_eq!(sensitivity, LocalFileSystem::platform_case_sensitivity());
        if cfg!(windows) {
            assert_eq!(sensitivity, CaseSensitivity::Insensitive);
        } else {
            assert_eq!(sensitivity, CaseSensitivity::Sensitive);
        }
    }

    /// A hidden name is hidden on every platform that has no flag of its own.
    #[test]
    fn a_hidden_name_is_hidden() {
        let filesystem = LocalFileSystem;
        assert!(filesystem.is_hidden(Path::new("/tmp/.hidden.mkv")));
        assert!(!filesystem.is_hidden(Path::new("/tmp/visible.mkv")));
    }

    /// The path model keeps the raw path and derives the identity from it.
    #[test]
    fn the_path_model_keeps_the_raw_path() {
        let filesystem = LocalFileSystem;
        let path = filesystem
            .media_path(Path::new("/tmp/Film.mkv"))
            .expect("the adapter answers");
        assert_eq!(path.raw(), Path::new("/tmp/Film.mkv"));
        assert_eq!(path.display(), "Film.mkv");
        assert_eq!(path.volume().as_str(), LOCAL_VOLUME);
    }
}
