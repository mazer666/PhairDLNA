# Roadmap and milestone plan — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Project Manager
Requirements: `../01-product/REQUIREMENTS-INDEX.md` (milestone column)
Traceability: `TRACEABILITY-MATRIX.md` · Quality gates: `../05-quality/QUALITY-GATES.md`

---

## 1. The shape of the plan

The plan is deliberately **boring**: each milestone is small, ends in something that can
be *observed*, and has an exit criterion that a reviewer can check.

Two rules govern the order:

1. **A working headless server comes first.** The first thing a user meets must be a
   server that a television can find and play from — not a beautiful app that cannot
   stream. The macOS application is the second act, not the first.
2. **Nothing is "done" before it is measured.** A milestone ends with evidence, not
   with a merged branch.

```
M0 Foundation ──▶ M1 Headless DLNA ──▶ M2 Library & UX ──▶ M3 Linux/Windows
                                        │
                                        └─▶ M4 Interop & hardening ──▶ (stable)
                                                              │
                                                              └─▶ M5 Transcoding (optional)
```

---

## 2. M0 — Foundation

**Goal:** a repository that another engineer can pick up and trust.

| Deliverable | Requirement / gate |
| --- | --- |
| Approved requirements baseline with stable IDs | G0, `DOC-001` |
| Architecture, ADRs, threat model, dependency review | G1, `PLT-001`, `SUP-002` |
| Rust workspace with the four crates, boundary check | `PLT-001`, gate QG-28 |
| CI that can be triggered manually | `../07-project/CI-POLICY.md` |
| Full AGPL text, licence and advisory gates green | QG-08, QG-09, `SUP-001` |
| Documentation structure with no duplicate documents | `DOC-006`, `DOC-011` |

**Exit criteria:** every requirement has an ID, a priority, a feature, a milestone and
a verification method; no open contradiction in the baseline; the owner has approved
the baseline; `cargo fmt`, `clippy` and `test` are green on all three platforms; the
licence and advisory gates have been observed at least once.

**Current state:** the requirements, architecture, security, quality and traceability
parts of this milestone are complete (2026-11-14). The implementation scaffolding is
not.

---

## 3. M1 — A working headless DLNA server on macOS

**Goal:** a television finds the server, browses it and plays a file.

| Step | Content | Requirements |
| --- | --- | --- |
| M1.1 | Configuration and typed settings | `CFG-001` … `CFG-006` |
| M1.2 | Platform traits: interfaces, volumes, paths | `PLT-003`, `PLT-005`, `PLT-011` |
| M1.3 | SSDP discovery and advertisement | `INT-001` … `INT-019` |
| M1.4 | Device and service description, SCPD | `INT-020` … `INT-029`, `INT-044`, `INT-045` |
| M1.5 | SOAP dispatch and the ContentDirectory service | `INT-030` … `INT-043`, `INT-046` … `INT-049` |
| M1.6 | ConnectionManager with an honest protocolInfo | `INT-050` … `INT-056` |
| M1.7 | DIDL-Lite generation | `INT-060` … `INT-079` |
| M1.8 | HTTP media delivery, ranges and seeking | `INT-080` … `INT-099`, `FUN-040` … `FUN-046` |
| M1.9 | A minimal catalogue over one root | `FUN-001` … `FUN-007`, `FUN-008` … `FUN-011` |
| M1.10 | The security baseline | `SEC-001` … `SEC-018`, `SEC-060` … `SEC-065`, `SEC-090` … `SEC-096` |
| M1.11 | The CLI: run, status, scan, interfaces | `OPS-001` … `OPS-008`, `OPS-011` |

**Exit criteria**

- A real client discovers the server and browses a folder (device test recorded in the
  interoperability matrix — **not** a claim).
- A file plays from beginning to end, and a seek lands at the right position.
- Every conformance check `PC-01` … `PC-26` is green.
- The threat-model negative tests exist for M1.10.
- Idle memory and warm start are measured against `PERF-001` and `PERF-002` on the
  reference hardware.
- The server runs for seven days without a leak (the soak test may start in M4; the
  memory and descriptor trend is already flat in M1).

**Explicitly not in M1:** the menu bar app, the administration surface, transcoding,
multiple roots, artwork, playlists.

---

## 4. M2 — Library, configuration, diagnostics and the native app

**Goal:** the product becomes pleasant to live with.

| Step | Content | Requirements |
| --- | --- | --- |
| M2.1 | Multiple roots, external volumes, network shares | `FUN-017` … `FUN-019`, `PLT-011` |
| M2.2 | Metadata extraction, artwork, thumbnails | `FUN-013`, `FUN-014` |
| M2.3 | Playlists, subtitles, organisation | `FUN-015`, `FUN-016`, `FUN-020` … `FUN-022` |
| M2.4 | Full configuration, access control, resource limits | `CFG-007` … `CFG-015` |
| M2.5 | Local administration surface and diagnostics | `OPS-020` … `OPS-027`, `OPS-016`, `OPS-017`, `UX-011`, `UX-012` |
| M2.6 | The macOS menu bar application | `UX-001` … `UX-010`, `UX-013` … `UX-020`, `MB-001` … `MB-009` |
| M2.7 | Service installation and lifecycle | `LIFE-001` … `LIFE-011`, `PLT-013` |
| M2.8 | Adaptive resource management | `RES-001` … `RES-009` |

**Exit criteria**

- A novice completes the setup in the documented study protocol and the result is
  recorded (or the study is honestly reported as not run).
- The application is operable with the keyboard alone and passes a VoiceOver audit.
- The administration surface survives the DNS-rebinding and CSRF negative tests.
- The service survives sleep/wake and a network change in a recorded trial.

---

## 5. M3 — Linux and Windows

**Goal:** the same core, the same behaviour, on the other two platforms.

| Step | Content | Requirements |
| --- | --- | --- |
| M3.1 | Platform traits for Linux and Windows | `PLT-002`, `PLT-003`, `PLT-015`, `PLT-016` |
| M3.2 | Service integration: systemd, Windows service | `OPS-009` |
| M3.3 | Packaging and installation | `PLT-016`, `SUP-005` |
| M3.4 | Cross-platform behaviour parity | `PLT-010` |

**Exit criteria:** the build matrix is green on all three platforms; the same protocol
fixture suite passes unchanged; a recorded device test on at least one non-macOS
client.

---

## 6. M4 — Interoperability and hardening

**Goal:** the difference between "works" and "works everywhere".

| Step | Content | Requirements |
| --- | --- | --- |
| M4.1 | Client compatibility profiles | `INT-100` … `INT-104` |
| M4.2 | The real-device matrix | `INT-102` |
| M4.3 | Fuzzing, property tests, resource-bound tests | `SEC-060` … `SEC-072` |
| M4.4 | Soak test, crash and corruption recovery | `REL-001` … `REL-006` |
| M4.5 | Security review of the whole surface | `../04-security/SECURITY-MODEL.md` |

**Exit criteria:** the device matrix has at least one observed result per priority
client family; the fuzz targets are clean; the seven-day soak shows flat memory,
descriptor and thread counts; the independent security review has no open critical or
high finding.

---

## 7. M5 — Optional transcoding (deferred)

**Goal:** format conversion for the files a client cannot play — explicitly, never by
surprise.

**Entry condition:** the user has explicitly enabled transcoding and selected a
target format. No transcoding happens because the server "thought it would help".

**Exit criteria:** a bounded, cancellable, isolated transcoding path; a per-stream
resource budget; a documented licence position (no FFmpeg shipped; a user-supplied
external binary only); and evidence that direct play is unaffected when transcoding is
off.

---

## 8. Dependency order

```
M0 ──▶ M1.1 ──▶ M1.2 ──┬─▶ M1.3 ──▶ M1.4 ──▶ M1.5 ──▶ M1.6
                       └─▶ M1.8 ──▶ M1.9 ──▶ M1.10 ──▶ M1.11
M1 ──▶ M2.1 ──▶ M2.2 ──▶ M2.4 ──▶ M2.5 ──▶ M2.6 ──▶ M2.7 ──▶ M2.8
M1 ──▶ M3.1 ──▶ M3.2 ──▶ M3.3 ──▶ M3.4
M1 ──▶ M4.1 ──▶ M4.2 ──▶ M4.3 ──▶ M4.4 ──▶ M4.5
M4 ──▶ M5
```

The critical path to the first usable product is
**M0 → M1.1 → M1.2 → M1.3 → M1.4 → M1.5 → M1.7 → M1.8 → M1.9**.

---

## 9. What "done" means for a milestone

A milestone is complete when **all** of the following are true and recorded:

1. Every `MUST` requirement assigned to it is implemented **and** verified.
2. Every gate in `../05-quality/QUALITY-GATES.md` that applies is green, with the
   observed output archived.
3. The negative test of every applicable gate has been demonstrated.
4. The feature specifications, the requirement index, the traceability matrix and
   `WHERE_TO_CHANGE.md` are updated.
5. The device matrix contains the observed results the milestone promised.
6. The documentation a newcomer needs is present and correct.
7. The owner has approved the milestone.

**No release and no tag without explicit owner approval.**
