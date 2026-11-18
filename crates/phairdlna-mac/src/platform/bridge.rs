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
use objc2::runtime::{AnyObject, Sel};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::{NSObject, NSString, NSTimer};

use crate::actions::MenuAction;

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

        /// The action "Start server".
        #[unsafe(method(phairStartServer:))]
        fn phair_start_server(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::StartServer.tag());
        }

        /// The action "Stop server".
        #[unsafe(method(phairStopServer:))]
        fn phair_stop_server(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::StopServer.tag());
        }

        /// The action "Add media folders".
        #[unsafe(method(phairAddFolders:))]
        fn phair_add_folders(&self, sender: &AnyObject) {
            let _ = sender;
            dispatch(MenuAction::AddMediaFolders.tag());
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

/// The selector of an action.
///
/// Every action has its own selector, because AppKit sends the action of a
/// menu item to the target without telling which entry the user chose.
pub fn selector_of(action: MenuAction) -> Sel {
    match action {
        MenuAction::OpenAdmin => sel!(phairOpenAdmin:),
        MenuAction::OpenMediaFolders => sel!(phairOpenFolders:),
        MenuAction::RescanLibrary => sel!(phairRescan:),
        MenuAction::OpenPreferences => sel!(phairPreferences:),
        MenuAction::AddMediaFolders => sel!(phairAddFolders:),
        MenuAction::ExportDiagnostics => sel!(phairExportDiagnostics:),
        MenuAction::StartServer => sel!(phairStartServer:),
        MenuAction::StopServer => sel!(phairStopServer:),
        MenuAction::Quit => sel!(phairQuit:),
        MenuAction::TogglePresentation => sel!(phairTogglePresentation:),
    }
}

/// The target that receives the actions of the menu.
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

    /// Creates a menu item that cannot be chosen.
    ///
    /// The item carries a line of text and no action, so that the user can
    /// read it and cannot trigger it.
    ///
    /// # Safety of the inner call
    ///
    /// The initializer of `NSMenuItem` is called on a freshly allocated item
    /// of this class, and the item keeps no ownership of the target.
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
