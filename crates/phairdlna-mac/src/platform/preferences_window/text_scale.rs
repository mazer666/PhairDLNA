//! Text-size controls for the native preferences window.
//!
//! What: builds and updates the 100%, 150% and 200% radio controls.
//! Why: the app-owned accessibility preference belongs to the app, not server config.
//! Example: `select` highlights the setting currently applied to the overview.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSButton, NSButtonType, NSControlStateValueOff, NSControlStateValueOn, NSTextField, NSView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::super::bridge::MenuTarget;
use crate::actions::MenuAction;
use crate::model::TextScale;
use crate::text::Strings;

/// Adds the localized label and controls to preferences.
pub(super) fn build_controls(
    mtm: MainThreadMarker,
    strings: &Strings,
    target: &MenuTarget,
    content: &NSView,
) -> (Retained<NSTextField>, Vec<Retained<NSButton>>) {
    let label = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.text_size")),
        mtm,
    );
    label.setFrame(NSRect::new(
        NSPoint::new(18.0, 16.0),
        NSSize::new(118.0, 28.0),
    ));
    content.addSubview(&label);

    let mut buttons = Vec::new();
    for (x, title, action) in [
        (140.0, "100%", MenuAction::TextSize100),
        (235.0, "150%", MenuAction::TextSize150),
        (330.0, "200%", MenuAction::TextSize200),
    ] {
        let button = NSButton::new(mtm);
        button.setButtonType(NSButtonType::Radio);
        button.setTitle(&NSString::from_str(title));
        button.setFrame(NSRect::new(NSPoint::new(x, 12.0), NSSize::new(82.0, 32.0)));
        target.button_action(&button, action);
        content.addSubview(&button);
        buttons.push(button);
    }
    (label, buttons)
}

/// Shows which app-owned text size is selected.
pub(super) fn select(buttons: &[Retained<NSButton>], scale: TextScale) {
    let selected_index = match scale {
        TextScale::Percent100 => 0,
        TextScale::Percent150 => 1,
        TextScale::Percent200 => 2,
    };
    for (index, button) in buttons.iter().enumerate() {
        button.setState(if index == selected_index {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
    }
}
