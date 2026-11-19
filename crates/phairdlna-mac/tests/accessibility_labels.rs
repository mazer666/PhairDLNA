//! Accessibility-name contract for values in the native overview.
//!
//! What: keep explicit localized names attached to dynamic status fields.
//! Why: VoiceOver needs a stable field name when the displayed value changes.
//! Example: the changing status value has the accessible name `Server status`.

const OVERVIEW_LAYOUT: &str = include_str!("../src/platform/main_window/layout.rs");

/// Dynamic overview values keep an explicit accessible name as their text changes.
#[test]
fn changing_overview_values_have_accessibility_names() {
    let expected_labels = [
        "overview.accessibility.status",
        "overview.accessibility.library",
        "overview.accessibility.folders",
        "overview.accessibility.connection",
        "overview.accessibility.activity",
        "overview.accessibility.notice",
    ];
    assert!(OVERVIEW_LAYOUT.contains("setAccessibilityLabel"));
    for label in expected_labels {
        assert!(
            OVERVIEW_LAYOUT.contains(label),
            "missing accessible name: {label}"
        );
    }
}
