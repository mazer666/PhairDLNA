# ADR-0011 — The picture pipeline parses in-process until the sandbox child exists

Status: **Accepted**
Date: 2026-11-16
Owner role: Architect
Requirements: `FUN-013`, `FUN-014`, `SEC-066`, `SEC-067`, `SEC-091`, `PERF-004`

## Context

ADR-0008 decides that **all** media metadata parsing and thumbnail generation happens in a
separate, sandboxed child process, and that the main server process never parses a media
container. The reason is structural: a media file is written by someone else, and a parser
that trusts a length field can be made to allocate ten gigabytes from four bytes.

The milestone **M2.2** delivers the picture pipeline — the embedded cover, the sidecar, the
thumbnail and the disposable cache. The sandbox child of ADR-0008 does not exist yet: it is
the work of **M-06**, and the roadmap places M-06 after M2.

The owner directed that the picture extraction of M2.2 be **pure Rust** and free of any
external program. The remaining question is therefore not *whether* to parse, but *where*.

Two things make the in-process variant defensible for this milestone, and one thing makes it
a deviation that must be recorded rather than glossed over:

- The picture pipeline of M2.2 is **small**. It reads a cover out of an ID3v2 tag and out of
  an MP4 container, and it decodes a JPEG or a PNG. It is not the whole media landscape.
- Every read and every allocation is **bounded**: a tag above 16 MiB is refused, an embedded
  cover above 8 MiB is refused, a sidecar above 16 MiB is refused, and a picture that claims
  more than 8192 pixels per edge or more than 40 million pixels is refused **before** a
  single pixel is allocated.
- It is nevertheless a **deviation** from an accepted ADR, and a deviation that is not
  written down is how a security model quietly stops being true.

## Decision

**Until the sandbox child of ADR-0008 exists, the picture pipeline of M2.2 parses in the
server process, in pure Rust, with the bounds named above.**

The parsers live behind a narrow interface:

- `phairdlna-core/src/media/id3/` — the cover of an ID3v2 tag (versions 2.2, 2.3 and 2.4),
- `phairdlna-core/src/media/mp4/` — the cover of an MP4 container (`covr`),
- `phairdlna-core/src/media/thumbnail.rs` — the thumbnail of a picture,
- `phairdlna-core/src/media/sidecar.rs` — the names of the sidecar candidates,
- `phairdlna-core/src/media/artwork/` — the order of the search and the containment rules.

Each of them takes a **file handle** and a **bound**, and returns `Option` — never a
`Result` with a parser error, never a panic. A file that cannot be understood gives no
picture, and the library stays browsable.

The pipeline is **not on the scan path**: the catalogue reads no file of the library while it
is scanned (`library.extract_artwork`, `PERF-004`). The picture of a file is looked for when
a client asks for it, and the answer is remembered under a key that carries the size and the
modification time of the file.

The deviation ends with M-06. When the child exists, the parsers move into it **without a
change to their interface**: the child takes the file handle and the bound and sends the
bytes of the picture over the pipe. The caller does not change, and the decision that the
server never parses a media container becomes true again.

## Consequences

- The milestone M2.2 can be delivered without the distribution cost of a second executable
  for every target platform, and without the message protocol of ADR-0008.
- The server process carries the risk of a parser bug in the picture pipeline until M-06.
  The bounds and the refusal of every broken file reduce the risk; they do not remove it, and
  this ADR does not claim that they do.
- The quarantine of SEC-066 — a file whose parsing fails is quarantined and reported — has no
  place yet, because there is no child to kill. Until M-06, a broken file gives no picture and
  no error.
- A contributor who writes a new parser for this pipeline must write it **pure Rust**, must
  bound every read and every allocation, and must return `Option` rather than panicking. The
  gate `scripts/check-no-unsafe.sh` and the review of the change are the enforcement.
- The parsers are written so that the move into the child is a change of the *caller*, not a
  rewrite of the *parser*.

## Alternatives considered

- **Build the sandbox child now.** Rejected for this milestone: it is the work of M-06, it
  needs a distribution pipeline that does not exist yet, and the owner directed the scope of
  M2.2. It remains the plan.
- **Parse in-process with the `image` crate and no bounds.** Rejected: the bounds are the
  part of ADR-0008 that can be honoured without the child.
- **Use an external program such as `ffprobe` or `ffmpeg`.** Rejected for the reasons of
  ADR-0008: a copyleft binary in an AGPL distribution, and an attack surface the project
  does not control.
- **Extract nothing and serve the sidecar image only.** Rejected: the embedded cover is the
  picture of a library that was ripped from a disc, and `FUN-013` asks for it.

## Revisit condition

This ADR is **superseded when the sandbox child of ADR-0008 exists** (M-06). At that moment
the parsers move into the child, the server process stops parsing media containers again, and
this ADR is marked *Superseded* with a link to the ADR that closes the gap.

The owner may also bring the child forward; in that case the roadmap is changed and this ADR
is closed earlier.
