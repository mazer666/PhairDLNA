//! Client compatibility profiles and quirk mitigation (INT-100 .. INT-104, F-07).
//!
//! What: declarative records of known client quirks, matching against request
//!       signals (e.g. User-Agent substring), and an evidence gate that refuses
//!       any profile without recorded evidence or with unknown adjustment types.
//! Why: INT-100 requires declarative profiles with symptoms and evidence;
//!      INT-101 forbids workarounds violating normative specifications for others;
//!      INT-104 allows disabling workarounds globally via configuration.

use std::collections::HashSet;

/// Known adjustments to output shaping that a profile may request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum QuirkAdjustment {
    /// Omit DLNA 4th protocolInfo field parameter for clients expecting 3-field headers.
    CompactProtocolInfo,
    /// Force uppercase HTTP method or header matching.
    RelaxedHeaderCasing,
    /// Provide single res element even when subtitles exist.
    SuppressSubtitleRes,
}

impl QuirkAdjustment {
    /// Returns the stable identifier of the adjustment.
    pub fn name(self) -> &'static str {
        match self {
            Self::CompactProtocolInfo => "compact_protocol_info",
            Self::RelaxedHeaderCasing => "relaxed_header_casing",
            Self::SuppressSubtitleRes => "suppress_subtitle_res",
        }
    }
}

/// A declarative quirk mitigation profile for a client family.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientProfile {
    /// Stable lowercase hyphenated profile identifier.
    pub id: String,
    /// Name of client family (e.g. "legacy-smart-tv").
    pub client_family: String,
    /// Substring to match in client User-Agent header.
    pub user_agent_substring: String,
    /// Observable symptom observed on the client.
    pub symptom: String,
    /// Output shaping adjustment.
    pub adjustment: QuirkAdjustment,
    /// Pointer to interoperability matrix evidence (mandatory under INT-100).
    pub evidence: String,
    /// Requirements touched by this adjustment (e.g. "INT-050").
    pub touches: Vec<String>,
}

impl ClientProfile {
    /// Validates the profile against the evidence gate (INT-100, INT-101).
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("profile id cannot be empty".to_owned());
        }
        if self.evidence.trim().is_empty() {
            return Err(format!(
                "profile \"{}\" is missing mandatory evidence pointer (INT-100)",
                self.id
            ));
        }
        if self.touches.is_empty() {
            return Err(format!(
                "profile \"{}\" must declare touched requirements (INT-100)",
                self.id
            ));
        }
        Ok(())
    }

    /// Evaluates whether a request with the given User-Agent matches this profile.
    pub fn matches(&self, user_agent: Option<&str>) -> bool {
        let Some(ua) = user_agent else {
            return false;
        };
        ua.contains(&self.user_agent_substring)
    }
}

/// List of built-in known device profile identifiers supported by the server.
pub const KNOWN_PROFILE_IDS: &[&str] = &[
    "legacy-renderer",
    "smart-tv-2018",
    "quirky-client",
    "sample-tv",
];

/// Returns true if the profile id is a known built-in device profile.
pub fn is_known_profile(id: &str) -> bool {
    KNOWN_PROFILE_IDS.contains(&id)
}

/// Profile registry and matcher.
#[derive(Debug, Clone, Default)]
pub struct ProfileStore {
    profiles: Vec<ClientProfile>,
    disabled: HashSet<String>,
}

impl ProfileStore {
    /// Creates a new profile store with default approved profiles.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            disabled: HashSet::new(),
        }
    }

    /// Adds a profile to the store, enforcing the evidence gate (INT-100).
    pub fn register(&mut self, profile: ClientProfile) -> Result<(), String> {
        profile.validate()?;
        if self.profiles.iter().any(|p| p.id == profile.id) {
            return Err(format!("duplicate profile id \"{}\"", profile.id));
        }
        self.profiles.push(profile);
        Ok(())
    }

    /// Disables a list of profile IDs (INT-104).
    pub fn disable(&mut self, ids: &[String]) {
        for id in ids {
            self.disabled.insert(id.clone());
        }
    }

    /// Finds the active matching profile for a request's User-Agent.
    /// Disabled profiles are skipped (INT-104).
    pub fn match_client(&self, user_agent: Option<&str>) -> Option<&ClientProfile> {
        for profile in &self.profiles {
            if self.disabled.contains(&profile.id) {
                continue;
            }
            if profile.matches(user_agent) {
                return Some(profile);
            }
        }
        None
    }

    /// Returns all registered profiles.
    pub fn all(&self) -> &[ClientProfile] {
        &self.profiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile(id: &str, ua: &str, evidence: &str) -> ClientProfile {
        ClientProfile {
            id: id.to_owned(),
            client_family: "Test TV".to_owned(),
            user_agent_substring: ua.to_owned(),
            symptom: "Fails playback without 3-field header".to_owned(),
            adjustment: QuirkAdjustment::CompactProtocolInfo,
            evidence: evidence.to_owned(),
            touches: vec!["INT-050".to_owned()],
        }
    }

    #[test]
    fn profile_without_evidence_is_refused() {
        let bad = sample_profile("bad-tv", "BadTV/1.0", "");
        let err = bad.validate().expect_err("must fail");
        assert!(err.contains("missing mandatory evidence"), "{err}");
    }

    #[test]
    fn profile_matches_matching_user_agent() {
        let mut store = ProfileStore::new();
        store
            .register(sample_profile(
                "sample-tv",
                "SampleTV/2.0",
                "qa/device-matrix.toml#row-12",
            ))
            .expect("register");

        let matched = store.match_client(Some("Mozilla/5.0 (SampleTV/2.0)"));
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "sample-tv");

        let non_matched = store.match_client(Some("VLC/3.0.0"));
        assert!(non_matched.is_none());
    }

    #[test]
    fn disabled_profile_restores_plain_behaviour() {
        let mut store = ProfileStore::new();
        store
            .register(sample_profile(
                "sample-tv",
                "SampleTV/2.0",
                "qa/device-matrix.toml#row-12",
            ))
            .expect("register");

        store.disable(&["sample-tv".to_owned()]);
        let matched = store.match_client(Some("Mozilla/5.0 (SampleTV/2.0)"));
        assert!(
            matched.is_none(),
            "disabled profile must be skipped (INT-104)"
        );
    }
}
