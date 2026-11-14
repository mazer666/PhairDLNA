# Usability and accessibility requirements

> **Non-normative.** This document is research and strategy. It defines **no**
> requirement IDs and creates no obligation. The binding requirements are in
> [`../01-product/TECHNICAL-REQUIREMENTS.md`](../01-product/TECHNICAL-REQUIREMENTS.md)
> and the cross-cutting view in
> [`../01-product/REQUIREMENTS-INDEX.md`](../01-product/REQUIREMENTS-INDEX.md).
> Where this document names an identifier that looks like a requirement ID, it is a
> **local proposal identifier**, not a canonical one.


Status: Draft — thresholds require validation.

UX-001 First run: nontechnical user can select a media folder and make it available on a supported LAN without editing config files or knowing SSDP/UPnP. Proposed acceptance: >=90% of at least 10 representative novices complete within 3 minutes on a prepared test environment; record failures and time.
UX-002 Status: server running/stopped/error, scanning progress, shared folders and last observed client activity visible from main view; tested in all states.
UX-003 Errors: human-readable cause, consequence and next action; technical details expandable; evaluate 10 common failure scenarios with novice comprehension tasks.
UX-004 Configuration: basic and advanced modes share one validated settings model; switching modes does not lose values.
UX-005 Safety: shared paths and network exposure are explicit before activation; no hidden Internet exposure.
UX-006 Accessibility: keyboard-only setup and operation, VoiceOver labels/focus order, scalable text, sufficient contrast, no color-only status; verify with macOS accessibility test plan.
UX-007 Control: user can pause/resume indexing, start/stop server and remove a share without deleting source files.
UX-008 Feedback: UI never freezes during scans; cancellation is observable; long operations report progress or indeterminate status honestly.
UX-009 Localisation: prepare strings for translation; define supported locales before v1 approval.
UX-010 Documentation: beginner can locate where to change a setting or feature via README -> GETTING-STARTED -> WHERE_TO_CHANGE within three navigation steps.

No user-study outcomes are claimed yet.
