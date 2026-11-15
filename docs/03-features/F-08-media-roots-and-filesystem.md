# F-08 — Media roots and filesystem safety

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-001, FUN-002, FUN-003, FUN-004, FUN-005, FUN-006, FUN-007, FUN-017, FUN-018, FUN-019, FUN-023, FUN-024, SEC-001, SEC-002, SEC-003, SEC-004, SEC-005, SEC-006, SEC-007, SEC-008, PLT-003, PLT-005, PLT-006, PLT-007, PLT-011, PLT-012, PLT-015, PLT-017
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

All criteria below are *planned*; no test file exists yet.

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

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Root configuration, union catalogue, hidden-file rules | `crates/phairdlna-core/tests/media_roots.rs` |
| Containment: traversal, NUL, encoded separators, re-check after resolution | `crates/phairdlna-core/tests/path_containment.rs` |
| Symbolic-link and alias policy | `crates/phairdlna-core/tests/symlink_policy.rs` |
| Mount, unmount, remount and re-keying | `crates/phairdlna-core/tests/volume_events.rs` |
| Incremental rescan behaviour | `crates/phairdlna-core/tests/incremental_scan.rs` |
| Scanner unit tests | `crates/phairdlna-core/src/catalogue/scanner.rs` (inline `#[cfg(test)]` module) |
| Scan command and schedule | `crates/phairdlna-cli/tests/cli_scan.rs` |
| Device behaviour on removable volumes and protected folders | `qa/device-matrix.toml` |
| Platform traits and their default implementations | `crates/phairdlna-core/tests/platform_traits.rs` |
| Names with spaces, non-ASCII characters and emoji | `crates/phairdlna-core/tests/path_containment.rs` |
| Windows long-path manifest, backslash traversal, long paths | `crates/phairdlna-cli/tests/windows_manifest.rs` |

**Method.** The scanner and the containment rules are exercised against a temporary directory tree created by the
test itself, which makes symlinks, aliases and permission errors reproducible on every platform. The volume
events are driven through the `VolumeWatcher` trait with a scripted event source, so the mount and unmount logic
is tested without a physical drive; the physical drive remains a manual device check. Containment tests are
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
construction, and a client holding an old id receives `701 No Such Object` (INT-039). *Closes when:* the owner
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
