//! The vertical size of the overview document.
//!
//! What: keeps short content balanced and preserves the full height of long content.
//! Why: the scroll view should not leave a large empty band or clip a section.
//! Example: a 400-point document in a 560-point viewport grows to 480 points.

/// Returns a document height that balances short content in the viewport.
pub(super) fn centered_document_height(flow_height: f64, viewport_height: f64) -> f64 {
    let centering_space = ((viewport_height - flow_height).max(0.0)) / 2.0;
    flow_height + centering_space
}

#[cfg(test)]
mod tests {
    use super::centered_document_height;

    /// Short content receives equal free space above and below.
    #[test]
    fn short_content_uses_half_of_the_available_space() {
        assert_eq!(centered_document_height(400.0, 560.0), 480.0);
    }

    /// Content that needs scrolling is never compressed.
    #[test]
    fn long_content_keeps_its_full_height() {
        assert_eq!(centered_document_height(720.0, 560.0), 720.0);
    }

    /// A document that exactly fills the viewport gains no extra height.
    #[test]
    fn content_equal_to_the_viewport_stays_the_same_size() {
        assert_eq!(centered_document_height(560.0, 560.0), 560.0);
    }
}
