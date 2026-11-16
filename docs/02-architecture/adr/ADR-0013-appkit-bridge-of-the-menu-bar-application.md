# ADR-0013 — The AppKit bridge of the menu bar application

Status: **Accepted**
Date: 2026-11-16
Owner role: Architect
Requirements: `MB-002`, `MB-003`, `MB-005`, `MB-006`, `UX-007`, `UX-009`, `UX-017`, `UX-018`, `PLT-004`

## Context

The project forbids `unsafe` by default: any use needs an architecture decision
record, a written safety argument and a comment at every `unsafe` block
(`PROJECT-STANDARDS` § 2.6). The menu bar application of `phairdlna-mac` needs
the classes of AppKit: the status item, its menu, the picture of the item and
the accessibility labels of the item.

Almost all of that is safe to write. `objc2` and `objc2-app-kit` give safe
bindings for the classes, the methods and the objects of AppKit, and the
application uses them: `NSStatusBar::systemStatusBar`,
`NSStatusItem::statusItemWithLength`, `NSMenu::new`, `NSMenuItem::separatorItem`,
`NSImage::imageWithSystemSymbolName_accessibilityDescription`,
`NSWorkspace::sharedWorkspace` and the rest are ordinary safe calls.

One thing is not safe to write: **the actions of the menu**. AppKit does not
tell a program which entry of a menu the user chose. It sends a *message* to a
*target* object, and the message is one of the selectors that the target
implements. A program that wants to know which entry was chosen therefore needs
a class of its own, and every binding that creates a class of its own at run
time — `objc2::define_class!` among them — is `unsafe`, because the runtime
registers the class in a global table and the methods of the class are called
by the system and not by the program.

The alternatives were considered and none of them keeps the promise of the
project:

- **A second language.** A small Swift helper would give the best accessibility
  and the best platform fit, and it would add a second language, a second
  build step and a second thing to sign and to notarise. The decision OD-03 of
  the project chose the pure Rust way, with the accessibility audit as the gate
  that may send the project back.
- **A menu without actions.** The menu bar application would show the state of
  the server and could do nothing. That is a stub, and a stub is not a
  feature.
- **A webview.** Rejected by OD-03 and by the privacy model of the project.

## Decision

1. **One file of the crate uses `unsafe`:** `crates/phairdlna-mac/src/platform/bridge.rs`.
   It holds the class that receives the actions of the menu and the tick of the
   timer, and nothing else. Every other file of the crate is free of `unsafe`
   and is portable, so that the logic of the menu is tested on every platform
   of the build matrix.
2. **Every `unsafe` block carries its safety argument in a comment.** The
   argument names the invariant that the block upholds: the superclass, the
   instance variables, the thread and the lifetime of the objects that the
   block touches.
3. **The class is created once, on the main thread, and lives as long as the
   menu.** The class holds no instance variable and implements no `Drop`; the
   target of the menu items is the object of this class, and the menu keeps it
   alive.
4. **The class is bound to the main thread.** The attribute
   `#[thread_kind = MainThreadOnly]` makes the binding refuse any call that
   does not come from the main thread, which is the thread AppKit sends the
   actions of a menu and the ticks of a timer on (`MB-002`).
5. **The gate QG-03 knows about this record.** The gate refuses every `unsafe`
   in the workspace, except in a file that carries the marker
   `gate: unsafe-exempt ADR-0013` and whose record exists. The gate is not
   weakened: a file with `unsafe` and without the marker still fails it, and
   the marker names the record that a reader must read.

## Consequences

- The crate carries one file with `unsafe`, and every other file is portable
  and testable without a window.
- The safety argument lives in three places that belong together: this record,
  the comments at the blocks and the marker of the file.
- The gate QG-03 changes from a purely lexical scan to a scan that knows one
  exemption by name. The exemption is narrow: one file, one record, and a
  marker that names both.
- The accessibility audit of the project remains the gate that may send the
  project back to a helper written in Swift (OD-03). If that happens, this
  record is revisited and not rewritten silently.

## Alternatives considered

- **A helper written in Swift.** Rejected for now: a second language, a second
  build step and a second thing to sign. The accessibility audit is the gate
  that may change this decision.
- **A menu bar application without actions.** Rejected: a stub is not a
  feature, and `UX-019` asks for the user to start the server and to read the
  library again from the menu.
- **The `muda` crate of the ecosystem.** Rejected: it is a wrapper around the
  same runtime and would move the `unsafe` out of sight instead of away. The
  project writes the safety argument itself.
- **Polling the state of the menu instead of receiving the actions.** Rejected:
  it reads the wrong thing and cannot tell which entry the user chose.

## Revisit condition

Revisit if the accessibility audit shows that `objc2-app-kit` cannot deliver
the VoiceOver behaviour that `UX-009` and `UX-018` ask for. The decision would
then be a helper written in Swift, and this record would name the helper and
the boundary between it and this crate.
