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
use super::grid::{
    GridLayout, BODY_SIZE, BOTTOM_SPACE, BUTTON_HEIGHT, FIELD_HEIGHT, GAP, GUIDANCE_SIZE,
    HEADING_SIZE, LINE_HEIGHT, MARGIN, PAD, PICKER_HEIGHT, SECTION_SIZE, STATUS_SIZE, TIGHT_GAP,
};
use super::placement::{
    place_folders_card, place_info_card, place_settings_card, place_status_card,
};
use super::OverviewViews;

/// The heights the text of the sections needs.
pub(super) struct Heights {
    pub folders: f64,
    pub connection: f64,
    pub activity: f64,
    pub notice: f64,
    pub read_only: f64,
}

impl OverviewViews {
    /// Reflows all sections and text frames for their current content.
    pub(in crate::platform::main_window) fn reflow(&self) {
        configure_fonts(self);
        let viewport_width = current_viewport_width(self);
        let grid = GridLayout::for_width(viewport_width);
        let heights = measure(self, grid.inner_width);
        let (doc_w, doc_h) = calculate_document_size(&grid, &heights, self);
        resize_document(self, doc_w, doc_h);
        place_sections(self, &heights, &grid, doc_h);
    }
}

fn current_viewport_width(views: &OverviewViews) -> f64 {
    views
        .document
        .enclosingScrollView()
        .map(|scroll| scroll.contentView().bounds().size.width)
        .unwrap_or(600.0)
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
        &views.rescan_button,
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
fn measure(views: &OverviewViews, inner_width: f64) -> Heights {
    let notice_text = views.notice.stringValue().to_string();
    let notice_lines = if notice_text.is_empty() {
        0
    } else {
        wrapped_lines(&notice_text, GUIDANCE_SIZE, inner_width)
    };
    Heights {
        folders: measured_height(&views.folders, inner_width),
        connection: measured_height(&views.connection, inner_width),
        activity: measured_height(&views.activity, inner_width),
        notice: 17.0 * notice_lines as f64,
        read_only: 17.0
            * wrapped_lines(
                &views.read_only.stringValue().to_string(),
                GUIDANCE_SIZE,
                inner_width,
            ) as f64,
    }
}

/// The height a text of the body needs.
fn measured_height(field: &NSTextField, inner_width: f64) -> f64 {
    LINE_HEIGHT * wrapped_lines(&field.stringValue().to_string(), BODY_SIZE, inner_width) as f64
}

/// Calculates the heights of the individual cards.
fn card_heights(heights: &Heights) -> (f64, f64, f64, f64) {
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

    let folders_card =
        20.0 + TIGHT_GAP + heights.folders + GAP + PICKER_HEIGHT + TIGHT_GAP + 32.0 + 2.0 * PAD;

    let settings_card = 20.0
        + TIGHT_GAP
        + FIELD_HEIGHT
        + TIGHT_GAP
        + FIELD_HEIGHT
        + GAP
        + BUTTON_HEIGHT
        + 2.0 * PAD;

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

    (status_card, folders_card, settings_card, info_card)
}

/// Calculates document dimensions given grid layout and measured card heights.
fn calculate_document_size(
    grid: &GridLayout,
    heights: &Heights,
    views: &OverviewViews,
) -> (f64, f64) {
    let (h_status, h_folders, h_settings, h_info) = card_heights(heights);
    let flow = if grid.is_two_col {
        let col1_flow = h_status + GAP + h_folders;
        let col2_flow = h_settings + GAP + h_info;
        36.0 + GAP + col1_flow.max(col2_flow) + BOTTOM_SPACE
    } else {
        36.0 + GAP + h_status + GAP + h_folders + GAP + h_settings + GAP + h_info + BOTTOM_SPACE
    };

    let viewport_height = views
        .document
        .enclosingScrollView()
        .map(|scroll| scroll.contentView().bounds().size.height)
        .unwrap_or(800.0);

    let doc_height = centered_document_height(flow, viewport_height);
    let doc_width = if grid.is_two_col {
        grid.col2_x + grid.col_width + MARGIN
    } else {
        grid.col_width + 2.0 * MARGIN
    };
    (doc_width, doc_height)
}

/// Resizes the document view.
fn resize_document(views: &OverviewViews, width: f64, height: f64) {
    views.document.setFrameSize(NSSize::new(width, height));
    views.document.setBounds(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(width, height),
    ));
}

/// Places every control in reading order inside styled card containers.
fn place_sections(
    views: &OverviewViews,
    heights: &Heights,
    grid: &GridLayout,
    document_height: f64,
) {
    let (h_status, h_folders, h_settings, h_info) = card_heights(heights);
    let mut top = document_height - 18.0;
    let heading_w = if grid.is_two_col {
        grid.col_width * 2.0 + GAP
    } else {
        grid.col_width
    };

    place(&views.heading, grid.col1_x, heading_w, 32.0, &mut top, GAP);

    if grid.is_two_col {
        let mut left_top = top;
        left_top = place_status_card(views, heights, grid, left_top, h_status);
        place_folders_card(views, heights, grid, left_top, h_folders);

        let mut right_top = top;
        right_top = place_settings_card(views, grid, right_top, h_settings);
        place_info_card(views, heights, grid, right_top, h_info);
    } else {
        top = place_status_card(views, heights, grid, top, h_status);
        top = place_folders_card(views, heights, grid, top, h_folders);
        top = place_settings_card(views, grid, top, h_settings);
        place_info_card(views, heights, grid, top, h_info);
    }
}

/// The number of lines a text needs in the width of the document.
fn wrapped_lines(text: &str, font_size: f64, inner_width: f64) -> usize {
    let chars_per_line = (inner_width / font_size).floor().max(1.0) as usize;
    text.lines()
        .map(|line| line.chars().count().max(1).div_ceil(chars_per_line))
        .sum::<usize>()
        .max(1)
}

/// Places a text field with custom frame and gap.
pub(super) fn place(field: &NSTextField, x: f64, width: f64, height: f64, top: &mut f64, gap: f64) {
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
        assert_eq!(wrapped_lines(&"x".repeat(42), GUIDANCE_SIZE, 512.0), 1);
        assert_eq!(wrapped_lines(&"x".repeat(43), GUIDANCE_SIZE, 512.0), 2);
    }

    /// Each hard line and each wide-script character receives conservative space.
    #[test]
    fn wrapped_line_estimate_counts_explicit_and_wide_text() {
        let wide = "界".repeat(43);
        assert_eq!(wrapped_lines(&wide, GUIDANCE_SIZE, 512.0), 2);
        assert_eq!(
            wrapped_lines(&format!("short\n{wide}"), GUIDANCE_SIZE, 512.0),
            3
        );
    }
}
