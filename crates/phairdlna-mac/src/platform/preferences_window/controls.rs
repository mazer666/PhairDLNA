//! The controls of the native preferences window.
//!
//! What: holds the fields and the views of the preferences with their fonts.
//! Why: the window uses the standard system fonts of macOS, so the controls
//!      need no scaling of their own and no reflow of their own.
//! Example: a field with a long server name keeps the height it needs.

use objc2::rc::Retained;
use objc2_app_kit::{NSFont, NSTextField, NSTextView};
use objc2_foundation::NSRect;

/// The extra height a long value needs below its field.
const WRAP_MARGIN: f64 = 55.0;

/// A field of the preferences window.
pub(super) struct PreferenceField {
    /// The field itself.
    pub(super) field: Retained<NSTextField>,
    /// The frame the field was placed in.
    pub(super) frame: NSRect,
    /// The size of the standard system font of the field.
    pub(super) font_size: f64,
    /// The text that must wrap, when the field holds a long value.
    pub(super) wrap_text: Option<String>,
}

impl PreferenceField {
    /// Applies the standard system font of the field.
    pub(super) fn apply(&self) {
        self.field
            .setFont(Some(&NSFont::systemFontOfSize(self.font_size)));
    }

    /// The height the document of the window needs for this field.
    ///
    /// The height covers the frame of the field and the lines a long value
    /// needs, so that a long server name is not clipped.
    pub(super) fn document_height(&self) -> f64 {
        let margin = match &self.wrap_text {
            Some(text) => {
                WRAP_MARGIN * lines_of(text, self.font_size, self.frame.size.width) as f64
            }
            None => 0.0,
        };
        self.frame.origin.y + self.frame.size.height + margin
    }
}

/// A read-only text view of the preferences window.
pub(super) struct PreferenceTextView {
    /// The view itself.
    pub(super) view: Retained<NSTextView>,
    /// The size of the standard font of the view.
    pub(super) font_size: f64,
    /// Whether the view uses the fixed-pitch font of the system.
    pub(super) fixed_pitch: bool,
}

impl PreferenceTextView {
    /// Applies the standard system font of the view.
    pub(super) fn apply(&self) {
        let font = if self.fixed_pitch {
            NSFont::userFixedPitchFontOfSize(self.font_size)
        } else {
            Some(NSFont::systemFontOfSize(self.font_size))
        };
        self.view.setFont(font.as_deref());
    }
}

/// The number of lines a text needs in a field of the given width.
///
/// The estimate budgets one full em per character, so that a wide script or a
/// fallback glyph does not push a line out of the field.
fn lines_of(text: &str, font_size: f64, width: f64) -> usize {
    let characters_per_line = (width / font_size).floor().max(1.0) as usize;
    text.split(is_line_break)
        .map(|line| line.chars().count().max(1).div_ceil(characters_per_line))
        .sum()
}

/// Whether a character ends a line.
fn is_line_break(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}
