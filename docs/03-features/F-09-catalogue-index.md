# F-09 — Catalogue index and incremental scanning

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-008, FUN-009, FUN-010, FUN-011, FUN-012, FUN-025, PERF-003, PERF-004,
PERF-005, PERF-008, PERF-009, PERF-012

## 1. Purpose and user story

The **catalogue** is the server's own list of the user's media: which folders exist, which
files they contain and what is known about each file. DLNA has no protocol with which a
server could read a client's disk; the server must know the library itself before it can
answer a *browse* request from a television.

The catalogue is kept in an **index** — one local SQLite database. The index is a *cache*,
not a source of truth: the user's files are the truth (ADR-0006). Deleting the index must
cost time and nothing else. That single rule shapes this whole feature: the server never
writes to the user's media, the index is always disposable, and every value in it can be
recomputed from the filesystem.

An **incremental scan** compares what is on disk with what the index already knows — size
and modification time — and re-reads only what changed. A **full scan** walks everything.
A **rebuild** throws the index away and creates it again from scratch. The object
identifiers a client saw before the rebuild must be the same afterwards, because clients
cache them and reuse them (INT-062).

**User story.** As a user, I want my library indexed once and then kept up to date
cheaply, so that my television's list is never stale for long; and I want the index to be
disposable, so that a damaged database costs me a re-scan and never my collection.

## 2. Scope

**In scope**

- The `catalogue` module of `phairdlna-core`: the library model, the scanner, the indexer,
  the SQLite schema and the object-identifier scheme (ARCHITECTURE §4.5, ADR-0006,
  ADR-0007).
- The scanner-to-indexer pipeline: filesystem events in, rows out, applied in bounded
  transactional batches (ARCHITECTURE §7).
- The index as a rebuildable cache: creation, integrity check, automatic rebuild after
  corruption, and the report that it did so (FUN-009, FUN-010).
- The deterministic object-identifier scheme and its stability across rebuilds, restarts
  and upgrades (FUN-011).
- Media probing for duration, dimensions, bitrate and codec where no external program is
  needed (FUN-012). Artwork and thumbnails are F-10.
- The index behaviour the browse path depends on: availability during a scan (FUN-008),
  no filesystem access while browsing (PERF-008) and a bounded first page (PERF-005).
- The visibility bound for a newly added file and the fallback used when the file system
  does not deliver reliable change events (FUN-025).

**Explicitly out of scope**

- The `Browse` action, its paging, filtering and sorting (F-03) and the DIDL-Lite
  rendering of catalogue objects (F-05).
- Root definition, the traversal of a root and the detection of added, changed and
  removed files on disk (F-08 owns FUN-001 … FUN-007, FUN-017 … FUN-019 and FUN-024); the
  identity key itself is FUN-020 in F-11.
- Delivering the bytes (F-06); artwork, sidecar images and thumbnails (F-10); playlists
  and subtitles (F-11).
- The user-facing configuration of roots and limits (F-12); the command line (F-13); the
  administration surface (F-14).
- Transcoding and format conversion (F-17), and the invocation of any external program to
  inspect a file. ARCHITECTURE §4.6 refers to "ADR-0008" for that decision, but no such
  ADR exists in `docs/02-architecture/adr/` — raised in §12.

## 3. Functional requirements

**FUN-008** — The server shall keep the catalogue usable while a rescan runs; a browse
during a scan shall return the previously indexed content. [DESIGN] The scanner produces
filesystem events and the indexer applies them in batches; readers use a separate read
connection and see the last committed state, never a half-applied one. The scan never
holds a write transaction open across a batch, and a batch is small enough that the
writer's lock is held briefly.

**FUN-009** — The server shall store its catalogue in a local SQLite database that can be
deleted and rebuilt without data loss for the user's media. [DESIGN] One database file,
created with user-only permissions in a user-owned directory (SEC-095, ADR-0006). The
database holds paths and derived metadata only; it never holds the only copy of anything.
Deleting the file is a supported recovery action. The SQLite binding is used in its
bundled configuration with `loadable_extension` disabled (ADR-0006), so the server does
not depend on the system's SQLite version and cannot load extensions.

**FUN-010** — The server shall recover from a corrupted index database by rebuilding it
automatically, and shall report that it did so. [DESIGN] Opening the database runs an
integrity check. On failure the server moves the damaged file aside, creates a new one,
rebuilds from the filesystem, and records the fact in the log and in the status output
(OPS-002). The user is never left with a server that needs manual repair.

**FUN-011** — The server shall assign stable object identifiers that do not change when
the catalogue is rebuilt. [DESIGN] An identifier is computed, never stored as a row
number: `object_id = encode(hash(volume_id || 0x00 || identity))` (ADR-0007), where
`identity` is the path relative to the root, normalised to Unicode NFC with a
per-component case fold computed for that root's detected case sensitivity. The
identifier is neither the raw path nor a database row id, and it leaks no path fragment
(INT-062, PLT-008). The standard containers keep the reserved ids `0` (root) and `1`
(content directory), because some clients start browsing at a hard-coded id (INT-038).
Uniqueness is a separate question: two entries are the same file when their
`(volume id, device id)` pairs match, not when their paths match (ADR-0007, platform
§5.7).

**FUN-012** — The server shall extract duration, dimensions, bitrate and codec
information for media files where it can do so without executing an external program.
[DESIGN] Pure-Rust container parsers are used for the formats on the allowlist
(ARCHITECTURE §4.6); no external program is invoked (M-08). A format that cannot be read
yields *no* duration, dimension or codec value — the server never guesses and never
reports a value it did not read. The probing itself runs in the out-of-process sandbox
child described in F-10 (SEC-066, security §8), so that a malformed file cannot harm the
server process.

**FUN-025** — The server shall make a newly added file visible in the catalogue within 30 seconds on a file
system that delivers reliable change events, and shall document the fallback behaviour when it does not.
[DESIGN] The catalogue follows the filesystem event stream, so the bound is the time from the event to the
committed index row — not the duration of a scan. The server never depends on events alone: the fallback is the
scheduled rescan (FUN-024) together with the explicit scan command (OPS-003), and the fallback that is in force
for a root is documented in the user documentation and named in the diagnostics report. When the event stream
loses events — an `IN_Q_OVERFLOW` on Linux, for example — the affected root is re-scanned rather than left
stale (platform-integration §8.2). The fallback interval is raised in §12.

**PERF-003** — An incremental rescan of a tree with 10 000 unchanged files shall complete
without re-reading file contents. [DESIGN] Change detection compares size and
modification time against the index; an unchanged file is never opened. This is a
structural property, verified by instrumenting the read path rather than by timing
(qa-performance §3.2).

**PERF-004** — The server shall index at least 200 media files per second on the
reference hardware when metadata extraction is disabled. [DESIGN] The figure is a target
and the method is recorded in §5; it is the indexing path only, with no media probing
and no artwork work.

**PERF-005** — A `Browse` request over a container with 10 000 children shall return its
first page within 100 ms. [DESIGN] The children of a container are an indexed query with
a bounded page, so the cost does not grow with the size of the container beyond the page
itself. The browse path touches the index only and never the filesystem (PERF-008,
ARCHITECTURE §5).

**PERF-008** — The server shall not perform a full directory scan on a browse request.
[DESIGN] This is structural: the browse path opens the index and nothing else. It is
verified with an audit hook that records every filesystem call made during a browse,
which must be zero.

**PERF-009** — The catalogue database shall support 100 000 objects with an index scan
that stays within a documented budget. [OPEN] The baseline fixes no number for the
budget; §5 proposes one and §12 asks the owner to confirm it.

**PERF-012** — The server shall degrade gracefully when a catalogue is larger than the
tested envelope: it shall remain responsive and shall not exhaust memory. [DESIGN] The
index lives on disk and only hot pages are cached, so a larger library costs disk and
query time, not resident memory (qa-performance §3.1). The server keeps answering browse
requests above the tested envelope; it never refuses to start because the library is
large.

## 4. Technical requirements

- **Module.** `phairdlna-core/src/catalogue/` owns the library model, the identifier
  scheme, the SQLite index, the scanner and the incremental update logic
  (ARCHITECTURE §4.5). The scanner produces a stream of filesystem events — added,
  changed, removed — and the indexer turns them into rows. The scanner never writes to
  the database directly; that is what makes the write path testable without a disk.
- **Three identities per object, one identifier.** Every catalogue row stores the *raw*
  path exactly as the operating system reported it (FUN-019, used verbatim for I/O), the
  *device identity* `(volume id, device id)` used for uniqueness and change detection
  (platform §6.2 rule 9), and the *normalised identity* used for the key. The object
  identifier is derived from the volume identity and the normalised identity
  (ADR-0007), never from the raw path and never from a row id.
- **Path handling.** The raw path is never re-encoded or re-normalised before use
  (FUN-019). APFS is normalisation-insensitive but preserves the on-disk form, so a
  stored path string can stop resolving even though the file never moved (platform
  §5.1). The index therefore keeps the raw path *and* the device identity, and resolves
  by device identity wherever possible.
- **Transactions.** A scan applies its changes in bounded transactions (ARCHITECTURE
  §7); a crash mid-scan leaves the index consistent, not half-updated. Readers use a
  separate connection and see the last committed state.
- **Volume events.** Mount, unmount and remount are first-class events (PLT-011); the
  affected rows are re-keyed when a volume reappears under a different identity. The
  platform trait supplies volume identity and change notification (platform §6.2 rules 6
  and 8, ARCHITECTURE §12).
- **Quarantine.** A subtree that cannot be read because of a permission error is marked
  quarantined and the rest of the root continues to be indexed (platform §6.2 rule 7,
  PLT-012). A quarantined subtree is reported in diagnostics; it is not silently skipped.
- **No writes to the media.** The server never writes to, renames or deletes a file in a
  media root, with the single exception of its own cache directory when the cache is
  configured inside the root (FUN-018, PRD-007).
- **SQL.** Every query is parameterised; no value from the filesystem or from a client
  is ever concatenated into SQL text (SEC-005).
- **Containment.** Every path taken from the index is canonicalised and re-checked
  against its root before it is opened, including after symbolic-link resolution
  (SEC-001, SEC-003). The symbolic-link policy itself is a per-root configuration
  setting (CFG-007, F-12).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.
Every measurement follows qa-performance §4: release build, at least three discarded
warm-up runs, at least 30 measured iterations, median and 95th percentile reported, on
the reference profile of qa-performance §2.

- [DESIGN] Target (PERF-003): an incremental rescan of 10 000 unchanged files performs
  zero content reads. Method: instrument the file-read path of the scanner and assert
  that the read count is 0 while the stat count equals the file count. Structural check,
  not a timing.
- [DESIGN] Target (PERF-004): at least 200 files per second are indexed with metadata
  extraction disabled. Method: `hyperfine` (qa-performance §4) over a generated tree of
  100 000 files on the reference profile, ≥ 30 iterations, median and p95 reported.
- [DESIGN] Target (PERF-005): the first page of a browse over a container with 10 000
  children is returned in under 100 ms. Method: timed at the browse handler, excluding
  network time (qa-performance §3.2).
- [DESIGN] Target (FUN-025): a newly added file is visible in the catalogue within 30 seconds of its
  creation on a file system that delivers reliable change events. Method: create a file in a temporary root,
  poll the catalogue until the object appears and record the elapsed time; the fallback path is exercised
  separately with the event source replaced by a stub that delivers nothing, and the elapsed time in that
  case is reported against the fallback interval rather than against the 30-second bound.
- [DESIGN] Target (PERF-008): zero directory scans during a browse. Method: a filesystem
  audit hook that records every directory read; the count must be zero.
- [DESIGN] Target (PERF-009): an index scan over 100 000 objects stays within a
  documented budget. *Proposed budget, not in the baseline:* 20 ms median for the
  children query of one container. Method: `criterion` (qa-performance §4) over a
  generated 100 000-object index, ≥ 30 iterations, median and p95 reported. §12 Q2.
- [DESIGN] Target (PERF-012): with 400 000 indexed objects — four times the tested
  envelope — the server still answers a browse within the PERF-005 budget and its
  resident memory stays within the PERF-001 envelope. Method: the same browse benchmark
  plus `ps` sampling of the resident set.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effect is that the television's
  list is complete and up to date, and that the list does not go blank while the server
  is re-indexing.
- [UX-013] A scan reports progress and can be cancelled. The progress report is the
  number of files seen, changed and skipped, and it is available from the command line
  (OPS-003, F-13).
- [UX-004] When a root cannot be read, the message the user sees states what happened
  (the subtree is quarantined), why (a permission error) and what to do next (grant
  access to that folder). The message names the folder, not a stack trace.
- [UX-001] Everything here is operable without a GUI (PRD-006).
- [DOC-005] *catalogue*, *index*, *scan*, *incremental scan*, *rebuild* and *quarantine*
  are explained at first use.

## 7. Security and privacy

- The index is a local file with user-only permissions, in a user-owned directory, and
  never in a shared or world-readable location (SEC-095, ADR-0006).
- The index holds file names and paths — personal data in the sense of SEC-093 — but
  nothing else: no browsing history, no client address, no usage statistics. It is
  local, it is not transmitted, and it can be deleted at any time.
- Every path in the index is re-validated against its root before use, so a stale or
  tampered row cannot become a read outside the library (SEC-001, SEC-003, SEC-017).
- The scanner opens files read-only and never asks for write permission on a media
  directory (SEC-008).
- A file whose metadata cannot be parsed is quarantined by the sandbox, not by this
  feature; the index records the quarantine reason so the user can see it
  (SEC-066, security §8).
- Log records about the index use the redaction rules of OPS-007 (F-13); a path in a log
  line is replaceable by a stable pseudonym (SEC-094).
- The index contains no client data and no browsing state, so a client that disconnects
  leaves nothing behind in it.

## 8. Interoperability

- The catalogue is what the ContentDirectory service exposes. The identifiers this
  feature computes become the `id` attribute of every DIDL-Lite item and container
  (INT-060, INT-062, F-05); the reserved ids `0` and `1` are what some clients expect
  when they start browsing (INT-038).
- The identifier must be identical on every platform for the same library, so that a
  client cached from a macOS server keeps working after the same library is served from
  Linux (PLT-010, ADR-0007).
- A browse must never depend on the filesystem being available: a client browsing while
  a removable volume is unplugged receives the previously indexed content plus the
  diagnostics entry for the missing volume, not an error page (FUN-007, F-08).
- No DLNA or UPnP requirement is changed by this feature; the catalogue is an
  implementation detail behind the ContentDirectory contract.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| The index file is missing | a new index is created and a full scan is scheduled | FUN-009 |
| The index file is corrupted | the file is moved aside, a new one is created, the catalogue is rebuilt and the rebuild is reported in the log and in the status output | FUN-010 |
| The index cannot be written (read-only directory) | start-up aborts with a message naming the file and the directory; no scan is attempted | UX-004 |
| A subtree cannot be read (permission error) | the subtree is quarantined, the rest of the root is indexed, the quarantine is reported | platform §6.2 rule 7, PLT-012 |
| A root is unmounted or replaced | the affected rows are marked unavailable and re-keyed when the volume returns; the catalogue as a whole keeps working | PLT-011, FUN-007 |
| A file disappears between the scan and the index write | the row is removed by the next scan; a stale row is never served | FUN-006 (F-08) |
| A media file cannot be parsed | the sandbox quarantines the file; the object carries no duration or dimensions | FUN-012, SEC-066 |
| A scan is cancelled mid-way | the committed rows stay; the scan stops at the next batch boundary | UX-013 |
| The file system delivers no change events, or loses them (event-queue overflow) | the affected root falls back to the scheduled rescan, the fallback in force for that root is named in the diagnostics, and the catalogue is never left knowingly stale | FUN-025, FUN-024, platform §8.2 |

No failure of this feature may take down the HTTP task or the SSDP task: a database
error produces a diagnostic and a degraded catalogue, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* an indexed library, *when* a rescan starts and a browse is issued while the scan is running, *then*
the browse returns the previously indexed content, the scan is not blocked by the browse, and the browse
completes without a directory scan. Requirements: FUN-008, PERF-008 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/catalogue_index.rs`

**AC-02** — *Given* an indexed library, *when* the index file is deleted and the server is started again, *then* the
catalogue is rebuilt from the filesystem, no media file is modified, moved or deleted, and the rebuilt
catalogue contains the same objects. Requirements: FUN-009 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/catalogue_rebuild.rs`

**AC-03** — *Given* a deliberately corrupted index file, *when* the server starts, *then* it detects the corruption,
rebuilds the catalogue automatically, and reports in the log and in the status output that it rebuilt the
index. Requirements: FUN-010, OPS-002 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/catalogue_rebuild.rs`

**AC-04** — *Given* the same library, *when* the index is deleted and rebuilt, *then* every object has the same
identifier as before the rebuild, and no identifier contains a path fragment or a file name. Requirements:
FUN-011, INT-062, PLT-008 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/catalogue_identifiers.rs`

**AC-05** — *Given* two catalogue entries that name the same file through different paths (a hard link), *when* the
catalogue is built, *then* the entries are recognised as one file by their `(volume id, device id)` pair and
the library is not double-counted. Requirements: FUN-011 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/catalogue_identifiers.rs`

**AC-06** — *Given* a media file in a supported container format, *when* the catalogue is built, *then* the object
carries the duration, dimensions, bitrate and codec read from the file; and *given* a file in an unsupported
format, *then* the object carries no such value rather than a wrong one. Requirements: FUN-012 ·
Verification: automated test · Planned location:
`crates/phairdlna-core/tests/catalogue_metadata.rs`

**AC-07** — *Given* a media file that no parser can read, *when* the catalogue is built, *then* the file is quarantined
by the sandbox child, the server process is unaffected, and the catalogue contains no duration or dimension
value for that file. Requirements: FUN-012, SEC-066 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/media_sandbox.rs`

**AC-08** — *Given* a tree of 10 000 unchanged files, *when* an incremental rescan runs, *then* no file content is read,
and the files are still present in the catalogue afterwards. Requirements: PERF-003 · Verification:
automated test (instrumented read path) · Planned location:
`crates/phairdlna-core/tests/catalogue_incremental.rs`

**AC-09** — *Given* a tree of 100 000 files and metadata extraction disabled, *when* the catalogue is indexed, *then*
the throughput is at least 200 files per second, measured as median over at least 30 iterations on the
reference profile. Requirements: PERF-004 · Verification: benchmark · Planned location:
`crates/phairdlna-bench/benches/indexing.rs`

**AC-10** — *Given* a container with 10 000 indexed children, *when* a browse for the first page is issued, *then* the
first page is returned within 100 ms, measured at the browse handler. Requirements: PERF-005 · Verification:
benchmark · Planned location: `crates/phairdlna-bench/benches/browse.rs`

**AC-11** — *Given* a container with 10 000 indexed children, *when* a browse is issued, *then* the number of directory
reads made during the request is zero. Requirements: PERF-008 · Verification: automated test (filesystem
audit hook) · Planned location: `crates/phairdlna-core/tests/catalogue_index.rs`

**AC-12** — *Given* an index of 100 000 objects, *when* the children of one container are queried, *then* the query
completes within the documented budget stated in §5. Requirements: PERF-009 · Verification: benchmark ·
Planned location: `crates/phairdlna-bench/benches/index_scan.rs`

**AC-13** — *Given* an index four times the tested envelope, *when* a browse is issued, *then* the browse still returns
within the PERF-005 budget and the server's resident memory stays within the PERF-001 envelope.
Requirements: PERF-012 · Verification: benchmark · Planned location:
`crates/phairdlna-bench/benches/large_catalogue.rs`

**AC-14** — *Given* a root whose subtree cannot be read because of a permission error, *when* the catalogue is built,
*then* that subtree is quarantined, the rest of the root is indexed, and the quarantine is named in the
diagnostics. Requirements: FUN-008, PLT-012 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/catalogue_quarantine.rs`

**AC-15** — *Given* a removable volume that is unmounted and later reappears, *when* the catalogue is rebuilt, *then*
the objects of that volume keep their identifiers and the rows of the missing volume are marked unavailable
instead of being served as missing files. Requirements: FUN-011, PLT-011 · Verification: manual protocol
check on a host with a removable volume · Planned location: `qa/device-matrix.toml`

**AC-16** — *Given* a catalogue that has been built, *when* the whole tree is inspected for writes, *then* no file
inside a media root has been created, modified, renamed or deleted by the server. Requirements: FUN-009,
FUN-018, PRD-007 · Verification: automated test (filesystem audit hook) · Planned location:
`crates/phairdlna-core/tests/catalogue_rebuild.rs`

**AC-17** — *Given* a root on a file system that delivers reliable change events, *when* a media file is created
in it, *then* the object appears in the catalogue within 30 seconds of the creation; and *given* a file system
that delivers no change events, *when* the same file is created, *then* the file appears through the documented
fallback — the scheduled rescan or the explicit scan command — and the diagnostics name the fallback in force for
that root. Requirements: FUN-025, FUN-024, OPS-003 · Verification: automated test with a scripted event source
plus an automated test of the fallback path · Planned location:
`crates/phairdlna-core/tests/catalogue_incremental.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Index behaviour, browse-during-scan, no filesystem access | `crates/phairdlna-core/tests/catalogue_index.rs` |
| Rebuild, corruption recovery, no-writes-to-media | `crates/phairdlna-core/tests/catalogue_rebuild.rs` |
| Identifier stability, de-duplication by device identity | `crates/phairdlna-core/tests/catalogue_identifiers.rs` |
| Incremental scan, change detection, visibility bound and its fallback | `crates/phairdlna-core/tests/catalogue_incremental.rs` |
| Media probing and quarantine | `crates/phairdlna-core/tests/catalogue_metadata.rs`, `crates/phairdlna-core/tests/media_sandbox.rs` |
| Scanner and indexer unit tests | `crates/phairdlna-core/src/catalogue/scanner.rs`, `crates/phairdlna-core/src/catalogue/index.rs`, `crates/phairdlna-core/src/catalogue/identifiers.rs` (inline `#[cfg(test)]` modules) |
| Throughput and latency benchmarks | `crates/phairdlna-bench/benches/indexing.rs`, `.../browse.rs`, `.../index_scan.rs`, `.../large_catalogue.rs` |
| Removable-volume behaviour | `qa/device-matrix.toml` |

**Method.** The scanner and the indexer are separated (ADR-0006), so the scanner is
exercised against an in-memory filesystem and the indexer against an in-memory database;
the two are joined by a test that feeds synthetic filesystem events. The identifier
scheme is a pure function, so it is exercised directly. The "no file content is read"
criterion (AC-08) is proven by an instrumented read path rather than by timing
(qa-performance §3.2).

**Dependencies.** F-08 (roots, traversal, volume detection, raw-path handling), F-11
(the identity key of FUN-020), F-10 (the sandbox that probes media), F-03 (the browse
path that reads this index), F-05 (the identifiers this feature produces), F-12 (cache
size and worker limits) and the platform trait for volume identity and volume events
(ADR-0003, ARCHITECTURE §12).

## 12. Open questions and decisions

1. **[OPEN] "Root id" or "volume id" in the identifier formula.** ADR-0006 and
   ARCHITECTURE §8 describe the identifier as a hash of the *root id* and the path
   relative to that root; ADR-0007 defines it over the *volume id* and the normalised
   identity. The two differ when two roots point at the same volume (FUN-022) and when a
   root is remounted at a different mount point. *Proposed reading, used throughout this
   document:* the hash input is the volume identity plus the path relative to the root,
   exactly as ADR-0007 states. *Closes when:* the owner confirms the reading or amends
   ADR-0006/§8.
2. **[OPEN] The documented budget of PERF-009.** The baseline requires a budget but
   names no number. *Proposed:* 20 ms median for the children query of one container
   at 100 000 objects, on the reference profile. *Closes when:* the owner fixes the
   number in qa-performance §3.2.
3. **[OPEN] What "rebuild" costs at the tested envelope.** No requirement bounds the time
   a rebuild of 100 000 objects may take, and a user with a damaged index on a NAS-backed
   library will experience it. *Proposed:* the status output states the number of objects
   rebuilt and the elapsed time, and the rebuild is interruptible and resumable.
   *Closes when:* the owner decides whether a rebuild time budget is added.
4. **[OPEN] Quarantine visibility.** PLT-012 asks for a quarantined subtree that "needs
   consent", and FUN-012 and SEC-066 ask for a quarantined *file*. This document treats
   them as two different things: a permission-quarantined subtree is reported in
   diagnostics, and a parse-quarantined file is excluded from the catalogue. Whether a
   quarantined file should appear in the catalogue as an unreadable placeholder is
   undecided. *Closes when:* the owner decides.
5. **[OPEN] ADR-0008 is missing.** ARCHITECTURE §4.6 states that "an external program is
   never invoked in the baseline (ADR-0008)", but `docs/02-architecture/adr/` contains no
   ADR-0008. FUN-012 depends on that decision being recorded. *Closes when:* the ADR is
   written or the reference is corrected.
6. **[OPEN] The fallback interval when the file system does not deliver change events.**
   FUN-025 requires the fallback behaviour to be documented but names no interval. *Proposed
   reading:* the fallback is the scan schedule already configured for the root (FUN-024); when the
   schedule is "manual only", the fallback is the explicit scan command (OPS-003), and the
   diagnostics report says so instead of leaving the user with a catalogue that silently ages.
   *Closes when:* the owner confirms the reading or fixes a separate fallback interval.
7. **[OPEN] Where the index lives when the data directory is not writable.** The index
   needs a writable directory; on macOS a system-context daemon writes to
   `/var/lib`/`/var/log` while a user instance writes to the user's data directory
   (platform §4.2). *Closes when:* the platform adapter documents the per-instance data
   directory (F-13, OPS-010).
