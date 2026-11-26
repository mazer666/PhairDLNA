//! Responsive layout for the native overview.
//!
//! What: places overview sections according to the length of their text.
//! Why: a long text needs more rows and must push the later sections down.
//! Example: a long folder name gets extra rows and the overview scrolls.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAccessibility, NSAutoresizingMaskOptions, NSBox, NSButton, NSTextField, NSView,
};
use objc2_foundation::NSString;

use super::super::bridge::MenuTarget;
use super::super::icon;
use crate::actions::MenuAction;
use crate::app::ServerSettings;
use crate::text::Strings;

mod controls;
mod geometry;
mod grid;
mod placement;
mod reflow;
use controls::ServerControls;

/// The controls that form the scrolling overview document.
pub(super) struct OverviewViews {
    pub(super) document: Retained<NSView>,
    pub(super) status_card: Retained<NSBox>,
    pub(super) folders_card: Retained<NSBox>,
    pub(super) settings_card: Retained<NSBox>,
    pub(super) info_card: Retained<NSBox>,
    pub(super) heading: Retained<NSTextField>,
    pub(super) status: Retained<NSTextField>,
    pub(super) library: Retained<NSTextField>,
    pub(super) folders_heading: Retained<NSTextField>,
    pub(super) folders: Retained<NSTextField>,
    pub(super) connection_heading: Retained<NSTextField>,
    pub(super) connection: Retained<NSTextField>,
    pub(super) activity_heading: Retained<NSTextField>,
    pub(super) activity: Retained<NSTextField>,
    pub(super) notice: Retained<NSTextField>,
    pub(super) add_folders: Retained<NSButton>,
    pub(super) rescan_button: Retained<NSButton>,
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
        let (status_card, folders_card, settings_card, info_card) = build_cards(mtm, target);
        let (heading, status, library, folders_heading, folders) =
            build_status_labels(mtm, strings);
        let (connection_heading, connection, activity_heading, activity, notice, read_only) =
            build_info_labels(mtm, strings);
        let add_folders = folder_button(mtm, strings, target);
        let rescan_button = rescan_button(mtm, strings, target);
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

        assemble_document(
            &document,
            &[&status_card, &folders_card, &settings_card, &info_card],
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
            &[&add_folders, &rescan_button],
            &controls,
        );

        Self {
            document,
            status_card,
            folders_card,
            settings_card,
            info_card,
            heading,
            status,
            library,
            folders_heading,
            folders,
            connection_heading,
            connection,
            activity_heading,
            activity,
            notice,
            add_folders,
            rescan_button,
            read_only,
            controls,
        }
    }
}

type Cards = (
    Retained<NSBox>,
    Retained<NSBox>,
    Retained<NSBox>,
    Retained<NSBox>,
);

fn build_cards(mtm: MainThreadMarker, target: &MenuTarget) -> Cards {
    (
        card_box(mtm, target),
        card_box(mtm, target),
        card_box(mtm, target),
        card_box(mtm, target),
    )
}

type StatusLabels = (
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
);

fn build_status_labels(mtm: MainThreadMarker, strings: &Strings) -> StatusLabels {
    (
        label(mtm, strings.text("overview.heading")),
        label(mtm, ""),
        label(mtm, ""),
        label(mtm, strings.text("overview.folders")),
        label(mtm, ""),
    )
}

type InfoLabels = (
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
    Retained<NSTextField>,
);

fn build_info_labels(mtm: MainThreadMarker, strings: &Strings) -> InfoLabels {
    let notice = label(mtm, "");
    notice.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
    let read_only = label(mtm, strings.text("overview.read_only"));
    read_only.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
    (
        label(mtm, strings.text("overview.connection_title")),
        label(mtm, ""),
        label(mtm, strings.text("overview.activity_title")),
        label(mtm, ""),
        notice,
        read_only,
    )
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

/// Creates the library rescan button.
fn rescan_button(
    mtm: MainThreadMarker,
    strings: &Strings,
    target: &MenuTarget,
) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&NSString::from_str(strings.text("overview.rescan")));
    let img = icon::action_image_of(&MenuAction::RescanLibrary);
    target.configure_push_button(&button, img.as_deref());
    target.button_action(&button, MenuAction::RescanLibrary);
    button
}

/// Assembles all cards, labels, buttons and controls into the document view.
fn assemble_document(
    document: &NSView,
    cards: &[&NSBox],
    labels: &[&NSTextField],
    buttons: &[&NSButton],
    controls: &ServerControls,
) {
    for card in cards {
        document.addSubview(card);
    }
    for view in labels {
        document.addSubview(view);
    }
    for button in buttons {
        document.addSubview(button);
    }
    add_controls(document, controls);
}

/// Puts the controls that change the server into the document.
fn add_controls(document: &NSView, controls: &ServerControls) {
    document.addSubview(&controls.toggle);
    document.addSubview(&controls.folder_picker);
    document.addSubview(&controls.reveal_folder);
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

fn card_box(mtm: MainThreadMarker, target: &MenuTarget) -> Retained<NSBox> {
    let card = NSBox::new(mtm);
    target.configure_card_box(&card);
    card
}

fn label(mtm: MainThreadMarker, text: &str) -> Retained<NSTextField> {
    let field = NSTextField::wrappingLabelWithString(&NSString::from_str(text), mtm);
    field.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
    field
}
