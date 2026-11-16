# Decision log

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Architect

Every decision has an ID, a status, the date it was taken and who may change it.
Statuses:

- **Owner direction** — decided by the project owner; agents may not change it.
- **Accepted** — an architecture/process decision taken by the Lead within the mandate
  given by the owner; reversible by the Lead, recorded here.
- **Proposed** — a recommendation waiting for owner approval.
- **Superseded** — replaced by a later decision; the original text stays in this table.

Architectural decisions with options and trade-offs live as ADRs in
`../02-architecture/adr/` and are referenced from this table.

---

## Product and licensing

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-001 | Public, AGPL-3.0-or-later, ad-free, telemetry-free, privacy-first, no vendor account | Owner direction | 2026-11-14 | Owner |
| DEC-002 | Rust-first portable core; macOS first, Linux and Windows architecturally included from the start | Owner direction | 2026-11-14 | Owner |
| DEC-003 | Documentation is a product deliverable and must be readable by a programming beginner | Owner direction | 2026-11-14 | Owner |
| DEC-004 | GitHub Actions are manual-only; no unapproved releases or tags | Owner direction (confirmed) | 2026-11-14 | Owner |
| DEC-005 | Full, unmodified AGPL-3.0 text ships in `LICENSE`; the project is "AGPL-3.0-or-later" | Accepted | 2026-11-14 | Lead |
| DEC-006 | Offline/LAN-first: no internet dependency for normal operation, no automatic router port mapping, no cloud services | Owner direction | 2026-11-14 | Owner |
| DEC-007 | Media files are opened read-only by default; the server never modifies originals without explicit user consent | Accepted | 2026-11-14 | Lead |
| DEC-008 | The headless server must be fully functional without any GUI; the macOS menu bar app is an optional front end for the same core | Accepted | 2026-11-14 | Lead |

## Requirements and quality

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-009 | Requirements before implementation: the owner approves a requirements baseline before feature code is written | Owner direction | 2026-11-14 | Owner |
| DEC-010 | Requirement IDs are stable, prefixed by domain, and traceable to a feature, a milestone and a verification method | Accepted | 2026-11-14 | Lead (see `REQUIREMENTS-METHOD.md`) |
| DEC-011 | A gate result is only reported as passed when it was actually observed; `NOT RUN` and skip exit codes are never reported as success | Accepted | 2026-11-14 | Lead |
| DEC-012 | No performance, compatibility or security claim without recorded evidence | Owner direction | 2026-11-14 | Owner |

## Architecture

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-013 | Documentation is consolidated into the canonical `00`–`99` structure; the legacy `docs/requirements`, `docs/architecture`, `docs/security` and `docs/project` trees are migrated with an archive snapshot | Accepted | 2026-11-14 | Lead |
| DEC-014 | The Rust core is platform-neutral; everything OS-specific lives behind a narrow platform-adapter trait | Accepted | 2026-11-14 | Lead (ADR-0003) |
| DEC-015 | SSDP and the UPnP control layer are implemented in-project; no third-party UPnP/SSDP stack is taken as a dependency | Accepted | 2026-11-14 | Lead (ADR-0005) |
| DEC-016 | The SQLite index is the single source of truth for the catalogue; the index is rebuildable and never authoritative for file contents | Accepted | 2026-11-14 | Lead (ADR-0006) |
| DEC-017 | Transcoding is optional, isolated in its own process boundary, and off by default | Accepted | 2026-11-14 | Lead (ADR-0009) |
| DEC-018 | Media metadata and thumbnails are parsed in a sandboxed child process, never in the server process | Accepted | 2026-11-14 | Lead (ADR-0008) |
| DEC-019 | The runtime, HTTP server and XML parser are Tokio, Hyper/Axum, quick-xml and rusqlite, with the HTTP framing policy and the XML policy owned by the project | Accepted | 2026-11-14 | Lead (ADR-0002) |
| DEC-020 | The AGPL network-source offer is a product feature (an unauthenticated route on the running server), and the server never updates itself over the network | Accepted | 2026-11-14 | Lead (ADR-0010) |
| DEC-021 | `docs/05-quality/QUALITY-GATES.md` is the single register of the gate identifiers; every other document refers to a gate by the ID defined there | Accepted | 2026-11-15 | Lead |
| DEC-022 | The research briefs the specifications cite are committed under `docs/09-research/` as evidence, not as norm; a citation such as `standards §8.3 C12` resolves there | Accepted | 2026-11-15 | Lead |
| DEC-023 | The documentation carries one identifier scheme only. A draft that introduced its own IDs is archived or explicitly marked as non-normative | Accepted | 2026-11-15 | Lead |
| DEC-024 | The competitive bar is the macOS media player **Elmedia**: PhairDLNA must be the better macOS media *server* and must not be the worse macOS media *tool*. The bar is expressed as eleven testable criteria, not as a slogan | Owner direction | 2026-11-15 | Owner |
| DEC-025 | A competitor feature is adopted only after an evaluation record: source, user problem, our independent better outcome, architecture fit, security and privacy analysis, resource budget and acceptance tests. The record decides CORE / OPTIONAL / LATER / REJECT | Accepted | 2026-11-15 | Lead |
| DEC-026 | The Apple media libraries stay out of the baseline. They are a deliberate, per-library opt-in (OD-13), never a default | Accepted | 2026-11-15 | Lead |
| DEC-027 | The object identifiers of the catalogue are the blake3 hash of the volume identity, the root key and the relative identity, cut to 128 bits and rendered as hexadecimal | Owner direction | 2026-11-15 | Owner |
| DEC-028 | The portable filesystem adapter of the core reports the constant volume identity `local`, so that the object identifiers survive a remount and a move of the library; the real volume identity arrives with the platform adapter of F-13 | Accepted | 2026-11-15 | Lead (ADR-0007, `PLT-005`) |
| DEC-029 | The root key — the position of the root in the configuration — is part of the object identifier, so that two roots of one configuration keep their files apart; the name of a root is a label and is not part of it | Accepted | 2026-11-15 | Lead (FUN-022, ADR-0007) |
| DEC-030 | A folder of the library is a container in the DIDL-Lite document and not an item; the catalogue model carries `MediaKind::Folder` and the number of children | Accepted | 2026-11-15 | Lead (INT-042) |
| DEC-031 | The requirements baseline of the v1.0 documents is approved, so feature implementation may proceed | Owner direction | 2026-11-15 | Owner |
| DEC-032 | The minimum supported macOS version is macOS 15 (Sequoia) and later, with the launchd daemon model throughout (OD-05) | Owner direction | 2026-11-15 | Owner |
| DEC-033 | The asynchronous runtime (`tokio`), the HTTP server (`axum`) and the sockets live in `phairdlna-cli`; the core keeps the rules and stays portable and free of a runtime | Accepted | 2026-11-15 | Lead (M1.11, ADR-0002) |
| DEC-034 | The SSDP socket lives in the command-line crate; the core keeps the protocol — the parser, the matching table and the bytes of the advertisement and of the answer. The deviation from F-01 § 4 is recorded in that document | Accepted | 2026-11-15 | Lead (M1.11, F-01 § 12 item 7) |
| DEC-035 | The daemon serves the paths the device description advertises (`/ContentDirectory.xml`, `/ConnectionManager.xml`, `/ctl/CD`, `/ctl/CM`) and serves the conventional paths as well, because a client may hard-code them | Accepted | 2026-11-15 | Lead (M1.11, F-13 § 12 item 9) |
| DEC-036 | The `serve` command takes a `--port` switch, and the daemon accepts another port for the discovery socket and another path for the identity file, so that a test writes nothing into the configuration directory of the user and competes with no running server | Accepted | 2026-11-15 | Lead (M1.11, F-13 § 12 items 12 and 13) |
| DEC-037 | `FUN-008`, `FUN-009` and `FUN-010` — the catalogue that survives a rescan, the index on disk and its recovery from corruption — move to M2. The catalogue of M1 is clean, in memory and rebuilt at start-up; the index on disk is the work of M2 | Owner direction | 2026-11-16 | Owner (F-09, ADR-0006) |
| DEC-038 | The milestones M1.10 and M1.11 are reviewed together, because M1.11 was implemented before M1.10 and the security baseline must be proved against the code that is already written | Accepted | 2026-11-16 | Lead (M1.10, M1.11) |

## Reviews and evidence

| Review | Date | Result |
| --- | --- | --- |
| Independent beginner-experience review | 2026-11-14 | 6 blocker, 16 major, 11 minor, 4 info findings — **all addressed**; the report is kept at `../05-quality/reviews/BEGINNER-REVIEW-2026-11-14.md` |
| Independent consistency review (round 2) | 2026-11-15 | 2 blocker, 6 major, 15 minor, 4 info findings — **all addressed**; the report is kept at `../05-quality/reviews/CONSISTENCY-REVIEW-2026-11-15.md` |
| Independent review of M1.9 (the catalogue over one root) | 2026-11-15 | 3 blocker, 7 major, 11 minor, 3 info findings — the blockers and the majors are addressed by the milestone: the update identifier is monotonic and content-sensitive, the hard links are one object, the link policy is per root, the catalogue holds the target of a link, the diagnostics name no path, the root container has the parent `-1`, a browse answers for an item, and the tables of F-08 and F-09 name every criterion with its result |

## Open decisions requiring owner approval

These are collected in `../07-project/OPEN-DECISIONS.md` with options and consequences.
Summary: **OD-01** macOS distribution/signing model · **OD-02** transcoding dependency
policy (FFmpeg) · **OD-03** macOS UI technology (pure Rust vs. native Swift helper) ·
**OD-04** admin authentication model · **OD-06** policy on shipping a prebuilt binary vs.
source-only distribution. **OD-05** (the minimum supported macOS version) was decided on
2026-11-15: macOS 15 and later (DEC-032).
