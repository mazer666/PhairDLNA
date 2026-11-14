# ADR-0007 — Object identifiers are deterministic, opaque and path-free

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `INT-062`, `INT-060`, `PLT-008`, `FUN-011`, `FUN-019`, `FUN-020`, `SEC-013`

## Context

Every item and container in a ContentDirectory browse has an `id` attribute. Clients
use it as the key for everything they do afterwards: they request it again, they
compare it with what they cached last time, and they use it to build the parent chain.

That creates three constraints at once:

1. The id must be **stable**. A client that cached `id=42` yesterday must still be
   able to use it today — across a restart, across an index rebuild, and across a
   change of Unicode normalisation form on the volume.
2. The id must be **opaque**. It must not leak an absolute filesystem path, because
   the id travels over the network to every client on the LAN.
3. The id must be **cheap**. Browsing must not require a database round-trip per
   object, or a large catalogue becomes slow.

A database auto-increment fails (1) and leaks (2). A hash of the raw path fails (1) on
macOS: APFS is normalisation-insensitive but preserves the on-disk form, so a file
created over SMB (precomposed) and later read on APFS can come back in a different
form than it was stored — a stored path string can stop resolving even though the
file never moved.

## Decision

An object id is computed as:

```
object_id = encode( hash( volume_id || 0x00 || identity ) )
```

where

- `volume_id` is the stable identity of the volume the object lives on (ADR-0003,
  `MediaPath.volume`), and
- `identity` is the **normalised** path relative to the root: Unicode NFC with a
  per-component case fold, computed per root according to that root's detected case
  sensitivity (ADR-0003, rule 3).

Rules:

- The hash is a fast, non-cryptographic 128-bit function rendered in a fixed
  lowercase alphabet. It is not a security boundary: the id is not a secret, it is
  merely opaque and stable.
- The standard containers have **reserved ids**: `0` for the root container and `1`
  for the content directory, because some clients start browsing at a hard-coded id.
- The id of a container is derived exactly like an item's, so the parent/child
  relationship is computable without a lookup.
- The id is **never the file name** and never contains a path fragment.

**Uniqueness is a separate question from identity.** Two catalogue entries are the
same file when their `(volume id, device id)` pairs match, not when their paths
match. The catalogue therefore stores the device identity alongside the path and
de-duplicates on it. A hard-linked file appearing under two names is one file with
two paths.

## Consequences

- Ids survive an index rebuild, an upgrade and a move of the whole library to
  another machine, as long as the roots keep their order and identity.
- Ids leak nothing about the path and cannot be turned back into one.
- Computing an id costs a hash — no database access, which keeps browsing fast.
- Because the id is derived from the *normalised* identity, a change of the
  normalisation mode of a volume would change the ids. The mode is therefore detected
  per root and stored with the root, not as a global setting.
- The catalogue must store the raw path *and* the device identity alongside the
  normalised identity. That is three columns instead of one, and it is the price of
  correctness on macOS.

## Alternatives considered

- **Database row ids.** Rejected: unstable across a rebuild and leaks the number of
  objects.
- **The absolute path, URL-encoded.** Rejected: leaks the file system layout and the
  user's name.
- **A random UUID stored in the index.** Rejected: not computable without the
  database, so browsing pays a lookup per object.
- **The relative path itself as the id.** Rejected: leaks the folder structure and is
  unstable when a folder is renamed.
- **A hash of the raw path.** Rejected: the normalisation trap described above makes
  it unstable on macOS volumes.

## Revisit condition

Revisit if a client turns out to require ids that are meaningful to it, if the
catalogue ever gains entries that are not files and therefore have no path, or if the
catalogue has to be shared between two machines with different volume identities.
