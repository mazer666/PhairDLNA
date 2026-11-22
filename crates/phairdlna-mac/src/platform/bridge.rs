//! The bridge to the Objective-C runtime.
//!
//! What: the one file of this crate that speaks to the Objective-C runtime.
//!       It defines the class that receives the actions of the menu and the
//!       tick of the timer, and it offers the safe functions around them.
//! Why: AppKit hands the action of a menu item to a *target* object as an
//!       Objective-C message (`ADR-0013`).
//!
//! **The thread.** The handler is stored in the thread of the caller, because
//! AppKit sends the actions of a menu and the ticks of a timer on the main
//! thread. The handler therefore never leaves the main thread.
//!
//! **What the bridge does not do.** It draws nothing and it knows nothing
//! about the server. The bridge only turns an Objective-C message into a
//! number and a number into a call.

#![allow(unsafe_code)] // ADR-0013 — the safety argument of every block is in this file.

// gate: unsafe-exempt ADR-0013 — the safety argument of every block of this
// file is written in ADR-0013 and in the comment above the block itself. This
// file is the only file of the workspace that uses `unsafe`, and the gate
// QG-03 reads this marker before it refuses the file.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationDelegate, NSButton, NSMenuItem, NSWindow, NSWindowDelegate,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer};

use crate::actions::MenuAction;

mod menu_items;
mod selectors;
use selectors::selector_of;

/// The number the timer sends when it ticks.
pub const TICK: i64 = -1;

/// The handler that the Objective-C class calls.
type Handler = Box<dyn Fn(i64)>;

// The handler lives in the thread that installed it, because AppKit sends the
// messages of the menu and of the timer on the main thread.
thread_local! {
    static HANDLER: RefCell<Option<Handler>> = const { RefCell::new(None) };
}

define_class!(
    // SAFETY:
    // - The superclass `NSObject` has no requirement for a subclass.
    // - The class holds no instance variable and implements no `Drop`.
    // - Every method of the class is called on the main thread, which is the
    //   thread the class was created on.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    struct Target;

    impl Target {
        /// The action "Open the administration page".
        #[unsafe(method(phairOpenAdmin:))]
        fn phair_open_admin(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::OpenAdmin.tag());
        }

        /// The action "Open the media folders".
        #[unsafe(method(phairOpenFolders:))]
        fn phair_open_folders(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::OpenMediaFolders.tag());
        }

        /// The action "Read the library again".
        #[unsafe(method(phairRescan:))]
        fn phair_rescan(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::RescanLibrary.tag());
        }

        /// The action "Open the configuration file".
        #[unsafe(method(phairPreferences:))]
        fn phair_preferences(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::OpenPreferences.tag());
        }

        /// The action "Export diagnostics report".
        #[unsafe(method(phairExportDiagnostics:))]
        fn phair_export_diagnostics(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::ExportDiagnostics.tag());
        }

        /// The action "Save settings".
        #[unsafe(method(phairSaveSettings:))]
        fn phair_save_settings(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::SaveSettings.tag());
        }

        /// The action "Remove media folder".
        #[unsafe(method(phairRemoveFolder:))]
        fn phair_remove_folder(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::RemoveMediaFolder.tag());
        }

        /// The action "Add media folders".
        #[unsafe(method(phairAddFolders:))]
        fn phair_add_folders(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::AddMediaFolders.tag());
        }

        /// The action "Start Server".
        #[unsafe(method(phairStartServer:))]
        fn phair_start_server(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::StartServer.tag());
        }

        /// The action "Stop Server".
        #[unsafe(method(phairStopServer:))]
        fn phair_stop_server(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::StopServer.tag());
        }

        /// The action "Quit".
        #[unsafe(method(phairQuit:))]
        fn phair_quit(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::Quit.tag());
        }

        /// The action that switches the view.
        #[unsafe(method(phairTogglePresentation:))]
        fn phair_toggle_presentation(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::TogglePresentation.tag());
        }

        /// The tick of the timer.
        #[unsafe(method(phairTimerFired:))]
        fn phair_timer_fired(&self, timer: &NSTimer) {
            dispatch(TICK);
            let _ = timer;
        }
    }

    unsafe impl NSObjectProtocol for Target {}

    unsafe impl NSApplicationDelegate for Target {
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        fn application_should_handle_reopen(
            &self,
            _sender: &NSApplication,
            _has_visible_windows: bool,
        ) -> bool {
            dispatch(MenuAction::OpenAdmin.tag());
            false
        }
    }

    unsafe impl NSWindowDelegate for Target {
        #[unsafe(method(windowShouldClose:))]
        fn window_should_close(&self, sender: &NSWindow) -> bool {
            sender.orderOut(None);
            false
        }
    }
);

impl Target {
    /// Creates the target.
    ///
    /// # Safety of the inner call
    ///
    /// `init` is the initializer of `NSObject`; the class has no instance
    /// variable, so there is nothing to initialize before it. The call is
    /// made on an object of this very class, and the result is kept alive by
    /// the caller.
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: `NSObject`'s `init` returns the object it was called on.
        unsafe { msg_send![super(this), init] }
    }
}

/// Hands a number to the handler of the thread.
///
/// The handler is taken out of its place while it runs, so that a message
/// that arrives while the handler runs is dropped instead of borrowing the
/// place twice. A message of the runtime that arrives while the application
/// is busy is a message of a later moment anyway (`MB-002`).
fn dispatch(number: i64) {
    HANDLER.with(|cell| {
        let Ok(mut place) = cell.try_borrow_mut() else {
            return;
        };
        if let Some(handler) = place.as_mut() {
            handler(number);
        }
    });
}

/// Installs the handler that the Objective-C class calls.
///
/// The handler is called on the thread that installs it. Installing a second
/// handler replaces the first one; the application installs exactly one.
pub fn install_handler(handler: Handler) {
    HANDLER.with(|cell| {
        *cell.borrow_mut() = Some(handler);
    });
}

/// The target for actions from the menu and native controls.
#[derive(Clone)]
pub struct MenuTarget {
    /// The Objective-C object that receives the messages.
    target: Retained<Target>,
}

impl MenuTarget {
    /// Creates the target on the main thread.
    pub fn new(mtm: MainThreadMarker) -> Self {
        Self {
            target: Target::new(mtm),
        }
    }

    /// Creates a menu item that sends its action to this target.
    ///
    /// # Safety of the inner call
    ///
    /// The selector is one of this crate, and the class above implements each
    /// of them; the target of the item is the object of this class, which is
    /// kept alive as long as the menu is. The initializer of `NSMenuItem` is
    /// called on a freshly allocated item of this class.
    pub fn menu_item(
        &self,
        mtm: MainThreadMarker,
        title: &str,
        action: MenuAction,
    ) -> Retained<NSMenuItem> {
        let text = NSString::from_str(title);
        let empty = NSString::from_str("");
        // SAFETY: `NSMenuItem` is allocated here and initialized here; the
        // selector is one that this crate defines and implements.
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &text,
                Some(selector_of(action)),
                &empty,
            )
        };
        // SAFETY: the target is the object of this crate, which outlives the
        // menu item; the item keeps no ownership of it, and the class of the
        // target is the one that implements the selector.
        unsafe {
            item.setTarget(Some(&*self.target));
        }
        item
    }

    /// Connects a native button to one action implemented by this target.
    pub fn button_action(&self, button: &NSButton, action: MenuAction) {
        // SAFETY: this target implements every selector returned by
        // `selector_of`, and `MenuTarget` retains it for the app lifetime.
        // AppKit sends the selected action to this retained main-thread target.
        unsafe {
            button.setTarget(Some(&*self.target));
            button.setAction(Some(selector_of(action)));
        }
    }

    /// Applies the standard system font to an AppKit menu.
    pub fn apply_standard_menu_font(&self, menu: &objc2_app_kit::NSMenu) {
        let font = objc2_app_kit::NSFont::menuFontOfSize(13.0);
        // SAFETY: AppKit owns this menu on the main thread, and the font is a
        // system menu font of standard size.
        unsafe {
            menu.setFont(Some(&font));
        }
    }

    /// Connects the application delegate to receive reopen events from the Dock.
    pub fn attach_application_delegate(&self, app: &NSApplication) {
        let delegate = ProtocolObject::from_ref(&*self.target);
        app.setDelegate(Some(delegate));
    }

    /// Connects the window delegate to intercept window close requests.
    pub fn attach_window_delegate(&self, window: &NSWindow) {
        let delegate = ProtocolObject::from_ref(&*self.target);
        window.setDelegate(Some(delegate));
    }

    /// Creates the timer that ticks on the main thread.
    ///
    /// # Safety of the inner call
    ///
    /// The selector is `phairTimerFired:`, which the class above implements,
    /// and the target is the object of this class, which outlives the timer.
    pub fn timer(&self, interval: f64) -> Retained<NSTimer> {
        // SAFETY: the selector is one that this crate defines and
        // implements, and the target is the object of this crate.
        unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                interval,
                &self.target,
                sel!(phairTimerFired:),
                None,
                true,
            )
        }
    }
}
