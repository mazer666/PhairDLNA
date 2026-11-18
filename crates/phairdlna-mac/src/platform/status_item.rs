//! The status item and its menu.
//!
//! What: the small item in the menu bar and the menu that opens when the user
//!       clicks it.
//! Why: the menu bar is the one place the user looks without leaving what
//!       they are doing, so the state of the server belongs there (`F-15`).
//! Example: the item shows the symbol of the state; the menu shows the state
//!          as a sentence and the six actions.
//!
//! **The thread.** Everything in this module belongs to the main thread: the
//! AppKit classes of a menu may be used only there (`MB-002`).

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAccessibility, NSMenu, NSMenuItem, NSStatusBar, NSStatusBarButton, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSString, NSTimer};

use crate::menu::MenuEntry;
use crate::model::ServerState;

use super::bridge::MenuTarget;
use super::icon;

/// The item in the menu bar and the menu that belongs to it.
pub struct StatusItem {
    /// The item in the menu bar.
    ///
    /// The field is never read: it keeps the item of the menu bar alive for
    /// as long as the application runs.
    _item: Retained<NSStatusItem>,
    /// The menu of the item.
    menu: Retained<NSMenu>,
    /// The button of the item; it carries the picture and the labels.
    button: Retained<NSStatusBarButton>,
    /// The target that receives the actions of the menu.
    target: MenuTarget,
}

impl StatusItem {
    /// Creates the item and its empty menu.
    ///
    /// The answer is `None` when macOS does not give the item a button; that
    /// happens on a machine without a menu bar, and the application then says
    /// so instead of pretending to run.
    pub fn new(mtm: MainThreadMarker) -> Option<Self> {
        let bar = NSStatusBar::systemStatusBar();
        let item = bar.statusItemWithLength(NSVariableStatusItemLength);
        let button = item.button(mtm)?;
        let menu = NSMenu::new(mtm);
        item.setMenu(Some(&menu));
        Some(Self {
            _item: item,
            menu,
            button,
            target: MenuTarget::new(mtm),
        })
    }

    /// Builds an NSMenuItem for a menu action with icon and enabled state.
    fn action_item(
        &self,
        mtm: MainThreadMarker,
        title: &str,
        action: &crate::actions::MenuAction,
        enabled: bool,
        tooltip: Option<&str>,
    ) -> objc2::rc::Retained<NSMenuItem> {
        let item = self.target.menu_item(mtm, title, *action);
        item.setEnabled(enabled);
        if let Some(tip) = tooltip {
            let tip_ns = NSString::from_str(tip);
            item.setToolTip(Some(&tip_ns));
        }
        if let Some(icon) = icon::action_image_of(action) {
            item.setImage(Some(&icon));
        }
        item
    }

    /// Writes the entries of the menu.
    ///
    /// The menu is built again from the entries, so that the menu and the
    /// model of the menu can never disagree.
    pub fn set_entries(&mut self, mtm: MainThreadMarker, entries: &[MenuEntry]) {
        self.menu.removeAllItems();
        for entry in entries {
            match entry {
                MenuEntry::Status { title } => {
                    let item = self.target.status_item(mtm, title);
                    self.menu.addItem(&item);
                }
                MenuEntry::Separator => {
                    let separator = NSMenuItem::separatorItem(mtm);
                    self.menu.addItem(&separator);
                }
                MenuEntry::Action {
                    title,
                    action,
                    enabled,
                    tooltip,
                } => {
                    let item = self.action_item(mtm, title, action, *enabled, tooltip.as_deref());
                    self.menu.addItem(&item);
                }
            }
        }
    }

    /// Shows the symbol of a state.
    pub fn set_state(&self, state: &ServerState) {
        if let Some(image) = icon::image_of(state) {
            self.button.setImage(Some(&image));
        }
    }

    /// Writes the short text of the item.
    ///
    /// The text is short on purpose: the menu bar is narrow, and a wide text
    /// would push the other items of the user out of the screen (`MB-006`).
    pub fn set_badge(&self, text: &str) {
        let text = NSString::from_str(text);
        self.button.setTitle(&text);
    }

    /// Names the item for the screen reader.
    ///
    /// The label says what the item is, the value says what the server does.
    /// Both are text, so that the state is never carried by the picture alone
    /// (`UX-009`, `UX-018`).
    pub fn set_accessibility(&self, label: &str, value: &str) {
        let label = NSString::from_str(label);
        let value = NSString::from_str(value);
        self.button.setAccessibilityLabel(Some(&label));
        self.button.setAccessibilityValueDescription(Some(&value));
    }

    /// The timer that ticks on the main thread.
    pub fn timer(&self, interval: f64) -> Retained<NSTimer> {
        self.target.timer(interval)
    }
}
