# F-10 — Metadata extraction and artwork

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-013, FUN-014, PERF-004

## 1. Purpose and user story

A television's DLNA app shows a list of albums and films. Next to each entry it wants a
picture: the **cover art** of an album, a **thumbnail** of a photo, a **poster** still
from a film. Without a picture the list is a wall of text; with it, the user recognises
what they own at a glance.

Those pictures come from three different places:

- **Embedded artwork** — an image stored inside the media file itself, typically an ID3
  `APIC` frame or an MP4 cover-art atom.
- A **sidecar image** — a separate picture file next to the media file, such as
  `cover.jpg` beside `song.mp3`. This is how a music library that was ripped from
  physical discs usually looks.
- The media file itself, for an image: a **thumbnail** is a small re-encoded preview of
  a photo, and a **poster** is a still frame taken from a video.

Producing them means *parsing files the user does not control*: a file that arrived
through a sync client, a shared folder or a download can be malformed in ways that crash
naive parsers (security §4.5). So this feature is as much a security boundary as it is
a picture generator.

**User story.** As a user, I want my albums and photos to show a picture in my
television's list, so that I can find what I am looking for; and I want a corrupt file
in my library to cost me one missing picture, never a crashed server.

## 2. Scope

**In scope**

- The `media` module of `phairdlna-core`: artwork discovery and extraction, thumbnail
  generation and the disposable cache (ARCHITECTURE §4.6, §9).
- The rules that decide which picture belongs to which object: embedded artwork, sidecar
  image, thumbnail, poster (FUN-013, FUN-014).
- The out-of-process sandbox that performs every parse (SEC-066, M-06, security §8) and
  the quarantine path for a file whose parsing fails.
- The disposable on-disk cache of generated images, its size bound and its eviction
  (SEC-064, CFG-008).
- The resource bounds that keep the picture pipeline off the critical path: the
  indexing-throughput effect of PERF-004.

**Explicitly out of scope**

- Container probing for duration, dimensions, bitrate and codec (F-09 owns FUN-012); the
  artwork rules here apply to what the *picture* pipeline reads.
- The `upnp:albumArtURI` element, its absolute URL form and the `dlna:profileID`
  omission (F-05 owns INT-067); serving the image bytes over HTTP (F-06).
- Transcoding of any kind (F-17). No external program is invoked in the baseline
  (M-08); ARCHITECTURE §4.6 refers to "ADR-0008" for that decision, but no such ADR
  exists in `docs/02-architecture/adr/` — raised in §12.
- The administration surface (F-14) and the command line (F-13), except that both use the
  sandbox rules stated here.

## 3. Functional requirements

**FUN-013** — The server shall extract embedded artwork from audio files and use a
sidecar image when one exists. [DESIGN] Both sources are supported. A sidecar is a
separate image file whose name is derived from the media file's name (`cover.jpg`,
`folder.jpg`, `album.jpg`, or the media file's own name with an image extension) and
which lies inside the same root; it is resolved, contained and re-checked like any other
path (SEC-001, SEC-003). A sidecar that lies outside the root, or that is reached
through a symbolic link the root's policy does not allow, is not used. Embedded artwork
is read in the sandbox child and is subject to the same size bounds as any other parsed
data. *Which source wins when both exist is not stated by the requirement — the reading
used here is "a sidecar image, when one exists, is used";* raised in §12.

**FUN-014** — The server shall generate thumbnails for image files and video posters on
demand and cache them in a disposable cache. [DESIGN] *On demand* means the picture is
produced when a client asks for it, not while scanning: a scan of a photo library must
not decode every image. A generated picture is written to the **disposable cache** — a
storage class that is regenerable, evictable, size-bounded and safe to delete at any
time (ARCHITECTURE §9, SEC-064). The cache key is derived from the object's identity
and the requested size, so that a changed file yields a new key and a stale picture is
never served. *The video-poster half of this requirement depends on a decoder that the
baseline does not have —* raised in §12.

**PERF-004** — The server shall index at least 200 media files per second on the
reference hardware when metadata extraction is disabled. [DESIGN] The picture pipeline
must not be on the scan path: with extraction disabled the scan performs no artwork
work, no thumbnail work and no decode. This feature therefore contributes to PERF-004
by keeping itself out of the indexing path, and the target itself is owned and measured
in F-09.

## 4. Technical requirements

- **Module.** `phairdlna-core/src/media/` owns media inspection: artwork discovery,
  thumbnail generation and the cache (ARCHITECTURE §4.6). The module knows the
  catalogue's object identity and the security module's containment rules; it never
  opens a path that the security module has not approved.
- **Everything parses out of process.** SEC-066 is explicit: "the server shall parse
  media metadata and generate thumbnails in a separate, sandboxed process with no network
  access, a bounded memory and CPU budget and a time limit, and shall quarantine a file
  whose parsing fails." The parent process never decodes pixels and never parses a
  container (M-06, security §8). [OBSERVED] The reason is not theoretical: the Rust
  `image` crate has shipped soundness defects (`RUSTSEC-2020-0073`, `RUSTSEC-2019-0014`)
  and the EXIF parser `kamadak-exif` has an infinite-loop advisory
  (`RUSTSEC-2021-0143`); Matroska metadata parsing has produced a use-after-free in a
  media stack (security §4.5).
- **Sandbox bounds.** The child runs with no network access, a bounded resident set, a
  bounded wall-clock time per file, a file-size cap and an allowlist of formats
  (M-06). [DESIGN] The bounds proposed in security §8 — 5 s wall clock, 256 MiB
  resident set, 64 MiB input file, at most 2 concurrent parse jobs — are the starting
  values; the metadata extraction time is a configuration setting (CFG-008) that may
  only tighten them.
- **Declared dimensions are checked before any allocation.** An image header that
  declares enormous dimensions is rejected before a buffer is sized (SEC-067); the
  parent's resident memory must not follow the child's allocation (SEC-TH-20).
- **Quarantine.** A file whose parsing fails, times out or exceeds a bound is
  quarantined: it is excluded from the catalogue's picture pipeline, the reason is
  recorded, and the rest of the library continues (SEC-066, PLT-012). A quarantined file
  never causes a retry storm — the retry policy is raised in §12.
- **The cache.** One directory of generated pictures, in the user's data directory or,
  if the user configures it, inside a media root (FUN-018 allows exactly that one
  exception). Its size is bounded and entries are evicted when the bound is exceeded
  (SEC-064, CFG-008). Deleting the cache changes nothing the user can observe except
  that pictures are produced again on demand.
- **Serving.** The generated or extracted picture is reachable at an absolute HTTP URL
  that contains no path fragment (PLT-008, INT-067); the URL and the serving route
  belong to F-06, and the `upnp:albumArtURI` element to F-05.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.
Measurements follow qa-performance §4: release build, discarded warm-up runs, at least
30 iterations, median and 95th percentile, on the reference profile of
qa-performance §2.

- [DESIGN] Target (PERF-004): with metadata extraction disabled, indexing throughput
  stays at or above 200 files per second, because the picture pipeline performs no work
  during a scan. Method: the indexing benchmark of F-09 with the extraction switch off.
- [DESIGN] Target: a thumbnail for a photo of the reference corpus is produced within
  1 s wall clock, including the child start-up. Method: `hyperfine` (qa-performance §4)
  over the sample corpus, ≥ 30 iterations, median and p95 reported.
- [DESIGN] Target: a cached picture is served without re-encoding and without starting a
  child process. Method: a benchmark that requests the same picture twice and records
  that the second request performs no child start-up (process-spawn counter stays
  unchanged).
- [DESIGN] Target: the cache never exceeds its configured size bound, and the eviction
  keeps the served pictures correct. Method: fill the cache past the bound with a
  synthetic corpus and assert that the on-disk size stays within the bound.

## 6. Usability and accessibility

- The feature has no graphical surface; its user-visible effect is the picture next to
  an album or a photo in the television's list.
- A file with no artwork produces **no** `upnp:albumArtURI` at all rather than a broken
  image link; the client then shows its own placeholder (INT-067, F-05).
- [UX-004] When a picture cannot be produced, the reason is visible in the diagnostics
  in plain language — "this file could not be read as an image" — and never as a parser
  error with a stack trace (UX-005).
- [UX-013] The picture pipeline is cancellable and reports progress; a long picture
  generation does not block a browse (ARCHITECTURE §5, §10).
- [UX-011] The cache size and the number of quarantined files appear in the diagnostic
  report, so that a user who notices a growing disk footprint can act (OPS-013, F-13).
- [DOC-005] *artwork*, *sidecar*, *thumbnail*, *poster*, *quarantine* and *disposable
  cache* are explained at first use.

## 7. Security and privacy

- The dominant risk in this feature is a malicious media file (SEC-TH-19, SEC-TH-20):
  the file comes from a sync client, a shared folder or a download, and it is parsed by
  the server. Every parse therefore happens in a separate, sandboxed child with no
  network access and bounded memory, CPU and time (SEC-066, M-06).
- The child has no network access at all, so a malicious file cannot cause an outbound
  connection (SEC-091). This feature initiates no network activity of its own.
- Declared image dimensions are bounded before any buffer is allocated, so that a
  100 000 × 100 000 header cannot exhaust the host's memory (SEC-067, SEC-TH-20).
- A file whose parsing fails is quarantined rather than retried indefinitely; a
  repeated parse failure is a resource-exhaustion vector in its own right (security §8).
- The sidecar lookup is a path-resolution operation and is subject to the full
  containment rules: resolve, canonicalise, re-check against the root, and refuse a
  symbolic link that leaves the root (SEC-001, SEC-003, SEC-006).
- The disposable cache holds copies of the user's pictures inside the user's own
  directories; it is never world-readable by default and never leaves the machine
  (SEC-093, SEC-095).
- Log records about artwork and thumbnails use the redaction rules of OPS-007 (F-13);
  a file path in a log line is replaceable by a stable pseudonym (SEC-094).

## 8. Interoperability

- [SPEC] A media server advertises album art through the `upnp:albumArtURI` element,
  whose value is an absolute URL the client fetches over HTTP and which carries no
  `dlna:profileID` attribute (INT-067, F-05).
- [DESIGN] The picture itself is served with the same MIME registry that F-06 and
  F-04 use, so a client that asks for `image/jpeg` receives JPEG bytes (INT-051,
  INT-087, F-05).
- [DESIGN] The generated thumbnail is a plain, widely decodable image; no proprietary
  or progressive format is produced, because the target devices are televisions and
  phones of unknown vintage.
- The catalogue supplies the object's identity and its media type; the picture pipeline
  supplies only the picture. A client that receives no picture is still served a
  correct `Browse` response (F-03, F-05).

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| The file has no artwork and no sidecar | no `upnp:albumArtURI` is emitted; the item is otherwise complete | FUN-013, INT-067 |
| The sidecar lies outside the root | the sidecar is ignored; the search continues for another candidate | SEC-001, SEC-006 |
| The child exceeds its time or memory bound | the child is killed, the file is quarantined with the reason, the library continues | SEC-066, SEC-TH-19 |
| The image declares enormous dimensions | the file is rejected before allocation; the parent never allocates the buffer | SEC-067, SEC-TH-20 |
| The cache is full | the least recently used entries are evicted; generation continues | SEC-064 |
| The cache directory has been deleted | it is recreated; entries are generated again on demand | ARCHITECTURE §9 |
| The sandbox child cannot be started | the picture is not produced, the reason is reported, and no in-process fallback is attempted | SEC-066, M-06 |

No failure of this feature may take down the HTTP task, the SSDP task or the catalogue
task: a picture failure produces a missing picture, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* an audio file with embedded artwork, *when* the catalogue is built, *then* the embedded picture is
extracted, and the object carries an absolute artwork URL that contains no path fragment. Requirements:
FUN-013, INT-067, PLT-008 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/artwork.rs`

**AC-02** — *Given* an audio file with a sidecar image next to it, *when* the catalogue is built, *then* the sidecar is
used as the artwork for that object, and the sidecar is resolved and contained like any other path.
Requirements: FUN-013, SEC-001 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/artwork.rs`

**AC-03** — *Given* a sidecar image placed outside the media root, *when* the catalogue is built, *then* the sidecar is
not used, no file outside the root is opened, and the object carries no artwork URL. Requirements: FUN-013,
SEC-001, SEC-003, SEC-006 · Verification: automated test (filesystem audit hook) · Planned location:
`crates/phairdlna-core/tests/artwork.rs`

**AC-04** — *Given* an audio file with neither artwork nor sidecar, *when* the catalogue is built, *then* the object
carries no artwork URL and no error is raised. Requirements: FUN-013 · Verification: automated test ·
Planned location: `crates/phairdlna-core/tests/artwork.rs`

**AC-05** — *Given* an image file, *when* a client requests a thumbnail, *then* a thumbnail is produced within the time
budget of §5, is cached, and a second request for the same picture is served from the cache without
producing it again. Requirements: FUN-014 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/thumbnails.rs`

**AC-06** — *Given* an image file that is changed after its thumbnail was cached, *when* a client requests the thumbnail
again, *then* the changed file yields a different cache key, so that the stale picture is never served.
Requirements: FUN-014 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/thumbnails.rs`

**AC-07** — *Given* a configured cache size bound, *when* the cache is filled beyond that bound, *then* the cache on
disk never exceeds the bound, entries are evicted, and the pictures that remain are the correct ones for
their objects. Requirements: FUN-014, SEC-064, CFG-008 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/thumbnails.rs`

**AC-08** — *Given* a media file whose parsing makes the sandbox child exceed its time bound, *when* the catalogue is
built, *then* the child is killed at the bound, the file is quarantined, the server process is unaffected
and its resident memory stays within the cap. Requirements: FUN-013, FUN-014, SEC-066, SEC-TH-19 ·
Verification: automated test · Planned location: `crates/phairdlna-core/tests/media_sandbox.rs`

**AC-09** — *Given* an image that declares enormous dimensions, *when* a thumbnail is requested, *then* the file is
rejected before any buffer is sized, the child is killed, and the parent's resident memory never exceeds the
cap. Requirements: FUN-014, SEC-067, SEC-TH-20 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/media_sandbox.rs`

**AC-10** — *Given* the sandbox child, *when* it runs, *then* it has no network access — asserted by an egress counter
that stays at zero — and it can only read the file it was given and write to the cache directory it was
given. Requirements: SEC-066, SEC-091, M-06 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/media_sandbox.rs`

**AC-11** — *Given* a library of 100 000 files and metadata extraction disabled, *when* the catalogue is indexed, *then*
the throughput is at least 200 files per second and no child process was started, because the picture
pipeline is not on the scan path. Requirements: PERF-004 · Verification: benchmark · Planned location:
`crates/phairdlna-bench/benches/indexing.rs`

**AC-12** — *Given* a quarantined file, *when* the diagnostics are read, *then* the file is named and the reason is
given in plain language, and the rest of the library is indexed. Requirements: FUN-013, FUN-014, UX-004 ·
Verification: automated test · Planned location: `crates/phairdlna-core/tests/media_sandbox.rs`

**AC-13** — *Given* the disposable cache, *when* the whole cache directory is deleted, *then* the server continues to
serve media and produces the pictures again on demand, with no error to the client. Requirements: FUN-014 ·
Verification: automated test · Planned location: `crates/phairdlna-core/tests/thumbnails.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Artwork discovery, precedence, containment | `crates/phairdlna-core/tests/artwork.rs` |
| Thumbnail generation, caching, eviction | `crates/phairdlna-core/tests/thumbnails.rs` |
| Sandbox bounds, quarantine, no network | `crates/phairdlna-core/tests/media_sandbox.rs` |
| Artwork and thumbnail unit tests | `crates/phairdlna-core/src/media/artwork.rs`, `crates/phairdlna-core/src/media/thumbnails.rs`, `crates/phairdlna-core/src/media/sandbox.rs` (inline `#[cfg(test)]` modules) |
| Indexing throughput with extraction disabled | `crates/phairdlna-bench/benches/indexing.rs` |
| Picture rendering on a real device | `qa/device-matrix.toml` |

**Method.** The sandbox child is a separate executable, so it is exercised for real: a
malformed corpus (a truncated ID3 frame, a Matroska file with a hostile `CodecPrivate`,
a PNG with an EXIF payload and an image header declaring 100 000 × 100 000) is fed to
it and the parent's state is observed (security §12, N-10). The child's resource bounds
are asserted from the outside, by sampling the child's resident set and by the
wall-clock kill, not by trusting the child's own report.

**Dependencies.** F-09 (object identity, the catalogue, and the media probing that shares
this sandbox), F-05 (the `upnp:albumArtURI` element), F-06 (serving the picture over
HTTP), F-11 (the identity key), F-12 (cache size and extraction-time limits, CFG-008)
and F-14 (the administration surface that lists quarantined files).

## 12. Open questions and decisions

1. **[OPEN] Video posters cannot be produced in the baseline.** FUN-014 asks for
   thumbnails *and* video posters, but a poster requires decoding a video frame, and
   the baseline ships no video decoder: no external program is invoked (M-08, which
   explicitly says "do not ship FFmpeg in v1"), and no pure-Rust video decoder is on the
   dependency list. *Proposed reading:* the baseline delivers image thumbnails and
   artwork; the poster half of FUN-014 is deferred and tracked as an open item until a
   decoder decision exists. *Closes when:* the owner decides between deferring the
   poster, shipping an LGPL-only external decoder, or adding a pure-Rust decoder.
2. **[OPEN] Which artwork source wins.** FUN-013 requires both embedded artwork and a
   sidecar to be supported but does not rank them. *Proposed reading, used here:* the
   sidecar wins, because a user who placed a cover next to the file meant it to be seen,
   and because it is the cheaper path. *Closes when:* the owner confirms or reverses it.
3. **[OPEN] The size cap on embedded artwork.** An embedded cover can be a
   multi-megabyte JPEG. *Proposed:* embedded artwork larger than the child's file-size
   bound is discarded and the sidecar is used instead; no re-encoding of embedded
   artwork happens in the baseline. *Closes when:* the owner fixes the cap.
4. **[OPEN] Quarantine retry policy.** SEC-066 says a file whose parsing fails shall be
   quarantined; it does not say whether the quarantine is permanent or expires.
   *Proposed:* a quarantined file is not retried automatically, is reported in the
   diagnostics, and is retried only when the user asks for a rescan or the file's
   modification time changes. *Closes when:* the owner decides.
5. **[OPEN] Where the disposable cache lives.** ARCHITECTURE §9 defines the storage
   class but not the location, and FUN-018 permits the cache inside a media root only
   when it is configured that way. *Proposed:* the user's data directory by default.
   *Closes when:* the platform adapter documents the per-instance data directory
   (F-13, OPS-010).
6. **[OPEN] ADR-0008 is missing.** ARCHITECTURE §4.6 refers to "ADR-0008" for the
   decision that no external program is invoked, but `docs/02-architecture/adr/` has no
   such ADR; the video-poster decision of §12 Q1 belongs there. *Closes when:* the ADR
   is written or the reference is corrected.
