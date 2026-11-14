# F-MENUBAR — Native macOS menu-bar application

Status: Draft, subject to UX prototype and owner approval.

## Product intent
A quiet, accessible macOS utility: everyday use requires no terminal and minimal navigation. Simple and Advanced are two presentations of the same engine and validated settings.

## Simple experience
- Menu bar icon conveys Ready, Scanning, Degraded, Stopped and Error without relying on color alone.
- Primary popover: clear server status, shared libraries, scan progress, last known clients, start/stop, add folder, troubleshooting and Open Settings.
- First-run assistant: select folders -> review LAN visibility and permissions -> start server -> show discovery instructions/test.
- Plain-language errors: what happened, whether media is affected, what user can do next. Technical details on demand.
- No marketing, upsell, account sign-in, notifications spam or intrusive dialogs.

## Advanced experience
- Settings categories: General, Libraries, Network, Devices & Compatibility, Playback & Resource Limits, Security & Privacy, Diagnostics.
- Searchable settings, descriptions, defaults, effective values, validated edits, safe reset and export/import.
- Compatibility Inspector and Playback Doctor integrated as task-based tools, not a cluttered dashboard.
- Show impact and confirmation before network exposure, removal of shares or other potentially disruptive changes.
- Switching modes must never restart service, drop playback, alter server config or lose unsaved settings.

## Requirements
MB-001 Server can run headless independent of UI. UI reconnects and reflects actual state rather than cached assumptions.
MB-002 UI actions have visible acknowledgment, progress and meaningful errors; never block UI thread with scanning, metadata or network I/O.
MB-003 Keyboard navigation, VoiceOver labels, scalable text, contrast and reduced-motion preferences supported.
MB-004 Status and errors are localized-ready; default wording avoids SSDP/UPnP jargon.
MB-005 Opening the popover or settings must not start indexing or network scans unnecessarily.
MB-006 Only local admin communication by default; explicit authentication/authorization model and threat assessment.
MB-007 Menu bar icon and popover remain understandable in light/dark mode, small displays and when menu bar space is constrained.
MB-008 Onboarding includes optional sample verification flow for TV discovery and playback; no assumption of a connected TV.
MB-009 Safe uninstall/stop behavior and clear autostart setting; no hidden persistent helper.
MB-010 Simple and Advanced persist user preference, share a single settings schema and preserve warnings in both modes.

## Acceptance
- >=90% of >=10 representative novices complete folder-to-server setup within proposed 3 min on prepared reference Mac.
- 10 scripted error states: >=80% novices identify next action correctly; report task failures and confidence.
- Keyboard-only and VoiceOver audit covers first run, share changes, start/stop, diagnosis and mode switch.
- p95 <100ms for local standard UI actions on documented reference hardware excluding asynchronous I/O completion.
- 100 mode switches with active direct stream: zero interruption attributable to UI and identical effective server config.
- UI exit/relaunch while service runs: state accurate and no duplicate daemon.
- Real macOS sleep/wake, login/logout, permissions denial, disconnected volume and firewall scenarios.

## Implementation-neutral design constraints
Choose native macOS technology based on accessibility, maintenance and Rust integration. Avoid embedding server logic in UI. Document UI state model, action contracts, error taxonomy and WHERE_TO_CHANGE.
