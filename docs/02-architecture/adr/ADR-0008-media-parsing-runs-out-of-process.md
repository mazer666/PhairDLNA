# ADR-0008 — Media metadata is parsed in a sandboxed child process

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `SEC-066`, `SEC-067`, `FUN-012`, `FUN-013`, `FUN-014`, `PERF-004`

## Context

A media file is not a passive file. It is a container format written by someone
else, full of length fields, offsets and embedded structures that a parser must
trust to describe the size of the next allocation. The historical record of media
servers is not encouraging:

- ReadyMedia/MiniDLNA: a `strtol` signedness bug and a `strtol(...) > 0`
  boolean-assignment bug feeding a `memmove` (both in chunked-encoding handling), a
  DFF variable-length-array stack overflow, a TiVo `strcat` stack overflow, and a
  configuration-file command injection.
- Gerbera: a WavPack stack overflow caused by putting records on the stack.
- The general class is exactly one thing: **an attacker-controlled length or count
  used without validation**.

The files reach the server without any user action. A synchronisation client, a
shared folder, a download or a friend sending a file is enough. The attacker does
not need network access to the server at all — a file on the disk is enough.

Writing every parser in safe Rust would reduce the risk, but it would not remove
it: a safe parser can still be made to allocate ten gigabytes from a four-byte
header, and the media format landscape (ID3, MP4 atoms, Matroska, EXIF, DFF,
WavPack, JPEG) is large enough that "we wrote it ourselves" is not a defence.

## Decision

**All media metadata parsing and thumbnail generation happens in a separate,
sandboxed child process.** The main server process never parses a media container.

The child:

- has **no network access**,
- is started with an explicit argument array and **never** through a shell,
- runs with a **bounded memory budget**, a **bounded processor budget** and a
  **time limit** per file,
- is **killed** when the limit is exceeded, and the file is **quarantined** rather
  than retried indefinitely,
- communicates the result over a pipe with a small, fixed message format.

The main process:

- **never allocates a buffer from a length field** it has not validated against the
  actual file size (`SEC-065`),
- caps the **declared dimensions** of an image before any buffer is allocated, so a
  file claiming 100 000 × 100 000 pixels cannot exhaust memory (`SEC-067`),
- treats the child's crash as a *per-file* failure, not a *server* failure.

The pure-Rust parsers (`symphonia` for audio, a container parser for Matroska and
MP4) are used **inside the child**, so the good parsers are still used — they just
do not run where a bug would be fatal.

## Consequences

- A parser bug becomes a killed child and a quarantined file, not a compromised
  server. This is the single most valuable structural decision in the security model.
- Parsing costs a process start per batch. Batching many files into one child
  amortises it, and the scheduler's priority classes decide when a batch may run
  (`RES-001`, `RES-003`).
- The child must be built and shipped for every target platform. That is a real
  distribution cost, and it is accepted.
- The protocol between the parent and the child must be small and strict: a fixed
  message format, a length prefix, and a hard limit on the message size.
- The main server process becomes simpler: it never has to know a media format at
  all. It knows a metadata record.

## Alternatives considered

- **Parsing in-process, in safe Rust.** Rejected as the *only* defence: safe Rust
  prevents memory corruption, not resource exhaustion, and the media format surface
  is too large to audit completely.
- **Parsing in-process with a fuzzing campaign instead of isolation.** Rejected —
  fuzzing reduces the probability of a bug; isolation removes the consequence.
- **Never parsing metadata at all.** Rejected — duration, dimensions and artwork are
  what make a DLNA server usable rather than a file dump.
- **Using an external tool such as `ffprobe`.** Rejected for the baseline for two
  independent reasons: it drags an FFmpeg binary into an AGPL distribution (a
  licensing trap), and it moves the attack surface to a process the project does not
  control. The child process with our own pure-Rust parsers is the better shape.

## Revisit condition

Revisit if a trusted, memory-safe, pure-Rust parser exists for a format that we
still need and whose failure mode has been audited — in that case that format could
be parsed in-process, with the child remaining the default for everything else.
