# F-15 — Native macOS experience

Status: In progress — the regular native macOS application, main overview, TV connection guidance, preferences window and authenticated API client are implemented in part (M2.6); DEC-079 selects app-managed service registration and local bootstrap plus authenticated settings updates, while service and configuration writes remain gated by OD-15 evidence; universal packaging passes locally, while first-run, discovery and accessibility acceptance remain open

Owner role: Product/Requirements

Version/date: 2.80 / 2026-11-19
Purpose: Specify the native macOS application as the primary graphical setup and administration surface.
References: `../00-governance/PROJECT-STANDARDS.md`, `../00-governance/REQUIREMENTS-METHOD.md`, `F-14`, `F-13`, `../07-project/ROADMAP.md`, `DEC-075`, `DEC-076`, `OD-14`, `OD-15`, `OD-16`.

Requirements: UX-001, UX-002, UX-003, UX-004, UX-005, UX-006, UX-007, UX-008, UX-009, UX-010, UX-013, UX-014, UX-015, UX-016, UX-017, UX-018, UX-019, UX-020, MB-001, MB-002, MB-003, MB-004, MB-005, MB-006, MB-007, MB-008, MB-009, PLT-004
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

**PhairDLNA's server is a headless program.** It answers discovery requests on the local network and streams files; it has no window, no icon in the
Dock and no settings page. On macOS the product pairs that server with a **regular native application** that guides setup and everyday use. A menu bar
item — the small icon in the top-right strip of the screen — gives quick status and common actions; the application's window is the main place to
understand and manage the server. The approved service identity, launch relationship and access to selected folders remain open under OD-15; this
specification does not treat a LaunchDaemon or LaunchAgent as selected.

This feature specifies the user-visible half of that pair: the main window, the status item, the two presentation modes, the first-run assistant, the
accessibility behaviour and the installation story. It specifies the **front end only**. The server logic lives in `phairdlna-core` and `phairdlna-cli`; the
application talks to it over the local administration surface and never embeds it (MB-001, ADR-0003).

Three macOS facts shape everything below.

1. **macOS 15 introduced Local Network privacy** (platform-integration §1.2, Apple TN3179). A program that sends or *receives* local network traffic
   needs the user's permission. `launchd` **daemons** and any process running as **root** are exempt; a per-user `launchd` **agent** is **not** —
   Apple says so explicitly. This explains why PLT-013 currently describes a daemon, but it does not settle OD-15 or establish the service model for
   this product. The regular application can show a Dock icon and main window while its menu-bar status item gives quick access; its activation policy
   is regular (Apple AppKit `NSApplicationActivationPolicyRegular`).
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

**UX-004** — Every error message shall state what happened, why it happened and what the user can do next. [DESIGN] The application presents
server errors in a translated three-part shape and maps local configuration failures to specific guidance; it does not expose raw filesystem errors or
absolute filesystem paths (SEC-094).

**UX-005** — The server shall never abort with a panic message that lacks context; unexpected failures shall produce a structured internal error and a
readable one-line summary. [DESIGN] The panic hook is installed by the daemon, not by the application; a panic in the *application* must never take
the daemon down with it, because the daemon is a separate process (MB-001).

**UX-006** — The command line shall provide `--help` for every subcommand. [DESIGN] The menu bar application is a front end for those subcommands; its
help text names the equivalent command so that a user can leave the graphical world.

**UX-007** — The macOS application shall be accessible with the keyboard alone and shall expose the status of the server as text, not only as an icon.
[DESIGN] The status item's menu opens with the keyboard, every item has a text label, and the status line is a sentence ("Server running — serving 2
clients"), never a coloured dot.

**UX-008** — The macOS application shall respect the system accessibility settings for contrast and motion and let users enlarge app text to at least
200% with a layout that reflows without clipping. [DESIGN] Colours come from the system semantic palette. Apple says macOS does not support Dynamic
Type, so PhairDLNA offers its own text-size control (DEC-077).

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

**UX-017** — The user interface shall be operable with the keyboard alone, shall expose accessible labels and a deterministic focus order, shall
provide reflowing app-level text scaling and shall respect system contrast settings. [DESIGN] Focus order follows the visual order of the controls and
never depends on where the pointer was last; every control has a label that reads as a sentence.

**UX-018** — The user interface shall expose the status of the server as text for assistive technologies, not only as an icon. [DESIGN] Same mechanism
as UX-009: the server state has a textual rendering that is always available, whether or not the menu is open.

**UX-019** — The user shall be able to start and stop the server, pause and resume indexing, and remove a shared folder without touching the source
files. [DESIGN] "Stop" means "stop serving on the network", not "shut down the process"; the daemon keeps the catalogue. Removing a folder removes it
from the catalogue and the configuration; the files themselves are never touched (FUN-018, F-08).

**UX-020** — All user-facing strings shall be prepared for translation, and the set of supported locales shall be decided before the first release.
[DESIGN] Strings live in one resource file per locale, never inline in code, and no string is built by concatenation; a locale list is fixed before
the first release. The native macOS app selects the first supported language from the user's preferred macOS languages; process locale variables are
the fallback when none of those preferences is supported. The list is an open question (§12).

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
- **Lifecycle state is an upstream contract, not an app inference.** AC-20 includes *Degraded*, but `/api/status` currently has no lifecycle-state field and
  the app maps every successful status response to `Running`. The core `LifecycleState` and `Timeline` types have transition and redaction tests, but the
  daemon runtime does not use them. Do not infer *Degraded* from warnings or unavailable discovery. F-13 must provide an authoritative runtime state and
  reason, F-14 must expose that contract through the authenticated local API, and F-15 must map and test it before AC-20 can cover *Degraded*.
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
   action after the approved configuration and service path exists. Do not show full filesystem
   paths in the simple view.
3. **TV connection** — the server name and local address, followed by two plain steps: open the TV's
   media-source list and choose PhairDLNA. A discovery check is optional and reports only what the
   server observed (`MB-007`). If an older API omits the server name, use the validated name from
   the app's resolved configuration; if no valid configuration exists, explain that the name is
   unavailable. When no network address is reported, do not claim the server is local-only; tell
   the user to look for its name in the TV's media sources and check network discovery if absent.
4. **Recent activity** — the current scan or active playback when present; otherwise a quiet empty
   state. Diagnostics and technical details live behind a secondary action or disclosure.

The target empty state has one obvious action: add a media folder. DEC-079 selects app-managed
registration and a local bootstrap/settings path, but folder changes and service controls stay
disabled until OD-15's service-access experiment passes and the approved flows are implemented. The
app explains how to proceed using the installation instructions. During a scan, progress and a cancel action replace the
ready message; the window remains responsive. The ready state uses the product's
accent colour sparingly and includes the same status in text. A failure replaces the primary status
with what happened and the next useful action; technical details remain collapsed until requested.
The overview and common folder and scan summaries describe catalogue counts as media items in
everyday language, rather than using DLNA's internal term "objects".
The scroll area follows the resizable window. When the overview content is shorter than the
initial viewport, it is vertically balanced rather than leaving a large empty band above the
first heading. Longer lists and larger text remain reachable by vertical scrolling; resizing
never hides an action or clips a section.
The menu shows folder details only when the server supplied them; stopped or refusing states do not
claim that no folders are configured. It does not offer server start/stop actions until OD-15's
service identity and access model pass review and the approved registration flow is implemented; it
explains that the user must follow the installation instructions in the meantime.
Do not use a grid of metric cards, decorative gradients, protocol labels or colour as the only state
signal. The simple/advanced choice changes the amount of detail, not this task order.

The native preferences window is secondary. Under DEC-079's selected but not yet implemented flow,
server fields, shared-folder paths and the expert configuration document remain read-only; the dead
Save button stays removed. App-owned presentation preferences remain interactive. The first-run
path creates the initial typed config locally; later settings use validated API updates and show when
a saved value is not yet effective. Both paths remain gated by OD-15's service evidence and secure
implementation.

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
- The Dock icon should identify PhairDLNA as a local media library server rather than a video player. Use a simple library/server metaphor and keep it
  recognizable at small sizes without text or fine detail.
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
- The local-network privacy behavior depends on the approved server service identity, which remains open under OD-15. The macOS application runs in the
  user's session and may need local-network approval if it ever makes a local-network request; its current client traffic is loopback-only
  (platform-integration §1.2, §4.2).
- The simple overview and user-facing error messages omit absolute filesystem paths. Read-only
  Preferences may show configured folder paths so users can inspect the server configuration.
  Diagnostics and logs redact paths under SEC-094; client-facing media URIs omit local paths under
  PLT-008. The Preferences path display is local configuration information, not diagnostic output.
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
| The app cannot read its settings | explain that the file may be missing or invalid and that native first-time setup is not available yet; mention `phairdlna-cli setup` only conditionally if the CLI is installed, otherwise ask the user to contact whoever installed PhairDLNA; tell an existing user to make the settings file readable and valid before reopening | UX-004, AC-19 |
| The server is not running | the application shows "server stopped"; starting it from the app is a target gated by OD-14/OD-15, not a verified current capability | MB-001, UX-019 |
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

**AC-06** — *Given* the app text size is 100%, 150% or 200% and "Increase contrast" and "Reduce motion" are enabled, *when* the
application is used, *then* all content and actions remain readable and reachable without clipping or overlap, the contrast requirement is met and no
animation conveys state. Requirements: UX-008, UX-017, MB-003 · Verification: manual macOS usability protocol · Planned location:
`qa/macos-usability-protocol.md`.

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
`crates/phairdlna-cli/tests/build_matrix.rs`, `scripts/package-macos-app.sh` (mandatory `lipo -verify_arch`), and `qa/device-matrix.toml`

**AC-18** — *Given* every user-facing string, *when* the resource file is inspected, *then* no string is built by concatenation and every string is
present in the resource file for each supported locale; and *given* the supported-locale list, *then* it is recorded before the first release.
Requirements: UX-020, UX-010 · Verification: automated test · Planned location: `crates/phairdlna-mac/tests/strings.rs`

**AC-19** — *Given* a first launch with no media folders, *when* configuration cannot be read, *then* the app explains the problem, does not show
fallback values as real settings, does not expose an absolute path or raw filesystem error, and
keeps folder changes unavailable until valid configuration and approved service/configuration paths
exist; no picker opens and no server setting is written until the DEC-079 flow is implemented after
OD-15 evidence passes. Persistence failures must be shown as errors and never as successful setup,
and the overview must explain when saved folders become active. The
overview shows one clear next action, explains that files are read-only, and exposes no protocol settings in the simple view. Requirements: UX-002,
UX-010, PRD-007 · Verification: manual usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-20** — *Given* the server is checking, scanning, ready, stopped or degraded, *when* the main window is opened, *then* its first area names the current state
in text and presents only actions that apply to that state; the remaining areas keep the same order. When the server supplies its status, every configured shared
folder is listed and one whose volume is unavailable is identified in words. When the server is stopped or refuses access, the overview explains that folder
details are unavailable instead of claiming that no folders are configured; library counts and playback activity are also unavailable, never shown as zero or empty. The add-folder action follows the folder list and is visible without scrolling in the default window at 100% text size; it remains disabled until valid configuration and approved service/configuration paths exist. When the window is resized, the scroll area follows its new dimensions and every section remains reachable without clipping. Requirements: UX-002, UX-004, UX-014, UX-016, MB-002, MB-006, LIFE-001, LIFE-004, LIFE-009 · Verification:
automated view-model test plus manual macOS usability protocol · Planned location: `crates/phairdlna-mac/tests/overview.rs` and `qa/macos-usability-protocol.md`

**AC-21** — *Given* the application starts, *when* the optional first-run folder prompt has been completed or dismissed, *then* the native server overview is visible without requiring the user to find the menu bar item. Requirements: UX-014, MB-001 · Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

**AC-22** — *Given* the application has not received its first status response, *when* the overview or menu is shown, *then* it says that the status is being checked and offers neither a start nor a stop action until the response arrives. Requirements: MB-001, UX-007, UX-018 · Verification: automated view-model test · Planned location: `crates/phairdlna-mac/tests/overview.rs` and `crates/phairdlna-mac/tests/menu.rs`

**AC-23** — *Given* the user exports a diagnostics report, *when* the report is saved or the write fails, *then* the app confirms success in the overview or explains the save error in a native alert. Requirements: PRD-014 · Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

## 11. Test strategy and dependencies

**Test locations and their state.** The files of the first column that carry a
test exist; the ones that do not yet exist are named with the reason.

| Purpose | Path | State |
| --- | --- | --- |
| Pure overview document-height boundaries | `crates/phairdlna-mac/src/platform/main_window/layout/geometry.rs` | **exists** — short, exact-fit and scrollable heights are covered; rendered window behavior remains manual |
| Conservative multiline-copy line-count boundaries | `crates/phairdlna-mac/src/platform/main_window/layout/reflow.rs` | **exists** — checks 100% and 200% width boundaries, hard line breaks and wide Unicode; actual AppKit clipping remains manual |
| Mode switching, shared schema, warning visibility | `crates/phairdlna-mac/tests/presentation_modes.rs` | **exists** (AC-08, AC-09, AC-10) |
| The requests of the client and the answers of the surface | `crates/phairdlna-mac/tests/ipc_client.rs` | **exists** |
| The entries of the menu and the state as text | `crates/phairdlna-mac/tests/menu.rs` | **exists** — checking-state and Refused-state action coverage; both offer neither Start nor Stop |
| The content of the primary overview | `crates/phairdlna-mac/tests/overview.rs` | **exists** — checks checking, scanning, running, stopped and refused text, prevents stale-data display, lists all folders, marks missing volumes and verifies useful TV discovery guidance in German/English; does not verify AppKit rendering or degraded-state mapping |
| Localized accessible names for dynamic overview values | `crates/phairdlna-mac/tests/accessibility_labels.rs`, `crates/phairdlna-mac/tests/strings.rs` | **exists** — protects the accessibility-label bindings and locale key coverage; spoken VoiceOver behavior remains manual |
| Folder-action safety while service and configuration decisions are open | `crates/phairdlna-mac/tests/folder_actions.rs` | **exists** — covers missing, malformed and valid configuration; the action stays disabled and a direct action leaves the configuration byte-for-byte unchanged |
| First-run branch selection and startup continuation (AC-19, AC-21) | `crates/phairdlna-mac/src/onboarding.rs`, `crates/phairdlna-mac/tests/onboarding.rs` | **exists** — four cases cover missing and malformed configuration, valid configuration without roots, and valid roots; recovery states map to localized notice keys. The overview opens without a blocking alert and places the notice below server status; actual window visibility, notice rendering and VoiceOver remain manual. |
| The wiring: the state, the failure and the actions | `crates/phairdlna-mac/tests/application.rs` | **exists** |
| The thread that talks to the surface | `crates/phairdlna-mac/tests/worker.rs` | **exists** |
| Bundle metadata and permissions keys | `crates/phairdlna-mac/tests/bundle_metadata.rs` | **exists** (AC-14) |
| No network egress from the application | `crates/phairdlna-mac/tests/no_egress.rs` | **exists** (AC-13) |
| String resources and locale coverage | `crates/phairdlna-mac/tests/strings.rs`, `crates/phairdlna-mac/tests/strings/recovery_copy.rs`, `crates/phairdlna-mac/tests/support/locale_contract.rs` | **exists** (AC-18); includes preferred-language priority, environment fallback and truthful recovery-copy cases |
| Progress reporting and cancellation | `crates/phairdlna-mac/tests/scan_progress.rs` | **not built** — the progress of the scan of the library is a property of the catalogue of F-08, and the application observes the state of the surface instead of a progress of its own (AC-11) |
| No side effects from opening a view | `crates/phairdlna-mac/tests/view_side_effects.rs` | **not built** — the rule is covered by `presentation_modes.rs`, which shows that switching the view starts no work (AC-12) |
| Status item state machine | `crates/phairdlna-mac/src/platform/status_item.rs` | **exists** — the state machine is in `src/model.rs` and is tested without a window |
| Keyboard navigation, accessible labels, focus order | `crates/phairdlna-mac/src/platform/status_item.rs`, `crates/phairdlna-mac/src/platform/preferences_window/general.rs`, `crates/phairdlna-mac/src/platform/preferences_window/roots.rs`, `crates/phairdlna-mac/src/platform/preferences_window/tabs.rs`, `qa/macos-usability-protocol.md` | **partly implemented** — explicit spoken names are set for the status item, read-only server values, shared-folder list and expert configuration text. Keyboard focus order, native rendering and actual VoiceOver output remain `NOT RUN`; no separate accessibility module is needed (AC-05, AC-07) |
| Accessible names for overview values | `crates/phairdlna-mac/src/platform/main_window/layout.rs`, locale resources, `crates/phairdlna-mac/tests/strings.rs` | **implemented, runtime unverified** — changing status, library, folder, TV-help, activity and notice text fields now have localized accessibility labels in all shipped locales; the VoiceOver spoken output remains `NOT RUN` (AC-05, AC-07) |
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

1. **[OPEN, OD-15; preferred test candidate recorded in DEC-079] Service privilege and folder access.** PLT-013 and PLT-014 are owned by F-22 in M3;
   they are not unassigned. DEC-079 directs the team to test a per-user `LaunchAgent` first. Do not revise PLT-013 or ship that service until the actual
   server process proves ordinary-user identity, selected-folder read, unselected-sibling denial, revocation and Local Network behavior, followed by
   independent security review. *Closes when:* the OD-15 experiment and review pass and the owner approves the PLT-013 change.
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
7. **[DECIDED DIRECTION, DEC-079; implementation gated by OD-15] App controls and service integration.** The former `launchctl` Start/Stop prototype
   was removed because it targeted an unapproved per-user `LaunchAgents` plist. DEC-079 selects
   app-managed `SMAppService` registration and retains CLI lifecycle support; the bundled helper
   type follows the OD-15 result. User-selected folder access and runtime lifecycle are not yet
   established, so the menu has no service Start/Stop action. *Closes when:* the OD-15 experiment
   and independent review pass, the owner approves the matching PLT-013 change, and registration,
   start/stop and recovery are verified on a real Mac.

9. **[DECIDED DIRECTION, DEC-079; implementation gated by OD-15] Configuration update, bootstrap and activation boundary.** A fresh account without a config reaches a localized, nonmodal recovery
   notice in the overview, so AC-03 is not met. The first-run folder picker and app-side configuration writes have been removed while OD-16 is
   until OD-16 was decided; the main-window control remains disabled pending OD-15. The server constructs its media library from startup roots, and
   `/api/catalogue/rescan` refreshes that existing library without reloading config. F-14 has no API write route, and F-12 rejects runtime reload.
   Options A/B would use a validated API update and distinguish saved from effective settings; option C would keep server settings read-only and
   requires a scope change. DEC-079 selects local app creation of the initial typed config, then authenticated API updates with complete validation,
   atomic persistence and a controlled restart. *Closes when:* the bootstrap and write path pass negative, boundary and recovery review, and the
   approved service consumes the saved roots; implementation depends on OD-15 evidence.

10. **[DECIDED, DEC-077] Larger text and layout reflow.** The owner selected an app-owned size preference, independent of server settings, through
    200%. Apple's HIG says macOS does not support Dynamic Type. The app now stores the size separately, exposes 100%, 150% and 200% in preferences and the menu,
    and reflows overview sections vertically according to the chosen size and a conservative full-em text-width estimate, with character-boundary wrapping inside a vertical scroll view. The Preferences tabs also scale their text and vertical layout inside a scrollable tab area. The visual result,
    no-clipping behavior, system contrast and reduced-motion behavior still need manual verification.
    *Implementation closes when:* the rendered app passes 100%, 150% and 200% checks with the longest supported translations and an independent
    accessibility review.

### Revision history

| Version/date | Change |
| --- | --- |
| 2.81 / 2026-11-19 | Adds compatibility fallback for a missing server name in older local APIs and gives actionable TV discovery guidance when the API omits a network address. |
| 2.80 / 2026-11-19 | Records DEC-079 owner direction for app-managed service registration and app bootstrap/API settings flow; keeps service implementation conditional on OD-15's experiment and independent review. |
| 2.79 / 2026-11-19 | Records that user-domain app registration also exited successfully without restoring the missing Launch Services database path; GUI startup remains NOT RUN. |
| 2.78 / 2026-11-19 | Records that non-deleting Launch Services seeding did not restore the missing user database path; a full database reset needs owner approval and GUI launch remains NOT RUN. |
| 2.77 / 2026-11-19 | Rebuilds the current universal bundle and records valid signature/architecture checks; Launch Services and Spotlight still reject app discovery in this session, so GUI startup remains NOT RUN. |
| 2.76 / 2026-11-19 | Removes 37 untranslated-at-runtime legacy keys from all five locale resources and keeps the exact locale contract aligned with current app code. |
| 2.75 / 2026-11-19 | Distinguishes a missing configuration, which may use CLI setup, from an existing invalid configuration, which must be repaired; adds five-locale regression coverage. |
| 2.74 / 2026-11-19 | Moves the five-locale recovery-copy contract into a focused test module to keep locale coverage within the source-size target. |
| 2.73 / 2026-11-19 | Removes unsupported installer and app-menu lifecycle instructions from the empty-library, configuration-recovery, menu and server-stopped copy in all five locales; rendered text remains NOT RUN. |
| 2.72 / 2026-11-19 | Replaces the misleading play-button Dock icon with a high-resolution library/server mark; adds a bundle-resource regression check and real-Mac small-size icon review remains pending. |
| 2.71 / 2026-11-19 | Refines German overview copy for natural, precise everyday wording around stopped state, media counts, folder setup, read-only behavior and TV connection; rendered copy remains NOT RUN. |
| 2.70 / 2026-11-19 | Packages the shared-constant and boundary-test revision as a signed universal bundle; launch and rendered accessibility remain NOT RUN. |
| 2.69 / 2026-11-19 | Uses one shared guidance-font constant for AppKit text and the line-height estimate; adds 100% / 200%, hard-break and wide-Unicode boundary tests. GUI clipping remains NOT RUN. |
| 2.68 / 2026-11-19 | Rebuilds the font-aligned recovery layout into a signed universal arm64/x86_64 bundle; launch and rendered clipping remain NOT RUN. |
| 2.67 / 2026-11-19 | Sets the inline recovery notice and read-only guidance to the font sizes and wrapping rules used by their height calculation; GUI clipping remains NOT RUN. |
| 2.66 / 2026-11-19 | Records the fresh universal arm64/x86_64 package and strict ad-hoc signature verification for the inline-recovery build; GUI launch and rendering remain NOT RUN. |
| 2.65 / 2026-11-19 | Moves repeated first-run configuration recovery from a blocking startup alert into the overview below server status; adds tested recovery-notice mapping and updates the manual acceptance protocol. GUI rendering remains NOT RUN. |
| 2.64 / 2026-11-19 | Clarifies that the simple overview and errors omit absolute paths while read-only Preferences may expose them for configuration inspection; diagnostics and client URIs keep their separate redaction rules. |
| 2.63 / 2026-11-19 | Removes unapproved app-side folder writes and LaunchAgent start/stop commands; the UI explains why these controls are unavailable pending OD-14/OD-15/OD-16. |
| 2.62 / 2026-11-19 | Gives changing overview values explicit localized accessibility names and records that spoken output still needs VoiceOver verification. |
| 2.61 / 2026-11-19 | Extracts the pure first-run decision and covers missing, malformed, empty and populated configuration; modal and rendered-window acceptance remain separate. |
| 2.60 / 2026-11-19 | Identifies the untested first-run decision branches and defines a bounded, decision-free implementation and verification increment for AC-19/AC-21. |
| 2.59 / 2026-11-19 | Packages the conditional missing-settings recovery guidance in a verified arm64 bundle; runtime remains open. |
| 2.58 / 2026-11-19 | Makes missing-settings guidance honest about the unavailable native bootstrap and conditional on the separately installed CLI; gives existing users a recovery step in all shipped locales. |
| 2.57 / 2026-11-19 | Packages plural menu labels and tooltips for the multi-folder picker in a verified arm64 bundle. |
| 2.56 / 2026-11-19 | Makes folder-picker tooltips accurately describe selection of one or more folders in every shipped locale. |
| 2.55 / 2026-11-19 | Makes the status-menu Add media folders label match the native multi-select folder picker and records a two-folder acceptance case. |
| 2.54 / 2026-11-19 | Packages the revised five-locale unexpected-response copy in a verified arm64 bundle; runtime remains open. |
| 2.53 / 2026-11-19 | Rewrites the unexpected-server-response error in all shipped locales with a plain next step. |
| 2.52 / 2026-11-19 | Corrects the accessibility implementation map: labels now point to their real source files; keyboard and VoiceOver acceptance remain manual. |
| 2.51 / 2026-11-19 | Packages the action-module refactor as a verified arm64 bundle; runtime remains unverified. |
| 2.50 / 2026-11-19 | Separates menu-action execution from the app state facade to bring `app.rs` below the 300-line target. |
| 2.49 / 2026-11-19 | Aligns app crate documentation with DEC-075 and records that source review found no pre-registration Launch Services call; runtime cause remains unknown. |
| 2.48 / 2026-11-19 | Repackages the Preferences accessibility labels as a verified arm64 bundle; Launch Services and VoiceOver remain open. |
| 2.47 / 2026-11-19 | Gives the read-only expert TOML text area the localized tab name for VoiceOver; manual screen-reader acceptance remains open. |
| 2.46 / 2026-11-19 | Repackages the explicit Preferences accessibility names in a verified arm64 bundle; VoiceOver/runtime remain open. |
| 2.45 / 2026-11-19 | Gives read-only server values and the shared-folder list explicit VoiceOver names in Preferences; real VoiceOver observation remains open. |
| 2.44 / 2026-11-19 | Packages the current Preferences-scaling and resize-following source as an arm64 app with a verified ad-hoc signature; runtime remains open. |
| 2.43 / 2026-11-19 | Makes Preferences tab scrolling follow window resizing and explicitly enables vertical scrolling for all long-form tabs. |
| 2.42 / 2026-11-19 | Counts explicit line breaks in long server-name geometry so the Preferences document grows to keep every line reachable. |
| 2.41 / 2026-11-19 | Separates the Preferences footer from the tab viewport at 200% and wraps long server-name values in the General tab. |
| 2.40 / 2026-11-19 | Extends app text scaling to Preferences, including all three tabs, with vertical reflow and a scrollable tab viewport. |
| 2.39 / 2026-11-19 | Records the third AppKit registration abort and maps the accompanying Launch Services status `-10813` to unavailable requested data; runtime acceptance remains open. |
| 2.38 / 2026-11-19 | Disables the app's folder action when the validated configuration is unavailable and adds negative/recovery coverage. |
| 2.37 / 2026-11-19 | Records missing user Launch Services database metadata and another failing system-app control; runtime acceptance remains open. |
| 2.36 / 2026-11-19 | Records a second direct-launch abort with the same Launch Services database failure; runtime acceptance remains open. |
| 2.35 / 2026-11-19 | Rebuilds the current startup-order source as a signed universal arm64/x86_64 bundle; Launch Services runtime acceptance remains open. |
| 2.34 / 2026-11-19 | Adds a system-app control confirming the Codex session's Launch Services failure is not PhairDLNA-specific; real-Mac runtime acceptance remains open. |
| 2.33 / 2026-11-19 | Records the Launch Services database failure in this Codex session and keeps native runtime and rendered-window acceptance open; initializes AppKit before starting the network worker. |
| 2.32 / 2026-11-19 | Adds negative packaging checks for a missing architecture slice and a modified resource in a signed app bundle. |
| 2.31 / 2026-11-19 | Covers the short-content, exact-fit and long-content boundaries of vertical overview sizing. |
| 2.30 / 2026-11-19 | Corrects application refresh change reporting and aligns regression expectations with localized recovery guidance and the current transport module. |
| 2.29 / 2026-11-19 | Adds regression coverage for preferred-language priority and fallback to the process locale. |
| 2.28 / 2026-11-19 | Resizes the overview scroll area with its window and balances short content vertically in the available viewport. |
| 2.27 / 2026-11-19 | Selects the first supported macOS preferred language before environment fallbacks, so Finder-launched app UI follows the user's system language. |
| 2.26 / 2026-11-19 | Replaces the vague TV-connection fallback with a plain explanation and next step when the server name is missing. |
| 2.25 / 2026-11-19 | Hides unavailable folder details from the menu, suppresses an inapplicable start action when the server refuses access and uses plain-language state copy. |
| 2.24 / 2026-11-19 | Makes the packager fail if its output lacks the architecture slices required by the selected packaging mode; the universal build and final signature passed with this guard enabled. |
| 2.23 / 2026-11-19 | Replaces DLNA-specific “objects” wording in the main overview and common summaries with plain-language item counts. |
| 2.22 / 2026-11-19 | Records that missing-config startup cannot reach folder onboarding and adds initial configuration bootstrap to OD-16. |
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
| 2.1 / 2026-11-18 | Applies owner direction DEC-077 and makes app-owned 200% text scaling a requirement with layout reflow. |
| 2.2 / 2026-11-18 | Records the persisted preference and menu controls, responsive vertical overview layout, conservative wrapping estimate and still-open real-Mac acceptance. |
| 2.3 / 2026-11-18 | Makes the server fields and expert configuration view read-only while OD-16 is open, removes the dead Save button and records the remaining direct-write prototype paths. |
| 2.4 / 2026-11-18 | Shows an explicit unavailable state instead of default values when configuration cannot be loaded. |
| 2.5 / 2026-11-18 | Shares first-run folder persistence with the overview action, reports write failures, and states that saved folders require a server restart. |
| 2.6 / 2026-11-18 | Explains missing or invalid configuration during first launch instead of silently skipping onboarding. |
| 2.7 / 2026-11-18 | Confirms diagnostics export success and surfaces file-write errors. |
| 2.8 / 2026-11-18 | Removes obsolete preference strings that implied read-only server settings and folders could be edited and saved. |
| 2.9 / 2026-11-18 | Lists every configured shared folder in the overview and describes unavailable volumes in words instead of status punctuation or a hidden-count summary. |
| 2.10 / 2026-11-18 | Explains that folder details cannot be retrieved while the server is stopped or refuses access, instead of implying that the configuration has no folders. |
| 2.11 / 2026-11-18 | Places the add-folder action directly after the shared-folder list so the primary setup action appears in the default window. |
| 2.12 / 2026-11-19 | Hides library and playback values when the app cannot reach the server and replaces the terminal-specific missing-server message with plain language. |
| 2.13 / 2026-11-19 | Moves the full canonical locale-key inventory into a focused support module to keep the locale acceptance source below the project file-size target. |
| 2.14 / 2026-11-19 | Makes the macOS packager ad-hoc-sign the fully assembled app bundle and verify its final signature, as required for Apple silicon execution. |
| 2.15 / 2026-11-19 | Adds a configurable package output directory and records successful arm64-only and universal package/signature verification, while keeping runtime and release acceptance open. |
| 2.16 / 2026-11-19 | Clarifies that a universal package build does not prove release-artifact runtime acceptance. |
| 2.17 / 2026-11-19 | Removes claims that a LaunchDaemon is already selected and aligns service-control statements with OD-14/OD-15. |
| 2.18 / 2026-11-19 | Maps configuration read/write failures to localized guidance without exposing absolute paths or raw filesystem errors. |
| 2.19 / 2026-11-19 | Replaces terminal-oriented and raw transport/authentication errors with localized app guidance for everyday users. |
| 2.20 / 2026-11-19 | Removes raw system-error details from server controls, display preferences, diagnostics saving and item-opening messages. |
| 2.21 / 2026-11-19 | Applies the same plain-language handling to view-preference persistence and application startup errors. |
