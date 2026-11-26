//! Layout geometry and parameters for single-column and responsive two-column grid.
//!
//! What: defines dimensions, padding, font sizes and responsive layout calculations.
//! Why: keep geometry definitions centralized and manageable within budget.

/// Standard horizontal margin.
pub(super) const MARGIN: f64 = 24.0;
/// Threshold width at which layout shifts from 1 column to 2 columns (landscape / widescreen).
pub(super) const TWO_COL_BREAKPOINT: f64 = 750.0;
/// Inner horizontal padding for card boxes.
pub(super) const PAD: f64 = 16.0;
/// Space between cards and sections.
pub(super) const GAP: f64 = 14.0;
/// Tight space between related elements (e.g. heading and content).
pub(super) const TIGHT_GAP: f64 = 6.0;
/// Height of standard buttons.
pub(super) const BUTTON_HEIGHT: f64 = 30.0;
/// Height of input fields.
pub(super) const FIELD_HEIGHT: f64 = 24.0;
/// Height of field labels.
pub(super) const LABEL_HEIGHT: f64 = 18.0;
/// Height of pop-up picker.
pub(super) const PICKER_HEIGHT: f64 = 26.0;
/// Space below the lowest section.
pub(super) const BOTTOM_SPACE: f64 = 30.0;

/// Font sizes across sections.
pub(super) const HEADING_SIZE: f64 = 24.0;
pub(super) const STATUS_SIZE: f64 = 18.0;
pub(super) const SECTION_SIZE: f64 = 14.0;
pub(super) const BODY_SIZE: f64 = 13.0;
pub(super) const GUIDANCE_SIZE: f64 = 12.0;
pub(super) const LINE_HEIGHT: f64 = 19.0;

/// Layout mode and computed column parameters.
#[derive(Clone, Copy, Debug)]
pub(super) struct GridLayout {
    pub is_two_col: bool,
    pub col1_x: f64,
    pub col2_x: f64,
    pub col_width: f64,
    pub inner_width: f64,
}

impl GridLayout {
    /// Computes the grid parameters for a given viewport width.
    pub(super) fn for_width(width: f64) -> Self {
        if width >= TWO_COL_BREAKPOINT {
            let total_usable = (width - 2.0 * MARGIN - GAP).max(200.0);
            let col_w = (total_usable / 2.0).floor();
            Self {
                is_two_col: true,
                col1_x: MARGIN,
                col2_x: MARGIN + col_w + GAP,
                col_width: col_w,
                inner_width: col_w - 2.0 * PAD,
            }
        } else {
            let col_w = (width - 2.0 * MARGIN).max(300.0);
            Self {
                is_two_col: false,
                col1_x: MARGIN,
                col2_x: MARGIN,
                col_width: col_w,
                inner_width: col_w - 2.0 * PAD,
            }
        }
    }
}
