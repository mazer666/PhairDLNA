//! Native configuration window for macOS.
//!
//! What: interactive native settings window featuring general settings,
//!       media folders list, and an embedded expert TOML editor.
//! Why: lets users configure the server directly inside the native macOS app
//!      without requiring a browser or external editor (`UX-004`).

use std::path::{Path, PathBuf};

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAlertStyle, NSButton, NSFont, NSPanel, NSScrollView, NSTabView, NSTabViewItem, NSTextField,
    NSTextView, NSView, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use phairdlna_core::config::Config;

use super::panels;

/// Holds the open preferences window and its controls.
pub struct PreferencesWindow {
    /// The window panel itself.
    pub panel: Retained<NSPanel>,
    config_path: PathBuf,
    name_field: Retained<NSTextField>,
    port_field: Retained<NSTextField>,
    editor_view: Retained<NSTextView>,
}

impl PreferencesWindow {
    /// Builds the preferences window panel.
    pub fn new(mtm: MainThreadMarker, config_path: &Path, strings: &crate::text::Strings) -> Self {
        let panel = create_panel(mtm, strings);
        let Some(content) = panel.contentView() else {
            return Self::fallback(mtm, panel, config_path);
        };

        let config = Config::load(config_path).unwrap_or_default();
        let raw_toml = std::fs::read_to_string(config_path).unwrap_or_default();

        let tab_view = NSTabView::new(mtm);
        tab_view.setFrame(NSRect::new(
            NSPoint::new(10.0, 55.0),
            NSSize::new(640.0, 475.0),
        ));

        let (tab1, name_field, port_field) = build_general_tab(mtm, &config, strings);
        tab_view.addTabViewItem(&tab1);

        let (tab2, editor_view) = build_expert_tab(mtm, &raw_toml, strings);
        tab_view.addTabViewItem(&tab2);

        content.addSubview(&tab_view);

        let save_btn = NSButton::new(mtm);
        let save_title = NSString::from_str(strings.text("prefs.save"));
        save_btn.setTitle(&save_title);
        save_btn.setFrame(NSRect::new(
            NSPoint::new(450.0, 12.0),
            NSSize::new(180.0, 32.0),
        ));
        content.addSubview(&save_btn);

        Self {
            panel,
            config_path: config_path.to_path_buf(),
            name_field,
            port_field,
            editor_view,
        }
    }

    fn fallback(mtm: MainThreadMarker, panel: Retained<NSPanel>, config_path: &Path) -> Self {
        Self {
            panel,
            config_path: config_path.to_path_buf(),
            name_field: NSTextField::new(mtm),
            port_field: NSTextField::new(mtm),
            editor_view: NSTextView::new(mtm),
        }
    }

    /// Displays the window and brings it to front.
    pub fn show(&self) {
        self.panel.makeKeyAndOrderFront(None);
    }

    /// Saves settings back to config.toml.
    pub fn save(
        &self,
        strings: &crate::text::Strings,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        let edited_toml = self.editor_view.string().to_string();
        if !edited_toml.trim().is_empty() {
            if let Err(e) = Config::from_str(&edited_toml, &self.config_path.to_string_lossy()) {
                let err_msg = strings.format("prefs.save_failed", &[("reason", e.message())]);
                panels::show_alert(
                    mtm,
                    strings.text("prefs.title"),
                    &err_msg,
                    NSAlertStyle::Warning,
                );
                return Err(e.message().to_owned());
            }
            std::fs::write(&self.config_path, edited_toml).map_err(|e| e.to_string())?;
        } else {
            let mut config = Config::load(&self.config_path).unwrap_or_default();
            config.server.device_name = self.name_field.stringValue().to_string();
            if let Ok(p) = self.port_field.stringValue().to_string().parse::<u16>() {
                config.server.http_port = p;
            }
            config.save(&self.config_path).map_err(|e| e.to_string())?;
        }

        panels::show_alert(
            mtm,
            strings.text("prefs.title"),
            strings.text("prefs.save_success"),
            NSAlertStyle::Informational,
        );
        Ok(())
    }
}

fn create_panel(mtm: MainThreadMarker, strings: &crate::text::Strings) -> Retained<NSPanel> {
    let panel = NSPanel::new(mtm);
    panel.setContentSize(NSSize::new(660.0, 540.0));
    panel.setStyleMask(
        NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Resizable
            | NSWindowStyleMask::Miniaturizable,
    );
    let title_ns = NSString::from_str(strings.text("prefs.title"));
    panel.setTitle(&title_ns);
    panel
}

fn build_roots_view(
    mtm: MainThreadMarker,
    config: &Config,
    strings: &crate::text::Strings,
) -> Retained<NSScrollView> {
    let roots_scroll = NSScrollView::new(mtm);
    roots_scroll.setFrame(NSRect::new(
        NSPoint::new(20.0, 20.0),
        NSSize::new(595.0, 275.0),
    ));
    let roots_view = NSTextView::new(mtm);
    roots_view.setEditable(false);
    let mut roots_text = String::new();
    for root in &config.library.roots {
        roots_text.push('\t');
        roots_text.push_str(&root.name);
        roots_text.push(':');
        roots_text.push(' ');
        roots_text.push_str(&root.path);
        roots_text.push('\n');
    }
    if roots_text.is_empty() {
        roots_text = strings.text("menu.status.no_roots").to_owned();
    }
    roots_view.setString(&NSString::from_str(&roots_text));
    roots_scroll.setDocumentView(Some(&roots_view));
    roots_scroll
}

fn build_general_tab(
    mtm: MainThreadMarker,
    config: &Config,
    strings: &crate::text::Strings,
) -> (
    Retained<NSTabViewItem>,
    Retained<NSTextField>,
    Retained<NSTextField>,
) {
    let tab = NSTabViewItem::new();
    let tab_title = NSString::from_str(strings.text("prefs.tab_general"));
    tab.setLabel(&tab_title);

    let view = NSView::new(mtm);

    let name_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.server_name")),
        mtm,
    );
    name_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 390.0),
        NSSize::new(180.0, 24.0),
    ));
    view.addSubview(&name_lbl);

    let name_field = NSTextField::new(mtm);
    name_field.setFrame(NSRect::new(
        NSPoint::new(200.0, 386.0),
        NSSize::new(300.0, 26.0),
    ));
    name_field.setStringValue(&NSString::from_str(&config.server.device_name));
    view.addSubview(&name_field);

    let port_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.http_port")),
        mtm,
    );
    port_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 350.0),
        NSSize::new(180.0, 24.0),
    ));
    view.addSubview(&port_lbl);

    let port_field = NSTextField::new(mtm);
    port_field.setFrame(NSRect::new(
        NSPoint::new(200.0, 346.0),
        NSSize::new(120.0, 26.0),
    ));
    port_field.setStringValue(&NSString::from_str(&config.server.http_port.to_string()));
    view.addSubview(&port_field);

    let roots_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.roots_header")),
        mtm,
    );
    roots_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 305.0),
        NSSize::new(400.0, 24.0),
    ));
    view.addSubview(&roots_lbl);

    let roots_scroll = build_roots_view(mtm, config, strings);
    view.addSubview(&roots_scroll);

    tab.setView(Some(&view));
    (tab, name_field, port_field)
}

fn build_expert_tab(
    mtm: MainThreadMarker,
    raw_toml: &str,
    strings: &crate::text::Strings,
) -> (Retained<NSTabViewItem>, Retained<NSTextView>) {
    let tab = NSTabViewItem::new();
    let tab_title = NSString::from_str(strings.text("prefs.tab_expert"));
    tab.setLabel(&tab_title);

    let scroll = NSScrollView::new(mtm);
    scroll.setFrame(NSRect::new(
        NSPoint::new(20.0, 20.0),
        NSSize::new(595.0, 400.0),
    ));
    let editor_view = NSTextView::new(mtm);
    let font = NSFont::userFixedPitchFontOfSize(12.0);
    editor_view.setFont(font.as_deref());
    editor_view.setString(&NSString::from_str(raw_toml));
    scroll.setDocumentView(Some(&editor_view));
    tab.setView(Some(&scroll));

    (tab, editor_view)
}
