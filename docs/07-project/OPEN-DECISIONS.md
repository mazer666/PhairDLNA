# Open decisions requiring owner approval — PhairDLNA

Status: **Open — OD-05, OD-08, OD-14, OD-16, OD-17 and OD-18 have owner direction; OD-15 remains evidence-gated; current through OD-18** · Owner role: Project Owner · Compiled: 2026-11-19
Version/date: 2.1 / 2026-11-19
Related: `DECISION-LOG.md` (decisions already taken), `../01-product/TECHNICAL-REQUIREMENTS.md`

These are the decisions that materially affect **product scope, architecture,
security, licensing or operation**. Everything else has been decided by the team and
recorded as an ADR.

Each entry states the question, the options, a **recommendation with its reason**, and
what happens if the decision is deferred.

## The recommendations at a glance

If you read nothing else, use this table to see the recommendations and the effect
of deferring each decision. A recommendation is planning guidance, not owner approval
for a material architecture or product choice. Work that does not depend on an open
decision can continue; the dependent feature remains gated until its decision and
required evidence are complete.

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
| OD-14 | macOS app integration with the system service | **Decided by DEC-079:** app-managed `SMAppService` registration for the bundled service; keep CLI installation available | The helper type follows OD-15; runtime approval and recovery still need verification |
| OD-15 | macOS service privilege and selected-folder access | **Owner direction by DEC-079:** test a per-user `LaunchAgent` first; revise PLT-013 and ship only if selected access, sibling denial, revocation and Local Network behavior pass independent security review | Do not implement lifecycle controls until the real-Mac experiment and review pass |
| OD-16 | How the app bootstraps, saves and activates server settings | **Decided by DEC-079:** app creates the initial typed configuration locally; later changes use authenticated, validated, atomic API updates and a controlled restart | Keep app writes disabled until OD-15 establishes a safe service path and acceptance/recovery behavior is implemented |
| OD-17 | How users enlarge text in the macOS app | App-level text-size control with reflowing layouts; macOS does not support Dynamic Type | **Decided: option A by DEC-077** |
| OD-18 | Reconcile accepted XML/HTTP dependency targets with the current implementation | **Decided: option A by DEC-078** — implement the accepted targets | M-02 stays open until implementation, security review and required tests pass |

**OD-05** (the minimum macOS version) was decided on 2026-11-15 (macOS 15 and later).
**OD-14 and OD-16** have owner direction in **DEC-079**. OD-14 selects app-managed
`SMAppService` registration while retaining CLI lifecycle support. OD-16 selects local
first-config creation, followed by authenticated API updates and a controlled restart.
**OD-15** remains evidence-gated: first test a per-user service against selected-folder access,
sibling denial, revocation and Local Network behavior; only a passing independent review permits
the PLT-013 change and lifecycle implementation. Status and
other read-only M2.6 work can continue while these decisions are open.
**OD-17** is decided by **DEC-077**: the app has its own larger-text setting and layouts
must reflow through 200%. Implementation and real-Mac acceptance remain open.
**OD-18** was decided on 2026-11-19 by **DEC-078**: preserve ADR-0002 and implement
`quick-xml` and `tower-http`. The XML adapter and handler timeout are implemented and
their targeted tests pass. The accepted tracing role still needs a redaction-safe design
and review. M-02 remains open until that role, the dependency review, full parser-policy
evidence, socket-deadline preservation and independent security review are complete.

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

## OD-14 — How the macOS application integrates with the system service — direction decided

**Question:** if the approved system-daemon model remains, should the regular macOS
application register and control its bundled helper, or should installation and
lifecycle remain CLI-only?

| Option | Effect |
| --- | --- |
| **A. App-managed registration with `SMAppService`** | If OD-15 confirms a `LaunchDaemon`, the `.app` bundle carries its helper and property list. The user explicitly starts registration; macOS requires administrator approval in System Settings. The app shows `requiresApproval` and links to that panel. The CLI remains available for headless installation. |
| **B. CLI-only service lifecycle** | Keep the app as a status and configuration client. Users install, start and stop the system daemon with documented CLI commands. This avoids app-led privileged registration, but leaves a terminal step in the normal-user setup flow. |

**Owner direction (2026-11-19, DEC-079):** select **A**, app-managed registration through `SMAppService`, and retain the CLI for headless lifecycle use. The bundled helper type follows OD-15; the current preferred experiment candidate is a per-user `LaunchAgent`. Registration must expose authorization/status and recovery clearly. This direction does not establish folder access or approve shipping a helper before OD-15's evidence passes.

**If deferred:** the native overview and work that does not change the service lifecycle can continue; app start/stop controls and end-to-end first-run setup stay open. Evidence: `docs/09-research/platform-integration.md` § 4.3 and F-15 § 12.

**Current safety state (2026-11-19):** the app's `launchctl` Start/Stop prototype was removed.
No menu item or app action starts or stops a service while OD-15's service identity and access
model remain unverified.
Start/stop actions have no `MenuAction` variants or AppKit selectors; tags 9 and 10 remain unmapped
and are covered by the action-tag test. The former `service_control` module no longer exists. This
intentionally leaves UX-019 open.

---

## OD-15 — macOS service privilege and access to selected folders — evidence open

**Question:** should the macOS server remain a system-context `LaunchDaemon`, or run
as a per-user service so the operating system can enforce the user's file permissions?

**Baseline conflict:** PLT-013 specifies a system-context daemon and describes the user
interface as a per-user `LaunchAgent`, while DEC-076 requires the user-facing app to be a
regular Dock application with a main window and menu-bar status item. DEC-079 directs the
team to evaluate a separate, per-user server `LaunchAgent`; it does not treat the GUI app
itself as that agent. Do not package this helper or revise PLT-013 until the access and
Local Network experiment passes independent review.

| Option | Effect |
| --- | --- |
| **A. Keep a system-domain `LaunchDaemon` without elevated media access** | PLT-013 remains in force, but the service identity must satisfy OPS-015 and PLT-018. Verify the effective UID, Local Network behavior, selected protected-folder read, denial for an unselected sibling and revocation on a real Mac. A system-domain job must not be assumed to run safely as root or to inherit the UI's folder selection. |
| **B. Change to a per-user `LaunchAgent`** | The server runs with the user's file permissions and can receive user-session privacy approval. It stops at logout and may need explicit Local Network approval. This requires an owner-approved change to `PLT-013`, confirmation that OPS-015 and PLT-018 remain satisfied, and an updated service lifecycle. |

**Owner direction (2026-11-19, DEC-079):** evaluate **B**, a per-user `LaunchAgent`, first because it avoids elevated service identity and runs in the signed-in user's context. This is a preferred experiment candidate, not approval to change PLT-013 or ship the service. It must still prove selected-folder read, denial for an unselected sibling, denial after revocation, ordinary-user identity, Local Network behavior and login/logout lifecycle in the actual service process, followed by independent security review. If it fails, do not fall back to A automatically; return the conflict for a new owner decision. Apple's Local Network guidance identifies responsible-code and signature concerns; it does not make a folder-picker selection proof of service-process access (`docs/09-research/platform-integration.md` § 5.4, Q3).

**Independent design review (2026-11-19):** the per-user candidate runs with the logged-in user's filesystem permissions by default; that does not itself deny access to an unselected sibling. Before building the probe, determine whether a sandbox or another demonstrated access boundary can satisfy sibling-denial and revocation, and how service-process authorization is established. Do not weaken either criterion. A later source audit found no App Sandbox entitlements in the current app bundle; Apple documents matching-signature requirements for security-scoped app bookmarks, but does not prove that this service design works. The sandboxed-helper route remains an unverified design candidate. See `../05-quality/reviews/OD15-SECURITY-REVIEW-2026-11-19.md`.

**Experiment protocol:** `qa/macos-service-access-protocol.md` records the positive selected-folder read, unselected-sibling denial, process identity, access revocation, Local Network behavior and lifecycle evidence required for each candidate. Its current results are `NOT RUN`.

**If deferred:** do not implement GUI service controls or mark macOS folder consent complete. Status UI, API-bound configuration requirements and other work independent of process privilege can continue.

**Current safety state (2026-11-19):** the app has no LaunchAgent control path. It still reads
the authenticated status API and displays configured folders, but it does not claim that the GUI
has permission to grant access to those folders. The selected-folder access protocol remains
`NOT RUN`.

---

## OD-16 — How native app settings are initialized, saved and applied — direction decided

**Question:** how should the native application change server settings when F-12
rejects runtime configuration reload and the service lifecycle is still gated by
OD-14/OD-15?

**Bootstrap question:** option A cannot by itself serve a first run where the
configuration file does not exist: an authenticated API update requires an
available service and an existing configuration to validate against. The owner
must decide how the initial validated configuration is created before that API
can be used. Candidates are (1) an app-mediated bootstrap with typed defaults,
atomic creation and explicit ownership/access rules; (2) a separate installer or
CLI provisioning step, which adds work outside the app-first flow; or (3) an
approved scope change that makes first-run setup unavailable until provisioning
has happened. Do not infer any candidate from option A.

| Option | Effect |
| --- | --- |
| **A. Authenticated update, validate, persist atomically, then restart the service** | The API accepts only an explicit allowlist of settings, validates the complete candidate with F-12's typed schema, writes one atomic version, and reports that restart is required. The app uses the approved service lifecycle, waits for reconnection and confirms the effective state. Invalid updates leave the prior file and running state untouched. This preserves the no-runtime-reload decision but depends on OD-14/OD-15. |
| **B. Add bounded live reconfiguration** | The API validates and applies an explicitly limited subset of settings without restarting. This changes ADR-0004 and F-12, and requires defined rollback, conflict, persistence and recovery behavior for every supported setting. |
| **C. Keep server settings read-only in the app** | Users continue to edit/import configuration outside the app. This avoids a new API write path but does not complete normal-user folder management or the app-first setup flow; choosing it requires an owner-approved scope change to F-14/F-15. |

**Owner direction (2026-11-19, DEC-079):** choose **A** for ongoing changes, preceded by bootstrap candidate **(1)**: the app creates the initial typed configuration locally with restrictive ownership/permissions and an atomic write. Once the service is available, app changes use an explicit authenticated API allowlist, full-candidate validation, atomic persistence and a controlled restart; the UI confirms when saved settings become effective. This preserves ADR-0004's no-runtime-reload decision. No folder selection is treated as authorization; OD-15 must establish the service access model first.

**Current safety state (2026-11-19):** the folder picker and app-side configuration writes have
been removed pending implementation of the DEC-079 direction and OD-15's service evidence. The main-window and menu actions remain visibly
unavailable, and the app explains that users must follow their installation instructions. Tests
cover missing, malformed and valid configuration; a direct action leaves the configuration file
byte-for-byte unchanged. The app does not yet bootstrap a missing configuration, so UX-002/AC-03
remain incomplete.

**If deferred:** do not claim app-driven settings updates as supported. The unapproved folder picker and direct-write implementation have now been removed. The main-window button is disabled and clearly labelled, the menu item is disabled with a plain-language explanation, and app actions cannot write configuration or start/stop the service. A missing config still produces an explicit recovery message, and a valid config without roots produces a notice that folder changes are unavailable. Startup constructs the media library from the configuration it loaded; `/api/catalogue/rescan` refreshes that existing library without reloading settings. UX-002/AC-03 therefore remain incomplete until bootstrap, persistence and activation are decided and implemented.

**Evidence needed to close:** specify and review the missing-config bootstrap source, defaults, file ownership and permissions, atomic write and failure recovery; prove that a fresh account can choose a folder and produce a valid configuration, and that malformed, read-only and interrupted-write cases fail safely. For A/B, also approve the API schema and allowlist, stale-version behavior, authentication/CSRF/custom-header enforcement and restart-failure recovery. For C, approve an F-14/F-15 scope change and verify read-only controls with no app-side file writes. For every option, an independent security review and end-to-end macOS evidence must follow OD-14/OD-15, including confirmation that the approved service consumes saved roots.

---

## OD-17 — How users enlarge text in the native macOS app

**Question:** how should the native app meet the larger-text intent of UX-008,
UX-017 and MB-003 on a platform whose Human Interface Guidelines say macOS does not
support Dynamic Type?

| Option | Effect |
| --- | --- |
| **A. Add an app-level text-size preference and reflow the interface** | Provide a small user-facing size control in app preferences and menu commands. Persist it with app preferences, never server configuration. Reflow the main window and menus; preserve all content and actions at the largest supported size. This adds implementation and accessibility review work but gives the app a direct larger-text path. |
| **B. Rely on macOS zoom and standard AppKit text** | Use standard system fonts and semantic colors, document macOS Zoom as the available system magnification tool, and revise UX-008/UX-017/MB-003 so they do not claim text-size adaptation or Dynamic Type. This keeps the interface smaller but changes the accessibility promise. |

**Recommendation:** choose **A**. The main tasks are text-heavy and the current
fixed frames and line limits can clip longer strings. An app-owned setting is the
available way to let people enlarge this interface without relying on screen magnification.
It must work across the main window, menus and supported locales; merely swapping font
sizes without reflow is insufficient.

**Owner direction (2026-11-18, DEC-077):** option A is selected. UX-008 and UX-017 were
revised in the approved technical requirements. The preference is app-owned and never
changes server configuration.

**Implementation acceptance remains:** record a manual protocol at 100%, 150% and 200%,
covering keyboard use, all overview states and the longest supported translations. An
independent accessibility review must inspect the rendered app on macOS.

---

## OD-18 — Reconcile ADR-0002 dependency targets with the current implementation — decided

**Question:** should the implementation be brought into line with the accepted
`quick-xml` and `tower-http` targets, or should ADR-0002 and DEC-019 be amended to
describe the project-owned XML parser and Tokio-based timeout middleware that are in
the current source?

**Decision:** Option A, by owner direction recorded in **DEC-078** on 2026-11-19.
ADR-0002 and DEC-019 remain in force. This resolves the architecture choice; it does
not close M-02 or claim that the migration is complete.

| Option | Effect |
| --- | --- |
| **A. Keep ADR-0002 and implement its targets** | Integrate `quick-xml` at the bounded XML-reader boundary and use `tower-http` for the accepted tracing/timeout roles. Update the lock and licence assessment, preserve the existing rejection and resource limits, and review the resulting parser path. |
| **B. Amend ADR-0002 and DEC-019 to match the current implementation** | Keep the project-owned XML parser and Tokio-based timeout middleware. An ADR amendment must explain why this is preferable, preserve the DTD/entity and size/depth/node/text bounds, and pass an independent security review before the architecture is accepted. |

**Recommendation:** prefer **A** because it follows the accepted architecture decision. This
does not authorize removing safety bounds or adding dependencies without an updated licence
assessment. The security review must include the parser's internal allocation behavior and
the behaviour of every accepted XML construct. The current parser must also reject truncated
input at every parser boundary without hanging; a static review found that EOF after an open
element previously caused `read_text` to return an empty string without advancing the cursor.
An EOF error has now been added. The isolated regression in
`crates/phairdlna-core/tests/xml_reader/truncation.rs` rejects eleven malformed/truncated EOF
inputs across element, text, declaration, comment, CDATA, attribute and closing-tag boundaries,
then verifies valid-document recovery under a 2-second watchdog; all 22 tests in that reader
suite pass against `quick-xml` 0.42.0. These cases expand boundary coverage but do not establish
exhaustive malformed-input rejection, internal allocation bounds or the broader security
properties of the parser. For the HTTP option, the review must distinguish
the handler timeout from the per-read and per-write silence deadlines: adopting `tower-http`
must preserve the current socket deadline semantics where required by `SEC-016`.

**If deferred:** keep the current code, but report the architecture discrepancy as open.
Do not claim that the current XML parser implements ADR-0002 or close M-02 on the basis of
the source files' existence alone.

**Evidence still needed to close:** the lock now contains the selected versions and targeted
parser and handler-timeout tests pass. The dependency review must assess the complete resolved
lock graph; hostile-input and internal-allocation review of `quick-xml` remains open; the
accepted `tower-http` tracing role needs a redaction-safe policy and review; and the full socket
read/write deadline regression evidence plus independent security review must be recorded.
The complete required test suite must be run before M-02 can close.

---

## How to close a decision

1. State the decision in `../00-governance/DECISION-LOG.md` with its ID, date and the
   authority that took it.
2. Add or adjust the affected requirement IDs in
   `../01-product/TECHNICAL-REQUIREMENTS.md`.
3. Write an ADR if the decision constrains the architecture.
4. Update the affected feature specification and this file (mark it resolved).
5. Regenerate the requirement index and the traceability matrix.

## Revision history

| Version/date | Change |
| --- | --- |
| 2.1 / 2026-11-19 | Records DEC-079 owner direction for app-managed service registration, the conditional per-user service experiment, and local bootstrap plus authenticated API updates with restart. |
| 2.0 / 2026-11-19 | Records the removal of unapproved app configuration writes and LaunchAgent service controls while OD-14/OD-15/OD-16 remain open. |
| 1.9 / 2026-11-19 | Records XML and handler-timeout implementation evidence and the remaining tracing, dependency and security-review work. |
| 1.8 / 2026-11-19 | Records owner direction for option A; implementation and M-02 evidence remain open. |
| 1.7 / 2026-11-19 | Records the passing EOF watchdog/recovery regression and narrows the remaining evidence for each OD-18 option. |
| 1.6 / 2026-11-19 | Adds EOF truncation handling to OD-18's parser evidence and requires preserving handler and socket deadline semantics when reconciling `tower-http`. |
| 1.5 / 2026-11-19 | Clarifies that recommendations do not approve material choices and that dependent work remains gated. |
| 1.4 / 2026-11-19 | Adds the missing-config bootstrap decision to OD-16 and records that an API update cannot initialize a fresh installation by itself. |
| 1.3 / 2026-11-19 | Adds OD-18 to track the accepted ADR-0002 dependency targets that differ from the current XML parser and service implementation. |
| 1.2 / 2026-11-18 | Defines the real-Mac evidence protocol for OD-15 and records the experiment-only probe as a prerequisite. |
| 1.1 / 2026-11-18 | Records that first-run folder selections now use the shared persistence path, surface write failures and explain that saved roots need a server restart. |
