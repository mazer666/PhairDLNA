//! The metadata and explanation of every configuration setting.
//!
//! What: describes every setting of the configuration, its classification
//!       (Standard vs. Expert), a human-friendly label, a one-line summary,
//!       and an in-depth tooltip explaining how it works.
//! Why: ordinary users should never be confused by network protocols or queue
//!      depths, while power users need complete visibility and tuning (`UX-010`,
//!      `UX-011`).
//! Example: `SettingTier::Standard` holds `server.device_name`, while
//!          `SettingTier::Expert` holds `server.ssdp_port` and symlink policies.

use serde::{Deserialize, Serialize};

/// The tier of a configuration setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingTier {
    /// Essential settings every user understands (device name, roots, on/off).
    Standard,
    /// Advanced tuning parameters for network, performance, and cache limits.
    Expert,
}

impl SettingTier {
    /// The name of the tier as a text.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Expert => "expert",
        }
    }
}

/// The metadata of one configuration setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingDoc {
    /// The full key of the setting, for example `server.device_name`.
    pub key: String,
    /// The human-friendly label.
    pub label: String,
    /// The tier (Standard vs. Expert).
    pub tier: SettingTier,
    /// A single explanatory sentence.
    pub summary: String,
    /// Contextual help, details and best-practice tips.
    pub tooltip: String,
    /// The default value as a string representation.
    pub default_value: String,
}

/// Returns the complete registry of configuration settings documented for the user.
pub fn registry() -> Vec<SettingDoc> {
    let mut entries = standard_settings();
    entries.extend(expert_server_settings());
    entries.extend(expert_storage_settings());
    entries
}

/// The settings intended for everyday users.
fn standard_settings() -> Vec<SettingDoc> {
    vec![
        SettingDoc {
            key: "server.device_name".to_owned(),
            label: "Server Name".to_owned(),
            tier: SettingTier::Standard,
            summary: "The name shown on your TV, receiver, and media players.".to_owned(),
            tooltip: "This label identifies your server when browsing for DLNA devices on the local network.".to_owned(),
            default_value: "PhairDLNA".to_owned(),
        },
        SettingDoc {
            key: "library.roots".to_owned(),
            label: "Shared Media Folders".to_owned(),
            tier: SettingTier::Standard,
            summary: "The folders containing audio, video, photos, and playlists to share.".to_owned(),
            tooltip: "Each root can point to an internal or external drive. Removable drives are gracefully detected.".to_owned(),
            default_value: "[]".to_owned(),
        },
        SettingDoc {
            key: "library.extract_artwork".to_owned(),
            label: "Automatic Cover Art Extraction".to_owned(),
            tier: SettingTier::Standard,
            summary: "Finds and serves album artwork and movie covers automatically.".to_owned(),
            tooltip: "Extracts ID3 APIC, MP4 cover atoms, and sidecar images (cover.jpg/png) without transcoding.".to_owned(),
            default_value: "true".to_owned(),
        },
        SettingDoc {
            key: "security.admin_enabled".to_owned(),
            label: "Administration Interface".to_owned(),
            tier: SettingTier::Standard,
            summary: "Enables local web administration on loopback (127.0.0.1).".to_owned(),
            tooltip: "Allows local control point status checks, library rescans, and diagnostic exports via token.".to_owned(),
            default_value: "true".to_owned(),
        },
    ]
}

/// Advanced network and discovery settings.
fn expert_server_settings() -> Vec<SettingDoc> {
    vec![
        SettingDoc {
            key: "server.http_port".to_owned(),
            label: "HTTP Media Port".to_owned(),
            tier: SettingTier::Expert,
            summary: "TCP port on which the media streaming server listens.".to_owned(),
            tooltip: "Standard port is 8200. Change only if another application occupies this port.".to_owned(),
            default_value: "8200".to_owned(),
        },
        SettingDoc {
            key: "server.ssdp_port".to_owned(),
            label: "SSDP Discovery Port".to_owned(),
            tier: SettingTier::Expert,
            summary: "UDP port used for multicast discovery announcements.".to_owned(),
            tooltip: "Standard UPnP discovery uses port 1900. Moving this port isolates testing servers.".to_owned(),
            default_value: "1900".to_owned(),
        },
        SettingDoc {
            key: "server.discovery_enabled".to_owned(),
            label: "SSDP Discovery Broadcasts".to_owned(),
            tier: SettingTier::Expert,
            summary: "Controls whether SSDP NOTIFY multicast announcements are emitted.".to_owned(),
            tooltip: "When disabled, clients must know the server's device description URL directly.".to_owned(),
            default_value: "true".to_owned(),
        },
        SettingDoc {
            key: "server.symlink_policy".to_owned(),
            label: "Global Symlink Policy".to_owned(),
            tier: SettingTier::Expert,
            summary: "How symbolic links inside media folders are handled.".to_owned(),
            tooltip: "Skip (safest, default): ignore symlinks. Follow: resolve target if inside a root. IndexAsLink: index link itself.".to_owned(),
            default_value: "skip".to_owned(),
        },
    ]
}

/// Advanced database, cache, and logging tuning parameters.
fn expert_storage_settings() -> Vec<SettingDoc> {
    vec![
        SettingDoc {
            key: "library.cache_directory".to_owned(),
            label: "Artwork Cache Directory".to_owned(),
            tier: SettingTier::Expert,
            summary: "Path where scaled thumbnail images are cached.".to_owned(),
            tooltip: "Defaults to the user application cache. Can be wiped anytime without data loss.".to_owned(),
            default_value: "None (OS user cache)".to_owned(),
        },
        SettingDoc {
            key: "library.database_path".to_owned(),
            label: "Catalogue SQLite Database Path".to_owned(),
            tier: SettingTier::Expert,
            summary: "Path to the SQLite index database file.".to_owned(),
            tooltip: "Defaults to user application data. If corrupted, PhairDLNA auto-recovers and re-indexes.".to_owned(),
            default_value: "None (OS user data)".to_owned(),
        },
        SettingDoc {
            key: "library.scan_schedule".to_owned(),
            label: "Background Library Scan Schedule".to_owned(),
            tier: SettingTier::Expert,
            summary: "Controls automatic periodic rescans or manual-only scanning (FUN-024).".to_owned(),
            tooltip: "Configures interval in seconds (default 900) or manual_only so scans run only via explicit trigger.".to_owned(),
            default_value: "interval(900)".to_owned(),
        },
        SettingDoc {
            key: "logging.level".to_owned(),
            label: "Log Level".to_owned(),
            tier: SettingTier::Expert,
            summary: "Verbosity of diagnostic log output (debug, info, warn, error).".to_owned(),
            tooltip: "Info is recommended for normal operation; debug helps diagnose network client quirks.".to_owned(),
            default_value: "info".to_owned(),
        },
        SettingDoc {
            key: "logging.redact".to_owned(),
            label: "Path & Address Redaction".to_owned(),
            tier: SettingTier::Expert,
            summary: "Replaces real user paths and client IP addresses with pseudonyms in logs.".to_owned(),
            tooltip: "Keep enabled so exported logs never leak private folder structures or IP addresses.".to_owned(),
            default_value: "true".to_owned(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_and_expert_tiers_are_separated() {
        let all = registry();
        let standard: Vec<_> = all
            .iter()
            .filter(|s| s.tier == SettingTier::Standard)
            .collect();
        let expert: Vec<_> = all
            .iter()
            .filter(|s| s.tier == SettingTier::Expert)
            .collect();
        assert!(!standard.is_empty(), "must have standard settings");
        assert!(!expert.is_empty(), "must have expert settings");
        assert!(standard.iter().any(|s| s.key == "server.device_name"));
        assert!(expert.iter().any(|s| s.key == "server.symlink_policy"));
    }

    #[test]
    fn every_setting_has_label_summary_and_tooltip() {
        for setting in registry() {
            assert!(!setting.label.is_empty(), "{}: missing label", setting.key);
            assert!(
                !setting.summary.is_empty(),
                "{}: missing summary",
                setting.key
            );
            assert!(
                !setting.tooltip.is_empty(),
                "{}: missing tooltip",
                setting.key
            );
        }
    }
}
