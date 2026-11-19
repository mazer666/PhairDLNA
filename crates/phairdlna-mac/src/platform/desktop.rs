//! The application on the screen.
//!
//! What: the part that owns the menu bar item, the state of the application
//!       and the thread that talks to the administration surface.
//! Why: the classes of AppKit may be used only from the main thread, and the
//!      main thread must draw, must not wait for the network and must not
//!      carry the logic of the application (`MB-002`).
//! Example: the timer of the menu bar ticks once a second, takes the answer
//!          of the worker and writes the menu again.
//!
//! **The thread.** Everything in this module belongs to the main thread. The
//! worker carries the requests to the surface and answers them on another
//! thread; the main thread only takes the answers.

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_foundation::NSTimer;

use crate::actions::MenuAction;
use crate::app::{App, DesktopRequest};
use crate::worker::{Answer, Command, Worker};

use super::bridge;
use super::status_item::StatusItem;
use super::workspace;

mod windows;

/// How often the timer of the menu bar ticks, in seconds.
///
/// The tick is the pace at which the application asks the surface for the
/// state of the server. The target of the project is a state that is shown
/// within half a second; the number here is the pace of the first version
/// and not a measured result (`PERF-011`).
pub const TICK_SECONDS: f64 = 1.0;

/// The application on the screen of the user.
pub struct Desktop {
    /// The main thread of the application.
    marker: MainThreadMarker,
    /// The state of the application and the texts the user reads.
    app: App,
    /// The thread that talks to the administration surface.
    worker: Worker,
    /// The item in the menu bar.
    item: StatusItem,
    /// The timer that ticks on the main thread.
    ///
    /// The field is never read: it keeps the timer alive for as long as the
    /// application runs, so that the timer of the system does not stop.
    _timer: Retained<NSTimer>,
    /// The entries that the menu shows now, so that the menu is built again
    /// only when they change.
    written: Vec<crate::menu::MenuEntry>,
    /// The preferences window when open.
    prefs_window: Option<super::preferences_window::PreferencesWindow>,
    /// The native server overview when it has been opened.
    main_window: Option<super::main_window::MainWindow>,
}

impl Desktop {
    /// Builds the application and puts it into the menu bar.
    ///
    /// The answer is an error when macOS does not give the application a
    /// place in the menu bar; the application then says so instead of
    /// pretending to run.
    pub fn new(marker: MainThreadMarker, app: App, worker: Worker) -> Result<Self, String> {
        let Some(item) = StatusItem::new(marker) else {
            return Err("no-button".to_owned());
        };
        let timer = item.timer(TICK_SECONDS);
        let mut desktop = Self {
            marker,
            app,
            worker,
            item,
            _timer: timer,
            written: Vec::new(),
            prefs_window: None,
            main_window: None,
        };
        desktop.write();
        desktop.show_initial_configuration_guidance();
        desktop.show_main_window();
        Ok(desktop)
    }

    /// Handles a message of the system.
    ///
    /// The number is the tick of the timer or the number of an action of the
    /// menu. A number that belongs to no action is ignored (`MB-002`).
    pub fn handle(&mut self, number: i64) {
        if number == bridge::TICK {
            self.tick();
            return;
        }
        let Some(action) = MenuAction::from_tag(number) else {
            return;
        };
        let outcome = self.app.perform(action);
        self.ask_worker(action);
        self.apply(outcome);
        self.write();
    }

    /// The tick of the timer.
    ///
    /// The tick takes the answers that are there and asks for a new state. A
    /// scan that nobody answered for ends, so that the menu does not claim
    /// that a scan runs when none does.
    fn tick(&mut self) {
        while let Some(answer) = self.worker.take() {
            self.take(answer);
        }
        self.app.end_of_a_silent_scan();
        self.worker.ask(Command::Refresh);
        self.write();
    }

    /// Takes an answer of the worker.
    fn take(&mut self, answer: Answer) {
        match answer {
            Answer::State(snapshot) => {
                self.app.take_state(snapshot);
            }
            Answer::Scanned(report) => {
                self.app.take_rescan(&report);
            }
            Answer::Diagnostics(json) => {
                self.save_diagnostics(&json);
            }
            Answer::Failed(error) => {
                if self.app.is_scanning() {
                    self.app.take_failed_rescan(&error);
                } else {
                    self.app.take_failure(&error);
                }
            }
        }
    }

    /// Asks the worker for what the action needs.
    ///
    /// The scan of the library is the work of the worker: the application
    /// shows that it runs and the worker carries it out.
    fn ask_worker(&mut self, action: MenuAction) {
        if action == MenuAction::RescanLibrary {
            self.worker.ask(Command::Rescan);
        }
    }

    /// Carries out the request of an action of the menu.
    fn apply(&mut self, outcome: crate::app::ActionOutcome) {
        let crate::app::ActionOutcome::Request(request) = outcome else {
            return;
        };
        match request {
            DesktopRequest::OpenPath(path) => {
                if !workspace::open_path(&path) {
                    self.app.set_notice(self.app.cannot_open_text());
                }
            }
            DesktopRequest::OpenPaths(paths) => self.open_paths(&paths),
            DesktopRequest::ShowPreferences => self.show_preferences(),
            DesktopRequest::ShowMainWindow => self.show_main_window(),
            DesktopRequest::ExportDiagnostics => {
                let _ = self.worker.ask(Command::ExportDiagnostics);
            }
            DesktopRequest::Quit => self.quit(),
        }
    }

    /// Writes the state of the application into the menu bar.
    ///
    /// The menu is built again only when the entries change, so that the menu
    /// of the user does not flicker on every tick of the timer.
    fn write(&mut self) {
        let entries = self.app.entries();
        if let Some(ref window) = self.main_window {
            window.update(
                self.app.strings(),
                self.app.snapshot(),
                self.app.text_scale(),
                self.app.notice(),
                self.app.can_add_media_roots(),
            );
        }
        if let Some(ref window) = self.prefs_window {
            window.update_text_scale(self.app.text_scale());
        }
        if entries == self.written {
            return;
        }
        self.written = entries.clone();
        self.item
            .set_entries(self.marker, &entries, self.app.text_scale().factor());
        self.item.set_state(&self.app.snapshot().state);
        self.item.set_badge(&self.app.badge());
        let (label, value) = self.app.accessibility();
        self.item.set_accessibility(&label, &value);
    }
}

/// The application on the screen, shared with the handler of the runtime.
///
/// The application lives in a cell, because the handler of the Objective-C
/// runtime and the timer of the menu bar both reach it, and both run on the
/// main thread.
pub type Shared = Rc<RefCell<Desktop>>;

/// Makes the application that the handler of the runtime reaches.
pub fn shared(desktop: Desktop) -> Shared {
    Rc::new(RefCell::new(desktop))
}

/// Installs the handler that the runtime calls.
///
/// The handler runs on the main thread, and it takes the answer of the
/// worker and writes the menu.
pub fn install_handler(desktop: Shared) {
    bridge::install_handler(Box::new(move |number| {
        // A message that arrives while the application is busy is a message
        // of a later moment: the borrow that fails is a message that is
        // dropped and not a panic.
        if let Ok(mut application) = desktop.try_borrow_mut() {
            application.handle(number);
        }
    }));
}
