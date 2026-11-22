//! General tab of the native preferences window.
//!
//! What: shows the server name, port and configured folders.
//! Why: users need to inspect server settings while OD-16 keeps app edits gated.
//! Example: an unreadable configuration displays unavailable values instead of defaults.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAccessibility, NSLineBreakMode, NSTabViewItem, NSTextField, NSTextView, NSView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use phairdlna_core::config::Config;

use super::controls::PreferenceField;
use super::roots::build_roots_view;

struct ValueRow<'a> {
    title: &'a str,
    value: String,
    y: f64,
    width: f64,
    wrap_long_value: bool,
}

pub(super) struct GeneralTab {
    pub(super) tab: Retained<NSTabViewItem>,
    pub(super) text_fields: Vec<PreferenceField>,
    pub(super) roots_view: Retained<NSTextView>,
}

pub(super) fn build_general_tab(
    mtm: MainThreadMarker,
    config: &Config,
    config_available: bool,
    strings: &crate::text::Strings,
) -> GeneralTab {
    let tab = NSTabViewItem::new();
    tab.setLabel(&NSString::from_str(strings.text("prefs.tab_general")));
    let view = NSView::new(mtm);
    let mut text_fields = Vec::new();
    add_value_row(
        &view,
        mtm,
        ValueRow {
            title: strings.text("prefs.server_name"),
            value: server_name(config, config_available, strings),
            y: 390.0,
            width: 390.0,
            wrap_long_value: true,
        },
        &mut text_fields,
    );
    add_value_row(
        &view,
        mtm,
        ValueRow {
            title: strings.text("prefs.http_port"),
            value: server_port(config, config_available, strings),
            y: 350.0,
            width: 120.0,
            wrap_long_value: false,
        },
        &mut text_fields,
    );
    let roots_view = add_roots(
        &view,
        mtm,
        config,
        config_available,
        strings,
        &mut text_fields,
    );
    tab.setView(Some(&view));
    GeneralTab {
        tab,
        text_fields,
        roots_view,
    }
}

fn server_name(config: &Config, available: bool, strings: &crate::text::Strings) -> String {
    if available {
        config.server.device_name.clone()
    } else {
        strings.text("prefs.value_unavailable").to_owned()
    }
}

fn server_port(config: &Config, available: bool, strings: &crate::text::Strings) -> String {
    if available {
        config.server.http_port.to_string()
    } else {
        strings.text("prefs.value_unavailable").to_owned()
    }
}

fn add_value_row(
    view: &NSView,
    mtm: MainThreadMarker,
    row: ValueRow<'_>,
    text_fields: &mut Vec<PreferenceField>,
) {
    let label = NSTextField::wrappingLabelWithString(&NSString::from_str(row.title), mtm);
    let label_frame = NSRect::new(NSPoint::new(20.0, row.y), NSSize::new(180.0, 24.0));
    label.setFrame(label_frame);
    view.addSubview(&label);
    text_fields.push(PreferenceField {
        wrap_text: None,
        field: label,
        frame: label_frame,
        font_size: 13.0,
    });

    let field = NSTextField::new(mtm);
    let field_frame = NSRect::new(
        NSPoint::new(200.0, row.y - 4.0),
        NSSize::new(row.width, 34.0),
    );
    field.setFrame(field_frame);
    field.setStringValue(&NSString::from_str(&row.value));
    field.setEditable(false);
    field.setSelectable(true);
    field.setAccessibilityLabel(Some(&NSString::from_str(row.title)));
    field.setLineBreakMode(NSLineBreakMode::ByCharWrapping);
    field.setMaximumNumberOfLines(0);
    view.addSubview(&field);
    text_fields.push(PreferenceField {
        wrap_text: row.wrap_long_value.then(|| row.value.clone()),
        field,
        frame: field_frame,
        font_size: 13.0,
    });
}

fn add_roots(
    view: &NSView,
    mtm: MainThreadMarker,
    config: &Config,
    available: bool,
    strings: &crate::text::Strings,
    text_fields: &mut Vec<PreferenceField>,
) -> Retained<NSTextView> {
    let heading = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.roots_header")),
        mtm,
    );
    let heading_frame = NSRect::new(NSPoint::new(20.0, 320.0), NSSize::new(300.0, 24.0));
    heading.setFrame(heading_frame);
    view.addSubview(&heading);
    text_fields.push(PreferenceField {
        wrap_text: None,
        field: heading,
        frame: heading_frame,
        font_size: 13.0,
    });

    let hint = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.roots_read_only")),
        mtm,
    );
    let hint_frame = NSRect::new(NSPoint::new(20.0, 298.0), NSSize::new(595.0, 20.0));
    hint.setFrame(hint_frame);
    view.addSubview(&hint);
    text_fields.push(PreferenceField {
        wrap_text: None,
        field: hint,
        frame: hint_frame,
        font_size: 13.0,
    });
    let (roots_scroll, roots_view) = build_roots_view(mtm, config, available, strings);
    view.addSubview(&roots_scroll);
    roots_view
}
