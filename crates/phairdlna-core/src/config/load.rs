//! Reading, writing and migrating the configuration.
//!
//! What: the functions that turn a file into a validated `Config` and back.
//! Why: the configuration is the one thing every other part of the server
//!      depends on, so it is loaded once, validated once, and never read
//!      again from a second place.
//! Example: `Config::load(path)` returns the validated configuration or a
//!      `ConfigError` that says what to change.

use std::path::{Path, PathBuf};

use super::defaults::{Config, CURRENT_SCHEMA_VERSION};
use super::error::ConfigError;
use super::validate;

/// The environment variable that names the configuration file.
pub const CONFIG_PATH_ENV: &str = "PHAIRDLNA_CONFIG";

/// The configuration file name inside the user's configuration directory.
pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";

impl Config {
    /// Reads, parses, migrates and validates a configuration file.
    ///
    /// The order matters: the file is parsed first, then migrated, then
    /// validated. A file that cannot be parsed produces a parse error; a file
    /// that parses but is wrong produces a field error.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path)
            .map_err(|source| ConfigError::document(read_failure(path, &source)))?;
        let mut config: Self = toml::from_str(&text)
            .map_err(|error| ConfigError::document(parse_failure(path, &error)))?;
        config.migrate()?;
        validate::validate(&config)?;
        Ok(config)
    }

    /// Parses, migrates and validates a configuration from a string.
    ///
    /// Used by the tests and by the `config validate` command, which must be
    /// able to check a document that is not on disk yet.
    pub fn from_str(text: &str, origin: &str) -> Result<Self, ConfigError> {
        let mut config: Self = toml::from_str(text)
            .map_err(|error| ConfigError::document(parse_failure(Path::new(origin), &error)))?;
        config.migrate()?;
        validate::validate(&config)?;
        Ok(config)
    }

    /// Parses, migrates and validates a configuration from inline text.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        Self::from_str(text, "inline")
    }

    /// Writes the configuration, with its schema version.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = self.render()?;
        write_atomically(path, &text)
    }

    /// Renders the configuration as the TOML text that would be written.
    ///
    /// The format belongs to the core, so that every caller renders the same
    /// document the same way — a second serialiser is a second dialect.
    pub fn render(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|error| {
            ConfigError::document(format!("cannot serialise the configuration: {error}"))
        })
    }

    /// Brings the document up to the current schema version.
    ///
    /// A document from an older known version is migrated in memory and the
    /// migration is reported. A document from a **newer** version is refused:
    /// this build cannot know what the newer one means, and guessing would be
    /// worse than stopping.
    pub fn migrate(&mut self) -> Result<(), ConfigError> {
        if self.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(ConfigError::document(format!(
                "the configuration declares schema version {} but this build understands at \
                 most version {CURRENT_SCHEMA_VERSION}; upgrade PhairDLNA or use a matching \
                 configuration file",
                self.schema_version
            )));
        }
        if self.schema_version < CURRENT_SCHEMA_VERSION {
            self.schema_version = CURRENT_SCHEMA_VERSION;
        }
        Ok(())
    }

    /// Whether the document had to be migrated to be understood.
    pub fn needs_migration(&self) -> bool {
        self.schema_version < CURRENT_SCHEMA_VERSION
    }
}

/// The path of the configuration file.
///
/// The command line wins over the environment variable, which wins over the
/// default location in the user's configuration directory. `CFG-001` asks for
/// exactly one file, and this is how it is found.
pub fn resolve_path(cli_path: Option<&Path>) -> PathBuf {
    if let Some(path) = cli_path {
        return path.to_path_buf();
    }
    if let Some(raw) = std::env::var_os(CONFIG_PATH_ENV) {
        return PathBuf::from(raw);
    }
    default_path()
}

/// The default location of the configuration file.
pub fn default_path() -> PathBuf {
    super::paths::user_config_dir().join(DEFAULT_CONFIG_FILE_NAME)
}

/// The message for a file that cannot be read.
fn read_failure(path: &Path, source: &std::io::Error) -> String {
    format!(
        "cannot read the configuration file \"{}\": {source}",
        path.display()
    )
}

/// The message for a file that cannot be parsed.
fn parse_failure(path: &Path, error: &toml::de::Error) -> String {
    format!(
        "the configuration file \"{}\" cannot be parsed: {error}",
        path.display()
    )
}

/// Writes a file atomically: to a temporary file, then a rename.
///
/// A half-written configuration file is worse than no configuration file, so
/// the old file survives a failure (`REL-005`).
fn write_atomically(path: &Path, text: &str) -> Result<(), ConfigError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|source| {
        ConfigError::document(format!(
            "cannot create the directory \"{}\": {source}",
            parent.display()
        ))
    })?;
    let temporary = parent.join(format!(
        "{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    std::fs::write(&temporary, text).map_err(|source| {
        ConfigError::document(format!(
            "cannot write \"{}\": {source}",
            temporary.display()
        ))
    })?;
    std::fs::rename(&temporary, path).map_err(|source| {
        ConfigError::document(format!("cannot replace \"{}\": {source}", path.display()))
    })
}
