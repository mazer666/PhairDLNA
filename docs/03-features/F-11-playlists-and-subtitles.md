# F-11 — Playlists, subtitles and organisation

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements:
FUN-015, FUN-016, FUN-020, FUN-021, FUN-022
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

Three small features make a media library feel organised rather than merely enumerated.

A **playlist** is a text file that lists media references — the `.m3u` next to a film series, the
`.pls` a ripping tool wrote. A **subtitle sidecar** is a separate text file that carries the
dialogue of a video; the television can fetch it and burn it into the picture. **Organisation** is
the set of rules that decide what the catalogue contains at all: which subfolders the user wants
to keep out, which files are the same file reached by two different paths, and what key identifies
a file in the first place.

**User story.** As a user, I keep a playlist beside my series and a subtitle file beside each
film, and I want the television to show the playlist as a list of things to play and to offer the
subtitle when it can use one; I want the folder I decided to leave out to stay out; and I want a
file that I shared through two overlapping folders to appear once, not twice.

## 2. Scope

**In scope** — reading playlists in M3U, M3U8 and PLS format and resolving their entries relative
  to the playlist file; the shape a playlist takes in the catalogue; discovering a subtitle sidecar
  next to a media file and exposing it as an additional `res` element; the identity key used for
  indexing and de-duplication; the exclusion of subdirectories of a root; and the behaviour when
  several roots point at the same volume.

**Explicitly out of scope** — the DIDL-Lite XML itself (F-05 writes it), the delivery of the
  subtitle bytes (F-06), the client profile that decides whether a subtitle is offered (F-07), the
  raw-path rule and the containment algorithm (F-08), writing playlists (the server never modifies a
  user file — FUN-018), transcoding or converting subtitle formats, and publishing a playlist as a
  `text/xml` document (see §12 and standards §9 Q12).

## 3. Functional requirements

**FUN-015** — The server shall support playlists in M3U, M3U8 and PLS format, resolving entries
  relative to the playlist file. [DESIGN] An entry that names no directory is resolved against the
  directory of the playlist file, never against the process working directory and never against a
  root. A playlist is presented in the catalogue as a **container** whose children are the resolved
  media objects, in playlist order; the UPnP class `object.container.playlistContainer` exists for
  exactly this purpose (standards §4.3). [DESIGN] The three formats are handled as follows: an M3U
  file is a list of paths, one per line, with `#` beginning a comment or an `#EXTINF` directive; the
  M3U8 form is the same structure encoded in UTF-8; a PLS file is an INI-style list with a
  `[playlist]` section whose `File1=`, `Length1=` and `Title1=` entries describe the items. [OPEN]
  These format descriptions were not verified against a primary format specification in this
  project's research; see §12.

**FUN-016** — The server shall expose a subtitle sidecar file (`.srt`, `.vtt`, `.sub`, `.ass`) as
  an additional `res` element where the client profile supports it. [DESIGN] A sidecar belongs to a
  media file when it sits in the same directory and has the same base name with one of those four
  extensions. The sidecar is an *additional* resource of the same catalogue object, not a separate
  item, and it carries its own `protocolInfo` and `size` (INT-063, INT-064); the file is served over
  the same media surface as any other resource (F-06). [DESIGN] The capability that decides whether
  the extra element is emitted comes from the client profile store (F-07); see §12 for the open
  question about the default when no profile exists.

**FUN-020** — The server shall compute a separate identity key for indexing and de-duplication by
  applying Unicode NFC and a per-component case fold to the path, scoped by the identity of the
  volume it lives on; it shall determine uniqueness from the platform's stable file identity rather
  than from the path. [DESIGN] The raw path is a separate value and is never rewritten (FUN-019,
  ADR-0003 rule 3). The case fold is used for matching only, never for display (platform-integration
  §5.1). Uniqueness is the pair *(volume id, device id)*, which is what makes a hard-linked file one
  file with two names (platform-integration §5.7; ADR-0007). [OPEN] The object identifier in
  ADR-0007 is derived from the volume identity and the path *relative to the root*, which does not
  line up with a volume-scoped key; see §12.

**FUN-021** — The server shall provide a way to exclude subdirectories of a root. [DESIGN] The
  exclusion is configured per root as a list of subdirectory paths relative to that root, is applied
  while scanning, and takes effect on the next scan. An excluded directory is neither indexed nor
  served, even if a client still holds an identifier from before the exclusion. [DESIGN] The same
  mechanism excludes the server's own cache directory when the cache is configured inside a root
  (FUN-018, F-08).

**FUN-022** — The server shall support multiple roots that point to the same underlying volume
  without producing duplicate catalogue entries. [DESIGN] Two roots on one volume, or one root that
  contains the other, must not make the same file appear twice. The rule follows from FUN-020: the
  catalogue entry is keyed by *(volume id, device id)*, so the second sighting of a file resolves to
  the existing entry instead of creating a new one. [OPEN] Which of the overlapping roots owns the
  entry, and what a browse shows, is not fixed by the requirement; see §12.

## 4. Technical requirements

**FUN-026** — The server shall offer a subtitle sidecar as an additional resource with
the MIME type that matches its format, for the formats SRT, WebVTT, SSA/ASS, SMIL and
SUB/IDX. [DESIGN] The MIME type is what tells the client what it is downloading; a
subtitle served as `application/octet-stream` is a subtitle the client may refuse.

**FUN-027** — The server shall deliver a subtitle file in a character set the client
can read, and shall state the character set in the response. [DESIGN] UTF-8 with or
without a byte-order mark is the default; a Latin-1 file that is not valid UTF-8 is
transcoded to UTF-8 on the way out, because the client would otherwise show mojibake.

**FUN-029** — The server shall expose each additional audio or subtitle track of a
media item as its own resource, so that a client may choose it. [DESIGN] The first
track stays the primary resource, so a client that only understands one resource keeps
working.

**INT-105** — The server shall give each additional resource of an item a
`protocolInfo` that names its own MIME type, so that a client can tell the tracks
apart. [SPEC] The `protocolInfo` string is four fields, the third of which is the
content format.


- **Modules.** Playlist parsing, subtitle pairing, the identity key and the de-duplication live in
  `catalogue`; the `res` element that carries a subtitle is written by `didl` (F-05); the containment rule that
  every playlist entry must pass lives in `security` (F-08); the platform facts — the stable file identity, the
  volume identity — come through the `platform` traits (ADR-0003).
- **Playlist parsing.** [DESIGN] The parser is bounded before it reads: a maximum file size, a maximum number of
  entries, a maximum line length and a maximum total entry length (SEC-060, SEC-011). It never executes
  anything and never resolves a URL — it produces strings that are then treated as untrusted paths.
- **Entry resolution.** [DESIGN] Every entry follows the rule of SEC-068 in this order: reject an entry that
  carries a scheme other than `file:` — a network scheme such as `http:`, `https:`, `rtsp:` or `smb:` is
  refused outright; reject an absolute path that is not inside a root; otherwise resolve the entry against the
  playlist's directory; then apply the ordinary containment check of F-08. The server makes no network request
  for any entry (SEC-091). The security research reaches the same conclusion for playlist formats, which it
  treats as attacker-supplied path containers (security-landscape §4.6).
- **Playlist in the catalogue.** [DESIGN] The playlist file is a catalogue object of class
  `object.container.playlistContainer` whose children are the resolved media objects in playlist order. An entry
  that resolves to nothing is kept as a *missing* child that is reported in diagnostics and not offered for
  playback, so that the user learns about it (UX-004). A file that appears twice in one playlist is one media
  object listed twice, not two catalogue entries.
- **Subtitle pairing.** [DESIGN] Pairing happens at scan time, when the sidecar file is next to the media file;
  it adds no I/O to a browse. The catalogue stores the sidecar as an additional resource of the media object.
  Where several sidecars match one base name, the one with the highest priority extension is chosen and the
  others are recorded as alternatives; see §12.
- **The additional `res` element.** [DESIGN] The DIDL writer emits the second `res` only when the client
  profile declares the subtitle capability; the element carries a `protocolInfo` whose MIME type is the type the
  server really serves for that file (INT-064, INT-051) and a `size` (INT-063). The element is additive: an
  item with a subtitle has one more `res`, and every other attribute of the item is unchanged, so a client that
  ignores it still plays the video (standards §6.6, §8.8 N9).
- **Identity key.** [DESIGN] `identity = NFC(casefold(component))` applied per component, prefixed by the
  volume identity. The fold is applied per component and not to the whole string, so that a separator is never
  folded. The key is computed without a database round-trip (ADR-0007).
- **De-duplication.** [DESIGN] The indexer stores the raw path, the identity key and the device identity next to
  each other; a row whose device identity is already present resolves to the existing object instead of creating
  a new one (FUN-020, FUN-022).
- **Exclusions.** [DESIGN] The scanner consults the exclusion list before descending into a directory. The list
  is matched against the path relative to the root, so it survives a change of the root's mount point.
- **Playlists are never written.** [DESIGN] The server reads a playlist and never modifies it, renames it or
  deletes it (FUN-018); the same holds for a subtitle sidecar, which is a user-owned artefact (SEC-008).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: a playlist of 1 000 entries is parsed and resolved in under **100 ms** on the reference
  hardware of qa-performance §5.3. Method: `criterion` benchmark over ≥ 30 iterations, median and p95 reported
  (PERF-011).
- [DESIGN] Target: computing the identity key of a path costs no filesystem access. Method: code review plus
  an assertion that no handle is opened while the key is computed.
- [DESIGN] Target: the presence of a subtitle does not add work to a browse. Method: an assertion that the
  browse path opens no additional file and performs no additional directory read.
- [DESIGN] Target: a catalogue with 10 000 objects in which half are reached through two overlapping roots
  still reports each file once. Method: a fixture that counts the objects in the index after the scan
  (PERF-009, PERF-012).

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effects are the playlist that appears in the client's
  list, the subtitle that the television offers, and the folder that the user decided to keep out.
- [UX-004] A playlist entry that cannot be resolved shall be reported as unavailable, with a reason ("file not
  found", "outside the shared folder", "network address not allowed"), and shall not silently disappear.
- [UX-010] The user-facing text shall say "playlist" and "subtitle", not "M3U8" or "sidecar".
- [UX-001] The feature is operable without a GUI; the exclusion list is a configuration entry and the missing
  entries appear in the diagnostics output (F-14).
- [DOC-005] *Playlist*, *sidecar*, *identity key* and *de-duplication* are explained at first use.

## 7. Security and privacy

- **Playlist entries are untrusted paths** (SEC-068). They are resolved, contained and refused when they use a
  network scheme. The server never performs a network request for a playlist entry, and never opens a file
  outside a root because a playlist named it (SEC-091, SEC-001).
- The parser is bounded: file size, entry count, line length and total entry length (SEC-060, SEC-011). A
  playlist cannot make the server allocate without limit.
- The parser never executes anything and never builds a command line from an entry (SEC-005).
- Playlist and subtitle files are user-owned artefacts: they are opened read-only and never modified
  (SEC-008, FUN-018).
- File names and entry strings are untrusted input of bounded length and unknown encoding (SEC-007).
- The catalogue stores the raw path, the identity key and the device identity; the identity key is a derived
  value and leaks nothing that the object id does not already leak (INT-062, PLT-008).
- Paths in logs and diagnostics are redacted (OPS-007, SEC-094).

## 8. Interoperability

- The UPnP class list contains `object.container.playlistContainer` and `object.item.playlistItem`
  (standards §4.3), which is what a playlist is presented as.
- The DLNA layer is additive: an additional `res` element does not change any other part of the object, so a
  client that does not understand it is unaffected (standards §6.6, §8.8 N9).
- [OBSERVED] The `sec:` namespace (`http://www.sec.co.kr/dlna`) carries `sec:CaptionInfo` and
  `sec:CaptionInfoEx` and is used by Samsung-family clients; the DLNA namespace carries `dlna:profileID` and
  `dlna:lifetime` (standards §4.1, §6.5). These are vendor extensions, not part of the baseline.
- [OPEN] Whether a subtitle is offered as an additional `res` element, as a `sec:CaptionInfo` element, or as a
  `dlna:ifoFileURI` resource. See §12.
- [OPEN] The DLNA `DIDL_V` / `DIDL_S` collection-profile constraints, which apply if a playlist is ever
  published as a `text/xml` document (standards §9 Q12). Publishing playlists as `text/xml` is out of scope for
  this feature.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Playlist entry uses a network scheme (`http:`, `https:`, `rtsp:`, `smb:`) | the entry is refused and reported; no network request is made | SEC-068 |
| Playlist entry resolves outside every root | the entry is refused and reported | SEC-068, SEC-001 |
| Playlist entry does not exist | the entry is kept as a missing child and reported in diagnostics | UX-004 |
| Playlist file is not valid for its format | the playlist is listed with a diagnostic; the rest of the catalogue is unaffected | UX-005 |
| Playlist exceeds the entry-count or size bound | parsing stops at the bound and the truncation is reported | SEC-060, SEC-011 |
| Subtitle sidecar is unreadable | the additional `res` element is omitted; the media object is unaffected | SEC-007 |
| Client profile does not support subtitles | no additional `res` element is emitted | FUN-016, F-07 |
| Excluded subdirectory still referenced by a stale identifier | the request is refused; the object is not served | FUN-021 |
| The same file is reached through two roots | one catalogue entry exists | FUN-022 |

A playlist problem never takes down a scan or a browse: the bad playlist produces a diagnostic,
never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* an M3U playlist whose entries are relative paths, *when* the catalogue is
  browsed, *then* the playlist appears as a container of class `object.container.playlistContainer`
  and every entry resolves against the directory of the playlist file. Requirements: FUN-015 ·
  Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/playlists.rs`

**AC-02** — *Given* an M3U8 playlist in UTF-8 and a PLS playlist with a `[playlist]` section,
*when* both are browsed, *then* their entries appear in the same container shape as the M3U
playlist and in playlist order. Requirements: FUN-015 · Verification: protocol conformance test ·
Planned location: `crates/phairdlna-core/tests/playlists.rs`

**AC-03** — *Given* a playlist containing the entry `http://example.invalid/x.mp4` and the entry
  `../../etc/passwd`, *when* the playlist is parsed, *then* both entries are refused, neither is
  indexed and no outbound request is attempted. Requirements: SEC-068, SEC-001, SEC-091 ·
  Verification: negative test · Planned location: `crates/phairdlna-core/tests/playlists.rs`

**AC-04** — *Given* a playlist whose entries number more than the configured bound, *when* it is
  parsed, *then* parsing stops at the bound, the truncation is reported and the server does not
  allocate without limit. Requirements: SEC-060, SEC-011 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/playlists.rs`

**AC-05** — *Given* a video file with a `.srt` file of the same base name beside it and a client
  profile that declares the subtitle capability, *when* the video object is rendered, *then* the
  object carries a second `res` element whose `protocolInfo` names the type the server serves for
  that file and whose `size` is the size of the subtitle file. Requirements: FUN-016, INT-063,
  INT-064 · Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/subtitles.rs`

**AC-06** — *Given* a video file with a subtitle sidecar and a client profile that does not
  declare the subtitle capability, *when* the video object is rendered, *then* the object carries
  exactly one `res` element and every other attribute is identical to the object rendered without a
  sidecar. Requirements: FUN-016, INT-101 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/subtitles.rs`

**AC-07** — *Given* the same file reachable through two roots on the same volume, *when* both
  roots are scanned, *then* the catalogue contains one entry for that file, and the identity key of
  both sightings is identical. Requirements: FUN-020, FUN-022 · Verification: automated test ·
  Planned location: `crates/phairdlna-core/tests/catalogue_identity.rs`

**AC-08** — *Given* a file whose name exists in two Unicode normalisation forms on a
  byte-preserving volume, *when* it is indexed, *then* the two names remain two distinct files,
  because the identity key is scoped by the volume and the volume is case-sensitive and
  byte-preserving. Requirements: FUN-020, PLT-005 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/catalogue_identity.rs`

**AC-09** — *Given* two names that differ only in case on a case-insensitive volume, *when* they
  are indexed, *then* they resolve to one catalogue entry. Requirements: FUN-020, PLT-005 ·
  Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/catalogue_identity.rs`

**AC-10** — *Given* a root with an excluded subdirectory, *when* the root is scanned, *then* the
  excluded directory and everything below it is absent from the catalogue, and a request for an
  object identifier that referred into it is refused. Requirements: FUN-021 · Verification:
  automated test · Planned location: `crates/phairdlna-core/tests/catalogue_identity.rs`

**AC-11** — *Given* a hard-linked file reachable under two names, *when* it is indexed, *then* one
  catalogue entry exists and both names resolve to it. Requirements: FUN-020 · Verification:
  automated test · Planned location: `crates/phairdlna-core/tests/catalogue_identity.rs`

**AC-12** — *Given* a playlist and a subtitle file in a root, *when* the server has been running,
*then* neither file was modified, renamed or deleted. Requirements: FUN-018, SEC-008 ·
Verification: automated test · Planned location: `crates/phairdlna-core/tests/playlists.rs`

**AC-13** — *Given* a playlist entry that names a file that does not exist, *when* the playlist is
  browsed, *then* the entry appears as unavailable with a reason, and the diagnostics output names
  it. Requirements: UX-004, FUN-015 · Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/playlists.rs`

**AC-14** — *Given* a catalogue that contains a subtitle, *when* a browse is served, *then* no
  additional file or directory is opened while the browse is produced. Requirements: PERF-008,
  FUN-016 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/subtitles.rs`

**AC-15** — *Given* a playlist file whose format is not valid, *when* the root is scanned, *then*
  the scan completes, the playlist is listed with a diagnostic and the other content of the root is
  indexed. Requirements: UX-005, FUN-015 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/playlists.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Playlist parsing, resolution, refusal of network entries | `crates/phairdlna-core/tests/playlists.rs` |
| Sidecar discovery and the additional `res` element | `crates/phairdlna-core/tests/subtitles.rs` |
| Identity key, de-duplication, exclusions, overlapping roots | `crates/phairdlna-core/tests/catalogue_identity.rs` |
| Parser and identity unit tests | `crates/phairdlna-core/src/catalogue/playlist.rs` and `crates/phairdlna-core/src/catalogue/identity.rs` (inline `#[cfg(test)]` modules) |
| Device behaviour per client family | `qa/device-matrix.toml` |

**Method.** The playlist fixtures are plain text files created by the test itself, one per format
and one per malformed case, so the parser is exercised without a device. The identity tests build
a small tree with case-variant names, a decomposed name and a hard link; the expected results
depend on the detected behaviour of the volume, which the test reads from the platform layer
rather than assuming. The subtitle test asserts on the DIDL-Lite document produced by the writer
(F-05), so the two features cannot drift.

**Dependencies.** F-05 (the `res` element and the object model), F-06 (serving the subtitle
bytes), F-07 (the client profile that carries the subtitle capability), F-08 (the raw path, the
containment rule and the exclusion of the cache directory), F-03 (the browse that returns
playlists and containers), the `platform` traits of ADR-0003, and ADR-0007 for the identifier
scheme.

## 12. Open questions and decisions

1. **[OPEN] The playlist format grammars were not verified against a primary specification.** This project's
   research covers the DLNA and UPnP standards and the security landscape; it did not retrieve the M3U, M3U8 or
   PLS format definitions. The descriptions in §3 are therefore marked as design decisions. *Closes when:* the
   format definitions are read from a primary source and the parser is checked against a fixture corpus built
   from real-world playlists.
2. **[OPEN] How a subtitle is offered.** The baseline says "as an additional `res` element" (FUN-016); the
   DLNA and vendor vocabulary observed in the field offers `sec:CaptionInfo` / `sec:CaptionInfoEx` and
   `dlna:ifoFileURI` instead (standards §4.1, §6.5), and the MIME type of `.srt`, `.sub` and `.ass` has no
   registered media type, which INT-051 and INT-064 require to be honest. *Closes when:* a device round shows
   which mechanism the target television families actually accept, and the type table is fixed with the
   candidate values named.
3. **[OPEN] "Where the client profile supports it" has no default.** The profile store is empty at the baseline
   (F-07, INT-100), so a literal reading means no subtitle is ever offered. *Proposed decision:* the profile
   store carries a capability flag with a documented default value, and the default is decided after the first
   device round; until then the feature is inert rather than wrong. *Closes when:* the owner fixes the default
   or the first profiles exist.
4. **[OPEN] Which sidecar wins when several match.** The rules in §4 pick the highest-priority extension, but a
   naming convention that carries the language (`film.en.srt`, `film.de.srt`) is not part of the baseline and
   cannot be assumed. *Closes when:* the owner decides whether the baseline supports one sidecar per media file
   or several with a language attribute.
5. **[OPEN] The identity key and the object identifier disagree about the scope.** FUN-020 scopes the identity
   key by the **volume**, while ADR-0007 derives the object id from the volume identity and the path **relative
   to the root**, so the same file reached through two roots on one volume has two identifiers under ADR-0007
   and one identity key under FUN-020. *Proposed decision:* the identity key stays volume-scoped, the object id
   is derived from it, and the catalogue resolves a second sighting of a file to the existing object — which
   means the identifier changes when a root is added or removed. *Closes when:* the owner confirms this or
   amends ADR-0007.
6. **[OPEN] Which overlapping root owns a shared file.** FUN-022 forbids duplicates but does not say whether the
   file is listed under the first root only, under every root that reaches it, or under a synthetic root. *Closes
   when:* the owner decides; the first-root-wins rule is used in the meantime.
7. **[OPEN] Publishing a playlist as `text/xml`.** The DLNA `DIDL_V` and `DIDL_S` collection profiles constrain
   what a published collection may contain and the research could not resolve them (standards §9 Q12).
   *Closes when:* the DLNA Part 2 profile text is obtained; publishing playlists that way stays out of scope
   until then.
