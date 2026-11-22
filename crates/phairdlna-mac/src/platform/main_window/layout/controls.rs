//! The controls that change the server in the native overview.
//!
//! What: builds the button that starts and stops the server, the fields of
//!       the settings, the button that saves them and the button that removes
//!       a shared folder.
//! Why: the overview is the place where the user steers the server, and every
//!      text of these controls comes from a resource file (`UX-020`).
//! Example: the server name field holds the name a client sees in its list.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAccessibility, NSButton, NSPopUpButton, NSTextField};
use objc2_foundation::NSString;

use super::super::super::bridge::MenuTarget;
use crate::actions::MenuAction;
use crate::app::{ConfiguredRoot, ServerSettings};
use crate::overview::ServerToggle;
use crate::text::Strings;

/// The controls of the overview that change the server.
pub(in crate::platform::main_window) struct ServerControls {
    /// The button that starts and stops the server.
    pub(super) toggle: Retained<NSButton>,
    /// The list of the shared folders the user may remove.
    pub(super) folder_picker: Retained<NSPopUpButton>,
    /// The button that removes the chosen folder.
    pub(super) remove_folder: Retained<NSButton>,
    /// The label of the server name.
    pub(super) device_name_label: Retained<NSTextField>,
    /// The field of the server name.
    pub(super) device_name_field: Retained<NSTextField>,
    /// The label of the HTTP port.
    pub(super) http_port_label: Retained<NSTextField>,
    /// The field of the HTTP port.
    pub(super) http_port_field: Retained<NSTextField>,
    /// The button that saves the settings.
    pub(super) save_settings: Retained<NSButton>,
    /// The paths of the folders, in the order of the list.
    roots: RefCell<Vec<String>>,
}

impl ServerControls {
    /// Builds the controls and fills them with the settings of the server.
    pub(super) fn new(
        mtm: MainThreadMarker,
        strings: &Strings,
        target: &MenuTarget,
        settings: &ServerSettings,
    ) -> Self {
        let toggle = button(mtm, "", MenuAction::StartServer, target);
        let folder_picker = NSPopUpButton::new(mtm);
        let remove_folder = button(
            mtm,
            strings.text("overview.remove_folder"),
            MenuAction::RemoveMediaFolder,
            target,
        );
        let (device_name_label, device_name_field) = editable_field(
            mtm,
            strings.text("overview.server_name"),
            &settings.device_name,
        );
        let (http_port_label, http_port_field) = editable_field(
            mtm,
            strings.text("overview.http_port"),
            &port_text(settings.http_port),
        );
        let save_settings = button(
            mtm,
            strings.text("overview.save_settings"),
            MenuAction::SaveSettings,
            target,
        );
        let controls = Self {
            toggle,
            folder_picker,
            remove_folder,
            device_name_label,
            device_name_field,
            http_port_label,
            http_port_field,
            save_settings,
            roots: RefCell::new(Vec::new()),
        };
        controls.set_roots(&settings.roots);
        controls
    }

    /// Shows the control that starts or stops the server.
    pub(in crate::platform::main_window) fn set_toggle(
        &self,
        strings: &Strings,
        toggle: ServerToggle,
        target: &MenuTarget,
    ) {
        self.toggle
            .setTitle(&NSString::from_str(strings.text(toggle.title_key)));
        target.button_action(&self.toggle, toggle.action);
        self.toggle.setEnabled(toggle.enabled);
    }

    /// Fills the list of the shared folders.
    ///
    /// The list holds the name of every folder; the path of a folder is kept
    /// beside it, because the name of a folder may be given twice and only
    /// the path identifies it.
    pub(in crate::platform::main_window) fn set_roots(&self, roots: &[ConfiguredRoot]) {
        self.folder_picker.removeAllItems();
        self.roots.borrow_mut().clear();
        for root in roots {
            self.folder_picker
                .addItemWithTitle(&NSString::from_str(&root.name));
            self.roots.borrow_mut().push(root.path.clone());
        }
        let empty = roots.is_empty();
        self.folder_picker.setEnabled(!empty);
        self.remove_folder.setEnabled(!empty);
    }

    /// The path of the folder the user chose.
    pub(in crate::platform::main_window) fn selected_root(&self) -> Option<String> {
        let index = self.folder_picker.indexOfSelectedItem();
        if index < 0 {
            return None;
        }
        self.roots.borrow().get(index as usize).cloned()
    }

    /// Writes the settings into the fields.
    pub(in crate::platform::main_window) fn set_values(&self, settings: &ServerSettings) {
        self.device_name_field
            .setStringValue(&NSString::from_str(&settings.device_name));
        self.http_port_field
            .setStringValue(&NSString::from_str(&port_text(settings.http_port)));
    }

    /// The values the user typed into the fields.
    pub(in crate::platform::main_window) fn values(&self) -> (String, String) {
        (
            self.device_name_field.stringValue().to_string(),
            self.http_port_field.stringValue().to_string(),
        )
    }
}

/// Builds a button that carries out one action.
fn button(
    mtm: MainThreadMarker,
    title: &str,
    action: MenuAction,
    target: &MenuTarget,
) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&NSString::from_str(title));
    target.button_action(&button, action);
    button
}

/// Builds a field the user may change, with the label that names it.
fn editable_field(
    mtm: MainThreadMarker,
    label: &str,
    value: &str,
) -> (Retained<NSTextField>, Retained<NSTextField>) {
    let label_field = NSTextField::wrappingLabelWithString(&NSString::from_str(label), mtm);
    label_field.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
    let field = NSTextField::new(mtm);
    field.setStringValue(&NSString::from_str(value));
    field.setEditable(true);
    field.setSelectable(true);
    field.setAccessibilityLabel(Some(&NSString::from_str(label)));
    (label_field, field)
}

/// The port as the text of a field; a port the app does not know stays empty.
fn port_text(port: u16) -> String {
    match port {
        0 => String::new(),
        port => port.to_string(),
    }
}
