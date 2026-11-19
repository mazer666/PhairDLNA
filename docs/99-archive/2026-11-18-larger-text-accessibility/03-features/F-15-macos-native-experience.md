# F-15 — Native macOS experience

Status: In progress — the regular native macOS application, initial main overview, folder action, TV connection guidance, preferences window and authenticated API client are implemented in part (M2.6); the complete first-run and service lifecycle flows, discovery verification, accessibility audit and universal binary remain open

Owner role: Product/Requirements

Version/date: 2.0 / 2026-11-18
Purpose: Specify the native macOS application as the primary graphical setup and administration surface.
References: `../00-governance/PROJECT-STANDARDS.md`, `../00-governance/REQUIREMENTS-METHOD.md`, `F-14`, `F-13`, `../07-project/ROADMAP.md`, `DEC-075`, `DEC-076`, `OD-14`, `OD-15`, `OD-16`.

Requirements: UX-001, UX-002, UX-003, UX-004, UX-005, UX-006, UX-007, UX-008, UX-009, UX-010, UX-013, UX-014, UX-015, UX-016, UX-017, UX-018, UX-019, UX-020, MB-001, MB-002, MB-003, MB-004, MB-005, MB-006, MB-007, MB-008, MB-009, PLT-004
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

**PhairDLNA's server is a headless program.** It answers discovery requests on the local network and streams files; it has no window, no icon in the
Dock and no settings page. On macOS the *product* pairs a **launchd daemon** that does the work with a **regular native application** that guides setup and
everyday use. A menu bar item — the small icon in the top-right strip of the screen — gives quick status and common actions; the application's window
is the main place to understand and manage the server.

This feature specifies the user-visible half of that pair: the main window, the status item, the two presentation modes, the first-run assistant, the
accessibility behaviour and the installation story. It specifies the **front end only**. The server logic lives in `phairdlna-core` and `phairdlna-cli`; the
application talks to it over the local administration surface and never embeds it (MB-001, ADR-0003).

Three macOS facts shape everything below.

1. **macOS 15 introduced Local Network privacy** (platform-integration §1.2, Apple TN3179). A program that sends or *receives* local network traffic
   needs the user's permission. `launchd` **daemons** and any process running as **root** are exempt; a per-user `launchd` **agent** is **not** —
   Apple says so explicitly. The always-on server is therefore a daemon. The regular application can show a Dock icon and main window while its menu-bar
   status item gives quick access; its activation policy is regular (Apple AppKit `NSApplicationActivationPolicyRegular`).
2. **The macOS application firewall is per application** (platform-integration §2). An ad-hoc-signed binary is not covered by the "automatically allow
   downloaded signed software" rule, so the firewall may prompt — and until the user answers, incoming connections are refused. The platform research
   calls this the single largest residual macOS risk and insists it be exercised on a real machine.
3. **A pure-Rust menu bar application is realistic** (platform-integration §7). `muda` implements its macOS backend on `objc2` + `objc2-app-kit`, that
   is on `NSStatusItem`. No Swift, no webview.

**User story.** As a Mac owner, I want one native PhairDLNA application for first-time setup and everyday use, with a menu bar item for quick status,
so I can add media folders, see whether the server is ready, and get help connecting my television without learning DLNA terms or opening a browser.
The server continues to run independently when I close the application.

## 2. Scope

**In scope** — the native application as the primary user-facing setup and administration surface; a focused main window for first run, server status,
media folders and connection help; the menu bar item for quick status and common actions; the two presentation modes (simple and advanced) over one
configuration schema; translation readiness; accessibility (keyboard, VoiceOver, text size, contrast, reduced motion); the relationship between the
application and the daemon (reconnection, no cached assumptions); the `.app` bundle and its permission metadata; the uninstall path; and the Apple
Silicon native build. The embedded browser interface is removed; the loopback API remains as the application's local service boundary (DEC-075).

**Explicitly out of scope** — the server itself, its protocol surface and its security model (F-01 … F-08, F-16); the administration surface and its
token, CSRF and `Host` rules (F-14, which this application is a *client* of); the CLI (F-13); the service installation mechanics as a product decision
(F-13, `OPS-009`); the code-signing and notarisation *plan* (F-19, `SUP-009`); the Linux and Windows service models (platform-integration §8, §9); and
the media-transcoding options (F-17).

## 3. Functional requirements

**UX-001** — The server shall be fully operable without any graphical interface. [DESIGN] Every action the menu bar application offers has a
command-line equivalent in `phairdlna-cli` (F-13); closing or removing the application never stops the daemon and never changes a setting.

**UX-002** — A first run shall require no more than one action (choosing a folder) before the server is usable on the network. [DESIGN] The
assistant's first screen asks for one folder; every later step has a safe default that can be accepted without comment. A first run with no folder
must still produce a running server and a clear "add a folder" state (F-14).

**UX-003** — The first-run output shall name the device name, the address it is reachable at and how to verify it from another device. [DESIGN] The
name is the configured `friendlyName`; the address is the interface-local address the daemon advertises (F-01); the verification step is the optional
assistant check of MB-007.

**UX-004** — Every error message shall state what happened, why it happened and what the user can do next. [DESIGN] The application renders the
server's structured errors verbatim in that three-part shape; it does not paraphrase an error into something the server did not say, and it never
shows an absolute filesystem path (SEC-094).

**UX-005** — The server shall never abort with a panic message that lacks context; unexpected failures shall produce a structured internal error and a
readable one-line summary. [DESIGN] The panic hook is installed by the daemon, not by the application; a panic in the *application* must never take
the daemon down with it, because the daemon is a separate process (MB-001).

**UX-006** — The command line shall provide `--help` for every subcommand. [DESIGN] The menu bar application is a front end for those subcommands; its
help text names the equivalent command so that a user can leave the graphical world.

**UX-007** — The macOS application shall be accessible with the keyboard alone and shall expose the status of the server as text, not only as an icon.
[DESIGN] The status item's menu opens with the keyboard, every item has a text label, and the status line is a sentence ("Server running — serving 2
clients"), never a coloured dot.

**UX-008** — The macOS application shall respect the system accessibility settings for contrast, motion and text size. [DESIGN] Colours come from the
system semantic palette. Apple says macOS does not support Dynamic Type, and the current overview does not adapt its fixed text sizes. OD-17 proposes
an app-level text-size control as the working direction; this is not part of the approved requirement baseline until the owner accepts the change.

**UX-009** — The macOS application shall show an accessible status item for assistive technologies. [DESIGN] The status item carries an accessibility
label and value that describe the server state in words and are updated when the state changes, so that VoiceOver announces a change without the user
opening the menu.

**UX-010** — The onboarding shall use plain language and avoid protocol jargon in the user-facing text. [DESIGN] The assistant says "your TV can now
find this server on your network", never "SSDP advertisement sent"; "media folder", never "media root"; "scanning", never "indexing". Technical terms
appear only in the advanced mode and only with an explanation.

**UX-013** — Long-running operations shall report progress and allow cancellation. [DESIGN] A scan started from the application shows a determinate or
indeterminate progress indicator and a *Stop* action that cancels the scan itself, not the server (FUN-024, F-08). The catalogue task owns the
operation; the application only observes it (MB-002).

**UX-014** — The user interface shall present the server state, the scan progress, the shared folders and the last observed client activity in one
view. [DESIGN] The simple mode's main view is that one view; the advanced mode adds detail beneath it rather than replacing it.

**UX-015** — The product shall provide a simple presentation mode and an advanced presentation mode over one configuration schema; switching between
them shall never reset a setting, restart the server or interrupt a transfer. [DESIGN] The mode is a *view* preference, stored outside the server
configuration and independent of it; it survives restarts and is reversible (see the mapping of the non-normative proposal in §4).

**UX-016** — A safety-critical warning raised in the advanced mode shall remain visible in the simple mode. [DESIGN] Warnings are properties of the
*server state*, not of the view: the simple mode shows the same warning, in simpler words, in the same place.

**UX-017** — The user interface shall be operable with the keyboard alone, shall expose accessible labels and a deterministic focus order, and shall
respect the system's text size and contrast settings. [DESIGN] Focus order follows the visual order of the controls and never depends on where the
pointer was last; every control has a label that reads as a sentence. The text-size expectation is unresolved under OD-17 because macOS does not
support Dynamic Type.

**UX-018** — The user interface shall expose the status of the server as text for assistive technologies, not only as an icon. [DESIGN] Same mechanism
as UX-009: the server state has a textual rendering that is always available, whether or not the menu is open.

**UX-019** — The user shall be able to start and stop the server, pause and resume indexing, and remove a shared folder without touching the source
files. [DESIGN] "Stop" means "stop serving on the network", not "shut down the process"; the daemon keeps the catalogue. Removing a folder removes it
from the catalogue and the configuration; the files themselves are never touched (FUN-018, F-08).

**UX-020** — All user-facing strings shall be prepared for translation, and the set of supported locales shall be decided before the first release.
[DESIGN] Strings live in one resource file per locale, never inline in code, and no string is built by concatenation; a locale list is fixed before
the first release. The list is an open question (§12).

**MB-001** — The server shall be able to run headless independently of the user interface; the interface shall reconnect and reflect the actual server
state rather than a cached assumption. [DESIGN] The daemon and the application are separate processes with separate lifetimes (platform-integration
§4.2); the application holds no server state of its own and re-reads it from the administration surface on connect and on every change notification.
The application never contains server logic: `phairdlna-mac` depends on `phairdlna-cli`, which depends on `phairdlna-core`, and never the other way
round (ARCHITECTURE §2, ADR-0003).

**MB-002** — The user interface shall acknowledge every action visibly, report progress and explain errors, and shall never block its event loop with
scanning, metadata or network input/output. [DESIGN] The menu bar application owns the AppKit main thread and pumps the event loop; every network call
to the administration surface is asynchronous and every long operation is observed by subscribing to the server's state rather than by polling on the
main thread.

**MB-003** — The user interface shall support keyboard navigation, assistive-technology labels, scalable text, sufficient contrast and the
reduced-motion preference. [DESIGN] No animation is used to convey state; every state change is also expressed in text.

**MB-004** — The user interface shall open no popover or settings view that starts an index or a network scan it does not need. [DESIGN] Opening a
view is a read. Only an explicit command starts work (UX-019); a view that wants fresh data asks for it and waits.

**MB-005** — The user interface shall communicate with the server only over the local administration surface by default, and shall use the
authenticated and authorised model defined for that surface. [DESIGN] The application holds the per-installation token and sends it as a bearer token
with the CSRF token and the custom header the surface requires (F-14, SEC-072). The application never opens the media port itself and never reads the
catalogue database directly.

**MB-006** — The menu bar status shall remain legible in light and dark appearance, on a small display and when the menu bar is crowded, and shall not
rely on colour alone. [DESIGN] The status item uses a template image that follows the menu bar appearance, a short text label rather than a wide
graphic, and a text status line in the menu; the state is always readable as text (UX-018).

**MB-007** — The first-run assistant shall offer an optional verification step that shows whether a television can discover and play the server,
without assuming that a television is present. [DESIGN] The step is *offered*, never required, and never claims a result it did not observe: it
reports "no client has found the server yet" as a neutral state and never "your TV cannot see this server". The evidence is the server's own
observation of incoming discovery requests.

**MB-008** — Uninstalling and stopping shall be safe and obvious, and the auto-start setting shall be visible; no hidden persistent helper shall
remain. [DESIGN] The application's removal action unregisters the service it registered, stops the daemon and says so; the auto-start state is shown
in the menu and changes only when the user asks. No helper is installed that the user cannot see and remove (LIFE-010).

**MB-009** — The native application shall ship as a bundle with the metadata the operating system requires for its own permissions and supported
service registration. [DESIGN] The application is a regular `.app` bundle with an `Info.plist` carrying `NSLocalNetworkUsageDescription` when it makes
a protected local-network request (platform-integration §1.3); selecting a folder does not establish access for the separate service process. If the
app registers a helper through `SMAppService`, the helper executable lives inside the bundle (platform-integration §4.3). It omits
`LSUIElement`, so macOS presents it as a normal application in the Dock and app switcher; the menu-bar item remains a quick-access control (DEC-076).

**PLT-004** — The server shall support macOS Apple Silicon natively without a Rosetta translation requirement. [DESIGN] The build matrix contains
`aarch64-apple-darwin` and the artefacts are universal (`arm64` + `x86_64`) so that the same file runs on Intel and Apple Silicon. On Apple Silicon an
arm64 binary must carry *some* valid signature to execute at all — ad-hoc is enough (platform-integration §3.1).

## 4. Technical requirements

- **Two processes, one contract.** The application (`phairdlna-mac`) is a client of the administration surface. PLT-013 currently requires a
  system-context launchd daemon for the server, but its effective identity and ability to honor selected-folder access without elevated privilege are
  unresolved (OD-15). The service implementation must wait for that decision and the real-Mac evidence.
- **Why the baseline specifies a daemon.** TN3179 exempts launchd *daemons* and root processes from Local Network privacy and explicitly does
  **not** exempt agents (platform-integration §1.2). This explains PLT-013, but does not settle how this product satisfies OPS-015/PLT-018 while
  serving only user-selected folders. OD-15 must resolve the complete process model.
- **Rust dependency set for the application** (platform-integration §7.3): `objc2`, `objc2-app-kit`, `objc2-foundation`, `block2`, `core-foundation`,
  `libc`, and `notify` for filesystem events. `cocoa` is **not** used (legacy `objc`, only 16.3 % documented, platform metadata omits
  `aarch64-apple-darwin`) and `dispatch` is **not** used (unreleased since 2020); GCD types come from `objc2::dispatch` when needed. `muda` +
  `tray-icon` are the evaluated menu crates; calling `objc2-app-kit` directly (`NSStatusItem` + `NSMenu`) is the documented alternative for a minimal
  menu.
- **Threading.** AppKit menus may be used only from the main thread; the application owns the main thread and pumps the run loop. All server
  interaction happens on other threads and posts results back (platform-integration §7.1, MB-002).
- **Bundle and metadata.** The application is an `.app` bundle with an `Info.plist` carrying `NSLocalNetworkUsageDescription`; the daemon, when
  installed as a helper, lives inside the bundle so that it inherits the bundle's code identity — which is what makes the "responsible code"
  attribution work for Local Network privacy (platform-integration §4.3).
- **Presentation modes.** One configuration schema, one validation layer, one transactional update path; the mode is a separate, reversible preference
  that changes the view and nothing else.
- **Distribution.** Ship an ad-hoc-signed universal binary now; design so that the *same* binary can later be Developer-ID-signed,
  hardened-runtime-enabled and notarised without code changes (platform-integration §3.3). The plan itself is F-19 / `SUP-009`.
- **Mapping of the non-normative proposal.** `docs/01-product/SIMPLE-ADVANCED-EXPERIENCE.md` is explicitly **non-normative**; its `UX-MODE-*`
  identifiers are local proposal identifiers, not requirement IDs. The canonical mapping used here is:

| Proposal identifier (non-normative) | Canonical requirement |
| --- | --- |
| UX-MODE-001 | UX-015, REL-006 |
| UX-MODE-002 | UX-016 |
| UX-MODE-003 | UX-002 |
| UX-MODE-004 | UX-007, UX-009, UX-017, UX-018, MB-003 |
| UX-MODE-005 | UX-006, UX-014 |
| UX-MODE-006 | UX-015 |

The proposal's acceptance thresholds are proposals; no measurement exists (the source document says so itself).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: the status item reflects a server-state change within 500 ms of the change happening. Method: timestamp the state change in the
  daemon and the repaint in the application over 100 changes, median and p95 reported (PERF-011).
- [DESIGN] Target: the application starts and shows a status within 2 s of being launched, on the reference hardware. Method: `hyperfine` over 20
  launches.
- [DESIGN] Target: the application consumes less than 1 % CPU averaged over one minute while the menu is closed. Method: `top` sampling over 60 s.
- [DESIGN] Target: the application never holds more than one outstanding request to the administration surface. Method: code review of the request
  helper.

## 6. Usability and accessibility

### Main window structure

The main window is the home for normal users. It uses a native title bar and a single vertical
reading order; the menu bar item stays compact and opens this window. The first release keeps four
areas in this order:

1. **Server status** — one sentence such as "Ready for your TV" or "Server stopped", a short reason
   when it is not ready, and one context-sensitive primary action.
2. **Media folders** — a short list of shared folder names and availability, with one "Add folder"
   action. Do not show full filesystem paths in the simple view.
3. **TV connection** — the server name and local address, followed by two plain steps: open the TV's
   media-source list and choose PhairDLNA. A discovery check is optional and reports only what the
   server observed (`MB-007`).
4. **Recent activity** — the current scan or active playback when present; otherwise a quiet empty
   state. Diagnostics and technical details live behind a secondary action or disclosure.

The empty state has one obvious action: add a media folder. During a scan, progress and a cancel
action replace the ready message; the window remains responsive. The ready state uses the product's
accent colour sparingly and includes the same status in text. A failure replaces the primary status
with what happened and the next useful action; technical details remain collapsed until requested.
Do not use a grid of metric cards, decorative gradients, protocol labels or colour as the only state
signal. The simple/advanced choice changes the amount of detail, not this task order.

The native preferences window is secondary. Basic settings use labelled controls with short
explanations. The raw configuration editor and server diagnostics belong in Advanced, with a clear
warning before a user edits technical values. Under OD-16 options A/B, app-driven settings would use
validated API updates and show when a saved value is not yet effective. Under option C, server
settings stay read-only in the app and the F-14/F-15 scope changes. Current direct file writes are a
prototype gap, not an accepted settings flow.

- The application is the primary graphical interface; the command line remains a complete headless path and the application remains optional to the
  server process (PRD-006, UX-001, MB-001).
- The main window opens on a calm overview: one plain-language server state, the chosen media folders, scan progress when active, recent client
  activity and one clear next action. Advanced settings, diagnostics and technical details are secondary destinations.
- First run follows one short path: choose a media folder, explain that files are only read, show scan progress, start the server through the supported
  service lifecycle, then offer an optional discovery check and explain how to find the server on a television.
- The menu bar item is a quick control and status entry point, not the only interface. Selecting its server-window action opens the native main window;
  no task opens a browser.
- Visual treatment is native and restrained: system appearance, semantic colours, clear text hierarchy, whitespace and few persistent panels. Avoid
  decorative dashboards, unexplained icons and technical jargon in the simple mode.
- The first-run assistant uses the system folder picker to let the user choose a media folder. The selection alone does not prove that the separate
  service process can read it, that access is limited to that folder, or that later revocation is enforced; the service access path remains gated by
  OD-15 and PLT-017 (platform-integration §5.4, Q3; F-22).
- Every error follows the "what happened / why / what to do" shape (UX-004), and the technical detail is available on demand behind a disclosure
  control rather than in the primary text (UX-010).
- The status is always available as text (UX-007, UX-018); no state is conveyed by a colour alone (MB-006).
- [DOC-005] *Menu bar*, *status item*, *launchd daemon*, *launchd agent*, *bundle*, *assistant*, *Local Network privacy* and *Rosetta* are explained
  at first use.

## 7. Security and privacy

- The application adds no data flow of its own: it stores its preferences in the user's own preferences directory and talks to the server over
  loopback. Folder selection is not described as a permission grant to the separate service; the exact service identity, access scope and revocation
  behavior remain subject to OD-15 (PRIVACY §9, F-22).
- The application is a client of the administration surface and uses its authentication model — bearer token, CSRF token, custom header (MB-005, F-14,
  SEC-072).
- The daemon's local-network privilege comes from being a launchd daemon. The macOS application runs in the user's session and may need local-network
  approval if it ever makes a local-network request; its current client traffic is loopback-only (platform-integration §1.2, §4.2).
- The application never shows an absolute filesystem path; it shows root names (SEC-094, PLT-008).
- The application performs no network access of its own — no update check, no telemetry (SEC-090, SEC-091, INT-059).

## 8. Interoperability

- The application speaks **no** DLNA or UPnP protocol. It is not a control point; it never sends `M-SEARCH` and never browses the catalogue through
  the protocol (F-01, F-03).
- It interoperates with the *operating system*: the menu bar, the accessibility APIs, the folder picker and the notification centre are the interfaces
  it must satisfy.
- [OBSERVED] The daemon and the agent have different permission behaviour on the same Mac, so the two halves of the product are not interchangeable at
  the platform level (platform-integration §1.2). This is a platform fact, not an interoperability claim.
- [PARTLY VERIFIED] Apple requires administrator approval in System Settings before a bundled `LaunchDaemon` starts and documents its registration
  states. The exact interaction and PhairDLNA's runtime behavior still need a real-Mac run (platform-integration §4.3). See §12.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| The daemon is not running | the application shows "server stopped" and offers to start it | MB-001, UX-019 |
| The daemon stops while the application is open | the application notices the closed connection, shows the state and reconnects when the daemon returns | MB-001, MB-002 |
| The daemon cannot be reached (permission, token) | a readable message saying the application could not talk to the server; the application keeps running | UX-004, MB-005 |
| A scan is cancelled by the user | the scan stops and the progress indicator disappears; the catalogue keeps the entries it already has | UX-013, FUN-024 |
| A folder the user chose is protected by macOS | the denial is explained with the name of the setting to change; the folder is quarantined, not fatal | PLT-017, UX-004, F-08 |
| The firewall blocks inbound connections | the daemon's log carries the one-line diagnostic "firewall may be blocking inbound connections"; the application shows the same hint | platform-integration §2.4, UX-004 |
| The application is closed | the daemon keeps serving; nothing else changes | UX-001, MB-001 |

No failure in the application may affect the daemon, and no failure in the daemon may take the application down with it: they are separate processes
(MB-001).

## 10. Acceptance criteria and traceability

The criteria below are *planned*; the automated ones that exist carry the path
of their test file in the table of § 11. The criteria that need a real Mac
stay manual until the application is installed on one.

**AC-01** — *Given* the daemon stopped and the application running, *when* the user activates the status item, *then* the state is reported as "server
stopped" in text, and activating "Start" brings the server up without the application restarting anything. Requirements: MB-001, UX-018, UX-019 ·
Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-02** — *Given* the application is the only thing running, *when* the daemon is started from the command line, *then* the application reflects the
new state without being relaunched and without showing a cached value. Requirements: MB-001, MB-002 · Verification: manual macOS usability protocol ·
Planned location: `qa/macos-usability-protocol.md`

**AC-03** — *Given* a first run with no configuration, *when* the assistant is opened, *then* at most one action — choosing a folder — is needed
before the server is reachable on the network, and the assistant shows the device name, the address and how to check from another device.
Requirements: UX-002, UX-003 · Verification: manual usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-04** — *Given* a failure reported by the server, *when* it is shown in the application, *then* the text states what happened, why and what to do
next, and contains no absolute filesystem path. Requirements: UX-004, SEC-094 · Verification: manual usability protocol · Planned location:
`qa/macos-usability-protocol.md`

**AC-05** — *Given* the application with the menu bar closed, *when* the server state changes, *then* the status item's accessibility label and value
change to describe the new state in words, and the state is also readable as text when the menu is open. Requirements: UX-007, UX-009, UX-018, MB-003
· Verification: manual macOS usability protocol (VoiceOver) · Planned location: `qa/macos-usability-protocol.md`

**AC-06 [PROPOSED, OD-17]** — *Given* the app text size is 100%, 150% or 200% and "Increase contrast" and "Reduce motion" are enabled, *when* the
application is used, *then* all content and actions remain readable and reachable without clipping or overlap, the contrast requirement is met and no
animation conveys state. Requirements: UX-008, UX-017, MB-003 · Verification: manual macOS usability protocol · Planned location:
`qa/macos-usability-protocol.md`. This replaces the earlier F-15 design interpretation only after owner approval.

**AC-07** — *Given* the application, *when* every control is reached with the keyboard only, *then* each control is reachable, has a spoken label and
a deterministic focus order. Requirements: UX-017, MB-003 · Verification: manual macOS usability protocol · Planned location:
`qa/macos-usability-protocol.md`

**AC-08** — *Given* an active transfer, *when* the user switches between the simple and the advanced mode, *then* the transfer is not interrupted, the
server is not restarted, no setting changes and no new background task starts. Requirements: UX-015, MB-004 · Verification: automated test · Planned
location: `crates/phairdlna-mac/tests/presentation_modes.rs`

**AC-09** — *Given* the advanced mode has a safety-critical warning displayed, *when* the user switches to the simple mode, *then* the warning is
still visible. Requirements: UX-016 · Verification: automated test · Planned location: `crates/phairdlna-mac/tests/presentation_modes.rs`

**AC-10** — *Given* the simple and the advanced mode, *when* both are inspected, *then* they read from the same configuration schema and expose the
same effective settings, and no setting is writable from one mode alone. Requirements: UX-015, UX-014 · Verification: automated test · Planned
location: `crates/phairdlna-mac/tests/presentation_modes.rs`

**AC-11** — *Given* a scan started from the application, *when* it runs, *then* progress is reported and a cancellation action stops the scan while
the server keeps serving, and the catalogue keeps the entries already indexed. Requirements: UX-013, MB-002 · Verification: automated test · Planned
location: `crates/phairdlna-mac/tests/scan_progress.rs`

**AC-12** — *Given* any view in the application, *when* it is opened, *then* no index scan and no network scan is started as a side effect.
Requirements: MB-004 · Verification: automated test · Planned location: `crates/phairdlna-mac/tests/view_side_effects.rs`

**AC-13** — *Given* the application, *when* it is inspected for outgoing connections, *then* it opens no connection other than to the loopback
administration surface, and it sends no telemetry, no update check and no identifier of any kind. Requirements: MB-005, SEC-090, SEC-091 ·
Verification: negative test · Planned location: `crates/phairdlna-mac/tests/no_egress.rs`

**AC-14** — *Given* the application bundle, *when* its metadata is inspected, *then* `Info.plist` contains `NSLocalNetworkUsageDescription` with a
plain-language string, declares macOS 15 as its minimum supported version and does not set `LSUIElement`; the app therefore runs with a regular activation
policy and appears in the Dock; and *given* a helper is registered through `SMAppService`, *then* the helper executable lives inside the bundle. Requirements:
MB-009, DEC-032, DEC-076 · Verification: automated test · Planned location:
`crates/phairdlna-mac/tests/bundle_metadata.rs`

**AC-15** — *Given* the auto-start setting is changed, *when* the user looks at the menu, *then* the current state of auto-start is visible and no
hidden persistent helper remains after the application is removed. Requirements: MB-008, LIFE-010 · Verification: manual macOS usability protocol ·
Planned location: `qa/macos-usability-protocol.md`

**AC-16** — *Given* the first-run assistant, *when* the verification step is offered and no client has discovered the server, *then* the assistant
reports the neutral state without claiming that a television is present and without declaring failure, and the step can be declined. Requirements:
MB-007 · Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-17** — *Given* the release artefact, *when* it is inspected, *then* it is a universal binary containing arm64 and x86_64, it runs natively on
Apple Silicon without a Rosetta translation, and it carries a code signature. Requirements: PLT-004 · Verification: build matrix · Planned location:
`crates/phairdlna-cli/tests/build_matrix.rs` and `qa/device-matrix.toml`

**AC-18** — *Given* every user-facing string, *when* the resource file is inspected, *then* no string is built by concatenation and every string is
present in the resource file for each supported locale; and *given* the supported-locale list, *then* it is recorded before the first release.
Requirements: UX-020, UX-010 · Verification: automated test · Planned location: `crates/phairdlna-mac/tests/strings.rs`

**AC-19** — *Given* a first launch with no media folders, *when* the native main window opens, *then* it shows one clear next action to add a folder,
explains that the files are read-only, and exposes no protocol settings in the simple view. Requirements: UX-002, UX-010, PRD-007 · Verification:
manual usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-20** — *Given* the server is checking, scanning, ready, stopped or degraded, *when* the main window is opened, *then* its first area names the current state
in text and presents only actions that apply to that state; the remaining areas keep the same order. Requirements: UX-004, UX-014, UX-016, MB-002,
MB-006, LIFE-001, LIFE-004, LIFE-009 · Verification: automated view-model test plus manual macOS usability protocol · Planned location:
`crates/phairdlna-mac/tests/main_window.rs`

**AC-21** — *Given* the application starts, *when* the optional first-run folder prompt has been completed or dismissed, *then* the native server overview is visible without requiring the user to find the menu bar item. Requirements: UX-014, MB-001 · Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-22** — *Given* the application has not received its first status response, *when* the overview or menu is shown, *then* it says that the status is being checked and offers neither a start nor a stop action until the response arrives. Requirements: MB-001, UX-007, UX-018 · Verification: automated view-model test · Planned location: `crates/phairdlna-mac/tests/main_window.rs` and `crates/phairdlna-mac/tests/menu.rs`

## 11. Test strategy and dependencies

**Test locations and their state.** The files of the first column that carry a
test exist; the ones that do not yet exist are named with the reason.

| Purpose | Path | State |
| --- | --- | --- |
| Mode switching, shared schema, warning visibility | `crates/phairdlna-mac/tests/presentation_modes.rs` | **exists** (AC-08, AC-09, AC-10) |
| The requests of the client and the answers of the surface | `crates/phairdlna-mac/tests/ipc_client.rs` | **exists** |
| The entries of the menu and the state as text | `crates/phairdlna-mac/tests/menu.rs` | **exists** — AC-22 checking-state coverage is still open |
| The wiring: the state, the failure and the actions | `crates/phairdlna-mac/tests/application.rs` | **exists** |
| The thread that talks to the surface | `crates/phairdlna-mac/tests/worker.rs` | **exists** |
| Bundle metadata and permissions keys | `crates/phairdlna-mac/tests/bundle_metadata.rs` | **exists** (AC-14) |
| No network egress from the application | `crates/phairdlna-mac/tests/no_egress.rs` | **exists** (AC-13) |
| String resources and locale coverage | `crates/phairdlna-mac/tests/strings.rs` | **exists** (AC-18) |
| Main-window checking, empty, scanning, ready, stopped and degraded states | `crates/phairdlna-mac/tests/main_window.rs` | **NOT RUN — acceptance test file does not exist** — initial overview shows live state, folders, an add-folder action, read-only explanation, current activity and advertised TV connection details; state-specific controls, degraded-state mapping and acceptance coverage remain open |
| Progress reporting and cancellation | `crates/phairdlna-mac/tests/scan_progress.rs` | **not built** — the progress of the scan of the library is a property of the catalogue of F-08, and the application observes the state of the surface instead of a progress of its own (AC-11) |
| No side effects from opening a view | `crates/phairdlna-mac/tests/view_side_effects.rs` | **not built** — the rule is covered by `presentation_modes.rs`, which shows that switching the view starts no work (AC-12) |
| Status item state machine | `crates/phairdlna-mac/src/platform/status_item.rs` | **exists** — the state machine is in `src/model.rs` and is tested without a window |
| Keyboard navigation, labels, focus order | `crates/phairdlna-mac/src/platform/accessibility.rs` | **not built** — the labels are written in `src/platform/status_item.rs` and the text of the state in `src/menu.rs`; the order of the keyboard is a property of the menu of the system and is checked by the manual protocol (AC-07) |
| Manual macOS usability protocol | `qa/macos-usability-protocol.md` | **written, not run** — first-launch, reconnection, keyboard, VoiceOver, appearance and TV scenarios are recorded; every real-Mac and TV observation remains `NOT RUN` (AC-01 … AC-07, AC-15, AC-16, AC-21) |
| Device and platform matrix rows | `qa/device-matrix.toml` | **not built** — the matrix belongs to M2.7 and to the release (AC-17) |

**Method.** The presentation-mode and side-effect rules are pure logic over the administration-surface client and are exercised without a GUI.
Everything that only a real Mac can answer — the menu bar, VoiceOver labels, the text-size setting, the Local Network prompt, the firewall prompt and
the bundle identity — is a **manual** protocol on a real machine, recorded with the device and the macOS version. The firewall behaviour is a **device
test** and is explicitly recorded as `NOT RUN — no device available` until a machine is available (QUALITY-GATES §1 rule 4).

**Dependencies.** F-13 (the CLI whose commands the application mirrors and the service lifecycle), F-14 (the administration surface, its token and its
first-run text), F-08 and F-12 (roots, scans and the configuration the assistant writes), F-16 (the security model the application inherits), F-19
(`SUP-009`, the signing plan), and the platform traits of ADR-0003.

## 12. Open questions and decisions

1. **[OPEN, OD-15] Service privilege and folder access.** PLT-013 and PLT-014 are owned by F-22 in M3; they are not unassigned. The macOS app depends
   on the F-22 service and privilege model. OPS-015 (`F-13`) and PLT-018 (`F-22`) require serving media from the user's own directories without elevated
   privileges, while PLT-013 requires a system-context daemon. The effective daemon identity, Local Network behavior for a non-root service, and
   selected-folder access must be established together before the service lifecycle is implemented. *Closes when:* OD-15 resolves the constraints and
   the real-Mac evidence and independent security review pass.
2. **[OPEN] The firewall is the largest macOS device-test risk.** Apple's documentation does not say what happens to the eventual service process
   when it binds a listening socket with the firewall on (platform-integration §2.3), and an ad-hoc signature is not covered by the auto-allow rule.
   The earlier LaunchDaemon recommendation is superseded by OD-15. Verify the firewall behavior and the recovery steps for the approved service model
   on a real Mac, then record the result in `qa/device-matrix.toml`.
3. **[OPEN] Developer ID and notarisation.** Option B (Developer ID + hardened runtime + notarisation) requires a paid Apple Developer Program
   membership (99 USD/year) and a named maintainer — a governance decision for an AGPL volunteer project, not a technical one (platform-integration
   §3.2, §3.3). The plan is `SUP-009` in F-19. *Closes when:* the owner decides whether to buy the membership before the first stable release.
4. **[PARTLY VERIFIED] `SMAppService` behaviour.** Apple documents that a bundled `LaunchDaemon` needs administrator approval in System Settings before
   it can start, and documents `notRegistered`, `enabled`, `requiresApproval` and `notFound` status values (platform-integration §4.3). The exact
   interaction, the supported-version runtime path and access to only user-selected media folders remain unverified on a real Mac. A pure-CLI install
   still has no bundle and therefore uses the explicit CLI service path.
5. **[OPEN] The supported locale list.** UX-020 requires the set to be decided before the first release but does not name it. *Proposed decision, not
   in the baseline:* English (United States) and English (United Kingdom) at launch, with the resource format able to carry more. Rationale: the
   project has no translation team, and an unbounded promise is not deliverable. *Closes when:* the owner fixes the list.
6. **[DECIDED 2026-11-18, DEC-075] Which surface is the "one view" of UX-014.** The native application's main window is the full status and
   administration view. The menu bar menu is a compact shortcut to common actions and status, not the primary workspace. The local API does not serve a
   browser UI.
7. **[OPEN] App controls and the approved daemon model.** The application currently calls `launchctl` for the service label
   `org.phairdlna.server`, and its service helper looks for a plist in `~/Library/LaunchAgents`. The approved platform decision is a system-context
   launchd daemon (`DEC-032`, `PLT-013`), which is a different service type and needs a verified install/control path. `SMAppService` is a documented
   candidate for the bundled-app path, but user-selected folder access and the full runtime lifecycle are not established. The current start and stop
   actions must not be presented as complete until their target is reconciled with that decision. The menu bar status item's own auto-start preference
   is a separate setting. *Closes when:* the application controls the approved daemon safely, and the install/start/stop/recovery path is verified on a
   real Mac. The owner decision is tracked as OD-14.

9. **[OPEN, OD-16] Configuration update and activation boundary.** The first-run wizard and folder action write the config file directly, and the
   wizard returns a configured result and requests a rescan even if persistence fails. The server constructs its media library from startup roots;
   the rescan refreshes that existing library and does not reload the config. A successfully saved new root therefore is not adopted by the running
   server through this path. The preferences Save button has no registered handler; the method behind it includes a non-atomic raw-file write path.
   F-14 has no API write route, and F-12 rejects runtime reload. Options A/B would use a validated API update and distinguish saved from effective
   settings; option C would keep server settings read-only and requires a scope change. *Closes when:* OD-16 selects the activation model and its
   acceptance criteria are reviewed; implementation also depends on OD-14/OD-15.

10. **[OPEN, OD-17] Larger text and layout reflow.** Apple's HIG says macOS does not support Dynamic Type. The overview uses fixed font sizes,
    fixed-height frames and line caps, so it can truncate content and offers no app-level text-size choice. The working proposal is an app-owned size
    preference, independent of server settings; the requirement and AC changes remain proposals until the owner approves them. *Closes when:* the owner
    confirms or changes OD-17 and the rendered app passes 100%, 150% and 200% checks with the longest supported translations.

### Revision history

| Version/date | Change |
| --- | --- |
| 0.2 / 2026-11-18 | DEC-075 makes the native app window the primary GUI, the menu bar item a quick-access surface, and removes the browser UI. Records the current direct-config-write gap. |
| 0.3 / 2026-11-18 | Adds the first native overview, its add-folder action and read-only explanation; AC-19/20 and the remaining flow gaps stay open. Clarifies the existing launchctl implementation against the approved daemon model. |
| 0.4 / 2026-11-18 | Adds current scan/playback activity to the overview and records that TV connection details are still unavailable through the API. |
| 0.5 / 2026-11-18 | Carries the daemon's UPnP friendly name and advertised LAN address through authenticated status into the TV connection area. |
| 0.6 / 2026-11-18 | Includes current SSDP task availability so TV guidance distinguishes active, unavailable and legacy/unknown discovery status. |
| 0.7 / 2026-11-18 | Aligns the application bundle's minimum OS version with DEC-032 (macOS 15). |
| 0.8 / 2026-11-18 | Uses a standard, centered main window with restrained semantic status colors; the menu bar remains quick access. |
| 0.9 / 2026-11-18 | Rewrites the first-run folder prompt in plain language and states that PhairDLNA does not change media files. |
| 1.0 / 2026-11-18 | Makes the macOS app a regular Dock application with a main window and menu-bar status item (DEC-076). |
| 1.1 / 2026-11-18 | Records Apple’s documented LaunchDaemon approval and status values; keeps runtime and user-folder access verification open. |
| 1.2 / 2026-11-18 | Links lifecycle work to OD-15 and records that selected-folder access by a separate service process is unverified. |
| 1.3 / 2026-11-18 | Removes the claim that folder selection itself grants the service access; aligns requirement ownership and surfaces the OPS-015/PLT-018 versus PLT-013 constraint. |
| 1.4 / 2026-11-18 | Clarifies that the app bundle's metadata does not grant access to folders for the service and gates the daemon assumptions on OD-15. |
| 1.5 / 2026-11-18 | Removes the stale firewall mitigation that treated a LaunchDaemon as the chosen service model. |
| 1.6 / 2026-11-18 | Links the direct-write gap to OD-16 and F-12's no-runtime-reload baseline. |
| 1.7 / 2026-11-18 | Makes app settings behavior conditional on OD-16 and records the first-run and preferences implementation gaps. |
| 1.8 / 2026-11-18 | Records that the first-run rescan refreshes the startup-configured library and cannot activate newly saved roots. |
| 1.9 / 2026-11-18 | Corrects the Dynamic Type claim for macOS and gates larger-text behavior and layout acceptance on OD-17. |
| 2.0 / 2026-11-18 | Proposes app-level text scaling under OD-17 and makes the proposed larger-text acceptance measurable through 200%. |
