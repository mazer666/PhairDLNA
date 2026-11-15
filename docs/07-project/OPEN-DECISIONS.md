# Open decisions requiring owner approval — PhairDLNA

Status: **Open** · Owner role: Project Owner · Compiled: 2026-11-14
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
| OD-05 | Minimum macOS version | macOS 15 and later | macOS 15 |
| OD-06 | Prebuilt binary or source only | Source always, plus a signed macOS binary | The same |
| OD-07 | The mandatory device matrix | The ten priority client families; three clients before stable | The ten families |
| OD-08 | Local administration surface | Yes, minimal, alongside the native app | Yes, minimal |
| OD-09 | The v1 format commitment | MP4 (H.264/AAC), MP3, JPEG, Matroska/WebM; SRT and WebVTT sidecars | The same list |
| OD-10 | The supported locales | English and German; strings prepared for translation | English and German |
| OD-11 | The reference hardware | The profile in the performance budgets; the machine is named before the first measurement | The same profile |
| OD-12 | Outside contributions | Accepted, with a requirement ID, a test and documentation | Accepted |
| OD-13 | The Apple media libraries | Own folders only; the libraries stay opt-in and unbuilt | Option A |

**The only decision that blocks work is OD-05** (the minimum macOS version), because
the service model and the first-run flow depend on it. Everything else can be
answered while M1 is already running.

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

**Consideration:** the **local-network privacy prompt** (macOS 15 and later) changes
how the operating system treats an SSDP multicast socket. It is also the reason the
always-on server must run as a launchd **daemon** rather than an agent
(`PLT-013`). Supporting older versions means a different permission story and a
different first-run experience.

**Recommendation:** **macOS 15 and later**, with the local-network permission handled
explicitly and the daemon model used throughout. The research is unambiguous here:
the local-network permission exists only on macOS 15 and later, so on an older system
the server cannot know whether a permission prompt appeared — which makes the
diagnostics worse, not better. One model instead of two.

**Default, if no decision is made:** macOS 15 and later, with the launchd daemon model throughout.

**If deferred:** the macOS service design and the first-run flow cannot be finalised.

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

**Question:** is a local web administration surface part of v1 alongside the macOS
application?

**Recommendation:** **yes, but minimal** — the surface exists for diagnostics and
automation (`OPS-020` … `OPS-027`), while the native application is the primary
interface. Both talk to the same core.

**Default, if no decision is made:** a minimal local administration surface exists from M2.2, alongside the native application. Both talk to the same core.

**If deferred:** F-14 shrinks; the CLI must carry the whole administrative load.

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

## How to close a decision

1. State the decision in `../00-governance/DECISION-LOG.md` with its ID, date and the
   authority that took it.
2. Add or adjust the affected requirement IDs in
   `../01-product/TECHNICAL-REQUIREMENTS.md`.
3. Write an ADR if the decision constrains the architecture.
4. Update the affected feature specification and this file (mark it resolved).
5. Regenerate the requirement index and the traceability matrix.
