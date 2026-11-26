//! The placement of the controls of the native overview.
//!
//! What: measures the visible text and places the controls in reading order.
//! Why: the document must grow when a long text needs more lines, so that no
//!      section is clipped.
//! Example: a long folder name gets extra rows and the overview scrolls.
//!
//! **The standard sizes of the system.** The overview uses the standard fonts
//! of macOS and needs no scaling of its own.

use objc2_app_kit::{NSFont, NSLineBreakMode, NSTextField};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::geometry::centered_document_height;
use super::OverviewViews;

/// The left edge of the content.
const X: f64 = 28.0;
/// The width of the content.
const WIDTH: f64 = 544.0;
/// The inner horizontal padding for card boxes.
const PAD: f64 = 16.0;
/// The inner content width inside cards.
const INNER_WIDTH: f64 = WIDTH - 2.0 * PAD;
/// The space between two blocks.
const GAP: f64 = 14.0;
/// The space between a heading and its content.
const TIGHT_GAP: f64 = 6.0;
/// The size of the font of the heading.
const HEADING_SIZE: f64 = 24.0;
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
const BUTTON_HEIGHT: f64 = 30.0;
/// The height of a field.
const FIELD_HEIGHT: f64 = 24.0;
/// The height of the label of a field.
const LABEL_HEIGHT: f64 = 18.0;
/// The height of the list of the folders.
const PICKER_HEIGHT: f64 = 26.0;
/// The space below the last section.
const BOTTOM_SPACE: f64 = 30.0;

/// The heights the text of the sections needs.
struct Heights {
    folders: f64,
    connection: f64,
    activity: f64,
    notice: f64,
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
        read_only: 17.0
            * wrapped_lines(&views.read_only.stringValue().to_string(), GUIDANCE_SIZE) as f64,
    }
}

/// The height a text of the body needs.
fn measured_height(field: &NSTextField) -> f64 {
    LINE_HEIGHT * wrapped_lines(&field.stringValue().to_string(), BODY_SIZE) as f64
}

/// Calculates the heights of the cards and total document flow.
fn card_heights(heights: &Heights) -> (f64, f64, f64, f64, f64) {
    // Status card: status + library + button + optional notice
    let status_card = 28.0
        + TIGHT_GAP
        + 20.0
        + TIGHT_GAP
        + BUTTON_HEIGHT
        + if heights.notice > 0.0 {
            TIGHT_GAP + heights.notice
        } else {
            0.0
        }
        + 2.0 * PAD;

    // Folders card: heading + folder list + picker + remove + add
    let folders_card =
        20.0 + TIGHT_GAP + heights.folders + GAP + PICKER_HEIGHT + TIGHT_GAP + 32.0 + 2.0 * PAD;

    // Settings card: heading + 2 form rows (label + field) + save button
    let settings_card = 20.0
        + TIGHT_GAP
        + FIELD_HEIGHT
        + TIGHT_GAP
        + FIELD_HEIGHT
        + GAP
        + BUTTON_HEIGHT
        + 2.0 * PAD;

    // Info card: connection heading + connection text + activity heading + activity + read-only
    let info_card = 20.0
        + TIGHT_GAP
        + heights.connection
        + GAP
        + 20.0
        + TIGHT_GAP
        + heights.activity
        + TIGHT_GAP
        + heights.read_only
        + 2.0 * PAD;

    let total = 36.0 // Page heading
        + GAP
        + status_card
        + GAP
        + folders_card
        + GAP
        + settings_card
        + GAP
        + info_card
        + BOTTOM_SPACE;

    (status_card, folders_card, settings_card, info_card, total)
}

/// The height the whole document needs.
fn flow_height(heights: &Heights) -> f64 {
    let (_, _, _, _, total) = card_heights(heights);
    total
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

/// Places every control in reading order inside styled card containers.
fn place_sections(views: &OverviewViews, heights: &Heights, document_height: f64) {
    let (h_status, h_folders, h_settings, h_info, _) = card_heights(heights);
    let mut top = document_height - 18.0;

    place(&views.heading, X, WIDTH, 32.0, &mut top, GAP);
    top = place_status_card(views, heights, top, h_status);
    top = place_folders_card(views, heights, top, h_folders);
    top = place_settings_card(views, top, h_settings);
    place_info_card(views, heights, top, h_info);
}

fn place_status_card(views: &OverviewViews, heights: &Heights, top: f64, height: f64) -> f64 {
    views.status_card.setFrame(NSRect::new(
        NSPoint::new(X, top - height),
        NSSize::new(WIDTH, height),
    ));
    let mut inner = top - PAD;
    place(
        &views.status,
        X + PAD,
        INNER_WIDTH - 150.0,
        26.0,
        &mut inner,
        TIGHT_GAP,
    );
    views.controls.toggle.setFrame(NSRect::new(
        NSPoint::new(X + WIDTH - PAD - 140.0, top - PAD - BUTTON_HEIGHT),
        NSSize::new(140.0, BUTTON_HEIGHT),
    ));
    place(
        &views.library,
        X + PAD,
        INNER_WIDTH,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    if heights.notice > 0.0 {
        place(
            &views.notice,
            X + PAD,
            INNER_WIDTH,
            heights.notice,
            &mut inner,
            0.0,
        );
    }
    top - height - GAP
}

fn place_folders_card(views: &OverviewViews, heights: &Heights, top: f64, height: f64) -> f64 {
    views.folders_card.setFrame(NSRect::new(
        NSPoint::new(X, top - height),
        NSSize::new(WIDTH, height),
    ));
    let mut inner = top - PAD;
    place(
        &views.folders_heading,
        X + PAD,
        INNER_WIDTH,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.folders,
        X + PAD,
        INNER_WIDTH,
        heights.folders,
        &mut inner,
        GAP,
    );
    views.controls.folder_picker.setFrame(NSRect::new(
        NSPoint::new(X + PAD, inner - PICKER_HEIGHT),
        NSSize::new(INNER_WIDTH, PICKER_HEIGHT),
    ));
    inner -= PICKER_HEIGHT + TIGHT_GAP;
    views.controls.remove_folder.setFrame(NSRect::new(
        NSPoint::new(X + PAD, inner - BUTTON_HEIGHT),
        NSSize::new(170.0, BUTTON_HEIGHT),
    ));
    views.add_folders.setFrame(NSRect::new(
        NSPoint::new(X + PAD + 180.0, inner - BUTTON_HEIGHT),
        NSSize::new(190.0, BUTTON_HEIGHT),
    ));
    top - height - GAP
}

fn place_settings_card(views: &OverviewViews, top: f64, height: f64) -> f64 {
    views.settings_card.setFrame(NSRect::new(
        NSPoint::new(X, top - height),
        NSSize::new(WIDTH, height),
    ));
    let mut inner = top - PAD;
    let (label_w, field_x) = (120.0, X + PAD + 130.0);

    views.controls.device_name_label.setFrame(NSRect::new(
        NSPoint::new(X + PAD, inner - LABEL_HEIGHT - 3.0),
        NSSize::new(label_w, LABEL_HEIGHT),
    ));
    views.controls.device_name_field.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - FIELD_HEIGHT),
        NSSize::new(260.0, FIELD_HEIGHT),
    ));
    inner -= FIELD_HEIGHT + TIGHT_GAP;

    views.controls.http_port_label.setFrame(NSRect::new(
        NSPoint::new(X + PAD, inner - LABEL_HEIGHT - 3.0),
        NSSize::new(label_w, LABEL_HEIGHT),
    ));
    views.controls.http_port_field.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - FIELD_HEIGHT),
        NSSize::new(100.0, FIELD_HEIGHT),
    ));
    inner -= FIELD_HEIGHT + GAP;

    views.controls.save_settings.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - BUTTON_HEIGHT),
        NSSize::new(180.0, BUTTON_HEIGHT),
    ));
    top - height - GAP
}

fn place_info_card(views: &OverviewViews, heights: &Heights, top: f64, height: f64) {
    views.info_card.setFrame(NSRect::new(
        NSPoint::new(X, top - height),
        NSSize::new(WIDTH, height),
    ));
    let mut inner = top - PAD;
    place(
        &views.connection_heading,
        X + PAD,
        INNER_WIDTH,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.connection,
        X + PAD,
        INNER_WIDTH,
        heights.connection,
        &mut inner,
        GAP,
    );
    place(
        &views.activity_heading,
        X + PAD,
        INNER_WIDTH,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.activity,
        X + PAD,
        INNER_WIDTH,
        heights.activity,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.read_only,
        X + PAD,
        INNER_WIDTH,
        heights.read_only,
        &mut inner,
        0.0,
    );
}

/// The number of lines a text needs in the width of the document.
fn wrapped_lines(text: &str, font_size: f64) -> usize {
    // Budget one full em per character to accommodate wide-script and fallback glyphs.
    let chars_per_line = (INNER_WIDTH / font_size).floor().max(1.0) as usize;
    text.lines()
        .map(|line| line.chars().count().max(1).div_ceil(chars_per_line))
        .sum::<usize>()
        .max(1)
}

/// Places a text field with custom frame and gap.
fn place(field: &NSTextField, x: f64, width: f64, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    field.setFrame(NSRect::new(
        NSPoint::new(x, *top),
        NSSize::new(width, height),
    ));
    *top -= gap;
}

#[cfg(test)]
mod tests {
    use super::{wrapped_lines, GUIDANCE_SIZE};

    /// The estimate changes at the exact width boundary of the standard font.
    #[test]
    fn wrapped_line_estimate_respects_the_width_boundary() {
        assert_eq!(wrapped_lines(&"x".repeat(42), GUIDANCE_SIZE), 1);
        assert_eq!(wrapped_lines(&"x".repeat(43), GUIDANCE_SIZE), 2);
    }

    /// Each hard line and each wide-script character receives conservative space.
    #[test]
    fn wrapped_line_estimate_counts_explicit_and_wide_text() {
        let wide = "界".repeat(43);
        assert_eq!(wrapped_lines(&wide, GUIDANCE_SIZE), 2);
        assert_eq!(wrapped_lines(&format!("short\n{wide}"), GUIDANCE_SIZE), 3);
    }
}
