//! Status-only menu items that cannot dispatch application actions.
//!
//! What: creates disabled labels for non-interactive menu status.
//! Why: action items and status rows have different safety behavior.
//! Example: the server state is shown as a disabled menu row.

use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::NSString;

use super::MenuTarget;

impl MenuTarget {
    /// Creates a readable menu item without an action the user can trigger.
    ///
    /// # Safety
    ///
    /// The allocated item has no action, so it cannot dispatch an unknown selector.
    pub fn status_item(&self, mtm: MainThreadMarker, title: &str) -> Retained<NSMenuItem> {
        let text = NSString::from_str(title);
        let empty = NSString::from_str("");
        // SAFETY: `NSMenuItem` is allocated here and initialized here; the
        // item has no action and therefore no selector to be wrong about.
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &text,
                None,
                &empty,
            )
        };
        item.setEnabled(false);
        item
    }
}
