//! Responsive text measurement and placement for the overview.
//!
//! What: measures visible copy and places the overview controls in reading order.
//! Why: the document must grow when larger text or long translations need more lines.
//! Example: selecting 200% recalculates heights before positioning the folder section.

use objc2_app_kit::{NSBox, NSButton, NSFont, NSLineBreakMode, NSTextField};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::geometry::centered_document_height;
use super::OverviewViews;
use crate::model::TextScale;

const X: f64 = 28.0;
const WIDTH: f64 = 544.0;
const GUIDANCE_FONT_SIZE: f64 = 12.0;

struct Heights {
    folders: f64,
    connection: f64,
    activity: f64,
    notice: f64,
    notice_gap: f64,
    read_only: f64,
}

impl OverviewViews {
    /// Reflows all sections and text frames for this size and their current content.
    pub(in crate::platform::main_window) fn reflow(&self, scale: TextScale) {
        let factor = scale.factor();
        let gap = 10.0 * factor;
        let body_size = 13.0 * factor;
        configure_fonts(self, factor, body_size);
        let heights = measure(self, factor, body_size, gap);
        let document_height = resize_document(self, flow_height(factor, gap, &heights));
        place_sections(self, factor, gap, &heights, document_height);
    }
}

fn configure_fonts(views: &OverviewViews, factor: f64, body_size: f64) {
    views
        .heading
        .setFont(Some(&NSFont::boldSystemFontOfSize(25.0 * factor)));
    views
        .status
        .setFont(Some(&NSFont::boldSystemFontOfSize(18.0 * factor)));
    views
        .library
        .setFont(Some(&NSFont::systemFontOfSize(body_size)));
    for heading in [
        &views.folders_heading,
        &views.connection_heading,
        &views.activity_heading,
    ] {
        heading.setFont(Some(&NSFont::boldSystemFontOfSize(14.0 * factor)));
    }
    for field in [&views.folders, &views.connection, &views.activity] {
        field.setFont(Some(&NSFont::systemFontOfSize(body_size)));
        field.setLineBreakMode(NSLineBreakMode::ByCharWrapping);
        field.setMaximumNumberOfLines(0);
    }
    for field in [&views.notice, &views.read_only] {
        field.setFont(Some(&NSFont::systemFontOfSize(GUIDANCE_FONT_SIZE * factor)));
        field.setLineBreakMode(NSLineBreakMode::ByCharWrapping);
        field.setMaximumNumberOfLines(0);
    }
    views
        .add_folders
        .setFont(Some(&NSFont::systemFontOfSize(13.0 * factor)));
}

fn measure(views: &OverviewViews, factor: f64, body_size: f64, gap: f64) -> Heights {
    let line_height = 19.0 * factor;
    let folder_height = measured_height(&views.folders, body_size, line_height);
    let connection_height = measured_height(&views.connection, body_size, line_height);
    let activity_height = measured_height(&views.activity, body_size, line_height);
    let notice_text = views.notice.stringValue().to_string();
    let notice_lines = if notice_text.is_empty() {
        0
    } else {
        wrapped_lines(&notice_text, GUIDANCE_FONT_SIZE * factor)
    };
    Heights {
        folders: folder_height,
        connection: connection_height,
        activity: activity_height,
        notice: 17.0 * factor * notice_lines as f64,
        notice_gap: if notice_lines == 0 { 0.0 } else { gap },
        read_only: 17.0
            * factor
            * wrapped_lines(
                &views.read_only.stringValue().to_string(),
                GUIDANCE_FONT_SIZE * factor,
            ) as f64,
    }
}

fn measured_height(field: &NSTextField, font_size: f64, line_height: f64) -> f64 {
    line_height * wrapped_lines(&field.stringValue().to_string(), font_size) as f64
}

fn flow_height(factor: f64, gap: f64, heights: &Heights) -> f64 {
    let sections = [
        34.0 * factor,
        gap,
        42.0 * factor,
        gap,
        heights.notice,
        heights.notice_gap,
        28.0 * factor,
        gap,
        24.0 * factor,
        gap / 2.0,
        heights.folders,
        gap,
        1.0,
        gap,
        36.0 * factor,
        gap / 2.0,
        24.0 * factor,
        gap / 2.0,
        heights.connection,
        gap,
        1.0,
        gap,
        24.0 * factor,
        gap / 2.0,
        heights.activity,
        gap,
        heights.read_only,
    ];
    sections.iter().sum::<f64>() + 40.0 * factor
}

fn resize_document(views: &OverviewViews, flow_height: f64) -> f64 {
    let viewport_height = views
        .document
        .enclosingScrollView()
        .map(|scroll| scroll.contentView().bounds().size.height)
        .unwrap_or(560.0);
    let height = centered_document_height(flow_height, viewport_height);
    views.document.setFrameSize(NSSize::new(600.0, height));
    views.document.setBounds(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(600.0, height),
    ));
    height
}

fn place_sections(
    views: &OverviewViews,
    factor: f64,
    gap: f64,
    heights: &Heights,
    document_height: f64,
) {
    let mut top = document_height - 20.0 * factor;
    place(&views.heading, 34.0 * factor, &mut top, gap);
    place(&views.status, 42.0 * factor, &mut top, gap);
    if heights.notice > 0.0 {
        place(&views.notice, heights.notice, &mut top, heights.notice_gap);
    }
    place(&views.library, 28.0 * factor, &mut top, gap);
    place(&views.folders_heading, 24.0 * factor, &mut top, gap / 2.0);
    place(&views.folders, heights.folders, &mut top, gap);
    place_rule(&views.folders_rule, &mut top, gap);
    place_button(&views.add_folders, 36.0 * factor, &mut top, gap / 2.0);
    place(
        &views.connection_heading,
        24.0 * factor,
        &mut top,
        gap / 2.0,
    );
    place(&views.connection, heights.connection, &mut top, gap);
    place_rule(&views.connection_rule, &mut top, gap);
    place(&views.activity_heading, 24.0 * factor, &mut top, gap / 2.0);
    place(&views.activity, heights.activity, &mut top, gap);
    place(&views.read_only, heights.read_only, &mut top, 0.0);
}

fn wrapped_lines(text: &str, font_size: f64) -> usize {
    // Budget one full em per character to accommodate wide-script and fallback glyphs.
    let chars_per_line = (WIDTH / font_size).floor().max(1.0) as usize;
    text.lines()
        .map(|line| line.chars().count().max(1).div_ceil(chars_per_line))
        .sum::<usize>()
        .max(1)
}

fn place(field: &NSTextField, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    field.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(WIDTH, height),
    ));
    *top -= gap;
}

fn place_rule(rule: &NSBox, top: &mut f64, gap: f64) {
    *top -= 1.0;
    rule.setFrame(NSRect::new(NSPoint::new(X, *top), NSSize::new(WIDTH, 1.0)));
    *top -= gap;
}

fn place_button(button: &NSButton, height: f64, top: &mut f64, gap: f64) {
    *top -= height;
    button.setFrame(NSRect::new(
        NSPoint::new(X, *top),
        NSSize::new(WIDTH, height),
    ));
    *top -= gap;
}

#[cfg(test)]
mod tests {
    use super::{wrapped_lines, GUIDANCE_FONT_SIZE};

    /// The estimate changes at the exact width boundary for 100% and 200% text.
    #[test]
    fn wrapped_line_estimate_respects_scaled_font_boundaries() {
        assert_eq!(wrapped_lines(&"x".repeat(45), GUIDANCE_FONT_SIZE), 1);
        assert_eq!(wrapped_lines(&"x".repeat(46), GUIDANCE_FONT_SIZE), 2);
        assert_eq!(wrapped_lines(&"x".repeat(22), GUIDANCE_FONT_SIZE * 2.0), 1);
        assert_eq!(wrapped_lines(&"x".repeat(23), GUIDANCE_FONT_SIZE * 2.0), 2);
    }

    /// Each hard line and each wide-script character receives conservative space.
    #[test]
    fn wrapped_line_estimate_counts_explicit_and_wide_text() {
        let wide = "界".repeat(46);
        assert_eq!(wrapped_lines(&wide, GUIDANCE_FONT_SIZE), 2);
        assert_eq!(
            wrapped_lines(&format!("short\n{wide}"), GUIDANCE_FONT_SIZE),
            3
        );
    }
}
