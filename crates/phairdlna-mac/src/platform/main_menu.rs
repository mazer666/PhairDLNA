//! Main menu bar creation for the macOS application.
//!
//! What: builds the system application menu bar shown at top-left.
//! Why: macOS applications must provide a main menu bar so standard shortcuts and
//!      actions (Preferences, About, Quit, Window controls) are available when active.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::NSString;

use super::bridge::MenuTarget;
use crate::actions::MenuAction;
use crate::text::Strings;

/// Installs the macOS application main menu bar at the top-left of the screen.
pub fn install_main_menu(mtm: MainThreadMarker, target: &MenuTarget, strings: &Strings) {
    let main_menu = NSMenu::new(mtm);
    main_menu.addItem(&build_app_menu(mtm, target, strings));
    main_menu.addItem(&build_file_menu(mtm, target, strings));
    main_menu.addItem(&build_window_menu(mtm, target, strings));

    let app = NSApplication::sharedApplication(mtm);
    app.setMainMenu(Some(&main_menu));
}

fn build_app_menu(
    mtm: MainThreadMarker,
    target: &MenuTarget,
    strings: &Strings,
) -> objc2::rc::Retained<NSMenuItem> {
    let app_menu_item = NSMenuItem::new(mtm);
    let app_menu = NSMenu::new(mtm);
    let title_app = NSString::from_str("PhairDLNA");
    app_menu.setTitle(&title_app);

    let about_item = target.menu_item(
        mtm,
        strings.text("prefs.tab_about"),
        MenuAction::OpenPreferences,
    );
    app_menu.addItem(&about_item);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));

    let prefs_item = target.menu_item(
        mtm,
        strings.text("menu.action.preferences"),
        MenuAction::OpenPreferences,
    );
    prefs_item.setKeyEquivalent(&NSString::from_str(","));
    app_menu.addItem(&prefs_item);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));

    let quit_item = target.menu_item(mtm, strings.text("menu.action.quit"), MenuAction::Quit);
    quit_item.setKeyEquivalent(&NSString::from_str("q"));
    app_menu.addItem(&quit_item);

    app_menu_item.setSubmenu(Some(&app_menu));
    app_menu_item
}

fn build_file_menu(
    mtm: MainThreadMarker,
    target: &MenuTarget,
    strings: &Strings,
) -> objc2::rc::Retained<NSMenuItem> {
    let file_menu_item = NSMenuItem::new(mtm);
    let file_menu = NSMenu::new(mtm);
    let title_file = NSString::from_str(strings.text("menu.title.file"));
    file_menu.setTitle(&title_file);

    let start_server = target.menu_item(
        mtm,
        strings.text("menu.action.start_server"),
        MenuAction::StartServer,
    );
    file_menu.addItem(&start_server);

    let stop_server = target.menu_item(
        mtm,
        strings.text("menu.action.stop_server"),
        MenuAction::StopServer,
    );
    file_menu.addItem(&stop_server);
    file_menu.addItem(&NSMenuItem::separatorItem(mtm));

    let add_folders = target.menu_item(
        mtm,
        strings.text("menu.action.add_folders"),
        MenuAction::AddMediaFolders,
    );
    add_folders.setKeyEquivalent(&NSString::from_str("o"));
    file_menu.addItem(&add_folders);

    let rescan = target.menu_item(
        mtm,
        strings.text("menu.action.rescan"),
        MenuAction::RescanLibrary,
    );
    rescan.setKeyEquivalent(&NSString::from_str("r"));
    file_menu.addItem(&rescan);

    file_menu_item.setSubmenu(Some(&file_menu));
    file_menu_item
}

fn build_window_menu(
    mtm: MainThreadMarker,
    target: &MenuTarget,
    strings: &Strings,
) -> objc2::rc::Retained<NSMenuItem> {
    let win_menu_item = NSMenuItem::new(mtm);
    let win_menu = NSMenu::new(mtm);
    let title_win = NSString::from_str(strings.text("menu.title.window"));
    win_menu.setTitle(&title_win);

    let overview = target.menu_item(mtm, strings.text("overview.title"), MenuAction::OpenAdmin);
    overview.setKeyEquivalent(&NSString::from_str("1"));
    win_menu.addItem(&overview);

    win_menu_item.setSubmenu(Some(&win_menu));
    win_menu_item
}
