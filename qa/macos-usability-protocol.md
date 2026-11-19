# macOS usability and accessibility protocol

Status: **Draft — app process launch observed after Launch Services reset; visual/accessibility run incomplete** · Owner role: QA / Accessibility

Version/date: 5.5 / 2026-11-19
Purpose: Record repeatable observations of first launch, the native overview, read-only server settings, language selection, keyboard use and VoiceOver.  
References: `../docs/03-features/F-15-macos-native-experience.md`, `../docs/05-quality/QUALITY-GATES.md`

Requirements: `UX-002`, `UX-003`, `UX-004`, `UX-007` … `UX-009`, `UX-017`, `MB-001` … `MB-003`, `MB-006`, `MB-007`, `MB-008`

## 1. Test record

Complete one record for each run. Use a disposable macOS account and disposable media folders so the run cannot change a personal library.

| Field | Record |
| --- | --- |
| Result | `PASS` / `FAIL` / `NOT RUN — reason` |
| App build or commit | |
| macOS version | |
| Mac model and processor | |
| App language | |
| VoiceOver | On / Off |
| Increase Contrast | On / Off |
| Reduce Motion | On / Off |
| App text size | 100% / 150% / 200% |
| Tester and date | |
| Evidence location | |

`NOT RUN` is not a pass. Record the observed text or action and a screenshot or VoiceOver transcript for every failure. Do not include real media paths, bearer tokens or personal names in evidence.

## 2. First launch and overview — AC-19, AC-21

1. First test a genuinely fresh disposable account with no PhairDLNA configuration. Launch `PhairDLNA.app` from Finder or Spotlight. Confirm that the app opens the overview without a blocking alert or folder picker, and shows localized recovery guidance immediately below the server status. The guidance recommends `phairdlna-cli setup` only when the settings file is missing and the separately installed CLI is available. Mark bootstrap `NOT RUN — pending OD-16`; this is not a successful first run.
2. In a separate run, start with a valid PhairDLNA configuration containing no shared folders. Confirm that the overview opens and explains that folder changes are unavailable in the app while its service/configuration path is unresolved.
3. Confirm the overview's folder action is visibly disabled and labelled as unavailable. The app must not offer a native folder picker.
4. Confirm that the status menu contains no Start or Stop service action and directs the user to ask the person who set up PhairDLNA for help. Confirm that neither the menu nor the overview implies that folder setup or server lifecycle is available in the app.
5. Repeat with a configuration containing shared folders. Confirm the app lists the folders returned by the server and never displays their absolute paths in simple mode.
6. Repeat with a malformed configuration. Confirm the overview shows the localized configuration guidance below the status without a modal alert, directs the user to repair the existing file rather than recreate it with CLI setup, does not show fallback settings as real values, and keeps folder changes disabled.
7. Compare the disposable configuration before and after startup and interaction with the disabled folder control. It must remain byte-for-byte unchanged.
8. Record that first-run folder selection, app-managed service control and folder activation are unavailable pending OD-14/OD-15/OD-16; do not record AC-03 or UX-019 as passing.

Expected: the overview uses everyday language and explains the current limitation without implying that a folder was saved or the server was started. First-run bootstrap, folder selection, device address in an assistant and the optional TV discovery check remain `NOT RUN` pending decisions and implementation (AC-03, AC-16).

## 3. Status and reconnection — AC-01, AC-02, AC-04

1. Open the app before the server is reachable. Confirm the overview says that it is checking the status and does not claim “stopped” before its first response.
2. Start the server using the documented development foreground command. Confirm the overview updates without relaunching the app and shows a text status.
3. Stop the foreground server. Confirm the app drops stale folders, counts and activity rather than showing the last successful response as current; it must not replace unavailable values with “0 objects” or “nothing is playing”.
4. Restart the server while leaving the app open. Confirm that the app recovers on its next refresh.
5. Trigger an authentication or configuration error in the disposable account. Record whether the message explains what happened in plain language and what to do next, and whether it exposes an absolute path or unnecessary terminal jargon.
6. In both refused-access and stopped states, confirm the menu offers no Start or Stop service action while OD-14/OD-15 are open. It should direct the user to ask whoever set up PhairDLNA for help. Confirm that unavailable folder details are not described as an empty configuration.

Expected: live state replaces the checking state; loss of the server clears its stale snapshot; recovery appears without restarting the app. The app’s graphical start/stop lifecycle remains incomplete; mark AC-01 `NOT RUN — approved daemon control path incomplete` rather than treating the foreground process as proof of GUI lifecycle support.

## 4. Keyboard, text and VoiceOver — AC-05, AC-06, AC-07

1. Turn on full keyboard access in macOS and reach every control in the overview, menu and Preferences window without a pointer.
2. Record the focus order, the visible focus indicator and whether each action can be activated from the keyboard.
3. Repeat with VoiceOver enabled. Record the spoken name and state of the menu bar item, every button, the current server state and the headings in the overview.
4. Repeat at 100%, 150% and 200% app text size with Increase Contrast and Reduce Motion enabled. Open each Preferences tab at every size.
5. At each size, check that text remains readable, controls do not overlap or disappear, focus remains visible, and no state depends on color or animation alone. When a recovery notice or read-only guidance is shown, confirm its full text wraps and remains visible without clipping at 100%, 150% and 200%. In Preferences, verify the general values, folder paths, expert TOML and About copy remain reachable by scrolling and are not clipped. Check the longest supported translations at 200%.
6. Add temporary test data with a very long folder name without spaces and names made from wide Unicode characters (for example, CJK ideographs and emoji); confirm the overview wraps all text, exposes the full content by vertical scrolling, and has no horizontal clipping.
7. Stop the server while its configuration contains shared folders. Confirm the overview says that folder details are unavailable instead of saying no folders are selected. Restore the server, configure at least five shared folders and make one volume unavailable; confirm the overview lists every configured folder, labels the unavailable one in words and allows the complete list to be reached by scrolling.
8. At the default window size and 100% text, confirm the disabled folder-setup control follows the shared-folder list and is fully visible without scrolling. Its label must explain that setup is unavailable in the app. Repeat with no folders selected and with the longest supported translation.
9. Open Preferences. Confirm the selected text-size radio button is visible, server fields, shared folders and expert TOML are read-only, and no Save button is offered. With VoiceOver, confirm the server-name and HTTP-port values are announced with their field names, and the shared-folder list and expert configuration text area have useful names. At 100%, 150% and 200%, use a long server name and long folder path and confirm the full value can be read without clipping. With a missing, malformed or unreadable configuration in the disposable account, confirm the UI shows an unavailable value and an explanation instead of defaults.
10. Export a diagnostics report to a writable location and confirm the overview reports success. Repeat with a location where writing is denied; confirm the app names the failure and does not claim the report was saved.
11. In a disposable macOS account, set the preferred system language to a shipped language, then launch the app from Finder with conflicting or unset locale environment variables. Confirm the app uses the first supported macOS preference. Repeat with only unsupported preferred languages and confirm the documented environment fallback is used.
12. Resize the overview to its minimum supported height and then larger than its initial size. Confirm the scroll area follows the window, long content remains reachable, the disabled folder-setup control is not clipped, and short content does not leave a large empty band above the first heading.
13. Resize Preferences to its minimum size and larger. Confirm the General, Expert and About tab area follows the window, vertical scrolling remains available, and all values remain reachable at 100%, 150% and 200%.
14. Assess the overview at its default size as a first-time, non-technical user. The server state must be the first thing noticed; folders, TV instructions and activity must read in the documented order. Confirm the window uses native macOS appearance, restrained system colors and whitespace, with no decorative cards, gradients or extra always-visible technical panels. Confirm the disabled setup control and its recovery notice are legible, and the notice makes the supported next step clear without suggesting that the control can be used.
15. Inspect the app icon in Finder and the Dock at normal and small sizes. Confirm the library/server symbol remains recognizable, has no play-button meaning, and the macOS rounded mask does not clip any artwork.

Expected: the window has one clear visual priority and preserves the simple four-area reading order without crowding. Every control has a useful spoken label; the changing overview values are announced with their localized names (server status, library, shared folders, TV connection help, current activity and notice); status is available as text; the layout stays usable at all three app text sizes. Do not infer appearance or accessibility from source code alone: record the observed result.

## Revision history

| Version/date | Change |
| --- | --- |
| 5.5 / 2026-11-19 | Confirms the post-reset PhairDLNA overview is visible in a fresh screenshot and records its displayed running state and library count. |
| 5.4 / 2026-11-19 | Records successful Launch Services process submission after the reported reset/restart; window rendering and accessibility remain unverified. |
| 5.3 / 2026-11-19 | Records that non-deleting `lsregister -apps user -v` also exited successfully without restoring the missing user database path or enabling app launch. |
| 5.2 / 2026-11-19 | Records that the non-deleting `lsregister -seed` attempt did not restore the missing user database path; a destructive database reset is not performed without owner approval. |
| 5.1 / 2026-11-19 | Rebuilds the normal universal bundle and records that strict signature and architecture checks pass while this session's Launch Services and Spotlight still reject app discovery; GUI acceptance remains NOT RUN. |
| 5.0 / 2026-11-19 | Separates missing-file CLI setup from recovery of an existing invalid configuration in the first-run protocol; runtime remains NOT RUN. |
| 4.9 / 2026-11-19 | Replaces unsupported installer instructions and app-menu lifecycle claims with an actionable contact for the person who configured PhairDLNA; rendered copy remains NOT RUN. |
| 4.8 / 2026-11-19 | Repeats the known system-app control after icon packaging; Calculator still returns `kLSNoExecutableErr` (`-10827`), so no PhairDLNA runtime conclusion is drawn. |
| 4.7 / 2026-11-19 | Adds an app-icon check for accurate server meaning and legibility at Finder/Dock sizes. |
| 4.6 / 2026-11-19 | Adds a first-time-user visual hierarchy check for the restrained native overview, including the status priority, reading order, whitespace and disabled setup action; runtime remains NOT RUN. |
| 4.5 / 2026-11-19 | Records the signed universal package containing the shared guidance-font constant and reflow-boundary tests; GUI acceptance remains NOT RUN. |
| 4.4 / 2026-11-19 | Records the signed universal package for the font-aligned recovery layout; real window rendering remains NOT RUN. |
| 4.3 / 2026-11-19 | Adds an explicit render check that the recovery notice and read-only guidance wrap without clipping at each supported text size. |
| 4.2 / 2026-11-19 | Verifies first-run and malformed-settings recovery as an inline notice beneath server status, with no blocking alert. |
| 4.1 / 2026-11-19 | Replaces obsolete folder-picker/save checks with current negative acceptance for disabled folder changes and unapproved service controls. |
| 4.0 / 2026-11-19 | Consolidates duplicate notes for the same 09:49 incident into one detailed entry. |
| 3.9 / 2026-11-19 | Records the verified arm64 bundle containing the conditional missing-settings recovery copy; launch remains `NOT RUN`. |
| 3.8 / 2026-11-19 | Adds a manual check that the missing-configuration warning describes the unavailable native bootstrap and makes the separate CLI recommendation conditional. |
| 3.7 / 2026-11-19 | Records a signed arm64 package containing the plural folder labels and tooltips; runtime remains `NOT RUN`. |
| 3.6 / 2026-11-19 | Checks that menu label and tooltip both accurately promise multiple-folder selection. |
| 3.5 / 2026-11-19 | Aligns the Add media folders label with the picker’s multiple-selection behavior and adds an acceptance check for selecting two folders. |
| 3.4 / 2026-11-19 | Confirms Calculator's declared executable exists as Mach-O despite Launch Services reporting it missing; records system signature verification failure. |
| 3.3 / 2026-11-19 | Rechecks Launch Services through Calculator; its `kLSNoExecutableErr` persists, so no PhairDLNA launch was attempted. |
| 3.2 / 2026-11-19 | Records the arm64 package containing the localized unexpected-response copy; Launch Services runtime remains `NOT RUN`. |
| 3.1 / 2026-11-19 | Records the updated unexpected-response copy under AC-04; real error presentation remains part of the manual run. |
| 3.0 / 2026-11-19 | Records the current action-module refactor package and signature check; Launch Services runtime remains `NOT RUN`. |
| 2.9 / 2026-11-19 | Records source-order inspection around AppKit registration; no app-authored pre-registration Launch Services call was found. |
| 2.8 / 2026-11-19 | Records a signed arm64 package that includes all three Preferences accessibility names; launch and VoiceOver remain unverified. |
| 2.7 / 2026-11-19 | Adds a VoiceOver name check for the expert-configuration text area. |
| 2.6 / 2026-11-19 | Records a signed arm64 package containing the Preferences accessibility-name updates; runtime remains unverified. |
| 2.5 / 2026-11-19 | Adds a VoiceOver check for accessible names on read-only server values and the shared-folder list in Preferences. |
| 2.4 / 2026-11-19 | Records a signed arm64 package built from the current Preferences-resize revision; launch remains unverified. |
| 2.3 / 2026-11-19 | Adds Preferences minimum-size and resize-following acceptance for every tab and text size. |
| 2.2 / 2026-11-19 | Records dynamic vertical space for long and multiline server-name values in Preferences. |
| 2.1 / 2026-11-19 | Covers long server names at all Preferences text sizes and records the separated footer/scroll geometry. |
| 2.0 / 2026-11-19 | Extends 100%, 150% and 200% checks to every Preferences tab and its scroll behavior. |
| 1.9 / 2026-11-19 | Adds a check that folder actions are disabled without a readable configuration and return after recovery. |
| 1.8 / 2026-11-19 | Records the read-only Launch Services database inspection and a repeated Calculator control failure. |
| 1.7 / 2026-11-19 | Records the second direct-launch abort and repeated Launch Services database failure. |
| 1.6 / 2026-11-19 | Records successful universal packaging of the startup-order build while keeping runtime launch NOT RUN. |
| 1.5 / 2026-11-19 | Confirms `kLSNoExecutableErr` against Calculator as well and records that current Launch Services failures are session-wide. |
| 1.4 / 2026-11-19 | Records the valid packaged bundle and Launch Services database failures observed in the Codex session; keeps runtime and rendered-window acceptance NOT RUN. |
| 1.3 / 2026-11-19 | Adds acceptance for overview resizing and vertical balance. |
| 1.2 / 2026-11-19 | Tracks Finder-launched macOS language selection as a separate manual acceptance item. |
| 1.1 / 2026-11-19 | Adds acceptance for macOS preferred-language selection when launched from Finder. |
| 1.0 / 2026-11-19 | Checks that missing TV identity details produce a clear explanation and next step. |
| 0.9 / 2026-11-19 | Adds a refused-access check for misleading folder status and state-inappropriate start action. |
| 0.8 / 2026-11-19 | Separates a genuinely missing-config bootstrap run from the existing-config/no-folders prompt and marks bootstrap blocked by OD-16. |
| 0.2 / 2026-11-18 | Adds acceptance for the visible read-only state and configuration load failures in Preferences. |
| 0.3 / 2026-11-18 | Adds a first-run negative case for configuration write failure. |
| 0.4 / 2026-11-18 | Adds visible recovery guidance for missing or malformed server configuration. |
| 0.5 / 2026-11-18 | Checks confirmation after diagnostics export and visible recovery on a write failure. |

## 5. TV connection guidance and lifecycle — AC-15, AC-16

1. With the server running and a TV on the same local network, check whether the TV can find the configured server name. When the server name is missing from status, confirm the app says the TV name was not provided and gives a useful setup step.
2. Repeat with discovery disabled and with the TV on a different network segment. Record what the overview says and whether it avoids claiming that a TV is present.
3. Confirm that no app Start/Stop action is offered while OD-14/OD-15 remain open. Do not treat command-line service control as app lifecycle acceptance.

Expected: the app distinguishes discovery being active, unavailable and unknown. This protocol does not count a server status response as proof that a TV discovered the server. AC-15/16 and GUI lifecycle remain `NOT RUN` until their flows exist and have been exercised on a real Mac and TV.

## 6. Current execution status

| Area | Status |
| --- | --- |
| Protocol document | Written; the complete protocol has not been executed |
| macOS app runtime | **PARTIAL** — current revision launched and its overview was visible at the default size in the 2026-11-19 post-reset screenshot; other states and sizes remain unverified |
| macOS language selection | **PARTIAL** — direct arm64 executable used German macOS preference with English locale variables; Finder GUI and unsupported-language fallback remain `NOT RUN` |
| Window resizing and vertical balance | `NOT RUN — corrected layout needs a fresh macOS observation` |
| VoiceOver and keyboard | `NOT RUN — no real-Mac observation recorded` |
| Television discovery | `NOT RUN — no device observation recorded` |
| GUI daemon install/start/stop | `NOT RUN — approved control path incomplete` |

**Ad-hoc preview observation (2026-11-18):** macOS 27.2.1 on Mac17,3, arm64;
ad-hoc-signed preview bundle with an isolated `/tmp` configuration and five sample
roots, server stopped. The main window rendered, and the add-folder action was fully
visible after the folder list. The initial render exposed false zero/empty library
information and terminal-specific missing-server copy; those are corrected in source.
The [captured pre-correction preview](evidence/macos-27.2.1-overview-offline-before-fix.png)
records that observation; it is evidence of the issue, not a pass for the corrected UI.
This observation is not a full AC-20 pass, does not cover the corrected render, and
does not count as a GUI lifecycle, accessibility or device test.

**Preview launch failure (2026-11-18):** macOS terminated the preview with
`EXC_CRASH (SIGKILL (Code Signature Invalid))` / `Taskgated Invalid Signature`.
The preview bundle had been modified after its ad-hoc signature was applied. Re-signing
the completed temporary bundle and verifying it with `codesign --verify --deep --strict`
allowed the preview to launch. The repository packager now signs only after assembling
the bundle and immediately verifies that final signature. On 2026-11-19,
`BUILD_UNIVERSAL=0` packaging completed in a fresh temporary output directory; its arm64
bundle passed the packager's final signature verification. The default universal
packaging run also completed in a fresh temporary output directory: `lipo` reported both
x86_64 and arm64 slices, and the completed bundle passed final signature verification.
This verifies packaging only, not app launch, Gatekeeper acceptance or notarisation.

**Isolated preview attempt (2026-11-19):** a fresh arm64 bundle used a temporary
configuration, a temporary home directory and a missing temporary token file. Its
configuration validated, its signature verified, and its `PkgInfo` contained exactly
eight bytes. Launching the executable directly from the shell aborted in
`NSApplication.sharedApplication` / `HIServices::RegisterApplication`; the crash report
shows `SIGABRT`, not a signature rejection. This alone does not identify the cause.
Launching the same bundle with `/usr/bin/open` failed with `kLSNoExecutableErr` although
`Info.plist` named the existing executable and `codesign` verified the bundle. Launch
Services registration also could not scan the bundle in `/private/tmp` and the ignored
workspace `target/` path (`-10822`, Spotlight); `open` returned the same
`kLSNoExecutableErr` for both locations. No corrected window render was observed;
the visual acceptance remains `NOT RUN`. The attempt used only files under
`/private/tmp/phairdlna-isolated-preview-20261119` and did not use the user's normal
configuration or token.

**Preview verification attempt (2026-11-19):** `codesign --verify --deep --strict`
passed for `/tmp/PhairDLNA-preview.app`, and `open -n` returned successfully with the
isolated preview configuration. A fresh screenshot could not be captured:
`screencapture -i -W -x` exited with status 134 and reported
`dispatch_source_create returned NULL, invalid parameters passed to dispatch_source_create`.
The process list was also unavailable because `sysmond` was not present. Therefore the
corrected window appearance and continued runtime remain `NOT RUN`; the existing
pre-correction screenshot is the only visual evidence.

**Launch Services follow-up (2026-11-19):** A newly packaged arm64 bundle has a valid
`CFBundleExecutable`, executable Mach-O, eight-byte `PkgInfo` and strict ad-hoc signature.
`open -n` nevertheless returned `kLSNoExecutableErr` (`-10827`) both from `/private/tmp`
and from a temporary copy in the user's `Applications` folder; that copy was removed
after the attempt. `lsregister -f` returned `-10822`, `lsappinfo` reported a null
session/application count and an empty process list, and `pgrep` could not query the
process list because `sysmond` was absent. Together with the failed Calculator control,
these observations establish that this Codex session's Launch Services path is not
working; they do not establish whether the bundle will launch in a normal Mac user session
or prove that the application code is crash-free. The 09:49 incident is recorded once
below. Runtime, rendered-window and accessibility acceptance remain `NOT RUN`.

As a control, `open -n -g /System/Applications/Calculator.app` failed with the same
`kLSNoExecutableErr`, and `lsappinfo processList` returned no applications. This confirms
the failure is not specific to PhairDLNA's bundle. Do not change bundle metadata based on
this session's launch error; retry the protocol on a Mac session where Launch Services can
open a known system application.

**Read-only Launch Services inspection (2026-11-19):** The system `lsregister -dump`
command completed its integrity check, but its user database reported `Path: NOT FOUND`,
`CacheSequenceNum: NOT FOUND` and `path = '?'`. A new
`open -n -g /System/Applications/Calculator.app` attempt still failed with
`kLSNoExecutableErr` (`-10827`). This is consistent with the earlier session-level
Launch Services failure. No database reset or registration mutation was performed;
PhairDLNA runtime acceptance remains `NOT RUN`.

**Launch Services recheck (2026-11-19, 11:41 local):** `/usr/bin/open -n -g
/System/Applications/Calculator.app` returned `kLSNoExecutableErr` (`-10827`,
"The executable is missing"). This is a known system application and the attempt
did not involve PhairDLNA. The failure confirms that Launch Services still cannot
open a control app in this session; the current PhairDLNA bundle was not launched.
Do not interpret this as new evidence of a PhairDLNA packaging defect.

**Latest control recheck (2026-11-19, 15:15 CET):** `/usr/bin/open -n -g
/System/Applications/Calculator.app` again returned `kLSNoExecutableErr` (`-10827`).
Because the known system control still cannot open, this session cannot establish
whether the current, newly packaged PhairDLNA app launches or how its window and Dock
icon render. The PhairDLNA bundle was not launched in this recheck.

The control app itself is present: its `Info.plist` declares `CFBundlePackageType=APPL`
and `CFBundleExecutable=Calculator`, and
`Contents/MacOS/Calculator` exists as an executable universal arm64e Mach-O. A
read-only `codesign --verify --deep --strict` check nevertheless returned
`CSSMERR_TP_NOT_TRUSTED` for architecture `arm64e.x1`. This additional control-system
failure means this session cannot distinguish a Launch Services database problem from
a broader local code-trust problem; no system repair or registration change was made.

**Repeated launch report (2026-11-19, 09:49, incident
`A96EFF41-05E6-4E48-8F34-471DDCBCAA54`):** The user resubmitted the same incident report;
it again shows the app aborting in
`NSApplication.sharedApplication` / `HIServices::RegisterApplication`. Its utility
thread entered `_LSCopyLocalDatabase`; the reported `OSStatus` value `-10813` is
Launch Services' `kLSDataUnavailableErr` (requested resource data is unavailable).
This is consistent with the session-level Launch Services evidence above, but does not
identify why that data was unavailable or demonstrate an app-specific defect. Apple’s
[Launch Services result-code reference](https://developer.apple.com/documentation/coreservices/klsdataunavailableerr)
defines the status. Runtime and rendered-window acceptance remain `NOT RUN`.

**Current bundle check (2026-11-19):** `scripts/package-macos-app.sh` rebuilt
`target/bundle/PhairDLNA.app` as a universal arm64/x86_64 bundle using the current
`Info.plist` and icon resource. `codesign --verify --deep --strict` passed; the bundle's
CLI `--help` also exited successfully. `open -n -g` still returned `kLSNoExecutableErr`
(`-10827`) from the repository path, and `lsregister -f` returned `-10822` while scanning
that bundle (`from spotlight`). This session also cannot open the known system Calculator
app. No Launch Services database reset was attempted. The stale local build artifact has
been replaced, but the app's GUI launch and `NSApplication` runtime remain `NOT RUN` until
Launch Services can discover and launch an app in a functioning macOS user session.

**Non-deleting registration repair attempt (2026-11-19):** `lsregister -seed` exited zero,
but a subsequent read-only `lsregister -dump` still reported the user database path and
cache sequence as `NOT FOUND`; `open` continued to return `-10827`. The tool documents
`lsregister -delete` as deleting the database and requiring a reboot. That system-wide
reset was not run because it affects Launch Services outside the project and needs owner
approval. GUI acceptance remains `NOT RUN`.

**User-app registration recheck (2026-11-19):** `lsregister -apps user -v` exited zero
but the following dump still reported `Path: NOT FOUND` and `CacheSequenceNum: NOT FOUND`.
The current PhairDLNA bundle continued to fail at `open` with `-10827`; its code signature
remained valid. A full database reset has not been run.

**Post-reset launch check (2026-11-19, 15:57–16:03 CET):** following the user's report
that Launch Services had been reset and macOS restarted, `open -n -a
/Users/volker/Documents/Repos/PhairDLNA/target/bundle/PhairDLNA.app` exited zero.
Unified logs show Launch Services submitting the `phairdlna-mac` process (PID 2410)
successfully; AppKit initialized far enough to emit only App Intents registration
connection errors in the filtered error log. No new `NSApplication.sharedApplication`
abort or `kLSNoExecutableErr` was observed in this launch. `osascript` activation also
exited zero. A fresh screenshot at `/private/tmp/phairdlna-current-screen.png` shows the
PhairDLNA overview open beside Terminal, with “Server läuft”, 39 media items and the
configured shared folders visible. This confirms launch and visible overview for this run.
Process-list tools still fail because `sysmond` is unavailable. The screenshot is one
default-state observation; the remaining first-run, reconnect, layout, keyboard, VoiceOver,
localization and accessibility protocol steps remain `NOT RUN`.

**Source startup-order review (2026-11-19):** the app reads its preferred language before
AppKit initialization, then calls `NSApplication.sharedApplication` on the main thread
before it spawns the administration worker. Source search found no app-authored
`NSWorkspace` or other explicit Launch Services call before application registration.
This review found no concrete startup-order defect that explains the report; it cannot
rule out an AppKit/Foundation interaction or prove the operating-system cause. Runtime
acceptance remains `NOT RUN`.

After the startup-order change, the current source also packaged successfully as a
universal arm64/x86_64 bundle. Both architecture checks and strict signature verification
passed; the bundle has an eight-byte `PkgInfo`. This still does not verify runtime launch.

**Preferences-resize revision package (2026-11-19):** the current worktree packaged
successfully as an arm64 app at
`/private/tmp/phairdlna-current-package.vRYw6q/PhairDLNA.app`. The release build completed,
the packager's strict ad-hoc signature verification passed, the binary is arm64, and
`PkgInfo` is eight bytes. No launch or rendered-window result was obtained from this
packaging step; GUI acceptance remains `NOT RUN`.

**Recovery-layout revision package (2026-11-19):** after aligning the recovery and
read-only guidance fonts with the reflow calculation,
`OUTPUT_DIR=/private/tmp/phairdlna-inline-layout-preview BUILD_UNIVERSAL=1
./scripts/package-macos-app.sh` completed successfully. The bundle contains arm64 and
x86_64, has an eight-byte `PkgInfo`, and passes strict ad-hoc signature verification.
Launch Services, rendered wrapping and clipping remain `NOT RUN`.

**Reflow-boundary revision package (2026-11-19):** the current source packaged at
`/private/tmp/phairdlna-overview-reflow-preview/PhairDLNA.app` as a universal app.
The arm64 and x86_64 checks, eight-byte `PkgInfo` and strict ad-hoc signature verification
passed. Runtime, rendered clipping, keyboard and VoiceOver checks remain `NOT RUN`.

