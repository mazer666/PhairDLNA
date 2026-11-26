//! Status-only menu items and native control styling that cannot dispatch application actions.
//!
//! What: creates disabled labels for non-interactive menu status and styles buttons.
//! Why: action items, status rows and control bezels have AppKit safety behavior.
//! Example: the server state is shown as a disabled menu row.

// gate: unsafe-exempt ADR-0013 — menu item and button initialization uses the Objective-C AppKit bridge.

use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBezelStyle, NSBox, NSBoxType, NSButton, NSCellImagePosition, NSColor, NSImage, NSMenuItem,
    NSTextField, NSTextFieldBezelStyle,
};
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

    /// Styles an AppKit button as a native push button, optionally with an icon.
    pub fn configure_push_button(&self, button: &NSButton, icon: Option<&NSImage>) {
        button.setBezelStyle(NSBezelStyle::Push);
        if let Some(image) = icon {
            button.setImage(Some(image));
            button.setImagePosition(NSCellImagePosition::ImageLeading);
        }
    }

    /// Styles an editable text field with rounded borders.
    pub fn configure_rounded_field(&self, field: &NSTextField) {
        field.setBezelStyle(NSTextFieldBezelStyle::RoundedBezel);
        field.setBezeled(true);
    }

    /// Styles an NSBox as a macOS card container with rounded corners and subtle background.
    pub fn configure_card_box(&self, box_view: &NSBox) {
        box_view.setBoxType(NSBoxType::Custom);
        box_view.setCornerRadius(10.0);
        box_view.setBorderWidth(1.0);
        box_view.setBorderColor(&NSColor::separatorColor());
        box_view.setFillColor(&NSColor::quaternarySystemFillColor());
    }
}
