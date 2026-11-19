//! Tab construction for the native Preferences window.
//!
//! What: assembles the general, expert and About tabs with scalable content.
//! Why: the Preferences window keeps server values inspectable at larger text sizes.
//! Example: the validated config feeds both the General tab and the read-only TOML tab.

use std::path::Path;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAccessibility, NSAutoresizingMaskOptions, NSFont, NSScrollView, NSTabView, NSTabViewItem,
    NSTextField, NSTextView, NSView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use phairdlna_core::config::Config;

use super::general::build_general_tab;
use super::scaling::{ScaledTextField, ScaledTextView};

pub(super) const TAB_WIDTH: f64 = 640.0;
pub(super) const TAB_HEIGHT: f64 = 475.0;

pub(super) struct PreferenceTabs {
    pub(super) tab_view: Retained<NSTabView>,
    pub(super) roots_scroll: Retained<NSScrollView>,
    pub(super) expert_scroll: Retained<NSScrollView>,
    pub(super) text_fields: Vec<ScaledTextField>,
    pub(super) text_views: Vec<ScaledTextView>,
}

pub(super) fn build_tabs(
    mtm: MainThreadMarker,
    config_path: &Path,
    strings: &crate::text::Strings,
) -> PreferenceTabs {
    let (config, config_available, raw_toml) = read_configuration(config_path, strings);
    let tab_view = NSTabView::new(mtm);
    tab_view.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
    tab_view.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(TAB_WIDTH, TAB_HEIGHT),
    ));
    let general = build_general_tab(mtm, &config, config_available, strings);
    tab_view.addTabViewItem(&general.tab);
    let (expert_tab, expert_view, expert_scroll) = build_expert_tab(mtm, &raw_toml, strings);
    tab_view.addTabViewItem(&expert_tab);
    let (about_tab, about_fields) = super::super::about_tab::build_about_tab(mtm, strings);
    tab_view.addTabViewItem(&about_tab);
    let mut text_fields = general.text_fields;
    text_fields.extend(
        about_fields
            .into_iter()
            .map(|(field, frame)| ScaledTextField {
                wrap_text: None,
                frame,
                field,
                font_size: 13.0,
            }),
    );
    PreferenceTabs {
        tab_view,
        roots_scroll: general.roots_scroll,
        expert_scroll,
        text_fields,
        text_views: vec![
            ScaledTextView {
                view: general.roots_view,
                font_size: 12.0,
                fixed_pitch: false,
            },
            ScaledTextView {
                view: expert_view,
                font_size: 12.0,
                fixed_pitch: true,
            },
        ],
    }
}

fn read_configuration(
    config_path: &Path,
    strings: &crate::text::Strings,
) -> (Config, bool, String) {
    let loaded_config = Config::load(config_path);
    let raw_config = std::fs::read_to_string(config_path);
    let config_available = loaded_config.is_ok() && raw_config.is_ok();
    let config = loaded_config.unwrap_or_default();
    let raw_toml =
        raw_config.unwrap_or_else(|_| strings.text("prefs.config_unavailable").to_owned());
    (config, config_available, raw_toml)
}

fn build_expert_tab(
    mtm: MainThreadMarker,
    raw_toml: &str,
    strings: &crate::text::Strings,
) -> (
    Retained<NSTabViewItem>,
    Retained<NSTextView>,
    Retained<NSScrollView>,
) {
    let tab = NSTabViewItem::new();
    tab.setLabel(&NSString::from_str(strings.text("prefs.tab_expert")));
    let scroll = NSScrollView::new(mtm);
    scroll.setFrame(NSRect::new(
        NSPoint::new(20.0, 20.0),
        NSSize::new(595.0, 400.0),
    ));
    scroll.setHasVerticalScroller(true);
    scroll.setHasHorizontalScroller(false);
    let editor = NSTextView::new(mtm);
    editor.setEditable(false);
    editor.setAccessibilityLabel(Some(&NSString::from_str(strings.text("prefs.tab_expert"))));
    let font = NSFont::userFixedPitchFontOfSize(12.0);
    editor.setFont(font.as_deref());
    editor.setString(&NSString::from_str(raw_toml));
    scroll.setDocumentView(Some(&editor));
    tab.setView(Some(&scroll));
    (tab, editor, scroll)
}

pub(super) fn add_tab_scroll(
    mtm: MainThreadMarker,
    content: &NSView,
    tabs: &NSTabView,
) -> Retained<NSScrollView> {
    let scroll = NSScrollView::new(mtm);
    scroll.setFrame(NSRect::new(
        NSPoint::new(10.0, 90.0),
        NSSize::new(640.0, 490.0),
    ));
    scroll.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    scroll.setHasVerticalScroller(true);
    scroll.setHasHorizontalScroller(false);
    scroll.setDocumentView(Some(tabs));
    content.addSubview(&scroll);
    scroll
}

pub(super) fn tab_scroll_frame(factor: f64, content_size: NSSize) -> NSRect {
    let bottom_controls_end = 46.0 * factor;
    let y = 90.0_f64.max(bottom_controls_end + 8.0);
    NSRect::new(
        NSPoint::new(10.0, y),
        NSSize::new(content_size.width - 20.0, content_size.height - y - 10.0),
    )
}

pub(super) fn add_notice(
    mtm: MainThreadMarker,
    content: &NSView,
    config_path: &Path,
    strings: &crate::text::Strings,
) -> ScaledTextField {
    let key = if Config::load(config_path).is_ok() {
        "prefs.read_only_notice"
    } else {
        "prefs.config_unavailable"
    };
    let notice = NSTextField::wrappingLabelWithString(&NSString::from_str(strings.text(key)), mtm);
    let frame = NSRect::new(NSPoint::new(418.0, 4.0), NSSize::new(230.0, 42.0));
    notice.setFrame(frame);
    notice.setFont(Some(&NSFont::systemFontOfSize(11.0)));
    notice.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
    content.addSubview(&notice);
    ScaledTextField {
        wrap_text: None,
        field: notice,
        frame,
        font_size: 11.0,
    }
}
