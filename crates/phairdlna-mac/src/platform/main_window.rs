//! Native overview window for the PhairDLNA server.
//!
//! What: the primary native window for server status and shared media folders.
//! Why: ordinary users need a clear home for setup and status without a browser.
//! Example: launching the app opens this standard window; the menu bar remains a quick route back.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSScrollView, NSViewController, NSWindow,
    NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::bridge::MenuTarget;
use crate::app::ServerSettings;
use crate::model::{ServerSnapshot, ServerState};
use crate::text::Strings;

mod layout;
use layout::OverviewViews;

/// The overview window and its responsive, scrollable controls.
pub struct MainWindow {
    window: Retained<NSWindow>,
    views: OverviewViews,
    /// Keeps the AppKit target alive for as long as its button exists.
    _target: MenuTarget,
}

impl MainWindow {
    /// Creates the overview using system window and text styles.
    pub fn new(
        mtm: MainThreadMarker,
        strings: &Strings,
        snapshot: &ServerSnapshot,
        target: MenuTarget,
        notice: Option<&str>,
        folder_action_available: bool,
        settings: &ServerSettings,
    ) -> Self {
        let views = OverviewViews::new(mtm, strings, &target, settings);
        let scroll = NSScrollView::new(mtm);
        scroll.setFrame(NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(600.0, 560.0),
        ));
        scroll.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        scroll.setHasVerticalScroller(true);
        scroll.setHasHorizontalScroller(false);
        scroll.setDocumentView(Some(&views.document));
        let controller = NSViewController::new(mtm);
        controller.setView(&scroll);
        let window = NSWindow::windowWithContentViewController(&controller);
        window.setContentSize(NSSize::new(600.0, 560.0));
        window.setStyleMask(
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable
                | NSWindowStyleMask::Resizable,
        );
        window.setContentMinSize(NSSize::new(600.0, 400.0));
        window.setTitle(&NSString::from_str(strings.text("overview.title")));
        window.center();
        target.attach_window_delegate(&window);

        let result = Self {
            window,
            views,
            _target: target,
        };
        result.update(strings, snapshot, notice, folder_action_available, settings);
        result
    }

    /// Updates visible state, then reflows every section.
    pub fn update(
        &self,
        strings: &Strings,
        snapshot: &ServerSnapshot,
        notice: Option<&str>,
        folder_action_available: bool,
        settings: &ServerSettings,
    ) {
        let content = crate::overview::overview_content(strings, snapshot);
        self.views
            .status
            .setStringValue(&NSString::from_str(&content.state));
        self.update_status_color(snapshot.state);
        self.views
            .library
            .setStringValue(&NSString::from_str(&content.library));
        self.views
            .folders
            .setStringValue(&NSString::from_str(&content.folders));
        self.views
            .connection
            .setStringValue(&NSString::from_str(&content.connection));
        self.views
            .activity
            .setStringValue(&NSString::from_str(&content.activity));
        self.views
            .read_only
            .setStringValue(&NSString::from_str(strings.text("overview.read_only")));
        self.views
            .notice
            .setStringValue(&NSString::from_str(notice.unwrap_or_default()));
        let folder_action = if folder_action_available {
            strings.text("overview.add_folders")
        } else {
            strings.text("overview.folder_setup_unavailable")
        };
        self.views
            .add_folders
            .setTitle(&NSString::from_str(folder_action));
        self.views.add_folders.setEnabled(folder_action_available);
        self.views.controls.set_toggle(
            strings,
            crate::overview::server_toggle(snapshot.state),
            &self._target,
        );
        self.views.controls.set_roots(&settings.roots);
        self.views.reflow();
    }

    /// The values the user typed into the server settings fields.
    pub fn input_values(&self) -> (String, String) {
        self.views.controls.values()
    }

    /// The path of the shared folder the user chose in the list.
    pub fn selected_root(&self) -> Option<String> {
        self.views.controls.selected_root()
    }

    /// Updates the settings fields with the stored values.
    pub fn set_settings(&self, settings: &ServerSettings) {
        self.views.controls.set_values(settings);
        self.views.controls.set_roots(&settings.roots);
    }

    fn update_status_color(&self, state: ServerState) {
        let color = match state {
            ServerState::Checking | ServerState::Stopped => {
                objc2_app_kit::NSColor::secondaryLabelColor()
            }
            ServerState::Running => objc2_app_kit::NSColor::systemGreenColor(),
            ServerState::Scanning => objc2_app_kit::NSColor::systemOrangeColor(),
            ServerState::Refused => objc2_app_kit::NSColor::systemRedColor(),
        };
        self.views.status.setTextColor(Some(&color));
    }

    /// Shows and activates the window.
    pub fn show(&self, mtm: MainThreadMarker) {
        let app = NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Regular);
        self.window.makeKeyAndOrderFront(None);
        app.activate();
    }
}

// `qa/macos-usability-protocol.md` is the manual acceptance gate for rendered behavior.
