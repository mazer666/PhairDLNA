# Open decisions requiring owner approval — PhairDLNA

Status: **Open — OD-05 and OD-08 decided; current through OD-17** · Owner role: Project Owner · Compiled: 2026-11-18
Related: `DECISION-LOG.md` (decisions already taken), `../01-product/TECHNICAL-REQUIREMENTS.md`

These are the decisions that materially affect **product scope, architecture,
security, licensing or operation**. Everything else has been decided by the team and
recorded as an ADR.

Each entry states the question, the options, a **recommendation with its reason**, and
what happens if the decision is deferred.

## The recommendations at a glance

If you read nothing else, this is the short version. Every decision has a
recommendation, and every recommendation is also the **default** — so the project
can continue without an answer, and your answer only ever changes the default, never
the momentum.

| ID | Question | Recommendation | If you do not decide |
| --- | --- | --- | --- |
| OD-01 | macOS distribution and signing | Ad-hoc signed universal binary now, Developer ID later | Option A |
| OD-02 | FFmpeg and transcoding | Never ship FFmpeg; a user-supplied binary only, out of process | No FFmpeg |
| OD-03 | macOS interface technology | Pure Rust with `objc2` + `objc2-app-kit` | Option A |
| OD-04 | Administration authentication | A `0600` bearer token plus CSRF, a custom header and a `Host` allowlist | The same model |
| OD-05 | Minimum macOS version | macOS 15 and later — **decided 2026-11-15** | macOS 15 |
| OD-06 | Prebuilt binary or source only | Source always, plus a signed macOS binary | The same |
| OD-07 | The mandatory device matrix | The ten priority client families; three clients before stable | The ten families |
| OD-08 | Local administration surface | Native app is primary; local API only, no browser UI | Decided: native app primary; authenticated local API remains |
| OD-09 | The v1 format commitment | MP4 (H.264/AAC), MP3, JPEG, Matroska/WebM; SRT and WebVTT sidecars | The same list |
| OD-10 | The supported locales | English and German; strings prepared for translation | English and German |
| OD-11 | The reference hardware | The profile in the performance budgets; the machine is named before the first measurement | The same profile |
| OD-12 | Outside contributions | Accepted, with a requirement ID, a test and documentation | Accepted |
| OD-13 | The Apple media libraries | Own folders only; the libraries stay opt-in and unbuilt | Option A |
| OD-14 | macOS app integration with the system service | Use `SMAppService` for a bundled helper; keep CLI installation available | Prefer app-managed registration if the process model is approved |
| OD-15 | macOS service privilege and selected-folder access | Satisfy OPS-015 and PLT-018 while validating PLT-013, Local Network behavior, folder scope and revocation; otherwise return the conflicting requirements to the owner | Resolve with a real-Mac security experiment and independent review before lifecycle implementation |
| OD-16 | How app changes to server settings become effective | Recommend authenticated API updates and a controlled restart, if OD-14/OD-15 approve a safe service path; preserve F-12's no-runtime-reload baseline | No app settings writes until the owner selects a model; depends on OD-14/OD-15 |
| OD-17 | How users enlarge text in the macOS app | App-level text-size control with reflowing layouts; macOS does not support Dynamic Type | Plan against A; do not implement until the owner approves the amended requirement baseline |

**OD-05** (the minimum macOS version) was the project-wide decision that blocked M2; it
was decided on 2026-11-15 (macOS 15 and later). **OD-15** gates macOS service-lifecycle
implementation until the process privilege and selected-folder access model is approved.
**OD-16** gates app-driven configuration writes until persistence, validation, and
activation semantics are agreed with F-12's no-runtime-reload baseline. Status and
other read-only M2.6 work can continue while these decisions are open.
**OD-17** resolves the mismatch between F-15's larger-text requirement and Apple's
macOS platform guidance; the current fixed-size overview is not larger-text support.

---

## OD-01 — macOS distribution and signing model

**Question:** how is the macOS build delivered to users?

| Option | Cost | Effect |
| --- | --- | --- |
| **A. Ad-hoc signed universal binary**, distributed as a release asset and through Homebrew | $0, no Apple account | Gatekeeper asks the user once ("Open Anyway") for browser downloads. A stable code identity is impossible, so the firewall may keep asking. `SUP-009` stays open |
| **B. Apple Developer Program**, Developer ID + hardened runtime + notarisation | $99/year | No Gatekeeper prompt, firewall auto-allow, stable identity. Requires a paid membership and a release process |
| **C. Source only** | $0 | The user needs a Rust toolchain. The highest-friction option, but the most transparent |

**Recommendation:** **A now, B when the project has users.** The product must not
depend on a paid membership to exist, and the honest friction of A is acceptable for
an early audience. The architecture must not prevent B later: the `ServiceSpec` must
carry the bundle identifier and the readiness notification from the start
(`LIFE-010`, `PLT-013`).

**Default, if no decision is made:** option A (ad-hoc signed universal binary). M1 is unaffected; only the macOS release experience stays undefined and `SUP-009` stays open.

**If deferred:** M1 can proceed on macOS; the macOS release experience stays undefined
and `SUP-009` cannot be closed.

---

## OD-02 — Transcoding and the FFmpeg question

**Question:** may the product ship or link a transcoder?

**Recommendation:** **No FFmpeg, ever, inside the product.** A GPL FFmpeg inside an
AGPL product makes the combined work GPLv3-governed and detaches the AGPL network
source obligation from it. The only acceptable shape is an **optional, user-supplied
external binary**, invoked out of process with an argument array, never a shell,
resource-capped and off by default (`FUN-047` … `FUN-049`).

**Default, if no decision is made:** no FFmpeg, no bundled transcoder. M5 stays deferred. Nothing in the baseline is blocked.

**If deferred:** M5 stays deferred and the baseline is unaffected. This decision does
not block anything.

---

## OD-03 — The macOS interface technology

**Question:** how is the menu bar application built?

| Option | Effect |
| --- | --- |
| **A. Pure Rust with `objc2` + `objc2-app-kit`** | No Swift, no webview, no extra toolchain. Uses the crates already assessed. The `NSStatusItem` API is reachable, but the team needs AppKit experience |
| **B. A small native Swift helper** for the interface, with the Rust core as a service | Best accessibility and platform fit; adds a second language and a build step |
| **C. A webview-based interface** | Rejected: it contradicts the "no webview, Rust-first" direction and weakens the privacy story |

**Recommendation:** **A**, with a documented fallback to B if the accessibility audit
in M2.4 shows that `objc2-app-kit` cannot deliver the required VoiceOver behaviour
(`MB-003`, `UX-017`, `UX-018`).

**Default, if no decision is made:** option A (pure Rust with `objc2` + `objc2-app-kit`). The accessibility audit in M2.4 is the gate that can send us back to option B.

**If deferred:** M2.6 cannot start, but nothing else is blocked.

---

## OD-04 — The administration authentication model

**Question:** how is the local administration surface authenticated?

**Recommendation:** a **per-installation random bearer token** in a file with mode
`0600` inside a `0700` directory, combined with a `Host` allowlist, a CSRF token and
a required custom header (`OPS-021` … `OPS-023`, `SEC-071`, `SEC-072`). This is the
model the security research supports against DNS rebinding and cross-site requests.

**Alternative considered:** no authentication at all, because the surface is loopback
only. Rejected: loopback is not a boundary (CVE-2022-26505 is the proof), and a
browser on the same machine is enough to attack it.

**Default, if no decision is made:** the bearer-token model described above. The administration surface does not ship without it; the server itself is unaffected either way.

**If deferred:** the administration surface must not ship. The server itself is
unaffected.

---

## OD-05 — The minimum supported macOS version

**Question:** which macOS versions must the first release support?

**Consideration at the time of the decision:** the **local-network privacy prompt**
(macOS 15 and later) changes how the operating system treats an SSDP multicast socket.
The then-current design used that behavior as a reason to require a launchd daemon
(`PLT-013`). Supporting older versions means a different permission story and a
different first-run experience. The service identity and folder-access implications
are now separately reviewed by OD-15.

**Recommendation at the time:** **macOS 15 and later**, with the local-network permission handled
explicitly and the then-current daemon model throughout. The research is unambiguous here:
the local-network permission exists only on macOS 15 and later, so on an older system
the server cannot know whether a permission prompt appeared — which makes the
diagnostics worse, not better. One model instead of two.

**Default at the time:** macOS 15 and later, with the then-current launchd daemon model.

**If deferred:** the macOS service design and the first-run flow cannot be finalised.

**Decision (2026-11-15, owner): macOS 15 and later (Sequoia).** The minimum-version choice is accepted.
DEC-032 also recorded the accompanying service-model rationale at that time. OD-15 now reviews whether
the current PLT-013 model can satisfy OPS-015/PLT-018 and selected-folder access; until the owner decides,
PLT-013 remains the baseline requirement. OD-15 does not reopen the minimum macOS version.

---

## OD-06 — Prebuilt binary versus source-only distribution

**Question:** does the project publish binaries at all?

**Recommendation:** publish **source always**, and a **signed macOS binary in
addition** once OD-01 is settled. Linux packages (deb, rpm) follow in M3; Windows an
MSI in M3. Every artefact carries a checksum and a software bill of materials
(`SUP-005`).

**Default, if no decision is made:** source always, plus a signed macOS binary once OD-01 is settled. The release checklist already assumes this.

**If deferred:** no impact on development; it changes the release process only.

---

## OD-07 — The mandatory device matrix

**Question:** which client families and physical devices must the compatibility
matrix cover before the first stable release?

**Recommendation:** the **ten priority families** listed in
`../05-quality/INTEROPERABILITY-MATRIX.md` § 4.1, with **at least three**
independently implemented clients before the first stable release, and a recorded
result — including failures — for each.

**Default, if no decision is made:** the ten priority families already listed in the interoperability matrix, with at least three independently implemented clients before the first stable release.

**If deferred:** the "stable" milestone cannot be defined; M4 has no exit criterion.

---

## OD-08 — The local administration surface versus the native application

**Decided 2026-11-18 (DEC-075):** The native macOS application is the primary user-facing
surface for setup, status and administration. The loopback service remains the local API used
by the application and automation; it does not serve a browser interface. The command line
remains available for headless operation and advanced use. The embedded browser UI is removed.
F-14 and F-15 are updated to reflect this decision. The API's authentication and loopback
protections remain in force.

**Question:** should a browser interface be part of v1 alongside the macOS application?

**Original recommendation (superseded by DEC-075):** keep a minimal local browser surface
for diagnostics and automation (`OPS-020` … `OPS-027`), while the native application is the
primary interface. Both talk to the same core.

**Original default (superseded):** a minimal local administration surface exists from M2.2,
alongside the native application. Both talk to the same core.

**Consequence of the rejected browser option:** automation can use the authenticated local API;
people use the native app or the CLI.

---

## OD-09 — The media format commitment for v1

**Question:** which formats and subtitle variants are contractual commitments
rather than best effort?

**Recommendation:** commit to **direct play for MP4 (H.264/AAC), MP3, JPEG and
Matroska/WebM**, and to **external subtitle sidecars in SRT and WebVTT** where the
client accepts them. Everything else is best effort and documented as such.

**Default, if no decision is made:** direct play for MP4 (H.264/AAC), MP3, JPEG and Matroska/WebM, plus SRT and WebVTT sidecars where the client accepts them. Everything else is best effort and documented as such.

**If deferred:** the acceptance criteria in F-05, F-06 and F-11 cannot be written
concretely.

---

## OD-10 — The supported locales for v1

**Question:** which languages must v1 ship?

**Recommendation:** **English and German**, with all strings prepared for translation
(`UX-020`) and no translation workflow beyond a plain string catalogue.

**Default, if no decision is made:** English and German, with all strings prepared for translation (`UX-020`).

**If deferred:** the string architecture in F-15 may need rework later.

---

## OD-11 — The performance reference hardware

**Question:** which machine defines the release targets?

**Recommendation:** the profile in `../05-quality/PERFORMANCE-BUDGETS.md` § 2, with
the **reference machine recorded by name** before the first measurement, and a
secondary profile for a NAS-backed library.

**Default, if no decision is made:** the profile in `../05-quality/PERFORMANCE-BUDGETS.md` § 2, with the reference machine named by the owner before the first measurement.

**If deferred:** no measurement can be compared against anything.

---

## OD-12 — The public repository's issue and contribution rules

**Question:** does the project accept outside contributions before the baseline is
implemented?

**Recommendation:** **yes**, with the rule that a contribution must come with a
requirement ID, a test and documentation. The feature template and the requirement
method exist for exactly this.

---

## OD-13 — The Apple media libraries

**Question:** shall the server be able to publish the user's macOS media libraries
(Music, TV, Podcasts, Books, Photos) alongside their own folders?

**The competitor:** FireStream does exactly this. It is the one place where the field is
ahead of our baseline, which is why it is written down instead of ignored.

| Option | Effect |
| --- | --- |
| **A. Own folders only** | Smallest surface, no access to personal libraries, no media-API permission. The gap to FireStream stays |
| **B. Apple libraries, opt-in per library** | Every library the user names, with the system's own consent prompt and the same metadata and artwork rules as a folder |
| **C. Apple libraries, always** | Rejected: it reaches into personal data the user never named, and it contradicts `PRD-003` |

**Recommendation:** **A now, B only if the owner asks for it.** The libraries are
personal data; opening them is a deliberate act by the user, not a default. If B is
chosen, the implementation rules are already written down: off by default, one
consent per library, the library appears as an ordinary root, and the same metadata
and artwork rules apply.

**If deferred:** nothing in the baseline is blocked. This is the one competitor gap
that we are choosing knowingly.

---

## OD-14 — How the macOS application integrates with the system daemon

**Question:** if the approved system-daemon model remains, should the regular macOS
application register and control its bundled helper, or should installation and
lifecycle remain CLI-only?

| Option | Effect |
| --- | --- |
| **A. App-managed registration with `SMAppService`** | If OD-15 confirms a `LaunchDaemon`, the `.app` bundle carries its helper and property list. The user explicitly starts registration; macOS requires administrator approval in System Settings. The app shows `requiresApproval` and links to that panel. The CLI remains available for headless installation. |
| **B. CLI-only service lifecycle** | Keep the app as a status and configuration client. Users install, start and stop the system daemon with documented CLI commands. This avoids app-led privileged registration, but leaves a terminal step in the normal-user setup flow. |

**Recommendation/default:** Prefer **A** if OD-15 confirms the system-daemon model; it avoids a terminal-only normal-user setup. This decision does not grant or establish folder access. Apple documents administrator approval and service states; PhairDLNA's folder-access behavior remains an independent security question.

**If deferred:** the native overview and work that does not change the service lifecycle can continue; app start/stop controls and end-to-end first-run setup stay open. Evidence: `docs/09-research/platform-integration.md` § 4.3 and F-15 § 12.

---

## OD-15 — macOS service privilege and access to selected folders

**Question:** should the macOS server remain a system-context `LaunchDaemon`, or run
as a per-user service so the operating system can enforce the user's file permissions?

| Option | Effect |
| --- | --- |
| **A. Keep a system-domain `LaunchDaemon` without elevated media access** | PLT-013 remains in force, but the service identity must satisfy OPS-015 and PLT-018. Verify the effective UID, Local Network behavior, selected protected-folder read, denial for an unselected sibling and revocation on a real Mac. A system-domain job must not be assumed to run safely as root or to inherit the UI's folder selection. |
| **B. Change to a per-user `LaunchAgent`** | The server runs with the user's file permissions and can receive user-session privacy approval. It stops at logout and may need explicit Local Network approval. This requires an owner-approved change to `PLT-013`, confirmation that OPS-015 and PLT-018 remain satisfied, and an updated service lifecycle. |

**Recommendation/default:** do not select A or B by assumption. A must demonstrate the effective identity and least-privilege folder access while satisfying OPS-015 and PLT-018. If it fails, B is a candidate only after its Local Network behavior is verified and the owner approves the `PLT-013` change. If neither path satisfies all three requirements, return the conflict to the owner for an explicit baseline revision. Apple documents scoped bookmarks for App Sandbox processes and per-process activation; it does not document transfer of an unsandboxed app's folder consent to a separate service process (`docs/09-research/platform-integration.md` § 5.4, Q3).

**If deferred:** do not implement GUI service controls or mark macOS folder consent complete. Status UI, API-bound configuration requirements and other work independent of process privilege can continue.

---

## OD-16 — How native app settings are saved and applied

**Question:** how should the native application change server settings when F-12
rejects runtime configuration reload and the service lifecycle is still gated by
OD-14/OD-15?

| Option | Effect |
| --- | --- |
| **A. Authenticated update, validate, persist atomically, then restart the service** | The API accepts only an explicit allowlist of settings, validates the complete candidate with F-12's typed schema, writes one atomic version, and reports that restart is required. The app uses the approved service lifecycle, waits for reconnection and confirms the effective state. Invalid updates leave the prior file and running state untouched. This preserves the no-runtime-reload decision but depends on OD-14/OD-15. |
| **B. Add bounded live reconfiguration** | The API validates and applies an explicitly limited subset of settings without restarting. This changes ADR-0004 and F-12, and requires defined rollback, conflict, persistence and recovery behavior for every supported setting. |
| **C. Keep server settings read-only in the app** | Users continue to edit/import configuration outside the app. This avoids a new API write path but does not complete normal-user folder management or the app-first setup flow; choosing it requires an owner-approved scope change to F-14/F-15. |

**Recommendation/default:** prefer **A** only after OD-14/OD-15 establish a safe service identity and restart path. It honors the existing no-runtime-reload baseline while keeping everyday setup in the native app. Until those prerequisites and owner approval are in place, the effective default is no app-side configuration writes. A UI selection must not be treated as authorization for the service to read the selected path (OD-15).

**If deferred:** do not claim app-driven settings updates as supported. The current native direct-write path remains a prototype gap and must be removed or made unavailable before M2.6 acceptance; no success message may imply that the running server adopted a change it has not loaded. Specifically, startup constructs the media library from the loaded roots, and `/api/catalogue/rescan` refreshes that existing library without reloading the config. The current wizard can therefore report `Configured` and request a scan after a failed save, or after a successful save whose new roots remain inactive in the running server.

**Evidence needed to close:** for A/B, an approved API schema and allowlist, positive/invalid/stale-version/interrupted-write criteria, authentication/CSRF/custom-header enforcement, and restart-failure recovery; for C, an approved F-14/F-15 scope change plus verified read-only controls and no app-side file writes. For every option, first-run persistence failures must be surfaced, preferences controls must match the choice, and an independent security review plus end-to-end macOS evidence must follow OD-14/OD-15.

---

## OD-17 — How users enlarge text in the native macOS app

**Question:** how should the native app meet the larger-text intent of UX-008,
UX-017 and MB-003 on a platform whose Human Interface Guidelines say macOS does not
support Dynamic Type?

| Option | Effect |
| --- | --- |
| **A. Add an app-level text-size preference and reflow the interface** | Provide a small user-facing size control in app preferences and menu commands. Persist it with app preferences, never server configuration. Reflow the main window and menus; preserve all content and actions at the largest supported size. This adds implementation and accessibility review work but gives the app a direct larger-text path. |
| **B. Rely on macOS zoom and standard AppKit text** | Use standard system fonts and semantic colors, document macOS Zoom as the available system magnification tool, and revise UX-008/UX-017/MB-003 so they do not claim text-size adaptation or Dynamic Type. This keeps the interface smaller but changes the accessibility promise. |

**Recommendation/default:** choose **A**. The main tasks are text-heavy and the current
fixed frames and line limits can clip longer strings. An app-owned setting is the
available way to let people enlarge this interface without relying on screen magnification.
It must work across the main window, menus and supported locales; merely swapping font
sizes without reflow is insufficient. A is the planning default while this decision
remains open. Requirement changes and implementation still require the owner's approval
of the amended accessibility baseline.

**If deferred:** continue planning against A, but do not implement the proposed text-size
control or claim that macOS Dynamic Type is supported. Keep larger-text acceptance open
until the owner approves the revised baseline and the rendered app passes the chosen size
and localization checks.

**Evidence needed to close:** the owner selects A or B; F-15 and the affected
requirements are revised consistently; if A is selected, record a manual protocol at
default, 150% and 200% sizes, covering keyboard use, all overview states and the longest
supported translations. An independent accessibility review must inspect the rendered
app on macOS.

---

## How to close a decision

1. State the decision in `../00-governance/DECISION-LOG.md` with its ID, date and the
   authority that took it.
2. Add or adjust the affected requirement IDs in
   `../01-product/TECHNICAL-REQUIREMENTS.md`.
3. Write an ADR if the decision constrains the architecture.
4. Update the affected feature specification and this file (mark it resolved).
5. Regenerate the requirement index and the traceability matrix.
