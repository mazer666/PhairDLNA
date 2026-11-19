//! Media-root controls and conversion for native preferences.
//!
//! What: builds the read-only folder list shown in native preferences.
//! Why: OD-16 keeps server-side changes read-only until persistence and activation are settled.
//! Example: users can select and copy a path without changing the server configuration.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAccessibility, NSScrollView, NSTextView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use phairdlna_core::config::Config;

pub(super) fn build_roots_view(
    mtm: MainThreadMarker,
    config: &Config,
    config_available: bool,
    strings: &crate::text::Strings,
) -> (Retained<NSScrollView>, Retained<NSTextView>) {
    let roots_scroll = NSScrollView::new(mtm);
    roots_scroll.setFrame(NSRect::new(
        NSPoint::new(20.0, 20.0),
        NSSize::new(595.0, 275.0),
    ));
    roots_scroll.setHasVerticalScroller(true);
    roots_scroll.setHasHorizontalScroller(false);
    let roots_view = NSTextView::new(mtm);
    roots_view.setEditable(false);
    roots_view.setAccessibilityLabel(Some(&NSString::from_str(
        strings.text("prefs.roots_header"),
    )));
    let mut roots_text = String::new();
    if config_available {
        for root in &config.library.roots {
            roots_text.push_str(&root.path);
            roots_text.push('\n');
        }
    } else {
        roots_text.push_str(strings.text("prefs.value_unavailable"));
    }
    roots_view.setString(&NSString::from_str(&roots_text));
    roots_scroll.setDocumentView(Some(&roots_view));
    (roots_scroll, roots_view)
}
