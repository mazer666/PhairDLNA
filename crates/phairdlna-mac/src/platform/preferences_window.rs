//! Native preferences window for macOS.
//!
//! What: shows server settings read-only and provides app-owned presentation preferences.
//! Why: the safe persistence and service-activation model is still open under OD-16.
//! Example: users can change text size here while server values remain available to inspect.

use std::cell::Cell;
use std::path::Path;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSButton, NSFont, NSPanel, NSScrollView, NSTabView, NSTextField, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::bridge::MenuTarget;
mod general;
mod roots;
mod scaling;
mod tabs;
mod text_scale;
use crate::model::TextScale;
use scaling::{scale_frame, ScaledTextField, ScaledTextView};
use tabs::{add_notice, add_tab_scroll, build_tabs, tab_scroll_frame, TAB_HEIGHT};
use text_scale::{build_controls, select};

/// Holds the open preferences window and its controls.
pub struct PreferencesWindow {
    /// The window panel itself.
    pub panel: Retained<NSPanel>,
    text_size_buttons: Vec<Retained<NSButton>>,
    text_size_label: Retained<NSTextField>,
    tab_view: Retained<NSTabView>,
    tab_scroll: Retained<NSScrollView>,
    roots_scroll: Retained<NSScrollView>,
    expert_scroll: Retained<NSScrollView>,
    text_fields: Vec<ScaledTextField>,
    text_views: Vec<ScaledTextView>,
    current_scale: Cell<Option<TextScale>>,
}

impl PreferencesWindow {
    /// Builds the preferences window panel.
    pub fn new(
        mtm: MainThreadMarker,
        config_path: &Path,
        strings: &crate::text::Strings,
        target: MenuTarget,
    ) -> Self {
        let panel = create_panel(mtm, strings);
        let Some(content) = panel.contentView() else {
            return Self::fallback(mtm, panel);
        };
        let mut tabs = build_tabs(mtm, config_path, strings);
        let tab_scroll = add_tab_scroll(mtm, &content, &tabs.tab_view);
        tabs.text_fields
            .push(add_notice(mtm, &content, config_path, strings));
        let (text_size_label, text_size_buttons) = build_controls(mtm, strings, &target, &content);
        Self {
            panel,
            text_size_buttons,
            text_size_label,
            tab_view: tabs.tab_view,
            tab_scroll,
            roots_scroll: tabs.roots_scroll,
            expert_scroll: tabs.expert_scroll,
            text_fields: tabs.text_fields,
            text_views: tabs.text_views,
            current_scale: Cell::new(None),
        }
    }

    fn fallback(mtm: MainThreadMarker, panel: Retained<NSPanel>) -> Self {
        Self {
            panel,
            text_size_buttons: Vec::new(),
            text_size_label: NSTextField::new(mtm),
            tab_view: NSTabView::new(mtm),
            tab_scroll: NSScrollView::new(mtm),
            roots_scroll: NSScrollView::new(mtm),
            expert_scroll: NSScrollView::new(mtm),
            text_fields: Vec::new(),
            text_views: Vec::new(),
            current_scale: Cell::new(None),
        }
    }

    /// Displays the window and brings it to front.
    pub fn show(&self, mtm: MainThreadMarker) {
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Regular);
        self.panel.makeKeyAndOrderFront(None);
        app.activate();
    }

    /// Highlights the currently selected app-owned text size.
    pub fn update_text_scale(&self, scale: TextScale) {
        if self.current_scale.get() == Some(scale) {
            return;
        }
        let factor = scale.factor();
        let required_height = self
            .text_fields
            .iter()
            .map(|field| field.minimum_document_height(factor))
            .fold(TAB_HEIGHT * factor, f64::max);
        select(&self.text_size_buttons, scale);
        self.text_size_label
            .setFont(Some(&NSFont::systemFontOfSize(13.0 * factor)));
        self.text_size_label.setFrame(scale_frame(
            NSRect::new(NSPoint::new(18.0, 16.0), NSSize::new(118.0, 28.0)),
            factor,
        ));
        for (index, button) in self.text_size_buttons.iter().enumerate() {
            let x = [140.0, 235.0, 330.0][index];
            button.setFont(Some(&NSFont::systemFontOfSize(13.0 * factor)));
            button.setFrame(scale_frame(
                NSRect::new(NSPoint::new(x, 12.0), NSSize::new(82.0, 32.0)),
                factor,
            ));
        }
        for field in &self.text_fields {
            field.apply(factor);
        }
        for view in &self.text_views {
            view.apply(factor);
        }
        self.roots_scroll.setFrame(scale_frame(
            NSRect::new(NSPoint::new(20.0, 20.0), NSSize::new(595.0, 275.0)),
            factor,
        ));
        self.expert_scroll.setFrame(scale_frame(
            NSRect::new(NSPoint::new(20.0, 20.0), NSSize::new(595.0, 400.0)),
            factor,
        ));
        self.tab_view
            .setFont(&NSFont::systemFontOfSize(13.0 * factor));
        let content_size = self
            .panel
            .contentView()
            .map(|view| view.bounds().size)
            .unwrap_or(NSSize::new(660.0, 590.0));
        let scroll_frame = tab_scroll_frame(factor, content_size);
        self.tab_scroll.setFrame(scroll_frame);
        self.tab_view.setFrame(NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(
                scroll_frame.size.width,
                required_height.max(TAB_HEIGHT * factor),
            ),
        ));
        self.current_scale.set(Some(scale));
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
