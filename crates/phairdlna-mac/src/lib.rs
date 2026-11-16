//! The native macOS front end of PhairDLNA.
//!
//! What: the menu bar application that shows what the headless server is
//!       doing and that lets the user steer it — the status item, its menu,
//!       the client of the administration surface and the texts the user
//!       reads.
//! Why: the server is the product; the application is a front end for it.
//!      The two are separate processes with separate lifetimes, so that the
//!      server keeps serving when the application is closed (`MB-001`,
//!      `UX-001`).
//! Example: the application asks the administration surface for the state of
//!          the server and shows one line of text — "Server running — 2
//!          active streams" — in the menu.
//!
//! **The parts.** [`model`] holds the state of the server as the application
//! understands it, [`menu`] turns that state into the entries of the menu,
//! [`ipc`] talks to the administration surface, [`text`] holds the strings of
//! the user interface, [`actions`] holds what the entries of the menu do,
//! [`worker`] keeps the network away from the main thread, and [`app`] wires
//! the parts together.
//!
//! **The platform.** [`platform`] is the AppKit adapter and exists only on
//! macOS. Everything else of this crate is portable and is tested without a
//! window, so that the logic of the menu can be checked in the continuous
//! integration of every platform of the build matrix (`F-15` section 11).

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod actions;
pub mod app;
pub mod ipc;
pub mod menu;
pub mod model;
#[cfg(target_os = "macos")]
pub mod platform;
pub mod preferences;
pub mod text;
pub mod worker;
