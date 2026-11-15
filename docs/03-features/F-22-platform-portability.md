# F-22 — Platform portability and the build matrix

Status: Draft
Owner role: Architect
Version/date: 0.1 / 2026-11-15
Requirements: PLT-001 … PLT-003, PLT-005 … PLT-018

Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` — a citation such as `standards §8.3 C12` points at that file and section.

---

## 1. Purpose and user story

A media server has to run on three operating systems, and on two architectures
inside one of them. Everything else in this project is written once, in portable
Rust; this feature owns the boundary that makes that possible, and the build matrix
that proves it.

**User story.** As someone who runs a server at home and a second one on a Linux
box, I want the same product with the same behaviour on both, so that a folder I
share from the Mac is browsable from the television in exactly the same way — and as
a contributor, I want to know that my change did not break the platform I cannot
test today.

Two things make this harder than it sounds, and both are about **the operating system
lying to you**:

1. **Path semantics differ.** macOS APFS is case-insensitive by default and
   *normalisation-insensitive* — two spellings of the same name are the same file —
   but it preserves whichever normalisation form the file was written in. Linux file
   systems are byte-preserving, so the same two spellings are two different files
   there. Windows NTFS is case-insensitive and does not normalise.
   `[OBSERVED]` (platform-integration §5.1)
2. **The same operation has different names.** A mounted volume is a device node on
   Linux, a mount point plus a notification on macOS and a drive letter plus a
   serial number on Windows. `[OBSERVED]` (platform-integration §6.1)

A server that assumes one of these behaves wrongly on the others, in ways that only
appear on the platform nobody tested.

---

## 2. Scope

**In scope**

- The rule that the core carries no OS-specific dependency, and the crate boundaries
  that enforce it (`PLT-001`, `PLT-002`, `PLT-003`).
- The detection of path semantics per media root: case sensitivity, Unicode
  normalisation, volume identity (`PLT-005`, `PLT-006`).
- The treatment of the raw path versus the normalised identity (`FUN-019`,
  `FUN-020` — owned by F-08).
- Path handling on Windows: long paths, both separator characters, symlink
  creation (`PLT-007`, `PLT-015`).
- The native service integration on each platform (`PLT-013`, `OPS-009`).
- The consent model where the operating system protects a folder (`PLT-017`).
- The distribution and packaging shape per platform (`PLT-009`, `PLT-016`).
- The build matrix that proves the portability claim (`PLT-002`).

**Explicitly out of scope**

- The behaviour of the server on a platform: that belongs to the features that
  implement it.
- The user interface of the macOS application (F-15).
- The service packaging details of M3, beyond the shape required here.

---

## 3. Functional requirements

**PLT-001** — The core crate shall not depend on any OS-specific crate and shall
compile on macOS, Linux and Windows from the same source. `[DESIGN]` The dependency
direction is checked mechanically by gate QG-28.

**PLT-002** — The project shall provide a build matrix covering macOS (arm64,
x86_64), Linux (x86_64) and Windows (x86_64). `[DESIGN]` The matrix also tests the
minimum supported Rust version, so that "portable" does not quietly mean "portable on
the version you happen to have".

**PLT-003** — All OS-specific behaviour shall be reachable through a documented trait
with a default implementation for unsupported platforms. `[DESIGN]` ADR-0003 defines
eight such traits; the default reports "not supported", so a missing implementation
degrades explicitly instead of misbehaving silently.

**PLT-005** — The server shall detect the case sensitivity, the Unicode normalisation
behaviour and the volume identity of each media root instead of assuming them.
`[OBSIGNED]` The detection is per root, because one server may serve a
case-sensitive Linux root and a case-insensitive APFS volume at the same time.

**PLT-006** — The server shall treat a path as case-insensitive only on
case-insensitive file systems, and shall not collapse two distinct files on a
case-sensitive volume into one catalogue entry. `[DESIGN]`

**PLT-007** — Windows path handling shall support long paths and shall reject path
traversal attempts using both separator characters. `[DESIGN]` A backslash is a
separator on Windows and a legal file-name character elsewhere; the two must not be
confused.

**PLT-009** — The build shall not require a C toolchain beyond what the platform
already provides, and any native dependency shall be documented. `[DESIGN]`

**PLT-011** — The server shall treat the mount, unmount and remount of a volume as a
first-class event and shall re-key the affected catalogue entries when a volume
reappears under a different identity. `[DESIGN]`

**PLT-012** — The server shall treat a permission error on a subtree as a quarantined
subtree that needs consent, and shall continue indexing the rest of the root.
`[DESIGN]` A single unreadable folder must not end a scan.

**PLT-013** — On macOS the always-on server shall run as a system-context launchd
daemon and the user interface as a per-user launchd agent, because the operating
system's local-network permission exempts daemons but not agents. `[OBSERVED]`
(platform-integration §1.2)

**PLT-014** — The macOS application shall be built as a universal binary for Apple
Silicon and Intel. `[DESIGN]`

**PLT-015** — The Windows installation shall declare long-path support in the
executable manifest and shall document that the system-wide `LongPathsEnabled`
setting is also required. `[OBSERVED]` Both are needed; either alone does nothing.

**PLT-016** — The Linux distribution shall provide native packages and shall not use
a filesystem-sandboxed package format as its primary distribution channel, because
such a sandbox blocks access to arbitrary media folders. `[DESIGN]`

**PLT-017** — The server shall obtain the user's consent for folders that the
operating system protects (macOS privacy prompts, Linux polkit, Windows elevation),
and shall explain the denial instead of failing silently. `[DESIGN]`

**PLT-018** — The server shall not require elevated privileges to serve media from the
user's own directories. `[DESIGN]`

---

## 4. Technical requirements

**The platform traits.** ADR-0003 defines eight: `Network`, `FileSystem`,
`VolumeWatcher`, `Consent`, `UserEnvironment`, `ServiceManager`, `LogSink`,
`UserNotifier`. Each returns data rather than performing behaviour, and each has a
default implementation that reports "not supported".

**The path model.** A path is a `MediaPath` with four parts (ADR-0003, rule 3):

| Field | What it is | Used for |
| --- | --- | --- |
| `raw` | The path exactly as the operating system reported it | Every file operation, verbatim. Never normalised |
| `identity` | Unicode NFC with a per-component case fold, scoped by the volume | Indexing, de-duplication, matching |
| `volume` | The stable identity of the volume it lives on | Scoping the identity; detecting a remount |
| `display` | The name shown to a user | Presentation only |

**Why the raw path is never normalised.** On macOS the on-disk form can differ from
the form a tool used to write the file (SMB shares store precomposed names, HFS+
stores decomposed ones), and APFS preserves whichever form is on disk. A path that
was cached as text can therefore stop resolving even though the file never moved.
`[OBSERVED]` (platform-integration §5.1)

**Uniqueness is not identity.** Two catalogue entries are the same file when their
`(volume id, device id)` pairs match, not when their paths match. A hard-linked file
under two names is one file with two paths. `FUN-020`, ADR-0007.

**The service model per platform.**

| Platform | The always-on server | The user interface |
| --- | --- | --- |
| macOS | launchd **daemon** (system context) — `PLT-013` | launchd **agent** (user session) |
| Linux | a systemd **system** unit, `Type=notify` | the same unit, or a user unit with lingering |
| Windows | a Windows **Service** (session 0) | a tray application |

**The consent model.** Where the operating system protects a folder, the server asks
for consent through the `Consent` trait and reports the refusal as a quarantined
subtree. It never attempts to work around a refusal, and it never asks for more
privilege than the task needs (`PLT-017`, `PLT-018`).

---

## 5. Performance requirements

| Requirement | Budget | Note |
| --- | --- | --- |
| `PLT-001` | The core compiles for every target in the matrix | No performance dimension |
| `PLT-005` | The per-root detection happens once per root, not per file | Measured with the scan of a root with 10 000 files |
| `PLT-011` | A volume event is handled within one second of delivery | A remount must not stall a browse |
| `PLT-013` | The daemon starts within the service manager's start-up budget | The first advertisement follows, not the other way round |

The platform layer must not add measurable overhead to the hot paths: browse, seek and
transfer are measured against `PERF-005` and `PERF-006`, and the platform code is on
the path of all three.

---

## 6. Usability and accessibility

- A user on macOS must never be asked for a permission the server does not need.
  The first-run output states what the server accesses and why.
- When the operating system refuses access, the message names the folder and the
  permission, in the user's language, and says what to do next (`UX-004`).
- The service must be startable and stoppable by the user without a terminal
  (`PLT-013`, `MB-008`).
- The user interface must not expose the platform differences as concepts: nobody
  needs to know what a "volume identity" is to browse a folder.

---

## 7. Security and privacy

| Property | Requirement | Threat |
| --- | --- | --- |
| The core pulls in no OS-specific dependency, so it pulls in no unexpected code | `PLT-001`, `PLT-003` | `SEC-TH-19` |
| A path is never re-normalised before use, so a traversal cannot be smuggled through a normalisation difference | `PLT-006`, `SEC-002` | `SEC-TH-17` |
| Both separator characters are rejected in a URL path | `PLT-007`, `SEC-002` | `SEC-TH-17` |
| A symbolic link or alias is resolved and re-checked against the root | `PLT-005`, `SEC-003`, `SEC-069` | `SEC-TH-18` |
| The server never asks for elevated privilege to read the user's own folders | `PLT-018` | — |
| A refused permission is explained, never worked around | `PLT-017` | — |

**The sandbox is a security boundary, not a packaging convenience.** On macOS the
local-network permission decides whether the server can announce itself at all; the
daemon/agent split is what keeps that working without a prompt in the user's face.

---

## 8. Interoperability

Portability must not cost compatibility: the protocol behaviour, the object
identifiers and the ordering are identical on every platform (`PLT-010`). A client
cannot tell which operating system the server runs on, and the interoperability
matrix does not record the operating system as a variable.

The conformance fixture suite is the proof: the same fixtures must produce the same
bytes on every target, and the build matrix runs them (`PLT-002`).

---

## 9. Error and recovery behaviour

| Situation | Expected behaviour | Requirement |
| --- | --- | --- |
| A root lives on a case-sensitive volume and the catalogue holds two files whose names differ only in case | Both are indexed and browsable; neither hides the other | `PLT-006` |
| A root lives on a case-insensitive volume | One entry, whichever spelling the file system returns | `PLT-006` |
| The same file is reached through two different spellings of its name | One catalogue entry, because the identity is normalised and the file identity matches | `PLT-005`, `FUN-020` |
| A volume is unmounted and remounted | The entries are re-keyed; the user sees the same library, not a duplicate one | `PLT-011` |
| A subtree is not readable | The subtree is quarantined, the rest of the scan continues, and the reason is reported | `PLT-012` |
| The operating system refuses a folder | The refusal is explained; nothing is worked around | `PLT-017` |
| The machine sleeps and wakes | The listeners are rebound on the permitted interfaces; discovery resumes there and nowhere else | `PLT-013`, `LIFE-002` |
| A path exceeds the platform's limit | The platform's own error is reported, with the path redacted; the server does not rewrite the path to make it fit | `PLT-007`, `PLT-015` |

---

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

| ID | Scenario | Requirement | Verification | Planned test |
| --- | --- | --- | --- | --- |
| **AC-01** | Given the core crate's dependency graph, when it is inspected, then no dependency is OS-specific and no crate depends downwards | `PLT-001`, `PLT-003` | Automated test over the `Cargo.toml` graph (gate QG-28) | `scripts/check-crate-boundaries.sh` |
| **AC-02** | Given the build matrix, when it runs, then the workspace builds and the tests pass on macOS arm64, macOS x86_64, Linux x86_64 and Windows x86_64, and on the minimum supported Rust version | `PLT-002` | Build matrix | `.github/workflows/ci-matrix.yml` |
| **AC-03** | Given a root on a case-sensitive volume with two files whose names differ only in case, when the root is indexed, then both appear and neither hides the other | `PLT-006` | Automated test | `crates/phairdlna-core/tests/path_semantics.rs` |
| **AC-04** | Given a root on a case-insensitive volume, when the same file is reached through two spellings of its name, then exactly one catalogue entry exists | `PLT-006`, `FUN-020` | Automated test | `crates/phairdlna-core/tests/path_semantics.rs` |
| **AC-05** | Given a media path, when it is opened, then the path handed to the operating system is byte-identical to the path the operating system reported | `PLT-005` | Property test | `crates/phairdlna-core/tests/properties/raw_path_preserved.rs` |
| **AC-06** | Given a media URL containing a backslash traversal, when it is requested, then it is rejected before canonicalisation and no filesystem call is made | `PLT-007`, `SEC-002` | Negative test | `crates/phairdlna-core/tests/path_traversal.rs` |
| **AC-07** | Given a path longer than the platform limit, when it is opened, then the platform error is reported with the path redacted and the path is not rewritten | `PLT-007`, `PLT-015` | Automated test | `crates/phairdlna-core/tests/long_paths.rs` |
| **AC-08** | Given a volume that is unmounted and remounted, when the catalogue is consulted, then the entries are re-keyed and no duplicate appears | `PLT-011` | Integration test | `crates/phairdlna-core/tests/volume_remount.rs` |
| **AC-09** | Given a subtree the operating system refuses, when the root is scanned, then the subtree is quarantined, the rest is indexed and the reason is reported | `PLT-012`, `PLT-017` | Integration test | `crates/phairdlna-core/tests/quarantined_subtree.rs` |
| **AC-10** | Given the macOS installation, when the server runs as a launchd daemon, then it is exempt from the local-network permission and the user interface as an agent is not | `PLT-013` | Device test on macOS 15 or later | `docs/05-quality/INTEROPERABILITY-MATRIX.md` § 4.2 |
| **AC-11** | Given the Windows installation, when the manifest is inspected, then `longPathAware` is present, and the documentation states that `LongPathsEnabled` is also required | `PLT-015` | Automated test over the manifest plus a documentation check | `crates/phairdlna-cli/tests/windows_manifest.rs` |
| **AC-12** | Given the Linux packaging, when the primary distribution channel is inspected, then it is a native package and not a filesystem-sandboxed format | `PLT-016` | Manual review | `docs/08-releases/RELEASE-POLICY.md` |
| **AC-13** | Given the protocol fixture suite, when it runs on two different platforms, then the produced bytes are identical | `PLT-010` | Automated test | `crates/phairdlna-core/tests/protocol_parity.rs` |
| **AC-14** | Given a folder the user has no permission for, when the server starts, then it asks through the platform's own consent mechanism and explains a refusal | `PLT-017`, `PLT-018` | Device test | `docs/05-quality/INTEROPERABILITY-MATRIX.md` § 4.2 |

---

## 11. Test strategy and dependencies

**Dependencies**

- ADR-0003 (the platform traits) and ADR-0007 (the identifier model) are the
  normative basis of this specification.
- The `Platform` traits are implemented by F-08 (paths and volumes), F-13 (the
  service) and F-15 (the macOS application).

**Layers**

| Layer | What it covers |
| --- | --- |
| Unit | The `MediaPath` construction, the identity normalisation, the volume comparison |
| Property | The raw path is preserved byte-for-byte; the identity is deterministic; a canonicalised path never escapes its root |
| Integration | A volume that appears and disappears; a subtree that is refused; a service that is installed and started |
| Build matrix | The same source compiles and the same fixtures pass on every target |
| Device | The macOS local-network permission, the Windows firewall rule, the Linux service unit |

**The negative tests matter more here than anywhere else**: the portability bugs are
the ones that only appear on the platform nobody tests.

---

## 12. Open questions and decisions

| ID | Question | What would close it |
| --- | --- | --- |
| F22-Q-01 | Which macOS version is the minimum? | Owner decision OD-05. The recommendation is macOS 15 and later, because the local-network permission only exists there and the daemon exemption is what keeps discovery working |
| F22-Q-02 | Which Linux distributions are supported in the first release? | A decision when M3 packaging starts; the requirement is only that native packages exist |
| F22-Q-03 | Is Windows in scope for the first stable release, or only the headless server? | Owner decision; the baseline requires the build to succeed and the service to install, not a graphical application |
| F22-Q-04 | Should a case-sensitive volume be supported at all on macOS? | A product decision. It is unusual, and the detection machinery handles it either way |
| F22-Q-05 | What is the exact set of volume events each platform reports? | A platform experiment per operating system; the research covers the API surface, not the delivery guarantees |
