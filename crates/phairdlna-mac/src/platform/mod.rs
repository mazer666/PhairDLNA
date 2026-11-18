//! The AppKit adapter of the application.
//!
//! What: the status item, its menu, the icon of the menu bar and the opening
//!       of addresses and files.
//! Why: AppKit may be used only from the main thread, and only the main
//!      thread may draw (`MB-002`).
//! Example: the status item shows the state of the server as a template
//!          image, so that it follows the appearance of the menu bar
//!          (`MB-006`).
//!
//! **The one file that speaks to the runtime.** [`bridge`] is the only file of
//! this crate that uses `unsafe`, because AppKit hands the action of a menu
//! item to an object of a class of our own, and a class of our own cannot be
//! written without it. The safety argument of every block is in that file and
//! in ADR-0013.

pub mod about_tab;
pub mod bridge;
pub mod config_dialog;
pub mod desktop;
pub mod icon;
pub mod panels;
pub mod preferences_window;
pub mod service_control;
pub mod status_item;
pub mod workspace;
