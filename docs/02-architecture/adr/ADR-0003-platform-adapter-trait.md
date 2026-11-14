# ADR-0003 — Small orthogonal platform traits keep the core portable

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `PLT-001`, `PLT-003` … `PLT-005`, `PLT-011` … `PLT-018`, `FUN-019`, `FUN-020`

## Context

A media server touches the operating system in a surprisingly small number of places:
it enumerates network interfaces, resolves paths on volumes, notices when a volume is
mounted or unmounted, asks for the user's consent when the OS protects a folder, finds
per-user directories, registers itself as a service and writes log records.

The risk is not the *count* of these operations but their *variation*:

- **Volume identity differs.** On macOS a volume has a stable identifier and a device
  id; on Linux the same comes from `/proc/self/mountinfo` and `statfs`; on Windows
  from the volume serial number.
- **Path semantics differ, and the differences are subtle.** APFS is
  **normalisation-insensitive** (NFC and NFD spellings are the same file) but
  **preserves the on-disk form**; HFS+ stores names decomposed; SMB uses precomposed
  form; Linux file systems are byte-preserving, so NFC and NFD are *two different
  files* there. A catalogue that keys on the raw path therefore either loses files or
  creates duplicates, depending on the volume.
- **Permissions differ.** macOS applies TCC (privacy prompts for Desktop, Documents,
  Downloads, network and removable volumes), Linux uses POSIX permissions and optional
  polkit, Windows uses ACLs and elevation.
- **Service models differ.** macOS uses launchd (daemon vs. agent — and the
  local-network permission exempts a daemon but not an agent), Linux uses systemd,
  Windows uses a Service in session 0.

If the core encodes any of these directly, it stops being portable and the
cross-platform build matrix becomes a wish.

## Decision

The core defines **small orthogonal traits** — not one large trait — plus a `Platform`
facade that bundles them. No Apple, Linux or Windows type appears in any signature;
the platform-specific crates are the only place such types exist.

| Trait | Purpose |
| --- | --- |
| `Network` | Enumerate interfaces, join a multicast group, report the local-network permission state |
| `FileSystem` | Turn an OS path into a `MediaPath { raw, identity, volume, display }`, report case sensitivity and normalisation behaviour, provide a stable file identity |
| `VolumeWatcher` | Emit `Mounted` / `Unmounted` / `Remounted` / `RescanRequired` events |
| `Consent` | Request user consent for a protected folder and explain a denial |
| `UserEnvironment` | Locate the per-user configuration and data directories |
| `ServiceManager` | Describe and install the service (including macOS bundle identifiers and readiness notification) |
| `LogSink` | Emit a log record to the platform's log facility |
| `UserNotifier` | Show a user-facing notification; unsupported when headless |

Four rules make the abstraction work:

1. **The traits return data, not behaviour.** The core decides what to do with a
   volume identity; the platform only tells it what the volume *is*.
2. **Every trait has a default implementation** that returns "not supported" or a
   conservative fallback. A new target therefore compiles without writing a platform
   module, and the core degrades explicitly instead of misbehaving silently.
3. **The raw path is never normalised.** `MediaPath.raw` is the path exactly as the OS
   reported it and is used verbatim for every file operation. `MediaPath.identity`
   (NFC + per-component case fold, scoped by the volume) is a separate, derived value
   used for indexing and matching. Re-normalising a path before use can break access
   to a file whose stored name is in the other form.
4. **Uniqueness comes from the file, not from the path.** The pair
   (volume id, device id) identifies a file; the path is only a way to reach it. This
   is what makes the catalogue correct across normalisation changes and hard links.

Case sensitivity, normalisation behaviour and volume identity are **detected per
root**, not assumed globally: one server may serve a case-sensitive Linux root and a
case-insensitive APFS volume at the same time.

## Consequences

- The core compiles on macOS, Linux and Windows from one source tree, and the build
  matrix proves it.
- Adding a platform means writing one module and one test — no changes in the core.
- The abstraction has a cost: a platform-dependent behaviour that does not fit a trait
  forces a widening of the traits. That is deliberate friction — it must be argued for
  in an ADR, and the "not supported" default means a missing implementation degrades
  instead of misbehaving.
- Because the traits are small and orthogonal, they are individually testable: a
  `FileSystem` test suite can run against a temporary directory on any platform.
- The `MediaPath` type becomes the central vocabulary of the catalogue. It must
  therefore be defined once, in the core, and used consistently.

## Alternatives considered

- **`cfg(target_os)` inside the core.** Rejected: the portability claim would become
  unverifiable, and every branch would need its own review.
- **One large `Platform` trait with every method.** Rejected: a trait with thirty
  methods forces every platform to implement methods it cannot support, and the
  "not supported" arms multiply.
- **A full virtual file system.** Rejected: the core needs nine facts about a
  filesystem, not a virtual filesystem; a full abstraction would be a large surface
  with no user-visible benefit.
- **Assuming POSIX everywhere.** Rejected: Windows is a requirement, and the
  normalisation trap would silently corrupt catalogue identity.

## Revisit condition

Revisit if a capability cannot be expressed as "return data", if a supported platform
needs behaviour (not just a fact) from the OS layer, or if the `MediaPath` type grows
more than a handful of fields.
