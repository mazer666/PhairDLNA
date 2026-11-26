//! Responsive layout for the native overview.
//!
//! What: places overview sections according to the length of their text.
//! Why: a long text needs more rows and must push the later sections down.
//! Example: a long folder name gets extra rows and the overview scrolls.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAccessibility, NSAutoresizingMaskOptions, NSBox, NSBoxType, NSButton, NSTextField, NSView,
};
use objc2_foundation::NSString;

use super::super::bridge::MenuTarget;
use super::super::icon;
use crate::actions::MenuAction;
use crate::app::ServerSettings;
use crate::text::Strings;

mod controls;
mod geometry;
mod reflow;
use controls::ServerControls;

/// The controls that form the scrolling overview document.
pub(super) struct OverviewViews {
    pub(super) document: Retained<NSView>,
    pub(super) heading: Retained<NSTextField>,
    pub(super) status: Retained<NSTextField>,
    pub(super) library: Retained<NSTextField>,
    pub(super) folders_heading: Retained<NSTextField>,
    pub(super) folders: Retained<NSTextField>,
    pub(super) folders_rule: Retained<NSBox>,
    pub(super) connection_heading: Retained<NSTextField>,
    pub(super) connection: Retained<NSTextField>,
    pub(super) connection_rule: Retained<NSBox>,
    pub(super) activity_heading: Retained<NSTextField>,
    pub(super) activity: Retained<NSTextField>,
    pub(super) notice: Retained<NSTextField>,
    pub(super) add_folders: Retained<NSButton>,
    pub(super) read_only: Retained<NSTextField>,
    /// The controls that change the server.
    pub(super) controls: ServerControls,
}

impl OverviewViews {
    /// Builds every overview control once; later updates change text and frames.
    pub(super) fn new(
        mtm: MainThreadMarker,
        strings: &Strings,
        target: &MenuTarget,
        settings: &ServerSettings,
    ) -> Self {
        let document = NSView::new(mtm);
        let heading = label(mtm, strings.text("overview.heading"));
        let (status, library) = (label(mtm, ""), label(mtm, ""));
        let folders_heading = label(mtm, strings.text("overview.folders"));
        let (folders, folders_rule) = (label(mtm, ""), separator(mtm));
        let connection_heading = label(mtm, strings.text("overview.connection_title"));
        let (connection, connection_rule) = (label(mtm, ""), separator(mtm));
        let activity_heading = label(mtm, strings.text("overview.activity_title"));
        let (activity, notice) = (label(mtm, ""), label(mtm, ""));
        notice.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
        let add_folders = folder_button(mtm, strings, target);
        let read_only = label(mtm, strings.text("overview.read_only"));
        read_only.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
        let controls = ServerControls::new(mtm, strings, target, settings);

        set_accessibility_labels(
            strings,
            &status,
            &library,
            &folders,
            &connection,
            &activity,
            &notice,
        );

        add_subviews(
            &document,
            &[
                &heading,
                &status,
                &library,
                &folders_heading,
                &folders,
                &connection_heading,
                &connection,
                &activity_heading,
                &activity,
                &notice,
                &read_only,
            ],
            &[&folders_rule, &connection_rule],
            &add_folders,
            &controls,
        );

        Self {
            document,
            heading,
            status,
            library,
            folders_heading,
            folders,
            folders_rule,
            connection_heading,
            connection,
            connection_rule,
            activity_heading,
            activity,
            notice,
            add_folders,
            read_only,
            controls,
        }
    }
}

/// Creates the add folders button.
fn folder_button(
    mtm: MainThreadMarker,
    strings: &Strings,
    target: &MenuTarget,
) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&NSString::from_str(strings.text("overview.add_folders")));
    let img = icon::action_image_of(&MenuAction::AddMediaFolders);
    target.configure_push_button(&button, img.as_deref());
    target.button_action(&button, MenuAction::AddMediaFolders);
    button
}

/// Puts the views that form the document into the scrollable view.
fn add_subviews(
    document: &NSView,
    labels: &[&NSTextField],
    rules: &[&NSBox],
    button: &NSButton,
    controls: &ServerControls,
) {
    for view in labels {
        document.addSubview(view);
    }
    for rule in rules {
        document.addSubview(rule);
    }
    document.addSubview(button);
    add_controls(document, controls);
}

/// Puts the controls that change the server into the document.
fn add_controls(document: &NSView, controls: &ServerControls) {
    document.addSubview(&controls.toggle);
    document.addSubview(&controls.folder_picker);
    document.addSubview(&controls.remove_folder);
    document.addSubview(&controls.device_name_label);
    document.addSubview(&controls.device_name_field);
    document.addSubview(&controls.http_port_label);
    document.addSubview(&controls.http_port_field);
    document.addSubview(&controls.save_settings);
}

fn set_accessibility_labels(
    strings: &Strings,
    status: &NSTextField,
    library: &NSTextField,
    folders: &NSTextField,
    connection: &NSTextField,
    activity: &NSTextField,
    notice: &NSTextField,
) {
    let labels = [
        (status, "overview.accessibility.status"),
        (library, "overview.accessibility.library"),
        (folders, "overview.accessibility.folders"),
        (connection, "overview.accessibility.connection"),
        (activity, "overview.accessibility.activity"),
        (notice, "overview.accessibility.notice"),
    ];
    for (field, key) in labels {
        field.setAccessibilityLabel(Some(&NSString::from_str(strings.text(key))));
    }
}

fn label(mtm: MainThreadMarker, text: &str) -> Retained<NSTextField> {
    let field = NSTextField::wrappingLabelWithString(&NSString::from_str(text), mtm);
    field.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
    field
}

fn separator(mtm: MainThreadMarker) -> Retained<NSBox> {
    let rule = NSBox::new(mtm);
    rule.setBoxType(NSBoxType::Separator);
    rule
}
