# F-15 — Native macOS experience

Status: Draft

Owner role: Product/Requirements

Version/date: 0.1 / 2026-11-14

Requirements: UX-001, UX-002, UX-003, UX-004, UX-005, UX-006, UX-007, UX-008, UX-009, UX-010, UX-013, UX-014, UX-015, UX-016, UX-017, UX-018, UX-019, UX-020, MB-001, MB-002, MB-003, MB-004, MB-005, MB-006, MB-007, MB-008, MB-009, PLT-004
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

**PhairDLNA's server is a headless program.** It answers discovery requests on the local network and streams files; it has no window, no icon in the
Dock and no settings page. On macOS the *product* is therefore two pieces that meet at one boundary: a **launchd daemon** that does the work, and a
**menu bar application** — the small icon in the top-right strip of the screen — that shows what the server is doing and lets the user steer it.

This feature specifies the user-visible half of that pair: the status item, the two presentation modes, the first-run assistant, the accessibility
behaviour and the installation story. It specifies the **front end only**. The server logic lives in `phairdlna-core` and `phairdlna-cli`; the
application talks to it over the local administration surface and never embeds it (MB-001, ADR-0003).

Three macOS facts shape everything below.

1. **macOS 15 introduced Local Network privacy** (platform-integration §1.2, Apple TN3179). A program that sends or *receives* local network traffic
   needs the user's permission. `launchd` **daemons** and any process running as **root** are exempt; a per-user `launchd` **agent** is **not** —
   Apple says so explicitly. The always-on server is therefore a daemon; the menu bar application is an agent.
2. **The macOS application firewall is per application** (platform-integration §2). An ad-hoc-signed binary is not covered by the "automatically allow
   downloaded signed software" rule, so the firewall may prompt — and until the user answers, incoming connections are refused. The platform research
   calls this the single largest residual macOS risk and insists it be exercised on a real machine.
3. **A pure-Rust menu bar application is realistic** (platform-integration §7). `muda` implements its macOS backend on `objc2` + `objc2-app-kit`, that
   is on `NSStatusItem`. No Swift, no webview.

**User story.** As a Mac owner, I want a small icon in my menu bar that tells me the server is running and shows me what is playing, so that I can
start it, pause a scan and see a problem without opening a terminal; and I want that icon to be the *only* thing I have to learn, so that the server
keeps working when I never touch it.

## 2. Scope

**In scope** — the menu bar status item and its menu; the two presentation modes (simple and advanced) over one configuration schema; the first-run
assistant; the translation readiness of user-facing strings; the accessibility behaviour (keyboard, VoiceOver labels, text size, contrast, reduced
motion); the relationship between the application and the daemon (reconnection, no cached assumptions); the `.app` bundle and its permission metadata;
the uninstall path; and the Apple Silicon native build.

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
system semantic palette so that the increase-contrast and reduce-motion settings apply; text is laid out in dynamic type so that the system text size
applies.

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
pointer was last; every control has a label that reads as a sentence.

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

**MB-009** — The menu bar application shall ship as a bundle with the metadata the operating system requires for its local-network and folder-access
permissions. [DESIGN] The application is an `.app` bundle with an `Info.plist` carrying `NSLocalNetworkUsageDescription` (platform-integration §1.3)
and, if it ever registers a helper through `SMAppService`, the helper executable lives inside the bundle (platform-integration §4.3). `LSUIElement`
marks it as a menu-bar-only application.

**PLT-004** — The server shall support macOS Apple Silicon natively without a Rosetta translation requirement. [DESIGN] The build matrix contains
`aarch64-apple-darwin` and the artefacts are universal (`arm64` + `x86_64`) so that the same file runs on Intel and Apple Silicon. On Apple Silicon an
arm64 binary must carry *some* valid signature to execute at all — ad-hoc is enough (platform-integration §3.1).

## 4. Technical requirements

- **Two processes, one contract.** The daemon (`phairdlna-cli` as a `launchd` daemon) owns everything; the application (`phairdlna-mac`) is a client
  of the administration surface. The daemon starts at boot with no user logged in; the application starts at login (platform-integration §4.2).
- **Why a daemon and not an agent for the server.** TN3179 exempts launchd *daemons* and root processes from Local Network privacy and explicitly does
  **not** exempt agents (platform-integration §1.2). The server is the component that sends and receives multicast, so it is the component that must
  not depend on a permission prompt.
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

- The application is an *optional* front end. The product is complete without it (UX-001, MB-001).
- The first-run assistant uses the system folder picker, so the user grants access by choosing a folder rather than by navigating a permission dialog
  (platform-integration §5.4; PLT-017, F-08).
- Every error follows the "what happened / why / what to do" shape (UX-004), and the technical detail is available on demand behind a disclosure
  control rather than in the primary text (UX-010).
- The status is always available as text (UX-007, UX-018); no state is conveyed by a colour alone (MB-006).
- [DOC-005] *Menu bar*, *status item*, *launchd daemon*, *launchd agent*, *bundle*, *assistant*, *Local Network privacy* and *Rosetta* are explained
  at first use.

## 7. Security and privacy

- The application adds no data flow of its own: it stores its preferences in the user's own preferences directory, talks to the server over loopback,
  and requests no macOS permission beyond the local-network permission and access to the folders the user chose (PRIVACY §9).
- The application is a client of the administration surface and uses its authentication model — bearer token, CSRF token, custom header (MB-005, F-14,
  SEC-072).
- The daemon's local-network privilege comes from being a launchd daemon, not from asking the user for permission at the wrong moment; the *agent* is
  where a prompt may legitimately appear (platform-integration §1.2, §4.2).
- The application never shows an absolute filesystem path; it shows root names (SEC-094, PLT-008).
- The application performs no network access of its own — no update check, no telemetry (SEC-090, SEC-091, INT-059).

## 8. Interoperability

- The application speaks **no** DLNA or UPnP protocol. It is not a control point; it never sends `M-SEARCH` and never browses the catalogue through
  the protocol (F-01, F-03).
- It interoperates with the *operating system*: the menu bar, the accessibility APIs, the folder picker and the notification centre are the interfaces
  it must satisfy.
- [OBSERVED] The daemon and the agent have different permission behaviour on the same Mac, so the two halves of the product are not interchangeable at
  the platform level (platform-integration §1.2). This is a platform fact, not an interoperability claim.
- [OPEN] Whether `SMAppService.daemon(plistName:)` still requires an administrator password, and what its exact `Status` values are, is not
  established (platform-integration §4.3). See §12.

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

All criteria below are *planned*; no test file exists yet.

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

**AC-06** — *Given* the system text size is increased and "Increase contrast" and "Reduce motion" are enabled, *when* the application is used, *then*
the layout adapts to the larger text, the contrast requirement is met and no animation conveys state. Requirements: UX-008, UX-017, MB-003 ·
Verification: manual macOS usability protocol · Planned location: `qa/macos-usability-protocol.md`

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
plain-language string and the bundle is marked as a menu-bar-only application; and *given* a helper is registered through `SMAppService`, *then* the
helper executable lives inside the bundle. Requirements: MB-009 · Verification: automated test · Planned location:
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

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Mode switching, shared schema, warning visibility | `crates/phairdlna-mac/tests/presentation_modes.rs` |
| Progress reporting and cancellation | `crates/phairdlna-mac/tests/scan_progress.rs` |
| No side effects from opening a view | `crates/phairdlna-mac/tests/view_side_effects.rs` |
| Bundle metadata and permissions keys | `crates/phairdlna-mac/tests/bundle_metadata.rs` |
| No network egress from the application | `crates/phairdlna-mac/tests/no_egress.rs` |
| String resources and locale coverage | `crates/phairdlna-mac/tests/strings.rs` |
| Status item state machine | `crates/phairdlna-mac/src/status_item.rs` (inline `#[cfg(test)]` module) |
| Keyboard navigation, labels, focus order | `crates/phairdlna-mac/src/accessibility.rs` (inline `#[cfg(test)]` module) |
| Manual macOS usability protocol | `qa/macos-usability-protocol.md` |
| Device and platform matrix rows | `qa/device-matrix.toml` |

**Method.** The presentation-mode and side-effect rules are pure logic over the administration-surface client and are exercised without a GUI.
Everything that only a real Mac can answer — the menu bar, VoiceOver labels, the text-size setting, the Local Network prompt, the firewall prompt and
the bundle identity — is a **manual** protocol on a real machine, recorded with the device and the macOS version. The firewall behaviour is a **device
test** and is explicitly recorded as `NOT RUN — no device available` until a machine is available (QUALITY-GATES §1 rule 4).

**Dependencies.** F-13 (the CLI whose commands the application mirrors and the service lifecycle), F-14 (the administration surface, its token and its
first-run text), F-08 and F-12 (roots, scans and the configuration the assistant writes), F-16 (the security model the application inherits), F-19
(`SUP-009`, the signing plan), and the platform traits of ADR-0003.

## 12. Open questions and decisions

1. **[OPEN] Ownership of PLT-013 and PLT-014.** The baseline assigns PLT-004 to F-15 but leaves PLT-013 (the daemon/agent split) and PLT-014 (the
   universal binary) with no feature in the requirements index. This specification describes the behaviour they imply but does not claim them. *Closes
   when:* the owner assigns them — PLT-013 and PLT-014 are the natural companions of PLT-004 here.
2. **[OPEN] The firewall is the largest macOS device-test risk.** Apple's documentation does not say what happens to a headless daemon that binds a
   listening socket with the firewall on (platform-integration §2.3), and an ad-hoc signature is not covered by the auto-allow rule. The mitigation
   documented in the research is to ship as a `LaunchDaemon` and tell users to add the binary to the firewall options if discovery fails. *Closes
   when:* a real Mac shows what the daemon does with the firewall enabled, and the result is recorded in `qa/device-matrix.toml`.
3. **[OPEN] Developer ID and notarisation.** Option B (Developer ID + hardened runtime + notarisation) requires a paid Apple Developer Program
   membership (99 USD/year) and a named maintainer — a governance decision for an AGPL volunteer project, not a technical one (platform-integration
   §3.2, §3.3). The plan is `SUP-009` in F-19. *Closes when:* the owner decides whether to buy the membership before the first stable release.
4. **[OPEN] `SMAppService` behaviour.** Whether `SMAppService.daemon(plistName:)` still requires an administrator password, and what its exact
   `Status` values are, is not established (platform-integration §4.3). A pure-CLI install cannot use `SMAppService` at all, because there is no
   bundle. *Closes when:* a real device answers the question; until then the install path writes the plist directly.
5. **[OPEN] The supported locale list.** UX-020 requires the set to be decided before the first release but does not name it. *Proposed decision, not
   in the baseline:* English (United States) and English (United Kingdom) at launch, with the resource format able to carry more. Rationale: the
   project has no translation team, and an unbounded promise is not deliverable. *Closes when:* the owner fixes the list.
6. **[OPEN] Which surface is the "one view" of UX-014.** The menu bar menu is small by construction; the requirement says "one view" but does not say
   whether that view is the menu, a popover or a window. *Proposed reading, used here:* the menu is the status surface and the one view is the popover
   it opens. *Closes when:* the owner confirms.
7. **[OPEN] "Auto-start" and the daemon.** MB-008 requires the auto-start setting to be visible and forbids a hidden helper; PLT-013 puts the server
   in a launchd daemon, which on macOS starts at boot regardless of a per-user auto-start switch. *Proposed reading, used here:* the visible switch
   controls the *menu bar application* only, and the daemon's boot-time start is a property of the installation, stated in the installation text.
   *Closes when:* the owner confirms.