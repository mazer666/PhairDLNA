//! The explanation of the configuration settings for the user.
//!
//! What: prints the documented settings, grouped by tier (Standard vs. Expert),
//!       including labels, summaries, default values, and tooltips.
//! Why: allows users to understand and tune settings without getting lost in
//!      complex parameters or raw configuration files (`UX-010`, `UX-011`).
//! Example: `phairdlna config explain` lists standard everyday settings;
//!          `phairdlna config explain --expert` lists everything including tuning.

use phairdlna_core::config::schema::{registry, SettingDoc, SettingTier};
use phairdlna_core::json::Json;

use crate::Outcome;

/// Explains the configuration settings to the user.
pub fn explain(expert: bool, as_json: bool) -> Outcome {
    let settings: Vec<SettingDoc> = registry()
        .into_iter()
        .filter(|s| expert || s.tier == SettingTier::Standard)
        .collect();

    if as_json {
        let json_entries: Vec<Json> = settings.into_iter().map(json_of_setting).collect();
        let document = Json::list(json_entries);
        Outcome::ok(format!("{document}\n"))
    } else {
        Outcome::ok(render_text(&settings, expert))
    }
}

/// Renders the settings as human-readable plain text.
fn render_text(settings: &[SettingDoc], expert: bool) -> String {
    let mut out = String::new();
    out.push_str("PhairDLNA Configuration Settings\n");
    out.push_str("================================\n\n");
    if expert {
        out.push_str("Showing all settings (Standard & Expert modes).\n\n");
    } else {
        out.push_str("Showing Standard settings. For advanced tuning, use:\n");
        out.push_str("  phairdlna config explain --expert\n\n");
    }

    for s in settings {
        out.push_str(&format!("• {} ({})\n", s.label, s.key));
        out.push_str(&format!("  Tier:    {}\n", s.tier.as_str()));
        out.push_str(&format!("  Default: {}\n", s.default_value));
        out.push_str(&format!("  Summary: {}\n", s.summary));
        out.push_str(&format!("  Help:    {}\n\n", s.tooltip));
    }
    out
}

/// Converts a `SettingDoc` into a JSON object.
fn json_of_setting(s: SettingDoc) -> Json {
    Json::map(vec![
        ("key".to_owned(), Json::text(s.key)),
        ("label".to_owned(), Json::text(s.label)),
        ("tier".to_owned(), Json::text(s.tier.as_str())),
        ("summary".to_owned(), Json::text(s.summary)),
        ("tooltip".to_owned(), Json::text(s.tooltip)),
        ("default_value".to_owned(), Json::text(s.default_value)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_mode_shows_only_standard_settings() {
        let outcome = explain(false, false);
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.contains("Server Name"));
        assert!(outcome.stdout.contains("server.device_name"));
        assert!(!outcome.stdout.contains("SSDP Discovery Port"));
    }

    #[test]
    fn expert_mode_shows_expert_settings() {
        let outcome = explain(true, false);
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.contains("Server Name"));
        assert!(outcome.stdout.contains("SSDP Discovery Port"));
        assert!(outcome.stdout.contains("server.ssdp_port"));
    }

    #[test]
    fn json_mode_emits_valid_json() {
        let outcome = explain(false, true);
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.starts_with('['));
        assert!(outcome.stdout.contains("\"key\":\"server.device_name\""));
    }
}
