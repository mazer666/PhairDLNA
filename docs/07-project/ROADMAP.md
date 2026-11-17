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

**Current state (2026-11-15):** the requirements, architecture, security, quality and
traceability parts are complete. The four crates now exist with the dependency
direction enforced by gate QG-28, the licence, advisory and ban gates are green on
the real dependency set, the configuration module is implemented with its tests, and
the command line offers `version`, `config path`, `config validate` and
`config show`.

The manual CI runs on Linux are green — all 13 gate steps, recorded as
[run 43699136828](https://github.com/mazer666/PhairDLNA/actions/runs/43699136828)
and the two runs before it.

M1.1 (the configuration), M1.2 (the platform traits) and M1.3 (the discovery
protocol) are implemented: the typed, validated, versioned configuration; the eight
capability traits with the path model and the Unicode-correct identity computation;
and the portable half of SSDP — parsing a search, deciding which of the five
advertised targets answers it, and writing the exact bytes of a reply or an
advertisement. 58 tests, 0 failed.

The socket itself is not yet written; the pure half was done first, because it is the
half that decides whether the server answers at all, and it is the half that can be
tested.

**Still open in M0:** the manual CI run on macOS and Windows, and the Intel macOS
target of the build matrix.

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
| M1.9 | A minimal catalogue over one root | `FUN-001` … `FUN-007`, `FUN-011`, `FUN-012` |
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

| Step | Content | Requirements | State |
| --- | --- | --- | --- |
| M2.1 | Multiple roots, external volumes, network shares | `FUN-017` … `FUN-019`, `PLT-011` | **done (2026-11-16)** — the roots keep their own identifiers and boundaries, the catalogue remembers the volume of every root, and the daemon reads the catalogue again when a drive comes or goes (`F-08` D-15 … D-20) |
| M2.2 | Metadata extraction, artwork, thumbnails | `FUN-013`, `FUN-014` | **done (2026-11-16)** — the cover of an ID3v2 tag and of an MP4 container is read in pure Rust, the image that lies next to a file wins over the cover in the tag, the picture is served at `/art/<id>` and the document names it in `upnp:albumArtURI`; the picture pipeline parses in the server process until the sandbox child of ADR-0008 exists (ADR-0011, F-10 D-01 … D-08) |
| M2.3 | Playlists, subtitles, organisation | `FUN-015`, `FUN-016`, `FUN-020` … `FUN-022` | **done (2026-11-16)** — the M3U, M3U8 and PLS formats are parsed within bounds and their entries are resolved against the folder of the playlist and kept inside the root; a playlist is a container of the class `object.container.playlistContainer` whose children keep the order of the playlist; a subtitle that lies next to a film is paired with it while the root is read and is offered as an additional `res` of the object; the identity key is volume-scoped and the uniqueness of a file comes from the platform, so that two roots that reach one file list it once (DEC-048 … DEC-057) |
| M2.4 | Full configuration, access control, resource limits | `CFG-007` … `CFG-015` | **in part (2026-11-16)** — the access-control list is read at the start and checked against the address of the socket, and an entry that cannot be read stops the server before a socket is opened (`CFG-009`); the settings of the administration surface exist and a configuration that would put the surface on an address of the network is refused unless the user says so (`CFG-005`); the export and the import of the configuration are commands, and the import writes nothing; the policy of the symbolic links is per root (`CFG-007`), an unknown key is an error (`CFG-013`) and the token file has the permissions 600 (`CFG-014`); the trusted-client list waits for the device profiles of F-07 (`CFG-010`), the limits of the extraction time and of the index workers wait for the components they bound (`CFG-008`), and the import that writes is still open (`CFG-012`) (DEC-061, DEC-062) |
| M2.5 | Local administration surface and diagnostics | `OPS-020` … `OPS-027`, `OPS-016`, `OPS-017`, `UX-011`, `UX-012` | **in part (2026-11-16)** — the surface lives in the crate `phairdlna-admin`, listens on the loopback interface only and on a port of its own, and starts only when the configuration asks for it; every request carries the bearer token of the installation, the `Host` header must name the loopback, the custom header `X-Phair-Admin` must be there, a request that changes something carries the token of the "cross-site request forgery" defence, and no answer carries a header that would let another origin read it (`OPS-020` … `OPS-024`, `OPS-027`); the surface answers the status, the diagnostics and the scan of the library, the paths of the user are redacted, and the source revision is embedded at build time (`OPS-017`, `UX-011`, `UX-012`); the source offer is not built (`OPS-016`), the plain-language first-run text belongs to the command line (`OPS-026`), and the address of the client is not known to the routes yet (`UX-012`) (ADR-0012, DEC-058 … DEC-064) |
| M2.6 | The macOS menu bar application | `UX-001` … `UX-010`, `UX-013` … `UX-020`, `MB-001` … `MB-009` | **in part (2026-11-16)** — the application is a client of the administration surface and holds no state of the server of its own; it reads the state on a timer of one second and shows it as a line of text, the icon of the system follows the appearance of the menu bar, the screen reader reads the label and the value of the item, and the texts live in one resource file per language (`UX-006`, `UX-007`, `UX-009`, `UX-010`, `UX-014`, `UX-018`, `UX-020`, `MB-001` … `MB-002`, `MB-004` … `MB-006`, `MB-009`); the requests run on a thread of their own, so that the main thread never waits for the network (`MB-002`); the two views — the short one and the detailed one — read the same state, and a warning of the server stays visible in the short view (`UX-015`, `UX-016`, `MB-004`); the bundle carries the metadata of the system (`MB-009`); the first-run assistant, the verification step, the audit of the screen reader and the universal binary are not built (`UX-001` … `UX-005`, `UX-008`, `UX-013`, `UX-017`, `UX-019`, `MB-003`, `MB-007`, `MB-008`, `PLT-004`), the application starts no program at all — the rule of the workspace that no program starts another one, which the test `crates/phairdlna-core/tests/ssdp_hardening.rs::the_server_runs_no_process_at_all` holds over the four crates (SEC-TH-11) — so the server is started with the command line and not from the menu (`UX-019`, `MB-008`) (ADR-0013, DEC-065 … DEC-071) |
| M2.7 | Service installation and lifecycle | `LIFE-001` … `LIFE-011`, `PLT-013` | — |
| M2.8 | Adaptive resource management | `RES-001` … `RES-009` | **in part (2026-11-17)** — one scheduler in the core arbitrates the work of the daemon in four classes (streaming, control plane, metadata, background); a free worker takes the task of the most important class that may run one, every class and every queue has a documented bound, and a task that does not fit is refused with an error that names the bound; the reading of the library runs in the background class and yields while the machine is busy, with a hysteresis band and a quiet period, so that the scanner does not flap; the control plane and the covers are served through the scheduler, so that a client is answered before the library is read; the counters of the scheduler are asked for locally and name no file and no address (`RES-001` … `RES-005`, `RES-007`, `RES-009`); the signals of the processor, the memory and the disk are not measured by a platform adapter yet, the report of the resources is not shown in the administration surface yet, and the hints of a battery or of a warm machine are not read (`RES-006`, `RES-008`) |
| M2.9 | The catalogue index on disk: the SQLite index, its rebuild and its recovery | `FUN-008`, `FUN-009`, `FUN-010` | — |

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
