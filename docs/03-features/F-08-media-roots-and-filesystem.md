# F-08 — Media roots and filesystem safety

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-001, FUN-002, FUN-003, FUN-004, FUN-005, FUN-006, FUN-007, FUN-017, FUN-018, FUN-019, FUN-023, FUN-024, SEC-001, SEC-002, SEC-003, SEC-004, SEC-005, SEC-006, SEC-007, SEC-008, PLT-003, PLT-005, PLT-006, PLT-007, PLT-011, PLT-012, PLT-015, PLT-017
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

A **media root** is a folder the user shares with the server. The server walks the roots, indexes what it finds
and serves the result as one catalogue. The filesystem is the hard part of a media server: the user's disk holds
symbolic links, aliases, hidden files, removable drives, network shares and folders the operating system
protects — and every one of those is a way for the server to read a file it was never given.

Two rules make this tractable. The **raw path** — the path exactly as the operating system reported it — is never
rewritten and is used verbatim for every file operation, because re-encoding it can stop a file from resolving
(ADR-0003 rule 3). And **containment** is decided after the path has been resolved: resolve the symbolic links,
then check where the file really is.

**User story.** As a user, I want to point the server at several folders — my films on the internal disk, my
music on an external drive, a folder on my NAS — and I want it to work, including when I unplug the drive; and I
want it to never read a file outside those folders, even if a link inside one of them points somewhere else.

## 2. Scope

**In scope** — the root configuration and the union catalogue; the scanner and the incremental rescan; the
  containment rule and everything it implies for opening a file; the symbolic-link policy; the treatment of hidden
  files, macOS sidecars and aliases; removable volumes, network shares and the events they raise; the quarantine of
  a subtree the user has not consented to; the per-root detection of case sensitivity, Unicode normalisation
  behaviour and volume identity; the scan schedule; the reporting of a root that is missing; the platform traits
  that keep the core portable (PLT-003); and the path handling of names with spaces, non-ASCII characters and emoji
  (PLT-006), of Windows traversal attempts written with either separator (PLT-007) and of Windows long paths
  (PLT-015).

**Explicitly out of scope** — metadata extraction (F-09), thumbnails and artwork (F-10), media delivery over
  HTTP (F-06), the DIDL-Lite rendering of the catalogue (F-05) and the `Browse` action (F-03); the identity key
  and the de-duplication rule (F-11, FUN-020, FUN-022); the exclusion of subdirectories of a root (F-11, FUN-021);
  the compatibility profiles that may shape what a client sees (F-07); the administration surface (F-14).

## 3. Functional requirements

**FUN-001** — The user shall be able to configure one or more media roots; each root shall have a name shown to
  clients and an absolute filesystem path. [DESIGN] The name is the label a client sees, never a path fragment
  (PLT-008); the path is absolute, because a relative root would depend on the daemon's working directory.

**FUN-002** — The server shall serve the union of all configured roots as one catalogue. [DESIGN] One browse
  tree, one root container; the root names appear as its top-level containers (F-05).

**FUN-003** — The server shall not follow a symbolic link that points outside its root unless the user
  explicitly allows it. [DESIGN] The allowance is a per-root setting, disabled by default (CFG-007); the same rule
  governs the scan and the serving of a media URL (SEC-006, SEC-001).

**FUN-004** — The server shall ignore hidden files and macOS resource-fork sidecar files (`._*`) when building the
  catalogue. [DESIGN] Hidden means the platform's notion of hidden — the `UF_HIDDEN` flag on macOS, the leading-dot
  convention elsewhere; `._*` are AppleDouble containers holding metadata, not media (platform-integration §5.6).

**FUN-005** — The server shall index audio, video, image and playlist files by extension and by sniffed content
  type. [DESIGN] The extension is a hint and the content is the evidence, because media files without an extension
  are common on macOS (platform-integration §5.5); the shared MIME registry decides the served type (F-04, F-06).

**FUN-006** — The server shall re-scan a root incrementally and detect added, changed and removed files without a
  full re-read of unchanged files. [DESIGN] The scanner compares size and modification time against the index and
  re-reads only what changed (ARCHITECTURE §7); a scan never blocks a browse.

**FUN-007** — The server shall detect a root that is unmounted, inaccessible or replaced by a different volume,
  and shall report it in diagnostics instead of failing the whole catalogue. [DESIGN] Detection compares the volume
  identity the root was configured with against the volume identity observed now (PLT-005, PLT-011); the other
  roots keep working.

**FUN-017** — The server shall support media stored on removable volumes and network shares, and shall recover
  when such a volume reappears. [DESIGN] A removable volume appears under `/Volumes/<name>` on macOS
  (platform-integration §5.2); a network share is mounted into the filesystem like any other directory
  (platform-integration §5.3). The volume watcher raises `Mounted` / `Unmounted` / `Remounted` / `RescanRequired`
  (ADR-0003).

**FUN-018** — The server shall never write to, rename or delete a file inside a media root, with the single
  exception of its own cache directory when the cache is configured inside the root. [DESIGN] Media files,
  playlists and subtitles are user-owned artefacts and are never modified (ARCHITECTURE §9); the cache directory
  is excluded from indexing.

**FUN-019** — The server shall keep the raw path exactly as the operating system reported it for every file
  operation, and shall never re-encode or re-normalise a path before using it. [DESIGN] `MediaPath.raw` is the
  path as reported; `MediaPath.identity` is a separate, derived value used for indexing and matching
  (ADR-0003 rule 3). APFS is normalisation-insensitive but preserves the on-disk form, so a stored path string can
  stop resolving even though the file never moved (platform-integration §5.1).

**FUN-023** — The server shall expose the free space of a root through the ContentDirectory service when the file
  system provides it. [DESIGN] `GetFreeSpace` is **not** a standard ContentDirectory action — it is absent from
  ContentDirectory 1–4 and from every shipped implementation the research read, which requires a client-specific
  extension to be kept out of the standard service description (standards §0.3, §8.3 C12; INT-048). The value is
  therefore exposed through a documented vendor extension in its own namespace. Raised in §12.

**FUN-024** — The server shall allow the user to configure a scan schedule (interval, or manual only). [DESIGN]
  The default is an interval; "manual only" means the catalogue is refreshed solely by an explicit scan command
  (OPS-003) or a filesystem event.

**SEC-001** — The server shall resolve every media path canonically and verify that it is inside an allowed root
  before opening it. [DESIGN] Canonicalise-then-contain is the primitive; textual collapsing of `..` never touches
  the filesystem and therefore cannot tell a contained path from an escaped one (security-landscape §4.4).

**SEC-002** — The server shall reject a path containing `..` segments, encoded separators (`%2f`, `%5c`), or a null
  byte, before canonicalisation. [DESIGN] The rejection happens before any filesystem call, so a hostile path never
  reaches the operating system (SEC-015, SEC-011).

**SEC-003** — The server shall resolve a symbolic link and re-check containment after resolution, to resist a
  link that changes between check and use. [DESIGN] Containment is checked twice — on the candidate path and on the
  resolved target — because a link can be repointed between the two moments (security-landscape §4.4, §7.4).

**SEC-004** — The server shall open files in a way that does not follow a final symbolic link when the file is
  outside the root. [DESIGN] Where the platform offers it, the open uses the no-follow flag so that the final
  component cannot be a link (security-landscape §4.4); the portable fallback is the re-check of SEC-003 plus a
  directory-relative open.

**SEC-005** — The server shall never construct a shell command from user, client or file metadata input.
  [DESIGN] No `Command::new`, no `sh -c`, no `system()` anywhere in the request path (security-landscape §3.3);
  metadata parsing happens in a separate, sandboxed process and never shells out to an external program (ADR-0008;
  SEC-066).

**SEC-006** — The server shall not follow a symbolic link that points outside a root, unless the user explicitly
  enabled it for that root, and it shall log the attempt. [DESIGN] The log record names the root and the link, with
  the target redacted like every other path (OPS-007, SEC-094).

**SEC-007** — The server shall treat all file names and metadata as untrusted input, including their length and
  encoding. [DESIGN] A name is bounded in bytes before it is copied into an index row, a URL or a log line; a name
  that is not valid UTF-8 is handled without being reinterpreted (SEC-060, SEC-011).

**SEC-008** — The server shall open media files read-only, without requesting write permission on the containing
  directory. [DESIGN] The scanner and the HTTP layer open read-only; the server never needs write access to the
  user's media and never asks for it (OPS-015, PLT-018).

**PLT-003** — All OS-specific behaviour shall be reachable through a documented trait with a default
implementation for unsupported platforms. [DESIGN] The platform layer is a set of small, orthogonal traits — the
filesystem facts, the volume identity, the volume events, the consent mechanism — and every trait has a default
implementation that reports "not supported here", so that the core compiles and runs on a platform for which no
adapter exists yet (platform-integration §10.1 rules 2 and 4). The traits are the only place where an operating
system type may appear in a signature; the core speaks in project-owned types and never asks "which platform is
this?" (platform-integration §10.1 rule 1; ARCHITECTURE §12).

**PLT-005** — The server shall detect the case sensitivity, the Unicode normalisation behaviour and the volume
  identity of each media root instead of assuming them. [DESIGN] The detection is per root and is stored with the
  root, because one server may serve a case-sensitive Linux root and a case-insensitive APFS volume at the same
  time (ADR-0003; platform-integration §5.1); the platform reports the facts, the core decides what to do with
  them (ADR-0003 rule 1).

**PLT-006** — The server shall run correctly when the user's home directory or a media root contains spaces,
  non-ASCII characters or emoji. [DESIGN] A path is never assembled by string concatenation and never
  percent-encoded by hand: the components are joined with the platform's own path type and encoded once, on the way
  into a URL, by one function (SEC-015, INT-098). The identity key is computed on the NFC form, so a folder whose
  name contains an emoji or an accented character is indexed and served exactly like any other folder (FUN-019).

**PLT-007** — Windows path handling shall support long paths and shall reject path traversal attempts using both
  separator characters. [DESIGN] On Windows the separator is the backslash, so a traversal attempt may arrive as
  `..\` as well as `../`; the rejection rule (SEC-002) is applied to both characters before any filesystem call is
  made. Long paths need two things at once — the executable must declare `longPathAware` and the machine must have
  the `LongPathsEnabled` registry value set (platform-integration §9.4; PLT-015) — so the path handling alone is
  not enough.

**PLT-011** — The server shall treat the mount, unmount and remount of a volume as a first-class event and shall
  re-key the affected catalogue entries when a volume reappears under a different identity. [DESIGN] The event bus
  carries the volume event; the catalogue re-keys the entries of the affected volume and bumps the system update id
  so that clients are told to refresh (INT-037).

**PLT-012** — The server shall treat a permission error on a subtree as a quarantined subtree that needs consent,
  and shall continue indexing the rest of the root. [DESIGN] A quarantined subtree is not an error state: it is
  listed in diagnostics, it is not served, and it can be released once the operating system has granted access
  (PLT-017).

**PLT-015** — The Windows installation shall declare long-path support in the executable manifest and shall document
  that the system-wide `LongPathsEnabled` setting is also required. [DESIGN] The Windows executable embeds an
  application manifest that contains `<ws2:longPathAware>true</ws2:longPathAware>`, and the installation
  documentation states that the registry value
  `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled` must be `1` for the opt-in to
  take effect (platform-integration §9.4). Both conditions are needed: the manifest alone does not raise the legacy
  limit of 260 UTF-16 code units, and the registry value alone does not opt the application in. Because the registry
  value is machine-wide and cached per process, it can only be set by an installer that runs with administrator
  rights — the per-user installation cannot set it (platform-integration §9.4, §11.1 D9).

**PLT-017** — The server shall obtain the user's consent for folders that the operating system protects (macOS
  privacy prompts, Linux polkit, Windows elevation), and shall explain the denial instead of failing silently.
  [DESIGN] Consent is a platform fact obtained through the `Consent` trait (ADR-0003): macOS asks through the
  system privacy prompt or a folder picker, Linux through polkit, Windows through elevation (platform-integration
  §11.1 D14; security-landscape §7.4). The server never requires Full Disk Access and never works around a denial.

## 4. Technical requirements

- **Modules.** The root configuration lives in `config`; the scanner, the indexer and the volume handling live in
`catalogue`; the containment primitives live in `security`; every platform-specific fact comes through the
`platform` traits (ARCHITECTURE §4.1, §4.5, §4.8, §4.9).
- **The path type.** `MediaPath { raw, identity, volume, display }` is the central vocabulary of the catalogue
(ADR-0003). `raw` is used verbatim for every I/O (FUN-019); `identity` is NFC plus a per-component case fold
scoped by the volume; `volume` is the stable volume identity; `display` is the name shown to the user. The
identity key and the de-duplication rule are specified by F-11 (FUN-020, FUN-022).
- **Containment algorithm.** [DESIGN] The order is fixed and is the one the security research requires
(security-landscape §7.4, M-05): (1) percent-decode exactly once; (2) reject NUL, control characters and
backslashes; (3) join to the configured root; (4) resolve the target — symbolic links **and** macOS Finder
aliases; (5) require the resolved target to be under the canonicalised root; (6) open with the no-follow flag
where the platform allows it, preferring a directory-relative open to close the check-to-use window. Steps 1–3
happen before any filesystem call (SEC-002); step 4 happens before the decision (SEC-003).
- **Aliases are not symbolic links.** [DESIGN] A macOS Finder alias is a regular file to the kernel, so
canonicalisation does not resolve it, and it can appear inside the root while pointing outside it. An alias is
therefore refused, or resolved explicitly and then contained by the same rule as a link (security-landscape §7.4).
The baseline requirement for this is SEC-069, which is not owned by this feature; see §12.
- **Symbolic-link policy.** [DESIGN] The setting is per root, disabled by default (CFG-007, FUN-003, SEC-006).
When disabled, a link whose resolved target leaves the root is refused and logged; the scanner does not descend
through it and the HTTP layer does not follow it. When enabled, the link may be followed, but the resolved target
must still satisfy containment, so "follow" never means "leave the root".
- **Scanning.** [DESIGN] A bounded worker pool walks the tree and emits filesystem events; the indexer turns them
into rows (ARCHITECTURE §4.5, §7). The scanner never writes to the database directly. Hidden files, `._*` files
and the server's own cache directory are skipped (FUN-004, FUN-018), and the scan is bounded by the configured
worker and time limits (SEC-062).
- **Volume events, quarantine and consent.** [DESIGN] The `VolumeWatcher` trait reports `Mounted`, `Unmounted`,
`Remounted` and `RescanRequired` (ADR-0003); the catalogue task consumes them, a removal marks the affected roots
absent, and a returning volume triggers a rescan (FUN-017, PLT-011). A permission error on a directory
quarantines that subtree: its contents are not indexed, the subtree is listed as needing consent, and the rest of
the root continues to be indexed (PLT-012); a single unreadable file quarantines the file, not the directory. When
a scan meets a protected folder the server asks the platform for consent and, on refusal, records an explanation
naming the folder and the denial reason (PLT-017, UX-004).
- **Free space and schedule.** [DESIGN] The free-space value comes from the platform layer and, when exposed to
clients, is exposed through a documented vendor extension action absent from the standard service description
(INT-048); it is also part of the local diagnostics output (F-14). The configuration carries either a scan
interval or "manual only" (FUN-024); the catalogue task owns the timer and the explicit scan command (OPS-003)
enters the same path.
- **Platform traits.** [DESIGN] Every operating-system-specific fact the core needs arrives through a small,
documented trait, and every trait has a default implementation that reports "not supported here" so that the core
compiles and runs on a platform with no adapter (PLT-003; platform-integration §10.1 rules 2 and 4). The traits
expose *capabilities* — "case sensitivity of this volume", "this volume's identity" — and never platform names.
The core owns the semantics and the adapter owns the mechanism (platform-integration §10.1 rule 3).
- **Path assembly and encoding.** [DESIGN] A path is built from path components with the platform's own path type
and is never concatenated as a string; percent-encoding happens exactly once, on the way into a URL, in one
function (SEC-015, PLT-006). A name with a space, an accented character or an emoji therefore needs no special
case anywhere: the raw path is used verbatim for I/O and the NFC identity is used for the key (FUN-019, FUN-020).
- **Windows path handling.** [DESIGN] The traversal rejection covers both separator characters — `../` and `..\` —
because on Windows the backslash is the separator, and a check that only knows the forward slash is a check that
can be bypassed (SEC-002, PLT-007). Long paths additionally need the `longPathAware` manifest element and the
`LongPathsEnabled` registry value on the machine (PLT-015; platform-integration §9.4); the server does not work
around a path that is still too long, it reports the error (UX-004).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: an incremental rescan of a tree with 10 000 unchanged files re-reads no file contents
(FUN-006). Method: instrument the reader and assert a read count of zero for unchanged entries, on the
reference hardware of qa-performance §5.3 (PERF-003).
- [DESIGN] Target: at least **200 media files per second** are indexed with metadata extraction disabled
(FUN-005). Method: benchmark over a synthetic tree of ≥ 10 000 files, median and p95 reported (PERF-004,
PERF-011).
- [DESIGN] Target: a browse never triggers a directory scan (PERF-008), and a scan of a root containing a slow
or broken network share does not block browse requests (PERF-014). Method: code review plus an assertion that no
directory handle is opened during a browse, and a fixture with an unresponsive mount.
- [DESIGN] Target: resident memory stays below 50 MB with 10 000 indexed objects (PERF-001). Method: RSS
sampling over the benchmark run on the reference hardware.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effects are the folder names a client shows, the
contents that appear, and what the diagnostics report says about a drive that is missing.
- [UX-004] Every error the user can meet — root missing, drive unplugged, permission denied — shall say what
happened, why and what the user can do next, and shall name the root by its configured name rather than by its
path.
- [UX-010] The user-facing text shall speak of "folders" and "drives", not of roots, volumes, mount points or
normalisation.
- [UX-001] The feature is fully operable without a GUI; the scan command (OPS-003) and the status output
(OPS-002) are the surfaces.
- [PLT-017] A denied consent shall be explained with the name of the operating-system setting the user has to
change, not with an error code.
- [DOC-005] *Media root*, *symbolic link*, *alias*, *quarantine* and *normalisation* are explained at first use.

## 7. Security and privacy

- The filesystem is untrusted input (ARCHITECTURE §1 rule 3). File names and metadata are untrusted strings of
bounded length and unknown encoding (SEC-007).
- Containment is the boundary: canonicalise, resolve, contain, then open (SEC-001 … SEC-004). The check is
repeated after resolution because a link can change between check and use (SEC-003).
- The server is read-only towards the user's media (SEC-008, FUN-018) and never runs a shell (SEC-005).
- The server never exposes an absolute filesystem path to a client: the media URI scheme is opaque and the root
name is the only human-visible label (PLT-008, INT-062, ADR-0007).
- Paths, client addresses and identifiers are redacted in logs and diagnostics (OPS-007, SEC-094); the
diagnostics report contains the root names and the pseudonymised paths only (UX-011).
- The server stores no personal data beyond the local index, the local configuration and the local logs
(SEC-093); the index file has user-only permissions (SEC-095). Scanning causes no outbound connection (SEC-091).

## 8. Interoperability

- The catalogue a client sees is one tree, whatever the number of roots (FUN-002); the root name is the
container title (F-05, INT-066).
- Platform differences are absorbed below the protocol: a client must see the same behaviour on macOS, Linux and
Windows (PLT-010). The per-root detection of case sensitivity and normalisation is what makes that true
(PLT-005).
- [OBSERVED] A hard-linked file appears under two names but is one file; the catalogue de-duplicates on the
platform's file identity rather than on the path (platform-integration §5.7; ADR-0007). The rule itself is
specified by F-11 (FUN-020, FUN-022).
- [OPEN] Whether a volume that reappears under a different device identity must keep its old object ids, and
whether an "external volume allowlist" should exist as a user setting. See §12.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Configured root path does not exist at start-up | configuration error naming the root and the path; the server does not start | CFG-002, UX-004 |
| Root is unmounted at runtime | the root is marked absent, its entries are withdrawn, the other roots keep serving; diagnostics report it | FUN-007, FUN-017 |
| Root is replaced by a different volume | detected by volume identity; the entries are re-keyed and the system update id is bumped | FUN-007, PLT-011 |
| Volume reappears | the volume is re-scanned and the entries are restored | FUN-017, PLT-011 |
| Permission error on a subtree | the subtree is quarantined and listed as needing consent; the rest of the root is indexed | PLT-012 |
| Consent is refused by the operating system | the denial is explained; the subtree stays quarantined; the server keeps running | PLT-017, UX-004 |
| Symbolic link leaves the root, policy "off" | the link is refused and the attempt is logged | FUN-003, SEC-006 |
| Symbolic link leaves the root, policy "on" | the resolved target is still contained; a target outside the root is refused | SEC-001, SEC-003 |
| Finder alias points outside the root | refused, and handled by the same rule as a symbolic link | SEC-069 (see §12) |
| Path contains `..`, an encoded separator or a NUL | rejected before canonicalisation; no filesystem call is made | SEC-002 |
| File is deleted while it is being scanned | the entry is skipped; no error is raised | FUN-006 |
| File cannot be read | the file is quarantined; the rest of the scan continues | PLT-012, SEC-007 |
| A media URL carries a traversal attempt written with a backslash separator | it is rejected before canonicalisation, exactly as the forward-slash form is; no filesystem call is made | SEC-002, PLT-007 |
| A path is longer than the operating system's path limit | the open fails with the platform's own error, which is reported with the path redacted; the server does not shorten or rewrite the path to make it fit | PLT-007, PLT-015, OPS-007 |

A filesystem problem never takes down the HTTP or SSDP tasks: a bad root produces a diagnostic, never a panic
(UX-005).

## 10. Acceptance criteria and traceability

The criteria below were *planned* when this document was written. The rows that carry a
**Result** line were checked by a test that exists; the rows that carry **NOT RUN** name
the reason. The implementation decisions are in § 12.1.

**AC-01** — *Given* two configured roots, *when* the catalogue is browsed, *then* one tree is returned whose
  top-level containers are named after the roots, and the union of both roots' contents is present. Requirements:
  FUN-001, FUN-002 · Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/media_roots.rs`

**AC-02** — *Given* a symbolic link inside a root whose target is outside it, *when* the root is scanned, *then*
  the target is not indexed and the attempt is logged; *given* the same root with the per-root link policy enabled,
*then* the link is followed only to a target that still satisfies containment, and a target outside the root is
refused. Requirements: FUN-003, SEC-006, SEC-001, CFG-007 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/symlink_policy.rs`

**AC-03** — *Given* a root containing a hidden file and a `._name.mkv` sidecar, *when* the root is scanned, *then*
  neither appears in the catalogue; and *given* a media file without an extension, *then* it is indexed with the
  type determined from its content. Requirements: FUN-004, FUN-005 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/media_roots.rs`

**AC-04** — *Given* a tree of 10 000 unchanged files, *when* an incremental rescan runs, *then* no file content is
  read and the scan completes inside the documented budget. Requirements: FUN-006, PERF-003 · Verification:
  benchmark · Planned location: `crates/phairdlna-core/tests/incremental_scan.rs`

**AC-05** — *Given* a root whose volume is unmounted, *when* the status output is produced, *then* the root is
  reported as absent with its name, the other roots are unaffected and the catalogue still answers a browse.
  Requirements: FUN-007, FUN-017 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/volume_events.rs`

**AC-06** — *Given* a root whose volume reappears under a different volume identity, *when* the reappearance is
  processed, *then* the affected entries are re-keyed, the system update id changes and the catalogue is scanned
  again. Requirements: FUN-017, PLT-011 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/volume_events.rs`

**AC-07** — *Given* a running server, *when* the media directory is inspected, *then* no file inside a root was
  created, renamed or deleted, apart from files inside the configured cache directory; and *given* a file whose
  name is stored in a decomposed Unicode form, *then* it opens and the identity used for indexing is the NFC,
  case-folded form of the path rather than the raw form. Requirements: FUN-018, FUN-019, FUN-020, PLT-005 ·
  Verification: automated test · Planned location: `crates/phairdlna-core/tests/media_roots.rs`

**AC-08** — *Given* a file system that reports free space, *when* the free space is requested, *then* it is
  reported and it is not exposed through a standard ContentDirectory action. Requirements: FUN-023, INT-048 ·
  Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/content_directory.rs`

**AC-09** — *Given* the scan schedule set to an interval, *when* the interval elapses, *then* a rescan starts;
*given* the schedule set to manual only, *when* the interval would elapse, *then* no automatic scan starts and
only an explicit scan command does. Requirements: FUN-024, OPS-003 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_scan.rs`

**AC-10** — *Given* a media URL containing `..`, an encoded separator or a null byte, *when* it is requested,
*then* it is rejected before any filesystem call and no file outside the root is opened. Requirements: SEC-002,
SEC-001, SEC-015 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/path_containment.rs`

**AC-11** — *Given* a symbolic link that is repointed between the containment check and the open, *when* the file
  is opened, *then* containment is re-checked after resolution and the file outside the root is not opened; *given* a
  media file that is itself a symbolic link to a file outside the root, *then* the final component is not followed
  and the request is refused. Requirements: SEC-003, SEC-004 · Verification: negative test · Planned location:
  `crates/phairdlna-core/tests/path_containment.rs`

**AC-12** — *Given* a file name of unusual length or a name that is not valid UTF-8, *when* the directory is
  scanned, *then* the name is handled within its documented bounds and no unbounded allocation or log line occurs;
*given* a media file that is served, *then* it is opened read-only and the server never requests write
permission on the containing directory. Requirements: SEC-007, SEC-060, SEC-011, SEC-008 · Verification:
automated test · Planned location: `crates/phairdlna-core/tests/path_containment.rs`

**AC-13** — *Given* two roots on volumes with different case sensitivity and Unicode normalisation behaviour,
*when* both are scanned, *then* each root's behaviour is detected and applied to that root, and neither root's
assumption leaks into the other. Requirements: PLT-005 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/media_roots.rs`

**AC-14** — *Given* a subtree that the operating system refuses to read, *when* the root is scanned, *then* the
  subtree is quarantined and listed as needing consent, the rest of the root is indexed, and the server keeps
  serving the catalogue. Requirements: PLT-012, UX-005 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/media_roots.rs`

**AC-15** — *Given* a folder the operating system protects, *when* access is denied, *then* the server records an
  explanation naming the setting the user has to change, does not fail silently and does not attempt to work around
  the denial. Requirements: PLT-017, UX-004 · Verification: manual protocol · Planned location:
  `qa/device-matrix.toml`

**AC-16** — *Given* a request that reaches the media layer, *when* the request is handled, *then* no shell
  process is spawned and no command line is built from any file name or metadata value. Requirements: SEC-005 ·
  Verification: automated test · Planned location: `crates/phairdlna-core/tests/path_containment.rs`

**AC-17** — *Given* the platform layer, *when* the operating-system-specific behaviours the core needs are
enumerated, *then* each one is reached through a documented trait, no operating-system type appears in a core
signature, and a trait that the current platform does not implement has a default implementation that reports
"not supported" instead of failing to compile. Requirements: PLT-003, PLT-001 · Verification: reviewer check plus
an automated test of the default implementations · Planned location:
`crates/phairdlna-core/tests/platform_traits.rs`

**AC-18** — *Given* a media root whose path contains a space, a non-ASCII character and an emoji, *when* the root
is configured, scanned and served, *then* the folder appears in the catalogue under its own name, the file is served
over its media URL, and the media URL contains no unescaped character and no path fragment. Requirements: PLT-006,
FUN-001, INT-098, SEC-015 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/path_containment.rs`

**AC-19** — *Given* a media URL that carries a traversal attempt written with a backslash separator, *when* it is
requested on Windows, *then* it is rejected before any filesystem call, exactly as the forward-slash form is; and
*given* a path longer than the legacy path limit, *then* the server opens it when the long-path opt-in is in force
and reports a clear error when it is not. Requirements: PLT-007, SEC-002, PLT-015 · Verification: negative test
plus a manual Windows check · Planned location: `crates/phairdlna-core/tests/path_containment.rs` and
`qa/device-matrix.toml`

**AC-20** — *Given* the Windows build, *when* the executable manifest is inspected, *then* it declares
`longPathAware` as true, and the installation documentation states that the `LongPathsEnabled` registry value must
be set to `1` for the opt-in to take effect. Requirements: PLT-015, PLT-007 · Verification: automated test over
the shipped manifest plus a reviewer check of the installation documentation · Planned location:
`crates/phairdlna-cli/tests/windows_manifest.rs` and `docs/06-development/BUILD-AND-TEST.md`

## 11. Test strategy and dependencies

**Test locations.** A file that exists today says *exists*; a file that does not exist
yet says *planned*.

| Purpose | Path | State |
| --- | --- | --- |
| Root configuration, union catalogue, hidden-file rules | `crates/phairdlna-core/tests/media_roots.rs` | exists |
| More than one root: two identifiers, independent containment, a root inside a root | `crates/phairdlna-core/tests/media_roots_multiple.rs` | exists |
| What the scanner reports when it cannot read | `crates/phairdlna-core/tests/media_diagnostics.rs` | exists |
| Symbolic-link and alias policy | `crates/phairdlna-core/tests/symlink_policy.rs` | exists |
| Containment: traversal, NUL, encoded separators, re-check after resolution | `crates/phairdlna-core/tests/path_containment.rs` | planned (F-16) |
| Mount, unmount, remount and re-keying | `crates/phairdlna-core/tests/volume_events.rs` | exists |
| The events the platform reports | `crates/phairdlna-core/tests/volume_platform_events.rs` | exists |
| Incremental rescan behaviour | `crates/phairdlna-core/tests/incremental_scan.rs` | planned (M2) |
| Scanner unit tests | `crates/phairdlna-core/src/catalogue/scanner/mod.rs`, `.../scanner/walk.rs` | exists |
| The memory of the volumes of the roots | `crates/phairdlna-core/src/catalogue/volumes.rs` | exists |
| Scan command and schedule | `crates/phairdlna-cli/tests/cli_scan.rs` | planned (M1.11) |
| Device behaviour on removable volumes and protected folders | `qa/device-matrix.toml` | planned |
| Platform traits and their default implementations | `crates/phairdlna-core/tests/platform_traits.rs` | planned |
| Names with spaces, non-ASCII characters and emoji | `crates/phairdlna-core/tests/media_roots.rs` | exists |
| Windows long-path manifest, backslash traversal, long paths | `crates/phairdlna-cli/tests/windows_manifest.rs` | planned (M3) |

**Method.** The scanner and the containment rules are exercised against a temporary directory tree created by the
test itself, which makes symlinks, aliases and permission errors reproducible on every platform. The volume
events are driven through a scripted platform: the test writes down the volume identity and the fingerprint the
platform reports for a path (`crates/phairdlna-core/tests/support/volumes.rs`), so the mount and unmount logic is
tested without a physical drive; the physical drive remains a manual device check. Containment tests are
negative tests: they assert that no file outside the root was opened, using an instrumented open hook rather
than by inspecting the response alone.
The platform traits are exercised through their default implementations, so that a
platform without an adapter is covered by the same test as one with it; the Windows-specific
behaviour — the manifest element and the long-path opt-in — is a Windows-only check and is
recorded as `NOT RUN — no Windows host` elsewhere.

**Dependencies.** F-03 (the browse that reads the catalogue), F-05 (the DIDL-Lite rendering of root names), F-06
(the media URL that the containment rule guards), F-09 (the metadata extraction the scanner triggers), F-11 (the
identity key and de-duplication that share the raw path), F-14 (the diagnostics that report a missing root), the
`platform` traits of ADR-0003, and the macOS privacy behaviour of PLT-017.

## 12. Open questions and decisions

### 12.1 Decisions taken by the implementation (WP5)

These decisions are implemented and covered by a test. The one marked **[owner decision]** changes a
documented behaviour and is reported to the owner with this work package.

**D-01 — The scanner reads the disk once, and the catalogue answers from memory (FUN-005, PERF-008).**
The scan walks the root, reads the metadata of every entry and reads the first bytes of a file whose name
says nothing. The result is an in-memory tree; a browse touches nothing on the disk. *Implemented.*

**D-02 — The object identifier is the blake3 hash of volume, root key and relative identity, cut to 128 bits
(INT-062, ADR-0007). [owner decision]** The owner directed blake3. The hash input is

```text
volume identity ‖ 0x00 ‖ root key ‖ 0x00 ‖ relative identity
```

and the identifier is the first 32 hexadecimal characters of the digest. The zero byte separates the parts,
so no two different triples can produce the same input. The identifier carries no part of the path: a file
called `Secret Documentary.mkv` yields an identifier that contains neither the name nor the format
(`SEC-005`). "Opaque" means *carries no part of the path* and not *is unreadable*: a reader who knows the
volume identity, the root key and a file name can confirm a guess, because the hash is not a secret
(ADR-0007). *Implemented.*

**D-03 — The volume identity of the portable adapter is the constant `local`.** No adapter reports a volume
identity yet, and an identity that changed with every remount would break every identifier a client cached
(`INT-062`). The portable adapter therefore answers `local` for every path, and the identifier of a file does
not change when the volume is remounted or the library is moved to another machine. The cost is that two roots
on two different volumes share a volume identity; the root key keeps them apart (D-04). *Implemented; the
volume identity of the platform adapter is still to come (F-13, `PLT-005`).*

**D-04 — The root key is the position of the root in the configuration.** The name of a root is a label for a
person and is not part of the identifier, so renaming a root does not invalidate the identifiers below it. Two
roots of the same configuration keep their files apart because their keys differ. The cost of the position is
written down here: **a root that is inserted at the top of the configuration moves the keys of the roots below
it**, and the identifiers of their files change with it. A client that cached an identifier sees `701 No Such
Object` for the moved root. The alternative — the name as the key — would move the identifiers on a rename
instead, and the rename is the change a person makes more often. *Implemented.*

**D-05 — A folder of the library is a container and not an item (INT-042).** The ContentDirectory service
lists the children of a container as items, so the catalogue model gained `MediaKind::Folder` and
`ContentItem::child_count`; the DIDL writer turns an item of that kind into a `container` element that carries
the number of its children. *Implemented.*

**D-06 — The case sensitivity of a volume is asked of the platform (PLT-005).** The `FileSystem` trait gained
`case_sensitivity` for the identity, and the portable adapter answers per platform: a Unix volume is assumed
to be case-sensitive, a Windows volume case-insensitive. The safe direction was chosen: folding two names that
a case-sensitive volume keeps apart would merge two files into one, while keeping them apart splits at most a
file in two. The identity of a path is computed by `platform::identity_of_with`, which folds only when the
volume is insensitive. *Implemented; the per-volume detection of the macOS adapter is still to come.*

**D-07 — A hidden file is a name that begins with a dot, and an AppleDouble sidecar is hidden as well
(FUN-004).** The question is asked of the platform through the new `FileSystem::is_hidden`, whose default is
the dot convention; the scanner additionally treats a name that begins with `._` as hidden, because that is
what the resource fork of a macOS file is called. *Implemented.*

**D-08 — A file is placed by its extension first and by its content second (FUN-005).** The extension is what
the person who named the file meant; a file whose extension says nothing is sniffed (JPEG, PNG, Matroska, ISO
base media, MPEG audio, MPEG transport stream). A file the sniffer cannot place stays out of the tree, because
a file the client cannot play is worse than a file the client does not see. A playlist is the one exception:
it is indexed, although the server cannot deliver it. *Implemented.*

**D-09 — The scanner never writes, never leaves the root and is bounded (FUN-018, SEC-003).** The walk opens
directories and files to read them and writes nothing. A symbolic link is followed only when the policy says
so, and only when its target lies inside the canonical root. The depth of the walk is limited (32 by default),
and a folder that is reached twice is not walked again, so a cycle of links cannot make the server disappear.
A folder that is deeper than the limit is **reported** in the diagnostics and not read: the cut is never
silent. *Implemented.*

**D-10 — The diagnostics of the server name no path of the disk (SEC-094, `UX-011`).**
A diagnostic says which folder below the root it is about — "the folder "a/b" is reached twice" — and
never the path of the disk. The name of the root is part of the message, because the owner named it. The
command line prints the path of a root in clear text, because it shows the owner the configuration he
asked for; that is a local display and not a log. *Implemented.*

**D-11 — The link policy is a setting of the root, and the setting of the server is its default
(`CFG-007`).** A root may name a policy of its own; a root that names none follows the setting of the
server. The two roots of one library may therefore behave differently, which is what the requirement
asks for. *Implemented.*

**D-12 — The catalogue holds the target of a link, and not the link (`SEC-001`).**
A link that is indexed as its own entry keeps the name of the link, but the path the catalogue holds is
the path of the **target**. A link that is repointed after the scan therefore cannot change what the
catalogue serves. The containment is checked again when the file is opened; that check belongs to the
delivery layer (F-16). *Implemented.*

**D-13 — Two names of one file are one object (`FUN-011`, ADR-0007).**
The scanner asks the platform for the identity of a file and keeps one object per identity, so a hard
link is not counted twice. A file system that gives no identity to its files is not asked to answer, and
the file is indexed as it is. *Implemented.*

**D-14 — A root that cannot be read is reported, and the rest of the library stays browsable (FUN-007).**
A root that is missing, unreadable or not a folder is reported in the diagnostics and marked absent; the
content directory and the other roots stay browsable. What the implementation cannot yet detect is a root that is *replaced* by
another volume: that needs a volume identity the portable adapter does not have (D-03). *Implemented with the
stated limit.*

**D-15 — The catalogue remembers the volume of every root between two reads (`FUN-007`, `FUN-017`, `PLT-011`).**
The memory lives in `catalogue::volumes::VolumeRegistry` and holds, for every root, the identity of the volume,
the fingerprint of the path and whether the root could be read. A catalogue that is built once and thrown away
cannot tell a drive that was unplugged from a drive that was never there; a catalogue that remembers can. The
memory belongs to the caller of the catalogue — the daemon keeps it while it runs — and the catalogue is read
again through `Catalogue::refresh`, which returns what changed at the roots. *Implemented.*

**D-16 — The fingerprint of a path is what the platform observes right now, and it is not the identity of the
volume (`FUN-007`, `PLT-005`).** The `FileSystem` trait gained `volume_fingerprint`, whose default says "not
supported": a platform that cannot see the volume of a path must not pretend that it can. The portable adapter
answers the device the path lives on, which changes when a volume is unmounted and its place on the disk is taken
by a folder of another device. The fingerprint says only that *something* at the path is the same as it was, or
that it is not; whether it is the same **volume** is a question only a platform with a real volume identity can
answer, and that identity is still to come (F-13). *Implemented.*

**D-17 — A volume that comes back under another identity is re-keyed (`PLT-011`, `INT-062`).**
The identifiers of the catalogue carry the identity of the volume (D-02). A volume that comes back under another
identity therefore gives its files new identifiers by construction, and the identifier of the catalogue grows, so
that a client that polls it browses again. A volume that comes back under the *same* identity keeps its
identifiers, and a client that cached them keeps working. *Implemented.*

**D-18 — A root inside another root is published once (`FUN-017`).**
Two roots that overlap would publish the same file twice, under two identifiers. The outer root therefore leaves
the inner root alone: the inner root is published under its own key and with its own boundary, and the files
below it are indexed once. *Implemented.*

**D-19 — The daemon watches the roots and reads the catalogue again when something changed (`FUN-017`).**
The watch asks a cheap question — "did anything change at the roots?" — and the question is answered without
reading a single file of the library. Only when the answer is "yes" is the catalogue read again, and the
catalogue that was there is replaced as a whole, so that a root that is not there any more takes its entries with
it. The catalogue is held behind a lock, and a request reads it under that lock, so that a drive that comes back
while a client is browsing cannot tear the tree in half. The work of the watch runs on a thread of its own, so
that a network share that does not answer cannot block the runtime of the server. Two limits are written down
here: the watch sees the **roots**, not the files inside them — a file that is added inside a root is noticed by
the next full scan, and the schedule of the scans is the work of `FUN-024` (M2.4); and the watch asks the
**observation** of the roots, not the events of the platform, because the portable adapter has no native events.
The core honours the events of the platform through `Catalogue::apply_volume_event`, and the native watcher
(DiskArbitration) is the work of the platform adapter (F-13). *Implemented.*

**D-20 — Two roots may not share a name and may not share a path (`CFG-002`, `FUN-017`).**
A name is how a person and a client tell the roots apart, and a path says which folder is published. Two roots
with one name would be two folders with one label, and two roots with one path would publish the same files twice
under two identifiers. The configuration refuses both, and the message names the field. *Implemented.*

**What is verified, and what is not.** Every criterion of § 10 is listed. A criterion
that is *implemented* has a test that fails when the behaviour breaks; a criterion
that is *partly implemented* says what is missing; a criterion that is **NOT RUN**
names the reason, because a skip is never a pass.

| Criterion | Result | Where |
| --- | --- | --- |
| AC-01 (two roots, one tree) | implemented | `crates/phairdlna-core/tests/media_roots.rs`, `crates/phairdlna-core/tests/catalogue_identifiers.rs`, `crates/phairdlna-core/tests/media_roots_multiple.rs` |
| AC-02 (a link out of the root) | implemented | `crates/phairdlna-core/tests/symlink_policy.rs` |
| AC-03 (hidden files, content sniffing) | implemented | `crates/phairdlna-core/tests/media_roots.rs` |
| AC-04 (incremental rescan) | **NOT RUN** — M1.9 has no index and no incremental scan; the criterion belongs to the index (F-09, M2) |
| AC-05 (an unmounted volume) | implemented | `crates/phairdlna-core/tests/volume_events.rs::a_drive_that_is_unplugged_takes_its_files_with_it`, `...::the_other_roots_are_not_touched_by_the_loss_of_a_drive`, `crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_was_unmounted_takes_its_files_with_it`, `crates/phairdlna-cli/src/serve/watch.rs::tests::the_watch_reads_the_catalogue_again` — the root is reported as absent, its files leave the catalogue, the other roots stay browsable, the identifier of the catalogue grows, and the running daemon notices the drive that is gone |
| AC-06 (a volume that reappears under a new identity) | implemented | `crates/phairdlna-core/tests/volume_events.rs::a_drive_that_comes_back_under_another_identity_is_given_new_identifiers`, `...::a_volume_that_comes_back_under_another_identity_is_a_replacement`, `...::a_drive_that_comes_back_keeps_its_identifiers`, `crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_comes_back_is_found_again` — the entries are re-keyed, the system update id changes, a volume that returns under the same identity keeps its identifiers, and a volume that returns under another identity is reported as a replacement |
| AC-07 (nothing is written) | implemented | `crates/phairdlna-core/tests/media_roots.rs::the_scanner_never_writes_to_the_library`; the raw path of a file is kept as the operating system reported it (`crates/phairdlna-core/tests/media_roots_multiple.rs::the_path_of_a_file_is_the_path_of_the_disk`) |
| AC-08 (the free space of a root) | **NOT RUN** — `FUN-023` and the vendor extension of open question 1 below; not part of M1.9 |
| AC-09 (the scan schedule) | **NOT RUN** — the scheduler is M1.11, and the schedule of a root is not configured yet |
| AC-10 (a media URL that walks out) | **NOT RUN** — the containment of a client-supplied path is F-16 and is not part of M1.9 |
| AC-11 (a link repointed after the scan) | partly implemented — the catalogue holds the **target** of a link, so a link that is repointed after the scan cannot change what the catalogue serves (`crates/phairdlna-core/tests/symlink_policy.rs::a_link_that_is_repointed_after_the_scan_cannot_escape_the_root`); the re-check of the path when the file is opened is F-16 |
| AC-12 (a name that is not valid UTF-8) | **NOT RUN** — the scanner reads a name with `to_string_lossy`, so two names that differ only in an unreadable byte would collapse into one identity; macOS refuses such names, and the Linux behaviour needs a fixture (qa/device-matrix) |
| AC-13 (two roots with different case sensitivity) | **NOT RUN** — the portable adapter answers per platform (`crates/phairdlna-core/src/platform/local.rs`), the per-volume detection of the macOS adapter is missing (F-13) |
| AC-14 (a subtree the system refuses to read) | partly implemented — the folder is reported in the diagnostics (`crates/phairdlna-core/tests/media_diagnostics.rs`); the list of quarantined subtrees that need consent does not exist yet |
| AC-15 (a protected folder) | partly implemented — the refusal is recorded as a diagnostic; the quarantine list of AC-14 does not exist yet |
| AC-16 (no shell is involved) | implemented by construction — the scanner starts no process and takes no command; there is no test, because the test would only assert the absence of a dependency |
| AC-17 (the platform layer) | partly implemented — the traits carry the facts the scanner needs and the portable adapter answers them (`crates/phairdlna-core/src/platform/local.rs`); the macOS adapter is missing (F-13) |
| AC-18 (a root path with a space, a non-ASCII character and an emoji) | implemented | `crates/phairdlna-core/tests/media_roots.rs::a_root_with_an_awkward_name_is_scanned` |
| AC-19 (a traversal attempt with a backslash) | **NOT RUN** — the containment of a client-supplied path is F-16 |
| AC-20 (the Windows manifest) | **NOT RUN** — the Windows build is M3 |

### 12.2 Open questions

1. **[OPEN] FUN-023 versus INT-048 — a genuine tension in the baseline.** FUN-023 asks for the free space of a
root "through the ContentDirectory service"; `GetFreeSpace` is **not** a ContentDirectory 1–4 action, and INT-048
forbids advertising a non-standard ContentDirectory action, while the research states that a client-specific
extension must be kept in its own namespace and documented as non-standard (standards §0.3, §8.3 C12).
*Proposed reading, used in this document:* the value is exposed through a documented vendor extension action in
its own namespace and through the local diagnostics, and is absent from the standard service description.
*Closes when:* the owner confirms the reading or amends FUN-023.
2. **[OPEN] The symbolic-link default contradicts the platform research.** The baseline disables link following
by default (CFG-007, FUN-003, SEC-006); the platform research recommends following links for library roots,
because users symlink to NAS mounts (platform-integration §11.1 D13). The baseline wins here, but the decision
to offer only a per-root boolean, or also a per-root allowlist of permitted targets, is open. *Closes when:* the
owner fixes the setting.
3. **[OPEN] macOS Finder aliases, and the ownership of SEC-069.** Canonicalisation resolves symbolic links but not
aliases, and whether `O_NOFOLLOW` has any effect on an alias is not established (security-landscape §7.4). The
baseline requirement SEC-069 covers aliases but is **not owned by any feature in this set**; this document
describes the behaviour it implies but does not claim the requirement. *Closes when:* a macOS fixture
establishes the behaviour of `canonicalize` and `O_NOFOLLOW` on an alias, and the ownership of SEC-069 is
assigned.
4. **[OPEN] An "external volume allowlist".** The containment rule of the security research permits a resolved
target that is on an explicitly allowed external-volume list (security-landscape §7.4, M-05); the baseline has
no requirement for such a list. *Proposed requirement, not in the baseline:* `SEC-009 — The server shall permit a
resolved media path that lies outside the root only when the target volume is on a user-configured allowlist.`
Rationale: without it, a user who links to a NAS has no supported way to serve it, and the alternative —
disabling containment — is not acceptable. *Closes when:* the owner accepts or rejects it.
5. **[OPEN] Volume re-keying and stable object ids.** The object id is derived from the volume identity and the
relative path (ADR-0007). When a volume reappears under a different identity, the ids of its entries change by
construction, and a client holding an old id receives `701 No Such Object` (INT-039). The re-keying itself is
implemented and tested (D-17); what is still open is the grace period. *Closes when:* the owner
decides whether a grace-period mapping from the old volume identity to the new one is required.
6. **[OPEN] Where the Windows installation requirements are documented.** PLT-015
   requires the `LongPathsEnabled` setting to be documented, and the repository has no installation
   guide yet — only `docs/06-development/BUILD-AND-TEST.md`. *Proposed reading:* the Windows
   installation section of that document carries the `longPathAware` manifest note and the
   `LongPathsEnabled` instruction, until a dedicated installation guide exists.
   *Closes when:* the owner decides where the installation documentation lives.
7. **[OPEN] The quarantine vocabulary.** "Quarantined subtree that needs consent" (PLT-012) is used here for a
permission error, while the same word is used by the security research for a file whose metadata parsing failed
(SEC-066). *Proposed decision:* keep one word and one meaning, or distinguish "not readable" from "not
parseable" in the user-facing text. *Closes when:* the owner picks one.
