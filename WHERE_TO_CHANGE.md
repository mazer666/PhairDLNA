# WHERE_TO_CHANGE — beginner navigation

**Start here:** [README.md](README.md) → [docs/06-development/GETTING-STARTED.md](docs/06-development/GETTING-STARTED.md) → this file → the feature specification → the source and test paths.

## The rules

| Rule | Where it is written down |
| --- | --- |
| Code size, complexity, error handling, `unsafe` | [docs/00-governance/PROJECT-STANDARDS.md](docs/00-governance/PROJECT-STANDARDS.md) |
| Document structure, naming, migration | [docs/00-governance/DOCUMENTATION-POLICY.md](docs/00-governance/DOCUMENTATION-POLICY.md) |
| Requirement IDs, priorities, traceability | [docs/00-governance/REQUIREMENTS-METHOD.md](docs/00-governance/REQUIREMENTS-METHOD.md) |
| Agent responsibilities | [docs/00-governance/AGENT-ROLES.md](docs/00-governance/AGENT-ROLES.md) |
| Approved exceptions | [docs/00-governance/EXCEPTIONS.md](docs/00-governance/EXCEPTIONS.md) |
| Decisions and ADRs | [docs/00-governance/DECISION-LOG.md](docs/00-governance/DECISION-LOG.md) · [docs/02-architecture/adr/](docs/02-architecture/adr/) |
| What the product must do | [docs/01-product/TECHNICAL-REQUIREMENTS.md](docs/01-product/TECHNICAL-REQUIREMENTS.md) |
| Requirement index (generated) | [docs/01-product/REQUIREMENTS-INDEX.md](docs/01-product/REQUIREMENTS-INDEX.md) |
| User journeys | [docs/01-product/USER-JOURNEYS.md](docs/01-product/USER-JOURNEYS.md) |
| Feature index and specifications | [docs/01-product/FEATURE-CATALOG.md](docs/01-product/FEATURE-CATALOG.md) · [docs/03-features/](docs/03-features/) |
| Architecture and module boundaries | [docs/02-architecture/ARCHITECTURE.md](docs/02-architecture/ARCHITECTURE.md) |
| Security and privacy | [docs/04-security/SECURITY-MODEL.md](docs/04-security/SECURITY-MODEL.md) · [docs/04-security/PRIVACY.md](docs/04-security/PRIVACY.md) |
| Dependencies and licences | [docs/04-security/DEPENDENCY-LICENSE-REVIEW.md](docs/04-security/DEPENDENCY-LICENSE-REVIEW.md) |
| Quality gates and tests | [docs/05-quality/QUALITY-GATES.md](docs/05-quality/QUALITY-GATES.md) · [docs/05-quality/TEST-STRATEGY.md](docs/05-quality/TEST-STRATEGY.md) |
| Interoperability and device evidence | [docs/05-quality/INTEROPERABILITY-MATRIX.md](docs/05-quality/INTEROPERABILITY-MATRIX.md) |
| Performance budgets | [docs/05-quality/PERFORMANCE-BUDGETS.md](docs/05-quality/PERFORMANCE-BUDGETS.md) |
| The research the requirements cite | [docs/09-research/](docs/09-research/) |
| Build and test commands | [docs/06-development/BUILD-AND-TEST.md](docs/06-development/BUILD-AND-TEST.md) |
| Roadmap, traceability, risks, decisions | [docs/07-project/](docs/07-project/) |
| Release policy | [docs/08-releases/RELEASE-POLICY.md](docs/08-releases/RELEASE-POLICY.md) |

## The source tree

### Exists today

| Path | Responsibility | Feature |
| --- | --- | --- |
| `Cargo.toml` | Workspace definition, shared package metadata | — |
| `rust-toolchain.toml` | Pinned toolchain and components | — |
| `deny.toml` | Licence, advisory, ban and source policy | F-19 |
| `crates/phairdlna-core/src/lib.rs` | The portable domain core — the crate root and the error type | — |
| `crates/phairdlna-core/src/error.rs` | The core error type | F-12 |
| `crates/phairdlna-core/src/xml/` | The one XML writer of the project — the escaping, the length rules and the writer | F-02, F-05 |
| `crates/phairdlna-core/src/platform/` | The capability traits, the path model, the portable filesystem adapter, the native filesystem change watcher (`fs_watcher.rs`, `change.rs`), the memory of the process (`memory.rs`) and the default implementation | F-08, F-09, F-13, F-14, F-15 |
| `crates/phairdlna-core/tests/catalogue_incremental.rs` | Acceptance test for incremental rescan verifying unchanged files are not re-read (PERF-003) | F-09, F-21 |
| `crates/phairdlna-core/tests/protocol_conformance.rs` | Acceptance test suite for protocol conformance checks (PC-01 .. PC-26) | F-01, F-02, F-03, F-05, F-07 |
| `crates/phairdlna-core/tests/platform_matrix.rs` | Acceptance tests for build matrix coverage and platform targets | F-22 |
| `crates/phairdlna-core/tests/parser_fuzz_bounds.rs` | Parser robustness, SSDP datagram fuzzing, boundary limits and path containment tests | F-16 |
| `crates/phairdlna-core/tests/offline_volume.rs` | Offline media root non-destructive handling and availability reporting | F-18 |
| `crates/phairdlna-cli/tests/crash_recovery.rs` | Crash and corruption recovery tests verifying immutability of media originals | F-18 |
| `crates/phairdlna-cli/tests/invalid_configuration.rs` | Atomic validation and rejection tests for invalid configurations | F-18 |
| `crates/phairdlna-core/src/ssdp/` | The discovery protocol: parsing a search, matching targets, building bytes, and replaying stored M-SEARCH diagnostics requests (`replay.rs`) | F-01, F-07 |
| `crates/phairdlna-core/src/identity.rs` | The persistent device identity | F-01, F-02 |
| `crates/phairdlna-core/src/catalogue/` | The catalogue — the model of the items and containers, the scanner of a root, the opaque identifiers, the identity key of a file, the playlists (`playlist.rs`), the subtitle sidecars (`subtitle.rs`), the memory of the volumes of the roots, the assembly of the tree, the sniffer of a file, the registry of the formats the server serves and the index of the catalogue on disk (`index/`) | F-05, F-08, F-09, F-11 |
| `crates/phairdlna-core/src/media/` | The pictures of the library — the discovery of the artwork of a media file (`artwork/`), the pictures that are embedded in it (`id3/`, `mp4/`), the images that lie next to it (`sidecar.rs`), the thumbnail of a photo (`thumbnail.rs`) and the disposable cache of the pictures that were produced (`cache.rs`) | F-10 |
| `crates/phairdlna-core/src/profile.rs` | Client compatibility profiles, declarative quirk mitigation and evidence gate | F-07 |
| `crates/phairdlna-core/tests/client_profiles.rs` | Acceptance tests for client compatibility profiles, evidence gate, and disable switch | F-07 |
| `crates/phairdlna-core/tests/device_matrix.rs` | Acceptance tests for machine-readable device trial matrix | F-07 |
| `qa/device-matrix.toml` | Machine-readable record of observed client device trials, media formats, and outcomes | F-07 |
| `crates/phairdlna-core/src/didl/` | The DIDL-Lite document — the object model, the `protocolInfo`, the value rules, the policy of the document and the writer | F-05, F-11 |
| `crates/phairdlna-core/src/http/` | The delivery of media over HTTP — the `Range` header, the headers of a media answer, the containment of a media URL and the evaluator of a request | F-06, F-11 |
| `crates/phairdlna-core/src/upnp/` | The device description, the service descriptions (SCPD), the SOAP dispatcher and the ContentDirectory and ConnectionManager actions | F-02, F-03, F-04 |
| `crates/phairdlna-core/src/config/` | The typed, validated, versioned configuration | F-12 |
| `crates/phairdlna-core/src/scheduler/` | The one scheduler of the server — the priority classes, the bounds of the workers and of the queues, the hysteresis of the pressure and the report of the resources | F-18 |
| `crates/phairdlna-cli/src/lib.rs` | The commands, as pure functions over their arguments | F-13 |
| `crates/phairdlna-cli/src/describe.rs` | Device description and SSDP header inspection command offline (`OPS-005`, `INT-015`) | F-01, F-02, F-13 |
| `crates/phairdlna-cli/src/interfaces.rs` | Interfaces listing and exposure evaluation command (`OPS-004`) | F-12, F-13 |
| `crates/phairdlna-cli/tests/cli_interfaces.rs` | Acceptance tests for network interfaces inspection command | F-12, F-13 |
| `crates/phairdlna-cli/tests/cli_diagnostics.rs` | Acceptance tests for offline device description and SSDP header blocks | F-01, F-13 |
| `crates/phairdlna-cli/src/ssdp_cmd.rs` | SSDP CLI command handlers: headers inspection and stored M-SEARCH replay diagnostics | F-01, F-07 |
| `crates/phairdlna-cli/src/setup.rs`, `crates/phairdlna-cli/src/setup/` | First-run orchestration, prompts, configuration validation and writing, plus user-facing setup guidance | F-13 |
| `crates/phairdlna-core/src/catalogue/index/` | The index of the catalogue on disk — the schema of the database, the writing of a catalogue, the reading of it back and the recovery from a damaged file | F-09 |
| `crates/phairdlna-core/tests/catalogue_rebuild.rs` | The index on disk — the round trip, the catalogue during a rescan, the restart and the file that belongs to the user alone | F-09 |
| `crates/phairdlna-core/tests/catalogue_recovery.rs` | The recovery of the index — the damaged file, the file of another schema and the rebuild | F-09 |
| `crates/phairdlna-cli/tests/catalogue_index.rs` | The index of the running daemon — the write, the restart and the damaged index | F-09 |
| `crates/phairdlna-cli/src/catalogue.rs` | The catalogue commands — the tree of the library and the document a client receives | F-08, F-09, F-13 |
| `crates/phairdlna-cli/src/serve/` | The daemon — the start of the server, the HTTP routes, the discovery socket, the library and the watch of the drives, the media of the library, the body of a media answer, the cache of the pictures, the log, the signals and the transfers that run (`transfers.rs`) | F-01, F-02, F-03, F-04, F-06, F-08, F-10, F-13 |
| `crates/phairdlna-cli/src/serve/limit.rs`, `crates/phairdlna-cli/src/serve/limit/tests.rs` | Shared connection-cap enforcement and per-read/per-write socket deadlines, with focused connection-budget tests | F-13, F-16 |
| `crates/phairdlna-cli/src/serve/guard.rs`, `crates/phairdlna-cli/src/serve/guard/tests.rs` | Request admission rules and their private-helper tests: client access, host, framing and bounded transfers | F-13, F-16 |
| `crates/phairdlna-cli/src/serve/setup.rs`, `crates/phairdlna-cli/src/serve/setup/` | The daemon startup facade and its configuration, socket, HTTP, discovery and ordered-startup modules | F-13, F-14 |
| `crates/phairdlna-cli/src/serve/admin.rs` | The wiring of the administration surface — the view of the daemon that the surface asks | F-14 |
| `crates/phairdlna-cli/src/serve/lifecycle.rs` | The daemon lifecycle controller and local timeline event recorder | F-13 |
| `crates/phairdlna-cli/src/serve/resources.rs` | The scheduler of the daemon — the classes of the work, the hints about the machine and the place of a transfer | F-18 |
| `crates/phairdlna-cli/src/serve/index.rs` | The index of the catalogue in the daemon — the opening of the file, the catalogue that is read out of it and the warnings that belong to the server | F-09 |
| `crates/phairdlna-cli/src/service/` | The background service commands (install, uninstall, start, stop, status), layout, managers, and systemd readiness notification | F-13 |
| `crates/phairdlna-cli/src/main.rs` | The argument parsing and the exit code | F-13 |
| `crates/phairdlna-core/tests/media_roots.rs` | The acceptance criteria of F-08 for one root | F-08 |
| `crates/phairdlna-core/tests/media_roots_multiple.rs` | The acceptance criteria of F-08 for more than one root | F-08 |
| `crates/phairdlna-core/tests/support/` | The fixtures of the tests of the core — the scripted platform of the volumes, the checker of a document and the catalogue of the protocol tests | F-08 |
| `crates/phairdlna-core/tests/playlists.rs` | The acceptance criteria of F-11 for the playlists — the formats, the order, the containment and the broken playlist | F-11 |
| `crates/phairdlna-core/tests/subtitles.rs` | The acceptance criteria of F-11 for the subtitle sidecars — the pairing, the additional resource and the reachability of the file | F-11 |
| `crates/phairdlna-core/src/upnp/xml_reader.rs`, `crates/phairdlna-core/src/upnp/xml_syntax.rs` | The bounded `quick-xml` SOAP parser adapter, entity and namespace policy | F-02, F-16 |
| `crates/phairdlna-core/tests/xml_reader.rs`, `crates/phairdlna-core/tests/xml_reader/truncation.rs` | Accepted XML semantics, resource limits, hostile references and watchdog-bounded truncation recovery | F-16 |
| `crates/phairdlna-core/tests/catalogue_identity.rs` | The acceptance criteria of F-11 for the organisation — the identity key, the de-duplication of two roots and the exclusion of a folder | F-11 |
| `crates/phairdlna-core/tests/volume_events.rs` | The acceptance criteria of F-08 for the volumes of a root — a drive that goes, comes back and comes back under another identity | F-08 |
| `crates/phairdlna-core/tests/volume_platform_events.rs` | The acceptance criteria of F-08 for the events the platform reports — a volume that was unmounted and one that was mounted again | F-08 |
| `crates/phairdlna-cli/tests/serve.rs` | The daemon that answers a client — the description, the service descriptions, the SOAP control, the media and the artwork | F-02, F-03, F-04, F-06 |
| `crates/phairdlna-cli/tests/subtitles.rs` | The daemon that serves a subtitle — the document that names it and the answer that carries it | F-11 |
| `crates/phairdlna-core/tests/artwork.rs` | The acceptance criteria of F-10 — the embedded cover, the sidecar, the thumbnail and the files that are broken | F-10 |
| `crates/phairdlna-cli/tests/artwork.rs` | The artwork of the daemon — the route that answers a request for a picture | F-10 |
| `crates/phairdlna-cli/tests/discovery.rs` | The discovery of the daemon — the advertisements, the answer of a search and the withdrawal | F-01 |
| `crates/phairdlna-cli/tests/serve_command.rs` | The `serve` command of the command line and the graceful shutdown | F-13 |
| `crates/phairdlna-cli/tests/service_notify.rs` | Acceptance checks for systemd notify readiness and shutdown reporting | F-13 |
| `crates/phairdlna-cli/tests/support/` | The fixtures of the tests of the daemon — the daemon in a thread and the small HTTP client | F-13 |
| `crates/phairdlna-core/tests/scheduler_priority.rs` | The acceptance criteria of F-18 — the order of the classes and the isolation of a slow client | F-18 |
| `crates/phairdlna-core/tests/scheduler_hysteresis.rs` | The acceptance criteria of F-18 — the hysteresis of the background work, with a synthetic clock | F-18 |
| `crates/phairdlna-core/tests/resource_exhaustion.rs` | The acceptance criteria of F-18 — the bounds of the queues and the refusal with a clear error | F-18 |
| `crates/phairdlna-cli/tests/resources.rs` | The resources of the daemon — the order of the work, the report and the place of a transfer | F-18 |
| `crates/phairdlna-admin/src/lib.rs` | The optional administration surface — the crate root and the public API | F-14 |
| `crates/phairdlna-admin/src/token.rs`, `crates/phairdlna-admin/src/token/` | Token generation, constant-time comparison, private token-file handling, typed errors and focused tests | F-14 |
| `crates/phairdlna-admin/src/guard.rs` | The rules every request of the surface must pass — host, framing, origin, token, custom header | F-14 |
| `crates/phairdlna-admin/src/guard/host.rs` | Host-header parsing and loopback-only validation for the administration surface | F-14 |
| `crates/phairdlna-admin/src/guard/request_checks.rs` | Request framing, browser-origin, fetch metadata, token, custom-header and CSRF checks | F-14 |
| `crates/phairdlna-admin/src/routes.rs` | The authenticated local API routes and their JSON answers; no browser UI | F-14 |
| `crates/phairdlna-admin/src/routes/timeout.rs` | The tower-http handler-future timeout and its delayed-handler regression | F-14, F-16 |
| `crates/phairdlna-admin/src/routes/status.rs` | The JSON status and shared-folder representation, including advertised device identity | F-14 |
| `crates/phairdlna-admin/tests/admin_api.rs` | The local API settings explanation and the refusal of browser document routes | F-14 |
| `crates/phairdlna-admin/src/backend.rs` | What the surface asks the server, and the answers it understands | F-14 |
| `crates/phairdlna-admin/src/json.rs` | The JSON writer of the surface | F-14 |
| `crates/phairdlna-admin/src/server.rs`, `crates/phairdlna-admin/src/listener.rs` | The local API listeners, shared connection cap, per-read/per-write silence deadlines and shutdown | F-14, F-16 |
| `crates/phairdlna-admin/tests/admin_surface.rs` | The acceptance criteria of F-14 — the answers and the refusals | F-14 |
| `crates/phairdlna-admin/tests/admin_surface/security.rs`, `crates/phairdlna-admin/tests/admin_surface/api.rs`, `crates/phairdlna-admin/tests/admin_surface/privacy.rs` | Security refusals, API behavior, diagnostics privacy and token-file permissions for F-14 | F-14 |
| `crates/phairdlna-admin/tests/admin_surface/timeout.rs` | Loopback proof that an incomplete HTTP header is closed at its configured read deadline | F-14, F-16 |
| `crates/phairdlna-cli/tests/admin.rs` | The administration surface of a running daemon | F-14 |
| `crates/phairdlna-mac/src/main.rs` | The primary graphical macOS app entry point — command-line options, localized startup and the event loop | F-15 |
| `crates/phairdlna-mac/src/lib.rs` | The native GUI client crate root and its portable app logic; the server remains independently usable headlessly | F-15 |
| `crates/phairdlna-mac/src/model.rs` | The state of the server as the application understands it, and the mode of the view | F-15 |
| `crates/phairdlna-mac/src/onboarding.rs` | Pure first-run configuration decisions and recovery-notice selection | F-15 |
| `crates/phairdlna-mac/src/overview.rs` | Localized state-dependent content for the native server overview | F-15 |
| `crates/phairdlna-mac/src/menu.rs`, `crates/phairdlna-mac/src/menu/sections.rs` | Menu state and the folder, playback and action groups | F-15 |
| `crates/phairdlna-mac/src/app.rs`, `crates/phairdlna-mac/src/app/state.rs`, `crates/phairdlna-mac/src/app/actions.rs`, `crates/phairdlna-mac/src/app/folders.rs` | App state and wiring, server-state transitions, menu-action handling, and read-only shared-folder actions | F-15 |
| `crates/phairdlna-mac/src/actions.rs` | What the entries of the menu do | F-15 |
| `crates/phairdlna-mac/src/worker.rs` | The thread that talks to the administration surface | F-15 |
| `crates/phairdlna-mac/src/preferences.rs` | App-owned preferences: presentation mode and text size, stored separately from server configuration | F-15 |
| `crates/phairdlna-mac/src/ipc/mod.rs` | The client of the administration surface — the crate root of the module | F-15 |
| `crates/phairdlna-mac/src/ipc/client.rs`, `crates/phairdlna-mac/src/ipc/client/` | The local API client, error and token handling, and its test cases | F-15 |
| `crates/phairdlna-mac/src/ipc/http.rs`, `crates/phairdlna-mac/src/ipc/http/transport.rs` | HTTP message parsing and the bounded loopback socket transport | F-15 |
| `crates/phairdlna-mac/src/ipc/json.rs` | The shared JSON value and error types used to interpret API replies | F-15 |
| `crates/phairdlna-mac/src/ipc/snapshots.rs` | What the answers of the surface become | F-15 |
| `crates/phairdlna-mac/src/text/strings.rs` | The texts of the user interface, one resource file per language | F-15 |
| `crates/phairdlna-mac/src/text/strings/resource.rs` | Embedded translation resources, locale matching and resource parsing | F-15 |
| `crates/phairdlna-mac/src/text/strings/format.rs` | Named placeholder replacement for translated sentences | F-15 |
| `crates/phairdlna-mac/src/platform/` | The AppKit adapter: status menu, native overview and preferences windows (`desktop/windows.rs`), runtime bridge and workspace; server lifecycle controls are not integrated while OD-14/OD-15 remain open | F-15 |
| `crates/phairdlna-mac/src/platform/service_control.rs` | The mechanism that starts and stops the job `org.phairdlna.server` with `launchctl`; the mechanism is built, but no user-facing control calls it while OD-14/OD-15 remain open | F-15 |
| `crates/phairdlna-mac/tests/service_control.rs` | Acceptance checks for the commands of `launchctl`, the refusal of the service manager and the absence of any privilege in the control | F-15 |
| `crates/phairdlna-mac/src/platform/desktop/windows.rs` | Main and preferences window lifecycle, inline initial-configuration guidance, and diagnostics save-panel flow | F-15 |
| `crates/phairdlna-mac/src/platform/main_window.rs` | The native overview window lifecycle and controls | F-15 |
| `crates/phairdlna-mac/src/platform/main_window/layout/` | Responsive card-based dashboard layout, grid calculation (`grid.rs`), card placement (`placement.rs`) and dynamic text reflow (`reflow.rs`) | F-15, UX-020 |
| `crates/phairdlna-mac/src/app/settings.rs` | Direct server settings management (device name, HTTP port) and atomic persistence | F-12, F-15 |
| `crates/phairdlna-mac/src/platform/main_menu.rs` | The native application main menu bar at top-left (App, File, Window menus) | F-15 |
| `crates/phairdlna-mac/src/platform/main_window/layout.rs`, `crates/phairdlna-mac/src/platform/main_window/layout/reflow.rs`, `crates/phairdlna-mac/src/platform/main_window/layout/controls.rs` | Overview controls, interactive server steering, localized accessibility names and responsive vertical reflow | F-15 |
| `crates/phairdlna-mac/src/platform/main_window/layout/geometry.rs` | Pure document-height calculation that centers short content and preserves scroll height for long content | F-15 |
| `crates/phairdlna-mac/src/overview.rs` | Localized state-dependent content for the native server overview | F-15 |
| `crates/phairdlna-mac/tests/overview.rs` | Acceptance checks for checking, scanning, running, stopped and refused overview content | F-15 |
| `crates/phairdlna-mac/tests/accessibility_labels.rs` | Source contract for localized accessible names on dynamic overview values; VoiceOver output remains a manual check | F-15 |
| `crates/phairdlna-mac/tests/folder_actions.rs` | Negative checks that app folder actions remain disabled and cannot alter server configuration | F-15 |
| `crates/phairdlna-mac/tests/onboarding.rs` | Missing, malformed, empty and already-configured first-run decision coverage | F-15 |
| `crates/phairdlna-mac/src/platform/preferences_window/general.rs` | General preferences tab with read-only server values and shared folders | F-15 |
| `crates/phairdlna-mac/src/platform/preferences_window/roots.rs` | Read-only media-root presentation in preferences | F-15 |
| `crates/phairdlna-mac/src/platform/preferences_window/controls.rs` | Preference controls and standard text placement in preferences | F-15 |
| `crates/phairdlna-mac/src/platform/preferences_window/tabs.rs` | Resizable outer viewport and tab construction for general, expert and About preferences | F-15 |
| `crates/phairdlna-mac/src/platform/preferences_window.rs`, `crates/phairdlna-mac/src/platform/about_tab.rs` | Preferences window lifecycle and About tab | F-15 |
| `crates/phairdlna-mac/src/platform/bridge/selectors.rs` | Mapping from application actions to the AppKit selectors implemented by the bridge | F-15 |
| `crates/phairdlna-mac/src/platform/bridge/menu_items.rs` | Disabled status rows for non-action menu information | F-15 |
| `crates/phairdlna-mac/resources/` | Localized UI text, bundle metadata, the app-icon source artwork (`AppIcon-v2.png`) and its multi-resolution ICNS resource | F-15 |
| `crates/phairdlna-mac/tests/` | The acceptance criteria of F-15 — the client, the menu, the application, the worker, the views, the bundle and the network | F-15 |
| `crates/phairdlna-mac/tests/support/mod.rs`, `crates/phairdlna-mac/tests/support/locale_contract.rs` | The reader of application source strings and the canonical locale-key/literal allowlists used by the F-15 locale checks | F-15 |
| `crates/phairdlna-mac/tests/strings/recovery_copy.rs` | Checks translated first-run and stopped-server recovery text against unavailable installer and app-control claims | F-15 |
| `qa/macos-usability-protocol.md` | Manual setup, keyboard, VoiceOver, text-size and read-only configuration acceptance protocol | F-15 |
| `qa/evidence/macos-27.2.1-overview-offline-before-fix.png` | Isolated preview screenshot documenting the offline overview issues observed before their source correction | F-15 |
| `qa/macos-service-access-protocol.md` | Real-Mac experiment for service identity, selected-folder scope, revocation and Local Network behavior | OD-15, F-22 AC-14 |
| `scripts/` | Repository gates and the requirement-view generator | F-19 |
| `scripts/package-macos-app.sh` | Builds the macOS app bundle, validates its eight-byte PkgInfo marker and architecture slices, applies its final ad-hoc signature, and verifies the signature | F-15, F-19 |
| `scripts/package-linux-deb.sh` | Packages standard unconfined Debian archive (`.deb`) with systemd service unit (PLT-016, M3.3) | F-13, F-20 |
| `scripts/soak.sh` | Automated soak and stability testing harness monitoring RSS, open descriptors, and thread counts | F-18 |
| `scripts/generate-release-artefacts.sh` | Builds the distribution tarball, generates SHA-256 checksums and software bill of materials (SBOM) | F-19, F-22 |
| `.github/workflows/` | Manual-only continuous integration | F-19 |
| `docs/` | The documentation tree | F-19 |
| `docs/99-archive/2026-11-19-dependency-license-review/` | Immutable copy and manifest of the prior approved dependency/licence review | F-19 |
| `docs/99-archive/2026-11-19-roadmap-v1.1/` | Immutable v1.1 roadmap snapshot, superseded by the DEC-075-aligned current plan | Project tracking |
| `docs/99-archive/2026-11-19-quality-gates-v1.0/` | Immutable approved v1.0 quality-gates baseline, preserved while QG-11 is expanded | F-19 |
| `docs/99-archive/2026-11-19-ci-policy-v1.0/` | Immutable approved v1.0 CI policy, preserved while current self-hosted runner coverage is recorded | F-19 |
| `docs/07-project/OPEN-DECISIONS.md` | Material owner decisions, including OD-18 for the ADR-0002 implementation discrepancy | F-19 |

> **A path moves into this table the day it exists.** Until then it belongs in the
> planned table below.

### Planned layout

The remaining paths below are planned and do not yet exist. They show where future work
belongs.

| Planned path | Responsibility | Feature |
| --- | --- | --- |
| `crates/phairdlna-core/src/ssdp/interfaces.rs` | The selection of the sending interface and the rate limiter of the answers | F-01 |
| `crates/phairdlna-core/src/platform/` | Platform traits and per-OS implementations | F-08, F-13, F-15 |
| `crates/phairdlna-core/src/lifecycle/` | Server states, sleep/wake and network recovery | F-13 |
| `crates/phairdlna-core/src/logging/` | Structured logging and redaction | F-13, F-16 |
| `crates/phairdlna-core/src/error.rs` | The crate error type | — |
| `crates/phairdlna-mac/src/platform/verify.rs` | The verification step that asks whether a client has found the server | F-15 |

### The scripts

| Script | Gate | State | What it does |
| --- | --- | --- | --- |
| `scripts/check-file-size.sh` | QG-13 | **exists and runs** | Enforces the file-length budget |
| `scripts/check-doc-links.sh` | QG-11 | **exists and runs** | Resolves relative documentation links and current source paths in this index |
| `scripts/check-no-unsafe.sh` | QG-03 | **exists and runs** | Finds `unsafe` outside of comments; a file that carries the marker `gate: unsafe-exempt ADR-0013` is exempt, and only when the record exists |
| `scripts/generate_requirement_views.py` | CI step "Requirement views are current" | **exists and runs** | Generates the requirement index and the traceability matrix |
| `scripts/check-crate-boundaries.sh` | QG-28 | **exists and runs** | Fails when a crate depends "downwards" (ADR-0001, amended by ADR-0012) |
| `scripts/soak.sh` | QG-22 | planned | Runs the seven-day soak and samples the signals |
| `scripts/device-matrix.sh` | QG-23 | planned | Records a device test in the interoperability matrix |
