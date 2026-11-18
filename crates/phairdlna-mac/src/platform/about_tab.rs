//! About tab for the native Preferences window.
//!
//! What: displays project metadata, version, license, and repository link.
//! Why: provides transparency and quick access to project resources (`UX-004`).

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSTabViewItem, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

/// The public GitHub repository URL of PhairDLNA.
pub const REPO_URL: &str = "https://github.com/mazer666/PhairDLNA";

/// Builds the 'About' tab view.
pub fn build_about_tab(
    mtm: MainThreadMarker,
    strings: &crate::text::Strings,
) -> Retained<NSTabViewItem> {
    let tab = NSTabViewItem::new();
    let tab_title = NSString::from_str(strings.text("prefs.tab_about"));
    tab.setLabel(&tab_title);

    let view = NSView::new(mtm);

    let title_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.about_title")),
        mtm,
    );
    title_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 390.0),
        NSSize::new(560.0, 30.0),
    ));
    view.addSubview(&title_lbl);

    let desc_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.about_desc")),
        mtm,
    );
    desc_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 320.0),
        NSSize::new(560.0, 60.0),
    ));
    view.addSubview(&desc_lbl);

    let ver_str = strings.format("prefs.about_version", &[("version", "0.1.0")]);
    let ver_lbl = NSTextField::wrappingLabelWithString(&NSString::from_str(&ver_str), mtm);
    ver_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 280.0),
        NSSize::new(560.0, 24.0),
    ));
    view.addSubview(&ver_lbl);

    let lic_lbl = NSTextField::wrappingLabelWithString(
        &NSString::from_str(strings.text("prefs.about_license")),
        mtm,
    );
    lic_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 240.0),
        NSSize::new(560.0, 24.0),
    ));
    view.addSubview(&lic_lbl);

    let repo_str = strings.format("prefs.about_repo", &[("url", REPO_URL)]);
    let repo_lbl = NSTextField::wrappingLabelWithString(&NSString::from_str(&repo_str), mtm);
    repo_lbl.setFrame(NSRect::new(
        NSPoint::new(20.0, 180.0),
        NSSize::new(560.0, 48.0),
    ));
    view.addSubview(&repo_lbl);

    tab.setView(Some(&view));
    tab
}
