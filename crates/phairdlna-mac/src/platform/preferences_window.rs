//! Native preferences window for macOS.
//!
//! What: shows the server settings read-only.
//! Why: the safe persistence and service-activation model is still open under
//!      OD-16, so the window shows the values of the server and changes
//!      nothing.
//! Example: users can inspect the server name, the port and the shared
//!          folders while the overview holds the fields they may change.

use std::path::Path;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSFont, NSPanel, NSScrollView, NSTabView, NSWindowStyleMask};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

mod controls;
mod general;
mod roots;
mod tabs;
use controls::{PreferenceField, PreferenceTextView};
use tabs::{add_notice, add_tab_scroll, build_tabs, tab_scroll_frame, TAB_HEIGHT};

/// Holds the open preferences window and its controls.
pub struct PreferencesWindow {
    /// The window panel itself.
    pub panel: Retained<NSPanel>,
    tab_view: Retained<NSTabView>,
    tab_scroll: Retained<NSScrollView>,
    text_fields: Vec<PreferenceField>,
    text_views: Vec<PreferenceTextView>,
}

impl PreferencesWindow {
    /// Builds the preferences window panel.
    pub fn new(mtm: MainThreadMarker, config_path: &Path, strings: &crate::text::Strings) -> Self {
        let panel = create_panel(mtm, strings);
        let Some(content) = panel.contentView() else {
            return Self::fallback(mtm, panel);
        };
        let mut tabs = build_tabs(mtm, config_path, strings);
        let tab_scroll = add_tab_scroll(mtm, &content, &tabs.tab_view);
        tabs.text_fields
            .push(add_notice(mtm, &content, config_path, strings));
        let window = Self {
            panel,
            tab_view: tabs.tab_view,
            tab_scroll,
            text_fields: tabs.text_fields,
            text_views: tabs.text_views,
        };
        window.apply_layout();
        window
    }

    fn fallback(mtm: MainThreadMarker, panel: Retained<NSPanel>) -> Self {
        Self {
            panel,
            tab_view: NSTabView::new(mtm),
            tab_scroll: NSScrollView::new(mtm),
            text_fields: Vec::new(),
            text_views: Vec::new(),
        }
    }

    /// Applies the standard system fonts and the height the content needs.
    ///
    /// The window uses the standard fonts of macOS; a long value keeps the
    /// height it needs, so that it is not clipped.
    fn apply_layout(&self) {
        for field in &self.text_fields {
            field.apply();
        }
        for view in &self.text_views {
            view.apply();
        }
        self.tab_view.setFont(&NSFont::systemFontOfSize(13.0));
        let required_height = self
            .text_fields
            .iter()
            .map(|field| field.document_height())
            .fold(TAB_HEIGHT, f64::max);
        let content_size = self
            .panel
            .contentView()
            .map(|view| view.bounds().size)
            .unwrap_or(NSSize::new(660.0, 590.0));
        let scroll_frame = tab_scroll_frame(content_size);
        self.tab_scroll.setFrame(scroll_frame);
        self.tab_view.setFrame(NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(scroll_frame.size.width, required_height),
        ));
    }

    /// Displays the window and brings it to front.
    pub fn show(&self, mtm: MainThreadMarker) {
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Regular);
        self.panel.makeKeyAndOrderFront(None);
        app.activate();
    }
}

fn create_panel(mtm: MainThreadMarker, strings: &crate::text::Strings) -> Retained<NSPanel> {
    let panel = NSPanel::new(mtm);
    panel.setContentSize(NSSize::new(660.0, 590.0));
    panel.setMinSize(NSSize::new(660.0, 590.0));
    panel.setStyleMask(
        NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Resizable
            | NSWindowStyleMask::Miniaturizable,
    );
    let title_ns = NSString::from_str(strings.text("prefs.title"));
    panel.setTitle(&title_ns);
    panel
}
