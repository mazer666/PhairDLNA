//! The placement of the controls of the native overview.
//!
//! What: measures the visible text and places the controls in reading order.
//! Why: the document must grow when a long text needs more lines, so that no
//!      section is clipped.
//! Example: a long folder name gets extra rows and the overview scrolls.
//!
//! **The standard sizes of the system.** The overview uses the standard fonts
//! of macOS and needs no scaling of its own.

use objc2_app_kit::{NSBox, NSButton, NSFont, NSLineBreakMode, NSPopUpButton, NSTextField};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::geometry::centered_document_height;
use super::OverviewViews;

/// The left edge of the content.
const X: f64 = 28.0;
/// The width of the content.
const WIDTH: f64 = 544.0;
/// The space between two blocks.
const GAP: f64 = 10.0;
/// The space between a heading and its content.
const TIGHT_GAP: f64 = 5.0;
/// The size of the font of the heading.
const HEADING_SIZE: f64 = 25.0;
/// The size of the font of the state.
const STATUS_SIZE: f64 = 18.0;
/// The size of the font of a section heading.
const SECTION_SIZE: f64 = 14.0;
/// The size of the font of the body.
const BODY_SIZE: f64 = 13.0;
/// The size of the font of the guidance.
const GUIDANCE_SIZE: f64 = 12.0;
/// The height of one line of the body.
const LINE_HEIGHT: f64 = 19.0;
/// The height of a button.
const BUTTON_HEIGHT: f64 = 32.0;
/// The height of a field.
const FIELD_HEIGHT: f64 = 24.0;
/// The height of the label of a field.
const LABEL_HEIGHT: f64 = 20.0;
/// The height of the list of the folders.
const PICKER_HEIGHT: f64 = 26.0;
/// The space below the last section.
const BOTTOM_SPACE: f64 = 40.0;

/// The heights the text of the sections needs.
struct Heights {
    folders: f64,
    connection: f64,
    activity: f64,
    notice: f64,
    notice_gap: f64,
    read_only: f64,
}

impl OverviewViews {
    /// Reflows all sections and text frames for their current content.
    pub(in crate::platform::main_window) fn reflow(&self) {
        configure_fonts(self);
        let heights = measure(self);
        let document_height = resize_document(self, flow_height(&heights));
        place_sections(self, &heights, document_height);
    }
}

/// Applies the standard fonts of the system to every control.
fn configure_fonts(views: &OverviewViews) {
    views
        .heading
        .setFont(Some(&NSFont::boldSystemFontOfSize(HEADING_SIZE)));
    views
        .status
        .setFont(Some(&NSFont::boldSystemFontOfSize(STATUS_SIZE)));
    views
        .library
        .setFont(Some(&NSFont::systemFontOfSize(BODY_SIZE)));
    for heading in [
        &views.folders_heading,
        &views.connection_heading,
        &views.activity_heading,
    ] {
        heading.setFont(Some(&NSFont::boldSystemFontOfSize(SECTION_SIZE)));
    }
    for field in [&views.folders, &views.connection, &views.activity] {
        field.setFont(Some(&NSFont::systemFontOfSize(BODY_SIZE)));
        field.setLineBreakMode(NSLineBreakMode::ByCharWrapping);
        field.setMaximumNumberOfLines(0);
    }
    for field in [&views.notice, &views.read_only] {
        field.setFont(Some(&NSFont::systemFontOfSize(GUIDANCE_SIZE)));
        field.setLineBreakMode(NSLineBreakMode::ByCharWrapping);
        field.setMaximumNumberOfLines(0);
    }
    for button in [
        &views.add_folders,
        &views.controls.toggle,
        &views.controls.remove_folder,
        &views.controls.save_settings,
    ] {
        button.setFont(Some(&NSFont::systemFontOfSize(BODY_SIZE)));
    }
    for field in [
        &views.controls.device_name_field,
        &views.controls.http_port_field,
    ] {
        field.setFont(Some(&NSFont::systemFontOfSize(BODY_SIZE)));
    }
}

/// Measures the text of the sections that change with their content.
fn measure(views: &OverviewViews) -> Heights {
    let notice_text = views.notice.stringValue().to_string();
    let notice_lines = if notice_text.is_empty() {
        0
    } else {
        wrapped_lines(&notice_text, GUIDANCE_SIZE)
    };
    Heights {
        folders: measured_height(&views.folders),
        connection: measured_height(&views.connection),
        activity: measured_height(&views.activity),
        notice: 17.0 * notice_lines as f64,
        notice_gap: if notice_lines == 0 { 0.0 } else { GAP },
        read_only: 17.0
            * wrapped_lines(&views.read_only.stringValue().to_string(), GUIDANCE_SIZE) as f64,
    }
}

/// The height a text of the body needs.
fn measured_height(field: &NSTextField) -> f64 {
    LINE_HEIGHT * wrapped_lines(&field.stringValue().to_string(), BODY_SIZE) as f64
}

/// The height the whole document needs.
fn flow_height(heights: &Heights) -> f64 {
    let sections = [
        34.0,
        GAP,
        42.0,
        GAP,
        BUTTON_HEIGHT,
        GAP,
        heights.notice,
        heights.notice_gap,
        28.0,
        GAP,
        24.0,
        TIGHT_GAP,
        heights.folders,
        GAP,
        1.0,
        GAP,
        PICKER_HEIGHT,
        TIGHT_GAP,
        BUTTON_HEIGHT,
        TIGHT_GAP,
        36.0,
        GAP,
        LABEL_HEIGHT,
        TIGHT_GAP,
        FIELD_HEIGHT,
        TIGHT_GAP,
        LABEL_HEIGHT,
        TIGHT_GAP,
        FIELD_HEIGHT,
        TIGHT_GAP,
        BUTTON_HEIGHT,
        GAP,
        24.0,
        TIGHT_GAP,
        heights.connection,
        GAP,
        1.0,
        GAP,
        24.0,
        TIGHT_GAP,
        heights.activity,
        GAP,
        heights.read_only,
    ];
    sections.iter().sum::<f64>() + BOTTOM_SPACE
}

/// Grows the document to the height it needs and returns that height.
fn resize_document(views: &OverviewViews, flow_height: f64) -> f64 {
    let viewport_height = views
        .document
        .enclosingScrollView()
        .map(|scroll| scroll.contentView().bounds().size.height)
        .unwrap_or(800.0);
    let height = centered_document_height(flow_height, viewport_height);
    views.document.setFrameSize(NSSize::new(600.0, height));
    views.document.setBounds(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(600.0, height),
    ));
    height
}

/// Places every control in reading order.
fn place_sections(views: &OverviewViews, heights: &Heights, document_height: f64) {
    let mut top = document_height - 20.0;
    place(&views.heading, 34.0, &mut top, GAP);
    place(&views.status, 42.0, &mut top, GAP);
    place_compact_button(&views.controls.toggle, 160.0, BUTTON_HEIGHT, &mut top, GAP);
    if heights.notice > 0.0 {
        place(&views.notice, heights.notice, &mut top, heights.notice_gap);
    }
    place(&views.library, 28.0, &mut top, GAP);
    place(&views.folders_heading, 24.0, &mut top, TIGHT_GAP);
    place(&views.folders, heights.folders, &mut top, GAP);
    place_rule(&views.folders_rule, &mut top, GAP);
    place_picker(&views.controls.folder_picker, &mut top, TIGHT_GAP);
    place_compact_button(
        &views.controls.remove_folder,
        180.0,
        BUTTON_HEIGHT,
        &mut top,
        TIGHT_GAP,
    );
    place_compact_button(&views.add_folders, 200.0, 36.0, &mut top, GAP);
    place(
        &views.controls.device_name_label,
        LABEL_HEIGHT,
        &mut top,
        TIGHT_GAP,
    );
    place_field(
        &views.controls.device_name_field,
        340.0,
        FIELD_HEIGHT,
        &mut top,
        TIGHT_GAP,
    );
    place(
        &views.controls.http_port_label,
        LABEL_HEIGHT,
        &mut top,
        TIGHT_GAP,
    );
    place_field(
        &views.controls.http_port_field,
        120.0,
        FIELD_HEIGHT,
        &mut top,
        TIGHT_GAP,
    );
    place_compact_button(
        &views.controls.save_settings,
        210.0,
        BUTTON_HEIGHT,
        &mut top,
        GAP,
    );
    place(&views.connection_heading, 24.0, &mut top, TIGHT_GAP);
    place(&views.connection, heights.connection, &mut top, GAP);
    place_rule(&views.connection_rule, &mut top, GAP);
    place(&views.activity_heading, 24.0, &mut top, TIGHT_GAP);
    place(&views.activity, heights.activity, &mut top, GAP);
    place(&views.read_only, heights.read_only, &mut top, 0.0);
}

/// The number of lines a text needs in the width of the document.
fn wrapped_lines(text: &str, font_size: f64) -> usize {
    // Budget one full em per character to accommodate wide-script and fallback glyphs.
    let chars_per_line = (WIDTH / font_size).floor().max(1.0) as usize;
    text.lines()
        .map(|line| line.chars().count().max(1).div_ceil(chars_per_line))
        .sum::<usize>()
        .max(1)
}

/// Places a text field.
fn place(field: &NSTextField, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    field.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(WIDTH, height),
    ));
    *top -= gap;
}

/// Places an input field with custom width.
fn place_field(field: &NSTextField, width: f64, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    field.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(width, height),
    ));
    *top -= gap;
}

/// Places a rule.
fn place_rule(rule: &NSBox, top: &mut f64, gap: f64) {
    *top -= 1.0;
    rule.setFrame(NSRect::new(NSPoint::new(X, *top), NSSize::new(WIDTH, 1.0)));
    *top -= gap;
}

/// Places a button with custom width.
fn place_compact_button(button: &NSButton, width: f64, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    button.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(width, height),
    ));
    *top -= gap;
}

/// Places the list of the folders.
fn place_picker(picker: &NSPopUpButton, top: &mut f64, gap: f64) {
    *top -= PICKER_HEIGHT;
    picker.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(WIDTH, PICKER_HEIGHT),
    ));
    *top -= gap;
}

#[cfg(test)]
mod tests {
    use super::{wrapped_lines, GUIDANCE_SIZE};

    /// The estimate changes at the exact width boundary of the standard font.
    #[test]
    fn wrapped_line_estimate_respects_the_width_boundary() {
        assert_eq!(wrapped_lines(&"x".repeat(45), GUIDANCE_SIZE), 1);
        assert_eq!(wrapped_lines(&"x".repeat(46), GUIDANCE_SIZE), 2);
    }

    /// Each hard line and each wide-script character receives conservative space.
    #[test]
    fn wrapped_line_estimate_counts_explicit_and_wide_text() {
        let wide = "界".repeat(46);
        assert_eq!(wrapped_lines(&wide, GUIDANCE_SIZE), 2);
        assert_eq!(wrapped_lines(&format!("short\n{wide}"), GUIDANCE_SIZE), 3);
    }
}
