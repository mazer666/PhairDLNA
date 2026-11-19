//! Text scaling for the native Preferences controls.
//!
//! What: applies app-owned font sizes and vertical reflow to preference content.
//! Why: larger text must remain reachable without overlapping fixed controls.
//! Example: the expert TOML view uses a larger fixed-pitch font at 200%.

use objc2::rc::Retained;
use objc2_app_kit::{NSFont, NSTextField, NSTextView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

pub(super) struct ScaledTextField {
    pub(super) field: Retained<NSTextField>,
    pub(super) frame: NSRect,
    pub(super) font_size: f64,
    pub(super) wrap_text: Option<String>,
}

impl ScaledTextField {
    pub(super) fn apply(&self, factor: f64) {
        self.field
            .setFont(Some(&NSFont::systemFontOfSize(self.font_size * factor)));
        self.field.setFrame(self.scaled_frame(factor));
    }

    pub(super) fn minimum_document_height(&self, factor: f64) -> f64 {
        let frame = self.scaled_frame(factor);
        let safety_margin = if self.wrap_text.is_some() {
            55.0 * factor
        } else {
            0.0
        };
        frame.origin.y + frame.size.height + safety_margin
    }

    fn scaled_frame(&self, factor: f64) -> NSRect {
        let mut frame = scale_frame(self.frame, factor);
        if let Some(text) = &self.wrap_text {
            let line_count = wrapped_lines(text, self.font_size, frame.size.width, factor);
            frame.size.height = frame.size.height.max(18.0 * factor * line_count as f64);
        }
        frame
    }
}

fn wrapped_lines(text: &str, font_size: f64, width: f64, factor: f64) -> usize {
    let characters_per_line = (width / (font_size * factor)).floor().max(1.0) as usize;
    text.split(is_line_break)
        .map(|line| {
            let character_count = line.chars().count().max(1);
            character_count.div_ceil(characters_per_line)
        })
        .sum()
}

fn is_line_break(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub(super) struct ScaledTextView {
    pub(super) view: Retained<NSTextView>,
    pub(super) font_size: f64,
    pub(super) fixed_pitch: bool,
}

impl ScaledTextView {
    pub(super) fn apply(&self, factor: f64) {
        let font = if self.fixed_pitch {
            NSFont::userFixedPitchFontOfSize(self.font_size * factor)
        } else {
            Some(NSFont::systemFontOfSize(self.font_size * factor))
        };
        self.view.setFont(font.as_deref());
    }
}

pub(super) fn scale_frame(frame: NSRect, factor: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(frame.origin.x, frame.origin.y * factor),
        NSSize::new(frame.size.width, frame.size.height * factor),
    )
}
