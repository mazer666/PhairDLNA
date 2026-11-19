# Project status — PhairDLNA

Status: **Baseline approved (2026-11-15) · M1.11 implemented · M2.1 implemented · M2.2 implemented · M2.3 implemented · M2.4 implemented (partly) · M2.5 implemented (partly) · M2.6 implemented (partly) · M2.7 implemented (partly) · M2.9 implemented** · Compiled: 2026-11-19
Owner role: Lead / Project Manager

> **Current verification (2026-11-19).** The latest complete local CI run completed all 11
> quality gates successfully after the F-13 direction update, visual-review protocol,
> German overview-copy refinements, F-14 token-module split, F-15 icon integration and
> daemon connection-limit module split, and the latest missing-versus-invalid configuration
> recovery wording and obsolete locale-key cleanup below. The latest run passed all 11
> gates; 31 source files remain above the 300-line target, and `miniz_oxide` remains
> duplicated by the PNG dependency paths.
> This covers workspace tests, Clippy, offline-build behavior, source-size policy, dependency checks and generated
> requirement views. The setup
> command was split into focused prompt, config-file and orchestration
> modules; its user-facing copy now describes the native app's authenticated local
> connection without referring to the removed browser page. The CLI setup tests (5/5),
> formatting, file-size, documentation-link and requirements-generator checks also pass.
> This is source/build evidence plus one observed default-size native overview screenshot.
> Window reflow, VoiceOver and device interoperability remain NOT RUN. This checkout contains a broad, uncommitted
> change set; OD-15 evidence and independent review remain gates before
> presenting the app setup/lifecycle milestone as complete.

> **macOS start failure follow-up (2026-11-19).** I rebuilt the normal universal app at
> `target/bundle/PhairDLNA.app`; it now contains the current icon reference and executable,
> passes strict deep signature verification, and includes both arm64 and x86_64 slices.
> Its CLI help runs. Launch Services still refuses to open it with `kLSNoExecutableErr`
> (`-10827`), and `lsregister -f` fails scanning the bundle with `-10822` from Spotlight.
> The same session has repeatedly failed to open Calculator, so native GUI launch is still
> blocked at the session's macOS Launch Services/Spotlight layer. The earlier preview's
> invalid signature and the later `NSApplication.sharedApplication` abort are distinct
> reports; this rebuild fixes the stale local bundle, but GUI startup is not verified.
> Non-deleting `lsregister -seed` and `lsregister -apps user -v` repair attempts exited
> successfully but did not restore the user database (`Path: NOT FOUND`) or app launch.
> A full Launch Services database deletion requires a reboot and is outside this repository
> change; it needs owner approval.

> **Post-reset app launch check (2026-11-19, 15:57–16:03 CET).** Following the user's
> report of a Launch Services reset and macOS restart, `open -n -a target/bundle/PhairDLNA.app`
> exited successfully. Unified logs show Launch Services submitting `phairdlna-mac` PID 2410;
> the new attempt produced App Intents registration connection errors but no observed
> `NSApplication.sharedApplication` abort or `kLSNoExecutableErr`. `osascript` activation
> also exited zero. A fresh screenshot shows the PhairDLNA overview open beside Terminal,
> displaying “Server läuft”, 39 media items and the configured shared folders. This confirms
> the visible overview for this run. Layout at other sizes, accessibility and the remaining
> usability protocol steps remain `NOT RUN`; evidence and limitations are in
> `qa/macos-usability-protocol.md` § Post-reset launch check.

> **Recovery copy correction (2026-11-19).** Five native-app messages across all five locales
> no longer refer to a nonexistent installer or imply that the app menu can start the server.
> The empty-library message tells users to ask whoever configured PhairDLNA for help. The
> localized resource and folder-action suites pass (23 tests); rendered copy and menu
> behavior remain `NOT RUN` pending a usable Launch Services session. The complete local CI
> run passed all 11 gates after this correction; source-size warnings briefly rose to 32.

> **Locale-test size follow-up (2026-11-19).** The new recovery-copy contract now lives in
> `tests/strings/recovery_copy.rs`; the general locale test file is under the 300-line target
> again. Targeted locale/folder tests, formatting and file-size checks pass.

> **Missing versus invalid configuration guidance (2026-11-19).** The five locales now
> recommend `phairdlna-cli setup` only when the configuration file is missing; an existing
> invalid file receives a repair instruction instead. The CLI source confirms setup refuses
> to replace an existing file unless `--force` is given. Automated locale tests check both
> conditions and their order. The current targeted runs pass; native rendering remains
> `NOT RUN`.

> **Obsolete locale-key cleanup (2026-11-19).** A source-to-resource audit found 37
> unreferenced keys per locale, including labels for removed folder pickers,
> configuration editing and service actions. Those keys were removed from all five locale
> files and the exact locale contract; the localized suite passes 19/19 and file-size,
> formatting, documentation-link and generated-requirement checks pass. The full 11-gate
> local CI also passes on this cleaned state; GUI rendering remains `NOT RUN`.

> **App-led direction consistency (2026-11-19).** F-13 now describes the native app as the
> primary graphical macOS experience and the CLI as the complete headless path. This aligns
> the feature specification with the approved product vision and J1 without weakening
> PRD-006's requirement that server capabilities remain usable without a GUI. Documentation
> links and generated requirement views pass after the edit.

> The macOS manual protocol now has an explicit first-time-user visual review: status
> hierarchy, reading order, whitespace, restrained native styling and clear recovery
> guidance. This makes the requested visual standard observable; current rendering remains
> `NOT RUN` because the session's Launch Services control app still cannot open.

> German overview copy has also been refined for natural everyday wording and clearer
> recovery instructions (stopped state, media count, folder setup, read-only behavior and
> TV connection). The locale-resource suite passed 17/17 and overview-content tests passed
> 6/6; these verify resource structure and content mapping, not rendered German typography.
> The German screen has not been rendered on the current build.

> **Token module size follow-up (2026-11-19).** `phairdlna-admin/src/token.rs` is now 277
> lines; its typed errors and nine focused token tests live in separate modules. The public
> `token::TokenError` path is preserved. Token tests (9/9), admin Clippy and the full 11-gate
> local CI run pass. The repository now has 32 source files above the 300-line target (down
> from 33); these are warnings, not failed gates.

> **App icon refinement (2026-11-19).** The Dock icon now uses a high-resolution local
> library/server mark rather than a play-button symbol. The versioned source artwork and
> ICNS resource are included in the app bundle; automated bundle metadata coverage passes
> 5/5. The release-mode universal package contains arm64/x86_64 slices and passes strict
> ad-hoc signature verification. The icon is readable in the extracted 32 px and 128 px
> resources; Finder/Dock masking and visual acceptance remain `NOT RUN` with Launch Services.

> **Daemon limit module size follow-up (2026-11-19).** `serve/limit.rs` is now 290 lines
> and its two connection-budget tests are in a focused module. The existing behavior and
> public listener types are unchanged. The targeted tests, CLI Clippy and full 11-gate local
> CI run pass; source-size warnings are now 31, down from 32.

---

> **Historical feature note (2026-11-17, commit `81a76d1e`, before the app-first decision).**
> The narrative below chiefly records the M2.9 gate run and earlier M2.6 work. More
> recently, code was committed for interactive **CLI setup** (`1bec4cb7`),
> CLI diagnostics and exports (`1bec4cb7`), **Simple/Advanced configuration
> explanations** (`ab4d3645`), an embedded local **settings-inspection web UI**
> (`be02dff8`) and a **diagnostics-export action in the macOS menu** (`81a76d1e`).
> The settings-inspection web UI was temporary and was removed the following day by
> DEC-075; the current app-first state is recorded below.
> Therefore any unqualified statement below that “first-run onboarding”,
> “plain-language diagnostics”, or “UI diagnostics export” does not exist
> refers to an **earlier implementation stage** and is superseded for the
> named features. **Full native GUI onboarding, physical-device usability and
> accessibility acceptance remain open.** The earlier 1,038-test result and three
> successful manual CI runs refer to earlier revisions. A current local gate run is
> recorded below; it does not replace real-Mac, accessibility or device acceptance.
> No real-device tests, performance benchmarks or seven-day soak tests have
> been recorded. The project remains not release-ready.

> **Service-model specification gap (2026-11-18).** Review found that PLT-013 still
> describes the user interface as a per-user LaunchAgent, while DEC-076 defines a
> regular Dock application. F-13 AC-11 and F-22 AC-10 are now explicitly blocked;
> OD-15 records the conflict. No macOS service packaging or launch relationship is
> approved until the owner resolves it and the real-Mac protocol is completed.

> **Observed release state (2026-11-18).** GitHub has one owner-published prerelease,
> `v0.1.0-alpha.1`, tagged at `66e94c9`. Current `main` is `9481e49`, a later commit
> that makes the native app primary and removes the browser UI. The alpha's notes refer
> to Web Admin; its installer is not the current app-first build. No release-quality
> installer for current HEAD is recorded, and the project remains not release-ready.

> **App-first UI decisions (2026-11-18, DEC-075 and DEC-076).** The embedded browser interface was removed;
> the native macOS application is the primary graphical experience, and the loopback
> administration service is its authenticated local API. The app is a regular Dock
> application with a main window and a menu-bar status item. An earlier implementation
> included a folder picker and first-run folder prompt; these controls were removed before
> the activation direction was selected. DEC-079 now selects local initial-config creation
> and authenticated API updates with restart; those paths remain unimplemented pending
> OD-15 service evidence. The app retains its status menu, read-only preferences and diagnostics
> export. An app-owned 100% / 150% / 200% text-size setting is saved outside the server
> config and is available in preferences and the menu; the overview and all three
> Preferences tabs scale their text and use scrollable vertical layouts. The Preferences
> viewport follows window resizing, and long and multiline server names expand the General
> tab document conservatively. Visual no-clipping behavior, keyboard and screen-reader
> behavior remain unverified on a real Mac. The native overview opens automatically;
> full setup, lifecycle onboarding and optional discovery remain open.

> **Preferences accessibility and package (2026-11-19).** Read-only server-name and
> HTTP-port values now have explicit accessibility labels, and the shared-folder text
> view has a name in the current macOS language. A fresh arm64 app package at
> `/private/tmp/phairdlna-current-package.XCKj0J/PhairDLNA.app` passed strict ad-hoc
> signature verification and bundle-structure checks. This is build/package evidence;
> Launch Services startup, actual VoiceOver announcements and keyboard acceptance remain
> NOT RUN. See the updated manual protocol before calling accessibility complete.

> **App model file-size follow-up (2026-11-19).** Menu-action execution now lives in
> `crates/phairdlna-mac/src/app/actions.rs`; `app.rs` is 244 lines and the new module is
> 76 lines, both within the 300-line target. This is a structure-only refactor; no
> behavior or milestone acceptance is claimed from it.

> The refactored source packaged as an arm64 app at
> `/private/tmp/phairdlna-current-package.YmsZ9s/PhairDLNA.app`. Strict ad-hoc signature
> verification and bundle checks passed. App launch remains `NOT RUN` because the
> documented Launch Services failure persists; packaging does not verify runtime.

> **F-15 accessibility traceability (2026-11-19).** The feature test strategy now maps
> explicit accessibility labels to their actual status-item and Preferences source
> files instead of the nonexistent `platform/accessibility.rs`. Source labels are
> implemented; focus order and spoken VoiceOver output still require manual acceptance.

> **Unexpected-response copy (2026-11-19).** The localized message now names PhairDLNA,
> says it received an unexpected server response, and tells the user to retry or check
> server setup if it continues. This improves the next-step guidance required by UX-004;
> real presentation still needs the AC-04 macOS usability run.

> **Launch Services recheck (2026-11-19, 11:41 local).** Opening the known system app
> Calculator through `/usr/bin/open` still returns `kLSNoExecutableErr` (`-10827`). I
> did not launch the current PhairDLNA bundle because the system control app also fails;
> its declared executable exists as a universal arm64e Mach-O. Strict system-bundle
> verification also returned `CSSMERR_TP_NOT_TRUSTED` on slice `arm64e.x1`. This points
> to a broader local Launch Services/code-trust problem; no system repair was attempted.

> A fresh arm64 package at
> `/private/tmp/phairdlna-current-package.45MaXX/PhairDLNA.app` contains these strings
> and passes strict ad-hoc signature and bundle-structure checks. This is packaging
> evidence only; runtime remains `NOT RUN`.

> **Multi-folder selection copy and package (2026-11-19).** The status-menu action and
> tooltip now accurately describe selecting several folders in all five shipped
> locales. The QA protocol checks selecting two folders, distinguishes saved settings
> from active server state, and confirms activation after restart. A fresh package at
> `/private/tmp/phairdlna-current-package.PHpiH0/PhairDLNA.app` passes signature and
> bundle checks; GUI acceptance remains `NOT RUN`.

> **First-run recovery copy (2026-11-19).** The localized missing-settings warning now
> states that native bootstrap is not available. It only recommends `phairdlna-cli setup`
> when the separately installed CLI is available; the app bundle does not contain that
> executable. The warning, CLI help and generated configuration header now use the real
> executable name. Existing users are told to restore a readable, valid settings file.
> The manual protocol checks this wording; bootstrap remains blocked by OD-16 and rendered
> copy is `NOT RUN`. A fresh arm64 preview at
> `/private/tmp/phairdlna-first-run-copy-preview/PhairDLNA.app` passes strict ad-hoc
> signature and bundle checks; its runtime remains unverified.

> **Onboarding decision extraction (2026-11-19).** The native first-run flow now maps
> configuration load results to three pure decisions: unavailable, valid with roots, or
> valid without roots. Four focused tests cover missing, malformed, empty and populated
> configuration; the targeted tests, formatting, all-target crate Clippy, documentation
> links and generated requirement views pass. Independent review confirmed the desktop
> still proceeds to request the overview after its modal path. This does not add bootstrap
> or approve the existing direct config-write path, which remains gated by OD-16; actual
> modal behavior and overview visibility remain `NOT RUN` on a real Mac.

> **Nonmodal first-run recovery (2026-11-19).** Missing or malformed settings now produce an inline localized notice immediately below the server status instead of a blocking alert on every launch. The overview remains available while bootstrap is unresolved, and the notice makes the existing recovery path visible without interrupting the user. Four onboarding tests now cover both recovery notice keys and the no-notice configured state. Source checks do not prove AppKit placement; real-Mac notice visibility and VoiceOver remain `NOT RUN` because this session's Launch Services path still fails.

> **Recovery-copy layout alignment (2026-11-19).** The overview now explicitly assigns the recovery notice and read-only guidance the 12-point scaled font and unlimited character wrapping assumed by their height calculation. The macOS crate tests and all-target Clippy pass after the change. This reduces a source-level mismatch; it does not prove rendered line heights or absence of clipping, which remain `NOT RUN` until a real window can be inspected.

> The resulting release-mode preview also packages as a universal arm64/x86_64 app with a valid strict ad-hoc signature and eight-byte `PkgInfo` (`/private/tmp/phairdlna-inline-layout-preview/PhairDLNA.app`). This is packaging evidence only; the known Launch Services failure still prevents runtime inspection.

> The same revision packaged at `/private/tmp/phairdlna-inline-recovery-preview/PhairDLNA.app` as a universal arm64/x86_64 bundle. Both architecture checks and strict ad-hoc signature verification passed. This does not establish successful launch or rendered placement; GUI acceptance remains `NOT RUN`.

> **AppKit startup source review (2026-11-19).** The current launch path creates
> `NSApplication` on the main thread before starting the API worker. Source search
> found no explicit app-authored Launch Services call before registration. This does
> not explain the app launch failure; see the incident-specific entry below. Runtime
> acceptance remains `NOT RUN`.

> **Overview accessibility labels (2026-11-19).** The main window's changing status,
> library, shared-folder, TV-help, activity and notice values now have explicit localized
> accessibility labels in all five shipped locales. `cargo test -p phairdlna-mac` passes,
> and a fresh universal bundle passes both architecture checks and strict ad-hoc signature
> verification. These checks do not establish what VoiceOver announces; actual GUI and
> spoken-label acceptance remain `NOT RUN` because this session's Launch Services path
> cannot open even the system Calculator app.

> **XML truncation hardening (2026-11-19).** Independent review of OD-18 found that
> the project-owned SOAP XML reader could repeatedly read an empty string at EOF after
> an unclosed element, without advancing its cursor. `xml_syntax::read_text` now returns
> a malformed-request error at EOF. A subprocess regression with a 2-second watchdog now
> covers EOF after an opening tag and inside text, then parses a valid document to confirm
> recovery. The parent requires a per-run completion marker after those assertions, so a
> child that runs zero tests cannot pass silently. All 18 XML reader tests and strict Core
> Clippy pass; an independent security review confirmed the watchdog and marker behavior.
> This does not establish the parser's broader security properties. OD-18 also requires
> comparing handler timeouts with existing per-read and per-write socket deadlines before
> adopting `tower-http`.

> **OD-18 implementation progress (2026-11-19).** DEC-078 records the owner direction to
> preserve ADR-0002 / DEC-019. `quick-xml` 0.42.0 now backs the bounded XML adapter; the
> 22-case XML reader suite passes, including eleven watchdog truncation cases, valid-document
> recovery, entity/DTD rejection, legal-reference checks and a depth boundary. `tower-http`
> 0.7.1 bounds handler futures. The admin listener now also wraps accepted loopback streams
> with independent read/write silence deadlines and a shared connection semaphore; its unit
> deadline tests and a loopback incomplete-header test pass. The 22 library tests of the
> admin crate and all CLI package tests pass with the socket-enabled run; strict Clippy and
> the no-admin CLI build pass. `cargo deny check licenses` passes. M-02 remains open: the
> dependency review and independent security review are not complete, the accepted
> `tower-http` tracing role has no redaction-safe implementation yet, and broader hostile XML
> and deadline-boundary coverage remains open. This is not a milestone exit.

> **Preferences accessibility follow-up (2026-11-19).** The read-only expert TOML
> text area now also exposes the localized Expert tab name as its accessibility label.
> A fresh arm64 package at `/private/tmp/phairdlna-current-package.8FuHCx/PhairDLNA.app`
> passed strict ad-hoc signature verification and bundle checks. Source checks pass;
> the independent code review found no concrete issue in the prior label additions and
> is reviewing this follow-up. Actual VoiceOver output and current-app launch remain
> `NOT RUN` due to the unresolved Launch Services/AppKit failure recorded above.

> **Overview usability update (2026-11-19).** When the server responds,
> the overview now lists all shared folders instead of hiding entries after the
> fourth; an unavailable volume is identified in localized text. If the server is
> stopped or refuses access, the app says it cannot retrieve folder details instead
> of implying that no folders are configured. The menu now omits unavailable folder
> rows and only offers “Start Server” when the server state is stopped; refusal copy
> is localized in plain language. Automated menu coverage now verifies that a refusing
> server offers neither Start nor Stop, keeps rescan disabled and leaves recovery actions
> available. Long-list scrolling, wrapping and VoiceOver behavior still require the manual
> macOS usability protocol. One local
> visual check on macOS 27.2.1 (Mac17,3, arm64) used an ad-hoc-signed preview bundle
> and isolated `/tmp` configuration at 100% text size. It confirmed the add-folder
> action is fully visible after the folder list. It exposed misleading “0 objects”
> and “nothing is playing” text, plus a terminal-specific message, while the server
> was stopped. The [captured pre-correction preview](../../qa/evidence/macos-27.2.1-overview-offline-before-fix.png)
> records the observed issue. Source and translations now show those data as unavailable and use
> plain language. A fresh preview view confirmed the running state, library count, folder list,
> section order and add-folder action. It also showed that the running server supplied no TV name,
> leaving a vague connection fallback; the copy now says which detail is missing and suggests
> checking server setup. The replacement copy has not been visually rechecked. The transient
> screenshot included the surrounding user desktop and was not retained in the repository. The
> full protocol, VoiceOver and long-list display remain NOT RUN.
> The preview was launched from Finder while macOS itself used German, but the app
> rendered English. The app previously selected its locale only from `LC_ALL`,
> `LC_MESSAGES` and `LANG`, which Finder did not provide as expected. The macOS app now
> reads Foundation's ordered preferred-language list first and uses environment values
> only when no shipped language matches. The implementation has not yet been visually
> rechecked with conflicting system and process locales; manual acceptance remains
> `NOT RUN` until observed. The Apple Silicon macOS target passed `cargo check` and
> Clippy with warnings denied; formatting, the 137-link documentation check, generated
> requirement-view consistency and `git diff --check` passed. Automated tests and the
> Finder language-selection protocol had not been run at that checkpoint; the macOS test
> targets compiled with `cargo check --tests`, but no cases had been executed.
> A later source review found excessive top whitespace in the existing overview capture.
> The scroll view now tracks window resizing, and the content-height calculation matches
> the sections actually placed; short content is balanced in the available viewport.
> The Apple Silicon macOS build and Clippy pass after this change, and the file-size gate
> remains green with 35 unrelated files above its 300-line target. A fresh GUI render and
> resize observation have not been recorded.
> The locale follow-up added regression cases for regional language matching, skipping
> unsupported preferred languages and falling back to the process locale. The targeted
> Apple Silicon macOS `strings` integration target passed all 17 tests after the locale
> source-literal allowlist was corrected for existing HTTP unit-test fixtures. This does
> not replace the pending Finder language-selection observation.
> The full Apple Silicon macOS crate suite initially exposed inverted `RefreshOutcome`
> reporting, stale expectations for localized recovery copy, and a stale source path in
> the loopback-only gate. Those defects and expectations are corrected. A subsequent
> `cargo test --target aarch64-apple-darwin -p phairdlna-mac` passed all 87 unit,
> integration and documentation tests. This is crate-level evidence, not a full workspace
> run or a real-GUI acceptance result.
> The overview's vertical document-height calculation has since been isolated as a pure
> helper with unit coverage for short content, exact viewport fit and scrollable long
> content. The Apple Silicon macOS tests and manual rendered-window observation remain
> the next verification steps for this layout change.
> A live locale check launched the arm64 executable with `LC_ALL`, `LC_MESSAGES` and
> `LANG` set to English; its `--help` output was German, matching the macOS preference.
> This confirms the Foundation preference takes precedence at process startup. Finder's
> GUI render and fallback when all preferred languages are unsupported remain unverified.
> The supplied crash report recorded `Taskgated Invalid Signature` after the temporary
> preview bundle had been modified post-signing. Re-signing and verifying the completed
> temporary bundle restored launch; the repository packager now signs after assembly and
> verifies the final signature. Both the arm64-only and default universal packaging runs
> passed in fresh temporary output directories on 2026-11-19; `lipo` confirmed x86_64 and
> arm64 slices and the universal bundle passed signature verification. The packager now
> also fails on a missing architecture instead of suppressing the `lipo` result; a fresh
> universal packaging run passed with separate arm64 and x86_64 checks enabled. GUI visual
> review, VoiceOver, device acceptance and notarisation remain NOT RUN.

> **Historical partial non-test gate audit (2026-11-19, before OD-18 implementation).** The worktree
> at that audit passed `cargo fmt
> --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, an Apple
> Silicon macOS build of `phairdlna-mac`, `scripts/check-no-unsafe.sh`,
> `scripts/check-file-size.sh`, `scripts/check-crate-boundaries.sh`,
> `scripts/check-doc-links.sh` (137 links), and the generated-requirements check
> (346 requirements). `cargo deny check licenses` passed without warnings after the
> active allow-list was narrowed to licenses in the current lock graph;
> `cargo deny check bans sources` passed with a warning for the two locked
> `miniz_oxide` versions. `cargo audit` exited 0 after scanning 124 dependencies and
> printed no advisory finding. This is not a full gate run: workspace tests and
> doctests, real-Mac usability acceptance and notarisation were then NOT RUN. This snapshot
> is superseded by the full local gate run recorded below. That audit also found an
> architecture discrepancy tracked as OD-18: ADR-0002 /
> DEC-019 select `quick-xml` and `tower-http`, while neither appears in the lockfile;
> the current source has a project-owned XML parser and Tokio-based timeout middleware.
> Do not treat that discrepancy as an accepted implementation result until it is reconciled.

> **Full local quality-gate run (2026-11-19).** The first run found two UI functions above
> the 60-line Clippy limit and a missing `admin` feature guard in the no-default-features
> build. The overview reflow and general Preferences tab were split into focused modules;
> the optional API startup call was guarded and extracted. After those fixes,
> `./scripts/local-ci.sh` exited 0: formatting, workspace Clippy, workspace tests and
> doctests, `--no-default-features` build, unsafe/file-size/boundary/link checks, licence,
> source/ban and advisory checks, and generated requirement views all passed. The file-size
> gate still reports files above its 300-line target; the dependency check warns that two
> locked `miniz_oxide` versions are present. This is local gate evidence for current HEAD,
> not a GitHub workflow run, milestone acceptance, GUI launch or device test.

> **Full gate rerun (2026-11-19, working tree based on `9481e49`).** After the shared
> guidance-font constant, overview reflow boundary tests and the request-guard test split,
> `CARGO_NET_OFFLINE=true ./scripts/local-ci.sh` exited 0 with all eleven gates passing.
> The workspace integration tests bound successfully with local loopback
> permission; within the sandbox the same tests are denied permission before binding. The
> run still reports the 300-line target warnings and the duplicate `miniz_oxide` versions.
> This does not change the open real-GUI, accessibility or device evidence.

> **Request-guard module split (2026-11-19).** Moved seven private-helper tests out of
> `crates/phairdlna-cli/src/serve/guard.rs` into `guard/tests.rs`; production behavior and
> visibility are unchanged. The production module is now 261 lines (from 375), the focused
> tests pass, CLI Clippy passes, and the file-size warning count drops from 35 to 34. The
> source index and F-16 test strategy now point to the test module.

> **Overview reflow boundary coverage (2026-11-19).** The 12-point guidance size is now a
> shared constant used by both AppKit font setup and conservative line estimation. Two
> unit tests cover the exact estimated-width boundaries at 100% and 200%, explicit line
> breaks and wide Unicode. The tests and macOS crate Clippy pass; rendered clipping remains
> `NOT RUN`.

> The exact source revision also packages as a universal app at
> `/private/tmp/phairdlna-overview-reflow-preview/PhairDLNA.app`; arm64/x86_64 checks,
> eight-byte `PkgInfo` and strict ad-hoc signature verification pass. Launch and rendering
> remain unverified.

> **Post-update F-15 check (2026-11-19).** After the refused-access menu correction,
> `cargo check --target aarch64-apple-darwin -p phairdlna-mac` and
> `cargo clippy --target aarch64-apple-darwin -p phairdlna-mac -- -D warnings` passed.
> The menu test now covers the refused state; `cargo test --target aarch64-apple-darwin
> -p phairdlna-mac --test menu` passed all seven cases, including that regression.

> **Overview state-model coverage (2026-11-19).** The state-dependent text for the native
> overview now comes from the pure `phairdlna_mac::overview` view model, which AppKit
> displays directly. Four new cases verify the initial checking state, scanning state,
> every configured folder and missing-volume wording, and suppression of stale counts,
> folders and activity for stopped/refused snapshots. The stale-data case first failed
> against the previous formatter, revealing that stopped/refused state relied on an
> upstream snapshot cleanup to hide old roots; the view model now enforces that invariant
> itself. The full `cargo test --target aarch64-apple-darwin -p phairdlna-mac` suite
> passed all 96 tests and doctests; all-target Clippy, formatting, generated requirement
> consistency, documentation links and diff checks passed. The automated model does not
> verify the rendered window, controls, resizing, VoiceOver or the remaining degraded
> state mapping; those remain open in F-15 and the macOS usability protocol.

> **Connection guidance coverage (2026-11-19).** Two more overview view-model cases now
> verify that the German missing-TV-name message points to server setup and that the
> English discovery-unavailable message retains both the server name and network address.
> The full macOS-Arm64 crate suite passes all 98 tests and doctests; all-target Clippy,
> formatting, generated requirement views, documentation links and diff checks pass.
> This checks localized content, not its rendered layout or a real television connection.

> **Native preview launch diagnosis (2026-11-19).** The supplied `SIGABRT` is distinct
> from the earlier `CODESIGNING` invalid-signature report: the main thread aborts in
> AppKit's `NSApplication.sharedApplication` / `HIServices::RegisterApplication` path.
> A fresh arm64 package passed configuration validation and strict signature verification;
> its `CFBundleExecutable` exists, the Mach-O is arm64 and `PkgInfo` is exactly eight
> bytes. `open -n` returned `kLSNoExecutableErr` for this bundle from both temporary and
> per-user Applications locations; `lsregister` returned `-10822`, and `lsappinfo` showed
> no session application list. A direct-launch report is recorded below. These
> observations show that Launch Services is not functioning in this Codex session; a
> control attempt to open the built-in Calculator app failed with the same `-10827`, so
> this is not specific to PhairDLNA's bundle. These observations do not prove whether the
> bundle launches in a normal user session or that the application runtime is crash-free.
> The corrected GUI render remains NOT RUN.

> **Launch Services database inspection (2026-11-19).** A read-only `lsregister -dump`
> completed its integrity check but reported the user database path and cache sequence as
> not found (`path = '?'`). `open -n -g /System/Applications/Calculator.app` still failed
> with `kLSNoExecutableErr` (`-10827`). This supports the existing session-level diagnosis;
> no Launch Services database reset or mutation was performed. The app runtime remains
> unverified.

> **Repeated native launch report (2026-11-19, 09:49).** The supplied report has the same
> incident ID, `A96EFF41-05E6-4E48-8F34-471DDCBCAA54`, and repeats the abort in
> `NSApplication.sharedApplication` / `HIServices::RegisterApplication`. The utility
> thread again enters `_LSCopyLocalDatabase`; its `OSStatus` `-10813` is Apple's
> `kLSDataUnavailableErr`. This adds a concrete Launch Services error code but does not
> identify the unavailable resource or establish a PhairDLNA defect. The earlier
> Calculator control still points to a session-level issue. No database mutation was
> attempted; app runtime and visual acceptance remain NOT RUN.

> **M2.7 runtime audit (2026-11-19).** Rechecking production references found that
> `LifecycleState` and `Timeline` are used only inside their core definitions and tests;
> the CLI serving process and admin `Status`/`/api/status` path do not use or expose them.
> The prior roadmap wording that claimed runtime sleep/wake, degraded transitions, retry
> integration and an operational timeline was too strong and has been corrected. M2.7
> remains partial: service-file management and pure transition/timeline primitives exist,
> but those do not prove runtime lifecycle behavior. The F-15 AC-20 `Degraded` view depends
> on F-13 supplying an authoritative state and reason and F-14 exposing them through the
> local authenticated API. The app must not guess this state from warnings or discovery.

> **Packaging regression checks (2026-11-19).** Two macOS-only negative tests now verify
> that `lipo` rejects a thin executable and that changing a sealed resource after ad-hoc
> signing fails strict bundle verification; source checks confirm the packager invokes
> these guards. Both negative cases fail as expected;
> the full macOS-Arm64 crate suite passes all 98 tests and doctests, with formatting and
> all-target Clippy clean. These checks validate the packaging guards, not Launch Services
> installation or a rendered application launch; they do not inject a faulty binary into
> the full packaging script. GUI acceptance remains NOT RUN. The independent review found
> no critical issue and confirmed this scope limitation.

> **Startup-order check (2026-11-19).** `NSApplication.sharedApplication` and the regular
> activation policy now run before `Worker::spawn`, so AppKit initialization completes
> before background work begins; if AppKit cannot initialize, the worker is not started.
> The targeted macOS crate suite passed (98 tests and doctests), and arm64-only plus
> universal bundles were rebuilt; the universal binary has both architecture slices,
> a valid strict signature and an eight-byte `PkgInfo`. Treat the ordering as a startup
> safeguard, not as a verified crash fix; the runtime failure is recorded in the
> incident-specific note above. GUI acceptance remains NOT RUN.

> **Preferences text scaling implementation (2026-11-19).** F-15 now applies 100%,
> 150% and 200% text sizes to the General, Expert and About tabs, grows the tab
> document for long or multiline server names, follows window resizing and keeps the scaled footer separate
> from the scroll viewport. Independent read-only reviews confirmed footer spacing,
> long/multiline-name document height and resize geometry at the three text sizes. The macOS
> crate passes Clippy with warnings denied, formatting, file-size, documentation-link,
> generated-requirement and diff checks pass. GUI rendering, keyboard, VoiceOver and
> no-clipping acceptance remain NOT RUN because the current Launch Services session
> cannot launch even its system-app control.

> **Current Preferences build artifact (2026-11-19).** The latest worktree built as an
> arm64 app at `/private/tmp/phairdlna-current-package.vRYw6q/PhairDLNA.app`. The
> packager passed strict ad-hoc signature verification, confirmed the arm64 executable
> and verified the eight-byte `PkgInfo`. This artifact establishes packaging only;
> Launch Services runtime, rendered layout and accessibility remain NOT RUN.

## 1. Where the project stands

The **requirements baseline is approved** (2026-11-15), and the implementation is running:
the milestones M1.1 to M1.11 are coded, and M1.9 has been through an independent review
whose findings are addressed. The server now **serves over the network** — but **no device
test has been performed**, so no milestone exit criterion is met yet. The repository
contains a Rust workspace, a complete documentation baseline and the gates that will hold
the implementation honest.

The first native overview window is now wired to the live app snapshot and shows
server state, library count, shared folders, current scan/playback and TV
connection name/address with live discovery availability in the selected app language. This is an implementation
slice. Before its first status response it now says that the status is being checked instead of incorrectly claiming the server is stopped. AC-19/20/22 remain open: first-run guidance, state-specific controls,
discovery verification, degraded-state mapping, automated acceptance coverage
and real-Mac usability review remain open. The activity area only shows current
work, not a historical event log.
Configuration read and folder-save failures now have separate localized guidance; the
app no longer displays the raw configuration error containing an absolute path. Connection,
authentication and server-response failures also use translated guidance instead of raw
technical details or terminal instructions. Server-control, preference-save, diagnostics-save,
item-opening and application-startup failures now use the same plain-language treatment.

The milestone that was closed last is **M1.11 — the live daemon**: `phairdlna-cli serve`
reads the configuration, loads the device identity, scans the library, opens the HTTP
server and the discovery socket, and answers a client until it receives `SIGINT` or
`SIGTERM`. The server serves the device description, the two service descriptions, the
SOAP control endpoints, the media with its byte ranges and the artwork route; the
discovery socket advertises the server, answers the searches of a client and withdraws its
advertisements when the daemon stops. The order of the stop is the one of `OPS-008`:
the advertisements go first, the discovery stops, the HTTP server drains its transfers,
and the process exits with the code 0.

**What is proven and how.** The daemon is tested as a client sees it, over a real socket:
`crates/phairdlna-cli/tests/serve.rs` reads the description, browses the catalogue with
a SOAP request, streams a file and asks for a byte range; `crates/phairdlna-cli/tests/discovery.rs`
speaks the real SSDP protocol and observes the advertisement, the answer of a search and
the withdrawal; `crates/phairdlna-cli/tests/serve_command.rs` starts the binary, reads the
description and ends it with a `SIGTERM`. The rules themselves stay in the core, and the
core is unchanged by M1.11 — it takes no new dependency.

**The first step of M2 is closed: M2.1 — multiple roots, external volumes and network
shares.** The catalogue keeps the roots apart: every root has its own key in the
identifiers, its own place in the tree and its own boundary, so a link from one root
into another is not followed and a root that lies inside another root is published
once. The catalogue now **remembers the volume of every root** between two reads, and
it says what changed: a drive that was unplugged takes its files with it and is
reported, a drive that comes back under the same identity keeps its identifiers, and a
drive that comes back under **another** identity is given new identifiers while the
identifier of the catalogue grows, so that a client browses again (`PLT-011`). The
daemon watches the roots — a cheap question that reads no file of the library — and
reads the catalogue again when the answer is "yes"; a file that is added *inside* a
root is noticed by the next full scan, and the schedule of the scans is the work of
`FUN-024` (M2.4). The evidence is
`crates/phairdlna-core/tests/media_roots_multiple.rs`,
`crates/phairdlna-core/tests/volume_events.rs` and
`crates/phairdlna-core/tests/volume_platform_events.rs`; the decisions are D-15 to
D-20 of `../03-features/F-08-media-roots-and-filesystem.md`.

**The second step of M2 is closed: M2.2 — metadata extraction, artwork and thumbnails.**
The library now shows its pictures. A **pure-Rust** parser reads the cover out of an ID3v2
tag (`APIC`, versions 2.2, 2.3 and 2.4) and out of an MP4 container (the `covr` atom), and
the image that lies **next to** a file — `cover.jpg`, `folder.jpg`, `album.jpg` or the name of
the file itself with an image extension — wins over the cover in the tag. A photo of the
library is its own picture: the server produces a small JPEG of it, never larger than it, and
keeps it in a **disposable cache** that may be deleted at any time, that never grows beyond
its bound and that removes its oldest pictures when it does. The picture is served at
`/art/<id>`, and the DIDL-Lite document names it in `upnp:albumArtURI` — an absolute address
of this server, with no path of the disk and no `dlna:profileID`. A file that is broken
gives no picture and no error: the library stays browsable and the daemon keeps serving. The
evidence is `crates/phairdlna-core/tests/artwork.rs` and `crates/phairdlna-cli/tests/artwork.rs`.

**The third step of M2 is closed: M2.3 — playlists, subtitles and organisation.** The library now
understands the lists the owner wrote by hand. A **playlist** — M3U, M3U8 or PLS — is read within
bounds (a mebibyte, ten thousand entries, a line of four kibibytes), and every entry is resolved
against the folder of the playlist itself. An entry that names a **network address** is refused
outright: the server makes no request for it. An entry that walks **out of the root** is refused
and reported. An entry that names a file that is not there stays in the playlist as a child that
carries no resource, so that the user learns where the track went instead of wondering. The
playlist is a **container** of the class `object.container.playlistContainer`, and its children keep
the **order** of the playlist — a list that is sorted is a different list. A broken playlist never
takes a scan down: it produces a diagnostic and the rest of the library is indexed.

A **subtitle** that lies next to a film is paired with it while the root is read, not while a
client browses. The pairing is an **additional `res`** of the media object, with its own
`protocolInfo` that names the type of the file (`text/vtt`, `application/x-subrip`, `text/x-ssa`,
`text/x-microdvd`), and the file is served from the media endpoint with that type and the
character set it is written in. The element is additive: a client that does not understand it still
plays the film. When several sidecars match one base name, the one with the highest priority
extension wins — WebVTT first, because it is the format a browser-based client reads without a
conversion.

The **organisation** is decided as well. The catalogue computes an **identity key** for every
file — the path, per component, with Unicode NFC and the case fold of the volume, scoped by the
identity of the volume — and it takes the **uniqueness** of a file from the stable file identity
the platform gives, not from the path. A file that two roots reach is therefore **one** catalogue
entry: the first root wins, the second sighting resolves to the entry that is already there, and
the identifier of the object does not change. The evidence is
`crates/phairdlna-core/tests/playlists.rs`, `crates/phairdlna-core/tests/subtitles.rs` and
`crates/phairdlna-cli/tests/subtitles.rs`; the decisions are DEC-048 to DEC-057 of
`../00-governance/DECISION-LOG.md`.

**The fourth step of M2 is closed in part: M2.4 — the configuration that means something.**
Two settings of the configuration were written down long ago and were never obeyed, and that is
the worst kind of setting: the user believes in it and it does not exist. The **access-control
list** (`security.allowed_clients`) is now read at the start, checked against the **address of
the socket** — and not against a header, which a client that is not allowed may write as it
likes — and a client that is not on the list is refused with `403` before a handler sees the
request. An entry of the list is an address (`192.168.1.20`), a network (`192.168.1.0/24`) or the
word `any`; an entry that cannot be read stops the server before a socket is opened, because a
list that is silently ignored is a list that the user believes in. The **administration surface**
has its own settings now: the port it listens on, the addresses it binds to, the file that holds
its token, and the word that lets it leave the machine — a configuration that would put the
surface on an address of the network is **refused** unless the user says so in the configuration
(`CFG-005`, DEC-061). The **export** and the **import** of the configuration exist as commands:
`phairdlna config export` writes the configuration as the server understands it, with the schema
version, and `phairdlna config import <file>` reads a file and reports the configuration it
would become — it writes nothing, and that is the part that is still open. The evidence is
`crates/phairdlna-cli/tests/serve.rs`, `crates/phairdlna-cli/tests/cli.rs` and
`crates/phairdlna-core/src/security/access.rs`.

**The fifth step of M2 is closed in part: M2.5 — the local administration surface.** The server
now has a door for its owner, and the door is narrow on purpose. The surface lives in the crate
`phairdlna-admin`, listens on the **loopback interface only** (`127.0.0.1` and `::1`), on a
port of its own, and is started only when the configuration asks for it; a build without the
cargo feature `admin` has no surface at all. Every request must carry the **bearer token** of
the installation — a random secret of 256 bits in a file with the permissions `600` inside a
folder with the permissions `700`; a file that others may read stops the surface instead of being
repaired silently. The surface answers only to a `Host` header that names the loopback, so that
the "DNS rebinding" attack finds nothing; it answers only to a request that carries the custom
header `X-Phair-Admin`, which a form in a browser cannot send; it carries **no header that would
let another origin read the answer**, and it answers no question about permissions. A request
that changes something carries the token of the **"cross-site request forgery"** defence, which
is bound to the session — the session *is* the token of the installation (DEC-060).

The surface answers three questions. **`/api/status`** says how long the server has been running,
how many streams are running, how many objects the catalogue holds, how much memory the process
occupies (where the platform can say), what the roots of the library look like and which
identifier the server tells its clients that it has changed. **`/api/catalogue/rescan`** reads
the library again. **`/api/diagnostics`** shows the records of the log and the warnings of the
server — with the paths of the user replaced by their pseudonyms, and the surface redacts again
what the daemon behind it hands over, so that a leak needs two mistakes and not one. The
transfers that run are registered, so that the diagnostics name the client and what it is
playing. The **source revision** is embedded into the binary when it is built, so that the
running server and the source it will offer cannot disagree. The evidence is
`crates/phairdlna-admin/tests/admin_surface.rs` and `crates/phairdlna-cli/tests/admin.rs`; the
decisions are DEC-058 to DEC-064 and **ADR-0012**.

**The sixth step of M2 is closed in part: M2.6 — the macOS application.** The regular app has a main overview and menu-bar status item. The crate
`phairdlna-mac` holds a real application now, and the application is nothing but a front end: it
holds **no state of the server of its own**, asks the administration surface of M2.5 what the
server does, and shows the answer as a line of text in the menu bar (DEC-065). The client of the
surface is **written in this project** — a small HTTP/1.1 client, a small reader of JSON with
bounds on its size and its depth, and a transport that is a trait, so that the tests answer the
client with a recorded answer and need no socket (DEC-066). The crate depends on the core of
the project and on the bindings of the system and on **nothing else**: no network library, no
serialization framework, and no connection to anything but the loopback interface — a test walks
the sources and the manifest of the crate and proves it (`tests/no_egress.rs`). The requests run
on a **thread of their own**, so that the main thread of the application never waits for the
network and the menu never stutters (DEC-067); the channel of the commands holds one command,
so that the application never has more than one request in flight. The token of the
"cross-site request forgery" defence is read out of the header that the surface hands with every
answer and sent back with the request that changes something (DEC-068).

The **texts of the user interface live in one resource file per language** — `en-US` and `en-GB`
— and in no other place: the lines of the menu, the labels that the screen reader reads, the
messages of a failure and the help of the command line (DEC-071). A test reads the sources of
the crate and refuses every text that a user would read. The **two views** — the short one and
the detailed one — read the same state, and the mode is a preference of the *application* that
lives in a file of its own, beside the configuration of the server and not inside it: switching
the view changes no setting of the server, restarts nothing and interrupts no transfer (DEC-070,
`UX-015`, `UX-016`). The state of the server is **text and never a colour alone**: the item
carries the label and the value that the screen reader reads, and the picture of the system
follows the appearance of the menu bar (`UX-009`, `UX-018`, `MB-006`).

The **only file of the workspace that uses `unsafe`** is the bridge to the Objective-C runtime of
the menu bar application, because AppKit hands the action of a menu item to a target object as a
message and a class of our own cannot be written without it. The record **ADR-0013** carries the
safety argument of every block, and the gate **QG-03** knows that one file by that record: a
file with `unsafe` and without the marker still fails the gate, and a marker that names a record
that does not exist fails the gate as well (DEC-069). The evidence is
`crates/phairdlna-mac/tests/ipc_client.rs`, `crates/phairdlna-mac/tests/menu.rs`,
`crates/phairdlna-mac/tests/application.rs`, `crates/phairdlna-mac/tests/worker.rs`,
`crates/phairdlna-mac/tests/presentation_modes.rs`, `crates/phairdlna-mac/tests/bundle_metadata.rs`,
`crates/phairdlna-mac/tests/no_egress.rs` and `crates/phairdlna-mac/tests/strings.rs`; the
decisions are DEC-065 to DEC-071.

**Current M2.6 state (2026-11-19).** The app retains its native overview, status menu, read-only
preferences, authenticated API client and text-size controls. Its folder picker, direct writes to
server configuration and LaunchAgent `launchctl` start/stop prototype have now been removed while
DEC-079 selects app-managed service registration and the app bootstrap/API/restart direction; OD-15's
service-process evidence and independent review remain open. The folder action is visible but disabled with an explanation; the
menu has no service start/stop action and the former service-control module was removed. Tests prove
the config remains unchanged. Start/stop actions and AppKit selectors are absent; tags 9 and 10 are unmapped. A missing config still shows a recovery warning, and a valid
config without roots shows that app folder setup is unavailable. This closes the unapproved
mutation path; it does not complete first-run setup or UX-019. The app cannot bootstrap a fresh
configuration, the optional discovery verification step is absent, and scan progress/cancellation
are absent. A current screenshot confirms the app launches and displays the overview at its
default size; window reflow, keyboard, VoiceOver and the remaining usability checks remain
`NOT RUN`. The written usability protocol is partially executed. Service registration and settings-write
implementation remain gated by OD-15 evidence and its independent review.

**F-15 navigation and documentation gate (2026-11-19).** Removed two obsolete paths from the
beginner source map, corrected its descriptions of the current onboarding and AppKit dialogs,
and renamed the startup helper to describe its actual recovery-guidance behavior. QG-11 now checks
current source paths in `WHERE_TO_CHANGE.md` as well as relative Markdown links. Its negative check
rejects a deliberately missing source path; the latest normal run passes all 137 links and 131
source paths. This prevents the source map from directing contributors to files that no longer
exist. It does not change the open service or folder-setup decisions above.

**Local API compatibility fix (2026-11-19).** The live app received catalogue and root status but no
usable friendly name or network address. It now falls back to the validated
name in its shared local configuration and tells the user to search for that name in TV Media Sources,
without incorrectly claiming local-only availability. The ARM64 preview bundle built and passed strict
signature verification; its screenshot shows the server name and updated guidance. Folder setup remains
disabled pending the OD-15 isolation and service-write path.

**Workspace validation and current Mac bundle (2026-11-19).** The complete
`cargo test --workspace --locked --offline` run passed after the sandboxed run was denied
loopback binding; the rerun with local loopback access passed the Admin, CLI, Core and Mac
tests and doctests. Workspace Clippy with warnings denied passed. The current universal
arm64/x86_64 bundle passes strict local signature verification. These results do not include
GUI launch, VoiceOver, real-device, or interoperability acceptance. The updated QG-11 and CI
policy documents are in review; their approved v1.0 copies are archived with SHA-256 manifests.

**The catalogue of the library now lives on disk as well: M2.9 — the index.** The
catalogue is still built in memory and answered from there, but a copy of it is written
into a **SQLite database** that belongs to the user alone: the file is created with the
permissions 600, and the directory that holds it with 700. The path of the file is the
setting `library.database_path`, and the data directory of the user holds it when the
setting is missing.

The index has **two connections**: one that writes and one that reads, and the journal
of the database is the write-ahead log. The whole catalogue is written in **one
transaction**, so a browse during a rescan sees the catalogue of before or the catalogue
of now, and never a mixture of the two (`FUN-008`). The server **takes the catalogue of
the index at its start** and reads the library again in the background, so a restart does
not cost a full scan and a client never meets an empty server because the index is empty
(`FUN-009`).

A file that is **damaged** — bytes that are not a database, a database that fails the
integrity check, or a database of another schema — is moved aside under the name
`.corrupt.<moment>`, a new index takes its place, the library is read again, and the
daemon says so **in the log and in the status of the server** (`FUN-010`). The index is a
cache: deleting it costs a re-scan and nothing else, and the media of the user is never
written to.

The evidence is `crates/phairdlna-core/tests/catalogue_rebuild.rs` and
`crates/phairdlna-cli/tests/catalogue_index.rs`; the decisions are D-01, D-05 and
D-12 to D-16 of `../03-features/F-09-catalogue-index.md`.

**One consequence of the index is on the record.** SQLite is compiled into the server
(the binding `rusqlite` with the feature `bundled`, as ADR-0006 and the dependency review
decide), so that the server does not depend on the SQLite of the system and cannot load
an extension into its own database. The price is a **C compiler at the time of the
build**, which sits in tension with `PLT-009` ("no C toolchain beyond what the platform
already provides") and with the claim of the workspace that the core is free of a C
compiler. The decision itself is the owner's and was taken in the dependency review; the
consequence is recorded here and in `../04-security/DEPENDENCY-LICENSE-REVIEW.md`, and it
is the one thing of this step that the owner may want to overrule.

**At the M2.9 checkpoint, two parts of M2.4 and M2.5 were still open.** The **source offer**
(`OPS-016`, `SEC-070`) does not exist: the archive, the route and the build step that produces
the artefact are not written, and the open question 2 of F-14 asks the owner how the archive is
to be produced. The **plain-language first-run and diagnostics CLI** were then open, but the later commit `1bec4cb7` adds `setup` and `diagnostics` commands. Their current full behavior and acceptance must be reviewed against the updated tests. The
**trusted-client list** (`CFG-010`) waits for the device profiles of F-07. The **address of the
client** is not known to the routes of the media server yet, so the diagnostics name the client
and the title but not the address.

**One deviation is on the record.** The picture pipeline of M2.2 parses **in the server
process**, not in the sandboxed child that ADR-0008 prescribes, because that child does not
exist yet — it is the work of M-06. Every read and every allocation of the pipeline is
bounded, and the parsers are written so that they move into the child without a change of
their interface. The deviation is written down in **ADR-0011** and in DEC-042, and it ends
with the child.

**What is still missing.** The IPv6 discovery, the interface selection of the sending
socket (the daemon has one address), the rate limiter of the answers, the video posters
(the baseline ships no video decoder), the sandbox child of the picture pipeline (M-06,
ADR-0011), the transfer limits, the incremental scan of the index and the change events of the
filesystem. Each of them is named in the requirement index with the state that is true
today.

The milestone before it, **M1.9 — the catalogue over one root**, is unchanged: a scanner
that reads a configured root, opaque blake3 identifiers that survive a rescan and a move
of the library, and a ContentDirectory browse that shows real folders and files. The
placeholder catalogue is retired; `phairdlna-cli catalogue scan` prints the tree with the
identifiers a client would see, and `phairdlna-cli catalogue browse <id>` prints the
document a client would receive.

**The configuration takes a list of roots**, and M2.1 is the milestone that makes the
list mean something: several roots are browsable at the same time, they keep their own
identifiers and their own boundaries, and a root that comes and goes with its drive is
handled while the server runs.

**The requirement group of M1.9 that was open is now decided.** The roadmap assigned
`FUN-008`, `FUN-009` and `FUN-010` — the SQLite index that survives a rescan, that can
be rebuilt and that recovers from corruption — to M1.9. The scope the owner approved for
M1.9 was the scanner, the opaque identifiers and the browse over one root; the index on
disk was not part of it, so the catalogue of M1 lives in memory. The owner decided on
2026-11-16 (**DEC-037**) that the three requirements **move to M2**, where the rest of the
index work lives (step M2.9 of the roadmap). The in-memory catalogue of M1 is the
accepted baseline, and the requirement index carries the new milestone.

| Area | State | Evidence |
| --- | --- | --- |
| Requirements | **Complete** | 346 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
| Traceability | **Complete** | Generated matrix requirement → feature → milestone → module → verification → evidence — `TRACEABILITY-MATRIX.md` |
| Architecture | **Complete** | Module boundaries, data flows, runtime model — `../02-architecture/ARCHITECTURE.md` |
| Decisions | **Complete** | The ADRs in `../02-architecture/adr/` plus 79 entries in the decision log — `../00-governance/DECISION-LOG.md` |
| Security model | **Complete** | Eight mitigations, 28 threats, each with its verification — `../04-security/SECURITY-MODEL.md` |
| Privacy model | **Complete** | Data inventory, network posture, source offer — `../04-security/PRIVACY.md` |
| Dependency review | **In review** | `cargo metadata --locked --offline --all-features` resolves 122 registry packages (121 distinct crate names); the licence gate passes, but the per-crate purpose, maintenance, alternatives and reviewer inventory is incomplete — `../04-security/DEPENDENCY-LICENSE-REVIEW.md` |
| Quality gates | **Defined** | 28 gates with their commands, the label vocabulary and 15 recorded negative tests; 13 gates can run today — `../05-quality/QUALITY-GATES.md` |
| Test strategy | **Complete** | Test layers, property tests, negative catalogue, fuzz targets — `../05-quality/TEST-STRATEGY.md` |
| Interoperability | **Framework complete, evidence empty** | 26 conformance checks defined; **no device test has been performed** — `../05-quality/INTEROPERABILITY-MATRIX.md` |
| Performance | **Budgets defined, not measured** | Budgets and the measurement protocol — `../05-quality/PERFORMANCE-BUDGETS.md` |
| Roadmap | **Complete** | Six milestones with exit criteria — `ROADMAP.md` |
| Risk register | **Complete** | 16 risks with mitigations and owners — `RISK-REGISTER.md` |
| Open decisions | **12 remain open** — OD-05, OD-08, OD-14, OD-16, OD-17 and OD-18 have owner direction; OD-15 remains evidence-gated before its conditional PLT-013 change | `OPEN-DECISIONS.md` |
| Glossary | **Complete** | The protocol, product, platform, security and project vocabulary — `../01-product/GLOSSARY.md` |
| Research inputs | **Complete** | The briefs the specifications cite — `../09-research/` |
| Implementation | **M0 done, M1.1–M1.11 implemented, M2.1, M2.2, M2.3 and M2.9 implemented, M2.4–M2.7 in part** | The four crates exist with the boundary enforced; configuration, platform traits, discovery, media serving, catalogue, CLI, diagnostics and native menu app are implemented in part or in full; the browser UI was removed (DEC-075); the initial native overview shows live state, item count and folders; M2.7 has service-file tooling and pure lifecycle primitives, but no lifecycle runtime integration, sleep/wake handling or app-visible degraded state; full app flows, approved service lifecycle, accessibility review and device discovery remain open; **no device test has been performed**, so no milestone exit criterion is met yet |
| Device evidence | **None** | Every device row reads `NOT TESTED` |
| Release | **Current HEAD not releasable; one historical alpha exists** | `RELEASE-BLOCKERS.md` |

---

## 2. What was actually run, and what it produced

| Command | Result |
| --- | --- |
| `./scripts/local-ci.sh` | **all eleven gates green** (2026-11-19, current working tree based on `9481e49`; rerun with loopback permission): format, clippy, workspace tests, no-default-features build, no unsafe, file size, crate boundaries, doc links, licences, bans and sources, advisories, requirement views |
| `cargo fmt --all -- --check` | clean (it failed before this phase — the CLI entry point was reformatted) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 0 in the 2026-11-19 full local gate run against the current working tree; the earlier 1,038-test count is historical and not attributed to this run |
| `scripts/check-file-size.sh` | `RESULT: PASS` |
| `scripts/check-crate-boundaries.sh` | `RESULT: PASS` (3 sibling dependencies across 4 crate manifests) |
| `cargo deny check licenses` | `licenses ok` (exit 0) |
| `cargo metadata --locked --offline --all-features --format-version 1` | 126 package entries: 4 workspace packages and 122 registry packages (121 distinct third-party crate names) |
| `cargo deny check bans sources` | `bans ok, sources ok` (exit 0) |
| `cargo audit` | exit 0 in the 2026-11-19 gate run — 1,550 advisories loaded, 126 locked crate dependencies scanned |
| `scripts/check-doc-links.sh` | `RESULT: PASS` (relative links checked) |
| `scripts/check-no-unsafe.sh` | `RESULT: PASS` (no unsafe code in the tracked Rust files) |
| `python3 scripts/generate_requirement_views.py --check` | up to date; 346 requirements indexed |
| GitHub Actions, manual runs | **success on earlier commit SHAs, not current HEAD** — `43698100636`, `43698591233`, `43699136828`, all 13 gate steps green each time: https://github.com/mazer666/PhairDLNA/actions/runs/43699136828 |
| `python3 scripts/generate_requirement_views.py` | 346 requirements indexed; index and traceability matrix regenerated |

Toolchain used for the M2.1 run (and for M1.11 before it): rustc 1.99.1, cargo 1.99.1,
clippy 0.1.99, rustfmt 1.8.0-stable (Homebrew). `rust-toolchain.toml` pins the channel `stable`, so the toolchain
is within the support window (`rust-version` is 1.89).

Everything else in this repository is **documented and unmeasured**. In particular:

- no performance number has been recorded,
- no device test has been performed,
- no fuzzing run has been performed,
- no soak test has been performed,
- no claim of DLNA certification exists and none will be made.

---

## 3. The baseline in numbers

| Number | Value |
| --- | --- |
| Requirements | 346 — 284 MUST, 57 SHOULD, 5 COULD |
| Feature specifications | 22 (F-01 … F-19, F-22, plus two proposals, F-20 / F-21) |
| Acceptance criteria | 330 across the 20 baseline specifications (F-20 and F-21 are proposals and define none) |
| Glossary entries | 77 terms across five vocabularies |
| Independent reviews | 5 — beginner-experience, consistency, M1.9, OD-18 security and OD-15 security reviews, all in `../05-quality/reviews/` |
| Protocol conformance checks | 26 (`PC-01` … `PC-26`) |
| Threats modelled | 28 (`SEC-TH-01` … `SEC-TH-28`) |
| Quality gates | 28 (`QG-01` … `QG-28`), of which 13 can run today |
| Milestone gates | 7 (`G0` … `G6`) |
| Architecture decisions | The ADRs in `../02-architecture/adr/` + 79 decision-log entries |
| Open owner decisions | 12 remain open; OD-05, OD-08, OD-14, OD-16, OD-17 and OD-18 have owner direction; OD-15 gates macOS service lifecycle and any PLT-013 revision |
| Risks | 16 (`R-01` … `R-16`) |
| Archived baselines | 7 snapshots, each with a SHA-256 manifest |
| Research briefs committed | 9 documents under `../09-research/` |
| Competitors mapped | 11 products with per-feature proposals — `../01-product/COMPETITOR-FEATURE-ATLAS.md` |
| Competitive criteria | 11 testable criteria against the macOS field — `../01-product/COMPETITIVE-PARITY-ELMEDIA.md` |

---

## 4. What we should do next — the proposal

### 4.1 M0 is closed, with two items still open

M0 was the foundation: the approved baseline, the architecture, the four crates with the
boundary enforced, the licence and advisory gates on the real dependency set, and the
manual CI workflow. All of it is done. Two items of it are not:

| Item | What | Done when |
| --- | --- | --- |
| M0.5 | The manual CI workflow has been run on macOS and on Windows, not only on Linux | three run URLs exist in the run list, one per platform, and every step passed |
| M0.6 | The Intel macOS target of the build matrix builds | the matrix shows a green x86_64-apple-darwin build |

The matrix definition now routes jobs by self-hosted OS and architecture labels
instead of sending every row to an arbitrary local runner. No new manual matrix run
is recorded, so M0.5 and M0.6 remain open; workflow correctness is not run evidence.

### 4.2 M1, in ten small steps

Each step ends in something observable, and each is small enough to review in one
sitting. The order is not negotiable: each step needs the one before it.

| # | Step | The observable that says it is done | State |
| --- | --- | --- | --- |
| 1 | Configuration and the platform traits | `phairdlna-cli config validate` accepts a generated configuration and rejects a broken one with a named field | done (M1.1, M1.2) |
| 2 | SSDP discovery | `phairdlna-cli ssdp headers` prints the exact bytes that will be sent, and a recorded `M-SEARCH` from a real client is answered | done (M1.3) |
| 3 | The device description and the SCPD | `curl http://localhost:PORT/description.xml` returns a document that a real client accepts | done (M1.4) |
| 4 | The SOAP dispatcher and ContentDirectory | A recorded `Browse` returns a DIDL-Lite document that validates against the vendored XSD (gate QG-17) | done (M1.5, M1.6); the XSD validation of QG-17 is still **planned** |
| 5 | ConnectionManager | `GetProtocolInfo` names only MIME types the server can actually serve | done (M1.6) |
| 6 | DIDL-Lite generation | The escaping property test passes, and the XSD validation passes | done (M1.7) |
| 7 | HTTP media delivery | A byte range returns 206 with a correct `Content-Range`; an unsatisfiable range returns 416 | done (M1.8) |
| 8 | The catalogue over one root | A browse shows one folder of real files with stable ids | **done (M1.9)** |
| 9 | The live daemon | `phairdlna-cli serve` answers a client over HTTP and over SSDP, and stops gracefully on a signal | **done (M1.11)** |
| 10 | The security baseline | Every threat row in F-16 has its negative test, and they pass | next |

**Step 10 is not optional and not last.** The threat rows are cheap to test while the
code is small, and expensive to retrofit. It is also the step that M1.11 was implemented
ahead of; it is closed next, against the code that is already written.

### 4.3 The first real device, and when

**No device test happens before steps 1 to 8 are green.** The conformance checklist
(`PC-01` … `PC-26`) is automated and runs first; a device is the *second* test, not
the first. The first device round is scheduled inside M1, and its result — including
failures — goes into the interoperability matrix the same day.

**Which device first:** a Samsung or LG television, because they are the most
frequently reported source of DLNA incompatibilities. If only one is available, use
VLC on a laptop, because it logs what it receives.

### 4.4 What remains outside the current M2.6 slice

- The app-first native experience is in progress. Transcoding, broad media-library
  integration and Linux/Windows packaging remain later scope; they do not replace the
  current M2.6 onboarding, safe settings and accessibility work.
- No plugin system, ever, without an ADR.
- No automatic CI. Manual only, until the gates have been run three times green and
  their negative tests demonstrated.

### 4.5 What the owner has to answer, and what happens if they do not

Twelve open decisions remain listed in `OPEN-DECISIONS.md`, each with a recommendation
**and** the default that applies if there is no answer. **OD-05** (the minimum macOS
version) was decided on 2026-11-15: macOS 15 (Sequoia) and later. OD-15 keeps the exact
service identity and selected-folder access model open; DEC-079 selects app-managed service registration and local bootstrap plus authenticated settings updates with restart, with implementation gated on OD-15 evidence. DEC-077 selects app-level larger-text scaling, and DEC-078 selects the accepted XML/HTTP dependency targets. The 100% / 150% / 200%
setting and responsive overview implementation now exist; accessibility and no-clipping
acceptance remain open until the manual Mac protocol is run.

### 4.6 The honest risk of this plan

The plan is a plan, not a forecast. The largest uncertainty is still the device
matrix: the first device round may reveal that a requirement is wrong. That is the
point of the matrix, and it is cheaper to learn now than after M2.

---

## 5. Honest limitations of this baseline

- **The interoperability matrix is empty.** The mechanism is defined; the evidence is
  not there yet. The first device round will change several requirements, and that is
  the intended purpose of the matrix.
- **The performance budgets are targets.** They were chosen to be plausible and
  measurable; they have not been checked against a real machine.
- **The macOS firewall and local-network behaviour are documented by Apple and not yet
  verified on a device.** They are the two largest macOS-specific risks (R-02, R-03).
- **The competitive landscape rests on public documentation, not on comparative
  testing.** No competitor was benchmarked.
- **Some protocol details could not be verified against a primary specification** and
  are recorded as open questions in the feature specifications — in particular the
  `writeStatus` attribute, the exact `dc:date` format constraints, and the DLNA
  guideline table that a media server must satisfy.
