# Architecture — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Architect
Decisions: `adr/` — one ADR per decision that a future contributor might otherwise
re-litigate.
Requirements: `../01-product/TECHNICAL-REQUIREMENTS.md`

This document describes *how the system is built*. It is not a tutorial of DLNA; the
protocol obligations are in the requirement catalogue, and each feature's behaviour is
in `../03-features/`.

---

## 1. Design rules

1. **The core is portable.** Nothing in `phairdlna-core` may depend on an
   OS-specific crate. Platform differences enter through one narrow trait
   (ADR-0003).
2. **The network is untrusted.** Every byte that arrives over a socket is parsed by a
   bounded parser. There is one place where trust is established, and it is explicit.
3. **The filesystem is untrusted.** Paths are canonicalised and re-checked against the
   root, including after symbolic-link resolution.
4. **Small modules, one responsibility.** A file that cannot be described in one
   sentence does not belong in the tree.
5. **The catalogue is a cache, the filesystem is the truth.** The index may be deleted
   and rebuilt; a rebuild must not change what the user sees.
6. **Direct play by default.** Nothing is transcoded unless the user asked for it.

---

## 2. Workspace layout

```
PhairDLNA/
├── Cargo.toml                 workspace
├── crates/
│   ├── phairdlna-core/        portable domain core (no OS-specific dependencies)
│   │   └── src/
│   │       ├── config/        typed, validated, versioned configuration
│   │       ├── upnp/          device model, SOAP dispatch, service state
│   │       ├── ssdp/          discovery: M-SEARCH, NOTIFY, interface selection
│   │       ├── didl/           DIDL-Lite generation (objects, escaping, classes)
│   │       ├── catalogue/      library model, index, scanning, identifiers
│   │       ├── media/          media inspection, metadata, artwork
│   │       ├── http/           media delivery, range handling, MIME
│   │       ├── security/       path containment, redaction, limits
│   │       ├── platform/       platform trait + per-OS implementations
│   │       ├── logging/        structured logging + redaction
│   │       └── error.rs        the crate error type
│   ├── phairdlna-cli/         headless daemon and command line
│   ├── phairdlna-admin/       optional loopback administration surface
│   └── phairdlna-mac/         optional native macOS menu bar application
├── scripts/                   repository gates and generators
└── docs/                      the documentation tree
```

**Dependency direction is one-way:**

```
phairdlna-mac  ─┐
phairdlna-admin ┼──▶ phairdlna-cli ──▶ phairdlna-core
phairdlna-cli  ─┘
```

`phairdlna-core` depends on no sibling crate. `phairdlna-cli` contains the process
lifecycle; `phairdlna-admin` and `phairdlna-mac` are optional front ends that talk to
the core through its public API.

The boundary check (`scripts/check-crate-boundaries.sh`, gate QG-07) is mechanical: it
parses the `Cargo.toml` dependency graph and fails when a crate depends "downwards".

---

## 3. Runtime model

The server is one process with a small number of long-lived tasks and a strict
shutdown path.

```
                     ┌──────────────────────────────────────────┐
   UDP 1900          │            phairdlna-core                │
   ┌────────┐        │                                          │
   │ ssdp   │◀──────▶│  ssdp task      (multicast + unicast)    │
   └────────┘        │                                          │
   ┌────────┐        │  http task      (media + admin ports)   │
   │  http  │◀──────▶│                                          │
   └────────┘        │  catalogue task (scan, index, metadata)  │
                     │                                          │
                     │  event bus (in-process, bounded)         │
                     └──────────────────────────────────────────┘
                                      │
                              ┌───────┴────────┐
                              │  SQLite index  │  (cache, rebuildable)
                              └────────────────┘
```

- **One event bus.** Modules communicate through a bounded in-process channel, never by
  calling into each other's internals. The bus carries catalogue-change events,
  transfer lifecycle events and log records.
- **Bounded everything.** Every channel has a capacity. A full channel drops the
  *lowest-priority* event and increments a counter — it never blocks a network task.
- **Graceful shutdown.** `SIGTERM`/`SIGINT` stop the SSDP task first (sending
  `ssdp:byebye`), then the HTTP task drains open transfers for a bounded time, then the
  catalogue task flushes the index, then the process exits with code 0.

---

## 4. Module responsibilities

### 4.1 `config`

Owns the configuration schema, its defaults, its validation and its migrations. No
other module reads an environment variable or a configuration file directly. The
schema is a versioned document; an older known version is migrated in memory, and the
migration is logged.

### 4.2 `ssdp`

Owns the socket, the `M-SEARCH` matching rules, the advertisement timer and the
interface selection. It knows nothing about the catalogue. It publishes the device
description URL and the advertised identity through a small trait so that the same
logic can be unit-tested without a network.

### 4.3 `upnp`

Owns the device model, the service list, the SCPD generation and the SOAP dispatcher.
The dispatcher is a table of actions; every action handler is a plain function with a
typed request and a typed response. An unknown action produces a UPnP fault, never a
panic.

### 4.4 `didl`

Owns the mapping from catalogue objects to DIDL-Lite. Escaping happens in exactly one
place, with a round-trip test. The object model carries the UPnP class, the resource
list and the metadata; the XML writer is dumb and deterministic.

### 4.5 `catalogue`

Owns the library model, the object identifier scheme, the SQLite index, the scanner
and the incremental update logic. The scanner produces a stream of *filesystem events*
(added, changed, removed); the indexer turns them into rows. The scanner never writes
to the database directly, which keeps the write path testable without a disk.

### 4.6 `media`

Owns media inspection: container parsing for duration and dimensions, tag extraction,
artwork discovery. Pure Rust parsers are used where they exist; an external program is
never invoked in the baseline (ADR-0008). Every parser has a size and time bound.

### 4.7 `http`

Owns media delivery: request routing, range parsing, MIME determination, streaming and
cancellation. It obtains a byte range from a `MediaSource` trait, so the HTTP layer can
be tested against an in-memory source.

### 4.8 `security`

Owns path containment, request-size limits, redaction and the "safe defaults" table.
It is a library, not a policy engine: the policies live in the modules that own them,
and `security` provides the primitives.

### 4.9 `platform`

Owns the trait described in ADR-0003 and its per-OS implementations. The default
implementation returns "not supported" for everything, so that a new platform compiles
without touching the core.

---

## 5. Data flow: browse

```
client ──HTTP POST──▶ http task
                        │
                        ├─▶ soap dispatcher (upnp)
                        │      └─▶ ContentDirectory::browse
                        │             ├─▶ catalogue: children(object_id, page)
                        │             │      └─▶ SQLite index (read-only)
                        │             └─▶ didl: objects → DIDL-Lite XML
                        │
                        └─◀── SOAP response 200 OK
```

No filesystem access happens during a browse. The index answers, which is what keeps
`PERF-005` and `PERF-008` satisfiable.

## 6. Data flow: playback

```
client ──HTTP GET (Range)──▶ http task
                              └─▶ media source (path → validated File)
                                   └─▶ read exactly the requested bytes
client ◀──206 Partial Content──
```

The path is validated by `security::path` before the file is opened. The transfer is
registered in a bounded table so that cancellation, limits and statistics work.

## 7. Data flow: scan

```
scanner (bounded worker pool)
   └─▶ filesystem events (added / changed / removed)
          └─▶ indexer ──▶ SQLite (transactional batches)
                 └─▶ event bus: "catalogue changed"
                        └─▶ didl refresh of cached metadata for changed items
```

The scan is incremental: the scanner compares size and modification time against the
index and only re-reads what changed (`FUN-006`, `PERF-003`).

---

## 8. Catalogue and identifiers

- The catalogue lives in one SQLite file (`FUN-009`), created with user-only
  permissions (`SEC-095`).
- Object identifiers are **opaque and stable** (`INT-062`): a deterministic hash of the
  root id and the path relative to that root, rendered in a fixed alphabet. They do not
  expose a filesystem path, they survive an index rebuild, and they are computed
  without a database round-trip.
- The standard containers have fixed ids (`0` = root, `1` = content directory), so
  that clients that start browsing at a hard-coded id find the right place.

## 9. Storage classes

| Class | Examples | Policy |
| --- | --- | --- |
| **Disposable cache** | thumbnails, generated artwork, DIDL fragment cache | Regenerable; evictable; bounded size; safe to delete at any time |
| **Persistent state** | catalogue index, configuration, device identity | Written atomically (write temp, fsync, rename); never promised to be securely erased |
| **User-owned artefacts** | media files, playlists, subtitles | Never modified; read-only access |

---

## 10. Concurrency and cancellation

- Every long-running operation is cancellable and reports progress (`UX-013`).
- Metadata extraction runs in a bounded worker pool with a per-file time limit
  (`SEC-062`).
- A transfer that the client abandons is detected through the read error or the
  connection close, and its slot is released (`INT-089`, `SEC-063`).
- No task holds a lock while performing I/O.

## 11. Error model

One error enum per crate, with a `thiserror`-style implementation. Errors carry the
context needed to explain them to a user: the operation, the path or object id, and the
cause. Errors that reach a user are rendered by one formatter that produces the
"what happened / why / what to do" shape required by `UX-004`. Internal error chains
never leak file system paths to a client (`SEC-013`).

## 12. Platform integration

The core needs exactly five things from the operating system:

1. enumerate network interfaces and their multicast capability,
2. resolve and validate a media path on a given volume,
3. detect that a volume changed (mounted, unmounted, replaced),
4. locate per-user configuration and data directories,
5. emit a log record and (optionally) a user notification.

These are the five methods of the `Platform` trait (ADR-0003). Everything else is
portable code.

## 13. What the architecture deliberately does not do

- It does not support internet-facing operation. There is no port-forwarding helper,
  no tunnel client and no "remote access" module.
- It does not implement the UPnP Internet Gateway Device protocol, even though it is
  the same discovery protocol. It is an attack surface with no user benefit here.
- It does not have a plugin system. A plugin system would make the security model and
  the licence audit unbounded; features are compiled in.
- It does not have a database abstraction layer over SQLite. SQLite is the store; a
  second store would double the test matrix for no user-visible benefit.

---

## 14. Related documents

| Question | Document |
| --- | --- |
| What must the product do? | `../01-product/TECHNICAL-REQUIREMENTS.md` |
| What does one feature do? | `../03-features/` |
| Why is it built this way? | `adr/` |
| How is it kept safe? | `../04-security/SECURITY-MODEL.md` |
| How do we know it works? | `../05-quality/QUALITY-GATES.md` |
| Where do I change what? | `../../WHERE_TO_CHANGE.md` |
