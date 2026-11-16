//! Where the files of the service of the server live.
//!
//! What: the folders of the job file, of the logs and of the data of the
//!       installation, for every platform and every scope.
//! Why: two modules that each guess where a service file belongs end up with
//!      two service files; the paths are decided once, here (`F-13` § 4).
//! Example: on macOS the job file of a per-user service belongs into
//!      `~/Library/LaunchAgents`.
//!
//! **The two scopes.** A *user* service belongs to the person who installed
//! it and needs no privilege; a *system* service belongs to the machine and
//! needs one. The user scope is the default, because the server never needs to
//! be root to serve the media of its owner (`OPS-015`).

use std::path::{Path, PathBuf};

use phairdlna_core::config::{user_config_dir, user_data_dir};

/// The label of the job of the server.
pub const LABEL: &str = "org.phairdlna.server";

/// The name of the unit file of the server on Linux.
pub const UNIT_NAME: &str = "phairdlna.service";

/// The folder the job file of the service belongs into.
pub fn unit_directory(platform: super::Platform, scope: super::Scope, home: &Path) -> PathBuf {
    use super::{Platform, Scope};
    match (platform, scope) {
        (Platform::MacOs, Scope::User) => home.join("Library").join("LaunchAgents"),
        (Platform::MacOs, Scope::System) => PathBuf::from("/Library/LaunchDaemons"),
        (Platform::Linux, Scope::User) => home.join(".config").join("systemd").join("user"),
        (Platform::Linux, Scope::System) => PathBuf::from("/etc/systemd/system"),
    }
}

/// The file that describes the service.
pub fn unit_path(platform: super::Platform, scope: super::Scope, home: &Path) -> PathBuf {
    let directory = unit_directory(platform, scope, home);
    match platform {
        super::Platform::MacOs => directory.join(format!("{LABEL}.plist")),
        super::Platform::Linux => directory.join(UNIT_NAME),
    }
}

/// The folder the logs of the service go to.
pub fn log_directory(platform: super::Platform, scope: super::Scope, home: &Path) -> PathBuf {
    use super::{Platform, Scope};
    match (platform, scope) {
        (Platform::MacOs, Scope::User) => home.join("Library").join("Logs").join("PhairDLNA"),
        (Platform::MacOs, Scope::System) => PathBuf::from("/var/log/phairdlna"),
        (Platform::Linux, Scope::User) => user_data_dir().join("logs"),
        (Platform::Linux, Scope::System) => PathBuf::from("/var/log/phairdlna"),
    }
}

/// The environment the server is given.
///
/// The environment is small on purpose: a service that inherits everything of a
/// shell inherits secrets as well (`SEC-090`).
pub fn environment(home: &Path) -> Vec<(String, String)> {
    vec![
        (
            "PATH".to_owned(),
            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin:/usr/sbin:/sbin".to_owned()),
        ),
        ("HOME".to_owned(), home.to_string_lossy().into_owned()),
    ]
}

/// The name of the user the service runs as.
///
/// The name comes from the environment of the shell; when the environment says
/// nothing, the name of the home directory is the best guess that exists.
pub fn user_name(home: &Path) -> String {
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            return user;
        }
    }
    home.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "phairdlna".to_owned())
}

/// The configuration file the server reads by default.
pub fn default_config_path() -> PathBuf {
    user_config_dir().join(phairdlna_core::config::DEFAULT_CONFIG_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::{default_config_path, environment, unit_path, LABEL, UNIT_NAME};
    use crate::service::{Platform, Scope};
    use std::path::{Path, PathBuf};

    /// The job file of macOS lies in the folder of the user.
    #[test]
    fn the_macos_job_file_lies_in_the_folder_of_the_user() {
        let path = unit_path(Platform::MacOs, Scope::User, Path::new("/Users/tester"));

        assert_eq!(
            path,
            PathBuf::from("/Users/tester/Library/LaunchAgents/org.phairdlna.server.plist")
        );
    }

    /// The job file of the system scope lies in the system folders.
    #[test]
    fn the_system_scope_lies_in_the_system_folders() {
        let path = unit_path(Platform::MacOs, Scope::System, Path::new("/Users/tester"));

        assert_eq!(
            path,
            PathBuf::from("/Library/LaunchDaemons/org.phairdlna.server.plist")
        );
    }

    /// The unit file of Linux bears the name of the service.
    #[test]
    fn the_linux_unit_bears_the_name_of_the_service() {
        let path = unit_path(Platform::Linux, Scope::User, Path::new("/home/tester"));

        assert!(path.ends_with(UNIT_NAME), "{path:?}");
        assert!(
            path.starts_with("/home/tester/.config/systemd/user"),
            "{path:?}"
        );
    }

    /// The environment of the service carries the home of the user and the
    /// path of the programs, and nothing else.
    #[test]
    fn the_environment_is_small() {
        let environment = environment(Path::new("/Users/tester"));

        assert_eq!(environment.len(), 2);
        assert!(environment.contains(&("HOME".to_owned(), "/Users/tester".to_owned())));
        assert!(environment.iter().any(|(name, _)| name == "PATH"));
    }

    /// The label of the job is the one the whole project uses.
    #[test]
    fn the_label_is_the_one_of_the_project() {
        assert_eq!(LABEL, "org.phairdlna.server");
    }

    /// The default configuration lives in the configuration folder of the user.
    #[test]
    fn the_default_configuration_is_in_the_folder_of_the_user() {
        let path = default_config_path();

        assert!(path.to_string_lossy().ends_with(".toml"), "{path:?}");
        assert!(path.to_string_lossy().contains("phairdlna"), "{path:?}");
    }
}
