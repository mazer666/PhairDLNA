# F-05 — DIDL-Lite metadata generation

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-060, INT-061, INT-062, INT-063, INT-064, INT-065, INT-066, INT-067, INT-068,
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
INT-069, INT-070, INT-071, INT-072, INT-073, INT-074, INT-075, INT-076, INT-077, INT-078, INT-079

## 1. Purpose and user story

When a control point (a television's DLNA app) asks the server "what is in this folder?", the server answers
with an XML document called **DIDL-Lite**. DIDL-Lite is the UPnP AV dialect in which a media server describes
its objects: each **item** (a song, a film, a photo) and each **container** (a folder) becomes one element with
attributes and child elements. The client reads the title, the duration and — most importantly — the `res`
element that says *where to fetch the bytes*.

```xml
<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/"
          xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/">
  <item id="a3f1…" parentID="1" restricted="1">
    <dc:title>Nightfall</dc:title>
    <upnp:class>object.item.audioItem.musicTrack</upnp:class>
    <res protocolInfo="http-get:*:audio/mpeg:*" duration="0:03:52.000" size="7340032">http://host/media/a3f1…</res>
  </item>
</DIDL-Lite>
```

**User story.** As a user, I want my television to show my files with their real titles, artists and durations,
and to start playing them straight from the list.

## 2. Scope

**In scope**

- The `didl` module of `phairdlna-core`: the object model, the XML writer and the escaping rules.
- The namespace set and the rule that every used prefix is declared (INT-071).
- The `<item>` / `<container>` attributes, the `upnp:class` values and the `res` attributes (INT-060,
  INT-061, INT-063, INT-064, INT-074, INT-075); the metadata elements `dc:title`, `upnp:class`,
  `upnp:artist`, `dc:creator`, `upnp:album`, `upnp:genre`, `upnp:originalTrackNumber`, `dc:date`,
  `upnp:albumArtURI`.
- The fourth protocolInfo field in its plain and DLNA-flavoured forms (INT-078, INT-079), the
  value-length policy (INT-072) and the second escaping into `<Result>` (INT-073).

**Explicitly out of scope**

- The `Browse` action, its arguments, paging and filters (F-03); the SOAP envelope — except the second escaping, which INT-073 places here (F-02); subtitle sidecars (FUN-016, F-11).
- Delivering the bytes the `res` element points at (F-06); extracting tags, durations and dimensions from media files (F-09); generating artwork and thumbnails (F-10); playlists as DIDL collections; the XSD validation harness, which consumes this output.

## 3. Functional requirements

**INT-060** — Every `<item>` and `<container>` shall carry the attributes `id`, `parentID` and `restricted`;
containers shall additionally carry `childCount`. [SPEC] `id`, `parentID` and `restricted` are required on both
elements; `childCount` is optional in the schema and required here for containers (standards §4.2, §8.5 L1).

**INT-061** — The server shall use the UPnP object classes `object.item.audioItem.musicTrack`,
`object.item.videoItem.movie`, `object.item.videoItem.videoBroadcast`, `object.item.imageItem.photo` and
`object.container.storageFolder` as appropriate. [SPEC] These are CD:4 allowed values (§4.3, §8.5 L3).
[DESIGN] A file maps to `movie`, a live source to `videoBroadcast`, a directory to `storageFolder`.

**INT-062** — The `id` attribute shall be a stable, opaque identifier for the object that does not change between
restarts and does not expose an absolute filesystem path. [SPEC] `id` is a required string attribute (standards
§4.2). [DESIGN] It is the opaque hash of ARCHITECTURE §8 — stable across an index rebuild and free of path
information (PLT-008).

**INT-063** — The `res` element shall carry `protocolInfo`, `duration` (for timed media, formatted
`H*:MM:SS.F*`), `size` and, for images, `resolution` formatted as `<width>x<height>`. [SPEC] `protocolInfo` is
required; `duration` uses the pattern `H*:MM:SS.F*` (any number of hour digits, two minute and second digits,
optional fraction); `resolution` matches `[0-9]+x[0-9]+` (§4.4, §8.5 L6).

**INT-064** — The `protocolInfo` string shall have the form `http-get:*:<mime>:<additional>` and shall name a
MIME type the server actually serves. [SPEC] `res@protocolInfo` is
`<transportProtocol>:<network>:<contentFormat>:<additionalInfo>` (standards §4.4); the MIME type is field three
and is the value the HTTP layer sends as `Content-Type`.

**INT-065** — The server shall XML-escape all title, artist, album, genre and description values: `&`, `<` and `>`
in element text; `&`, `<`, `>`, `"` and `'` in attribute values. [SPEC] XML 1.0 requires `&` and `<` to be escaped
in text, allows `>` as `&gt;`, and requires `"` as `&quot;` and `'` as `&apos;` in attribute values (§4.6, §8.5 L8).

**INT-066** — The server shall include `dc:title` and `upnp:class` for every item and container. [SPEC] They are
required child elements of every DIDL-Lite object (standards §4.2, §8.5 L2).

**INT-067** — The server shall include `upnp:albumArtURI` for audio items for which an artwork file exists, using
an absolute URL the client can fetch over HTTP and without a `dlna:profileID` attribute. [STANDARD/secondary] The
Windows Media Center compatibility guidance says `dlna:profileID` and `xmlns:dlna` must not appear there (§4.5, §8.5 L4, §8.8 N10).

**INT-068** — The server shall include `dc:date` when the file system provides a modification date, formatted as
an ISO 8601 date or date-time. [OBSERVED] Real-world values are inconsistent (`2024-05-28T02:28:47`,
`2022-05-26T16:22:39+0000`); ISO-8601 is the stable choice (standards §4.5, §8.5 L10). The schema constraint on
`dc:date` is still unverified (§9, Q6).

**INT-069** — The server shall include `upnp:artist`, `upnp:album` and `upnp:genre` for audio items when the tags
provide them. [OBSERVED] These are the properties real control points read (§4.5, §8.5 L11).

**INT-070** — The server shall include `upnp:originalTrackNumber` when the audio tags provide it. [OBSERVED]
Listed among the properties observed in real DIDL-Lite (standards §4.5).

**INT-071** — The server shall produce a `DIDL-Lite` document whose root element is `DIDL-Lite` in the namespace
`urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/`, and shall declare every namespace prefix it uses. [SPEC] The
default namespace is the DIDL-Lite namespace; `dc` and `upnp` are its companions (standards §4.1). [OBSERVED]
ReadyMedia bug #316 emitted `sec:` elements without declaring `xmlns:sec`, and the strict Cling parser rejected
the document as invalid XML.

**INT-072** — The server shall truncate a single metadata value to a documented maximum length so that no element
or attribute value exceeds 8 KB in its XML-escaped UTF-8 form. [OBSERVED] The DLNA client implementation guide
asks clients to support values of at most 8192 bytes in XML-escaped UTF-8 form (standards §4.6, §8.5 L9).

**INT-073** — The server shall escape the complete DIDL-Lite document a second time when embedding it into the
`Result` element of the SOAP body, because the document is carried as XML text. [SPEC] The `Result` argument
carries the document as text, so it is escaped once more (§3.3, §4.6). [OBSERVED] A live rclone
response contains `&lt;` and `&#34;` inside `<Result>`.

**INT-074** — The server shall report `res@bitrate` in bytes per second, as the metadata schema defines it. [SPEC]
The schema defines `bitrate` as "the bitrate in bytes/second of the resource" (standards §4.4). [OBSERVED] Some
servers emit bits per second, which confuses clients (§4.4).

**INT-075** — The server shall emit `res@sampleFrequency`, `res@nrAudioChannels` and `res@bitsPerSample` when the
media inspection provides them. [SPEC] All three are optional attributes of `res` (§4.4, §8.5 L6).

**INT-076** — The server shall use `dc:creator` in addition to `upnp:artist` for the artist of an audio item.
[OBSERVED] Both spellings appear in real DIDL-Lite and control points read either (§4.5, §8.5 L11).

**INT-077** — The server shall not emit the `writeStatus` attribute, because its definition could not be verified
against a primary specification. [OPEN] The research could not open the CD:4 Appendix-B row that defines it; it
appears there as a `upnp:` *property*, not an XML attribute (standards §4.2, §9 Q5).

**INT-078** — The DLNA signalling in the fourth `protocolInfo` field shall be additive: the server shall be able
to emit plain `http-get:*:<mime>:*` as well as a DLNA-flavoured string, and plain UPnP clients shall keep
working in both modes. [STANDARD/secondary] The Windows Media Center compatibility flags remove the DLNA
attributes from DIDL-Lite, so the DLNA layer must be separable from the plain layer (standards §6.6, §8.8 N9).

**INT-079** — The server shall emit `DLNA.ORG_PN` for a media resource only when the content actually conforms
to that DLNA Media Format Profile, and shall emit `DLNA.ORG_OP=01` only when byte-range seeking is supported.
[STANDARD/secondary] PROT-01: a renderer that sees a profile ID must be able to play conforming content
(standards §6.2, §8.8 N2). [OBSERVED] `DLNA.ORG_OP=01` means "range (byte) seek supported", `DLNA.ORG_CI=0`
means "not transcoded".

## 4. Technical requirements

- **Module.** The `didl` module of `phairdlna-core` owns the mapping from catalogue objects to
  DIDL-Lite. Escaping happens in exactly one place, with a round-trip test; the XML writer is
  dumb and deterministic (ARCHITECTURE §4.4).
- **Namespaces.** The root declares the default DIDL-Lite namespace and every prefix the document
  uses: `dc` = `http://purl.org/dc/elements/1.1/`, `upnp` =
  `urn:schemas-upnp-org:metadata-1-0/upnp/`, `dlna` = `urn:schemas-dlna-org:metadata-1-0/`
  (standards §4.1). [DESIGN] A prefix is declared only when the document uses it, always on the
  root element; the `sec:` namespace is not used at all.
- **The `res` URI.** One function builds the media URI and the HTTP layer serves exactly that URI
  (INT-090). The URI carries the opaque object identifier, never a filesystem path (INT-062, PLT-008).
  [DESIGN] The documented maximum value length is 8192 bytes of XML-escaped UTF-8; truncation applies to
  the *raw* value such that the escaped form fits, so an entity is never cut in half, and the truncated
  value ends with `…` (U+2026) so the user can see it was cut.
- **protocolInfo construction.** Field three is the MIME type from the shared MIME registry used
  by F-04 and F-06. [DESIGN] Field four follows the *mode*: plain mode writes `*`; DLNA mode
  writes `DLNA.ORG_PN=<profile>;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=<flags>` (§6.4).
  The `ORG_FLAGS` bits of the reference implementation are bit 30 time-based seek, bit 29
  byte-based seek, bit 28 playcontainer, bit 24 streaming transfer mode, bit 23 interactive, bit
  22 background, bit 21 connection stalling, bit 20 DLNA v1.5 ([OBSERVED], standards §6.4). The
  mode is a property of the client profile (F-07), never of the file.
- **Second escaping and validation.** The document is serialised to a string, then escaped once more
  when written into the `Result` element of the SOAP body (INT-073). The official DIDL-Lite XSD is
  public and is vendored for offline validation (qa-performance §1.3, item 4); a planned gate validates
  every generated document against it, and snapshot tests pin the element order.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: serialising 1 000 items takes under 20 ms median on the reference hardware of
  qa-performance §5.3, measured with `criterion` (qa-performance §5.2) over ≥ 30 iterations, with
  median and p95 reported; one document is bounded by the number of entries and by the value-length
  policy, so a browse of a large folder cannot produce an unbounded document (SEC-061).

## 6. Usability and accessibility

- The title the user sees in the television app is `dc:title`; a file whose name contains characters
  that XML cannot carry must still show a readable title (INT-065, INT-072). [UX-004] A truncated
  value shall be recognisable as truncated, so the user is not told a lie about a long title
  (INT-072). [UX-010] DLNA profile identifiers are jargon and stay out of user-facing text;
  [UX-001] nothing here needs a graphical interface.

## 7. Security and privacy

- File names and tags are untrusted input (SEC-007). Escaping keeps a file named
  `</dc:title><evil/>` from breaking the document or injecting elements into the client's parser
  (INT-065).
- The `id` attribute and the `res` URI shall not expose an absolute path or the user's directory
  layout (INT-062, PLT-008, SEC-093).
- `upnp:albumArtURI` points at the server's own HTTP endpoint; it is never a URL taken from a tag
  file, so a malicious tag cannot make the client fetch a third-party address. The document contains
  no DOCTYPE and no external entity (INT-024, SEC-010), and document size and every single value are
  bounded (INT-072, SEC-061), so a huge tag cannot exhaust memory.

## 8. Interoperability

- Normative: the namespace set (standards §4.1); the item/container attributes (§4.2); the
  `upnp:class` list (§4.3); the `res` attribute set, duration format and bitrate unit (§4.4); the
  escaping rules (§4.6).
- [OBSERVED] ReadyMedia bug #316: an undeclared `sec:` prefix made the strict Cling parser reject
  the whole response (§4.1). [OBSERVED] rclone declares `dlna` and `sec` on the root element.
- [STANDARD/secondary] The Windows Media Center rule that `dlna:profileID` must not appear in
  `upnp:albumArtURI` (§4.5). [OBSERVED] MiniDLNA and Gerbera nevertheless emit it.
- [OBSERVED] rclone maps a plain video file to `object.item.videoItem` rather than `.movie` (§4.3);
  INT-061 fixes the finer value, which the CD:4 class table allows. [OBSERVED] Some renderers are
  reportedly unhappy with `&#34;` and rewrite it before use (§4.6); we emit `&quot;` only in attribute
  values and leave `"` unescaped in element text.

## 9. Error and recovery behaviour

| Situation | Behaviour | Basis |
| --- | --- | --- |
| A tag is missing or unreadable | the optional element is omitted; the item is still described | INT-066, INT-069 |
| The file name cannot be represented in XML | the value is escaped or truncated | INT-065, INT-072 |
| The media inspection found no duration | `res@duration` is omitted rather than guessed | INT-063 |
| The artwork file disappeared after indexing | `upnp:albumArtURI` is omitted | INT-067 |
| A value exceeds the maximum length | the value is truncated on a character boundary, marked `…` | INT-072 |
| The DLNA profile of a file cannot be determined | the plain `*` fourth field is written | INT-078, INT-079 |

A metadata problem never fails the whole browse: one broken file must not cost the user the other 99 files in
the folder (F-03).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a generated document, *when* it is parsed, *then* the root element is `DIDL-Lite`
in the namespace `urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/`, every prefix the document uses is
declared on it, and each item and container carries `id`, `parentID` and `restricted` while each
container also carries `childCount`. Requirements: INT-071, INT-060 · Verification: protocol
conformance test · Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-02** — *Given* a music file, a film, an image and a folder, *when* their items are rendered, *then* their
`upnp:class` values are `object.item.audioItem.musicTrack`, `object.item.videoItem.movie`,
`object.item.imageItem.photo` and `object.container.storageFolder`, and the `id` attribute of one object is
identical before and after a restart and contains no path separator. Requirements: INT-061, INT-062 ·
Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-03** — *Given* an item with a known duration, an image with known dimensions and an audio file with a known
bitrate, *when* the `res` element is rendered, *then* it carries a `protocolInfo` with four colon-separated
fields whose third is a MIME type the HTTP layer serves for the same object, a `size`, a `duration` matching
`H*:MM:SS.F*`, a `resolution` matching `<width>x<height>` for the image and a `bitrate` in bytes per second.
Requirements: INT-063, INT-064, INT-074, INT-051 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/didl_lite.rs`

**AC-04** — *Given* a title containing `&`, `<`, `>`, `"` and `'`, *when* it is rendered as element
text and as an attribute value, *then* `&`, `<` and `>` are escaped in the text, all five characters
are escaped in the attribute, and the document still parses; and *when* a generated document is
embedded into the `Result` element of a SOAP response and the response is parsed and unescaped twice,
*then* the recovered document equals the original document. Requirements: INT-065, INT-073, INT-042 ·
Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/didl_escaping.rs`

**AC-05** — *Given* a title of 20 000 characters containing characters that expand when escaped,
*when* the document is rendered, *then* the escaped value is at most 8192 bytes long and ends with
the truncation marker. Requirements: INT-072 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/didl_lite.rs`

**AC-06** — *Given* an audio item with artwork, *when* the item is rendered, *then*
`upnp:albumArtURI` holds an absolute HTTP URL of this server and carries no `dlna:profileID`
attribute and no `xmlns:dlna` declaration. Requirements: INT-067 · Verification: protocol
conformance test · Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-07** — *Given* a file whose modification time is known and an audio item whose tags provide an artist and a
track number, *when* the item is rendered, *then* `dc:date` matches `YYYY-MM-DD` or `YYYY-MM-DDThh:mm:ss`, and
`upnp:artist`, `dc:creator` and `upnp:originalTrackNumber` carry the tagged values. Requirements: INT-068,
INT-069, INT-070, INT-076 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/didl_lite.rs`

**AC-08** — *Given* the plain mode and the DLNA mode, *when* the same item is rendered twice,
*then* the plain document contains `http-get:*:<mime>:*` and the DLNA document contains the
`DLNA.ORG_PN`, `DLNA.ORG_OP`, `DLNA.ORG_CI` and `DLNA.ORG_FLAGS` tokens, and both parse.
Requirements: INT-078, INT-055, INT-079 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-09** — *Given* a file whose content does not conform to any known DLNA Media Format Profile, *when* the
item is rendered, *then* no `DLNA.ORG_PN` token is emitted, `DLNA.ORG_OP=01` is emitted only when byte-range
seeking is available, and no attribute named `writeStatus` appears anywhere. Requirements: INT-079, INT-077 ·
Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Document shape and class mapping | `crates/phairdlna-core/tests/didl_lite.rs` |
| Escaping round-trip (property test) | `crates/phairdlna-core/tests/didl_escaping.rs` (qa-performance §2.6, P1) |
| Writer, truncation and namespace unit tests | `crates/phairdlna-core/src/didl/` (inline `#[cfg(test)]` modules) |
| Shape snapshots and schema validation | `crates/phairdlna-core/tests/snapshots/` (`insta`); vendored `qa/xsd/didl-lite.xsd` |
| Fuzzing of the parser | `fuzz/fuzz_targets/fuzz_didl_xml.rs`, corpus `qa/corpus/didl/*.xml` (qa-performance §4.2) |
| Device behaviour | `qa/device-matrix.toml` (qa-performance §3.2) |

**Method.** The document is produced from an in-memory object model, so the module is testable without a
catalogue, a filesystem or a socket. The escaping tests are property-based: generate arbitrary strings, render
them, parse them back and compare (qa-performance §2.6); the XSD gate proves schema-legality.

**Dependencies.** F-03 (the `Browse` action that consumes the document), F-06 (the media URI and its MIME type),
F-07 (the client profile that selects the mode), F-09 (the metadata), F-16 (the parsing and size limits).

## 12. Open questions and decisions

1. **[OPEN] The truncation policy and `writeStatus` (INT-072, INT-077).** INT-072 asks for "a
   documented maximum length"; this document proposes 8192 bytes of escaped UTF-8, truncation on the
   raw value and a trailing `…`. INT-077 forbids `writeStatus` because the research could not verify
   its definition (standards §9, Q5). *Closes when:* the owner approves the number and the marker, and
   when the CD:4 Appendix-B row is available.
2. **[OPEN] The DLNA Media Format Profile list.** Only the reference implementation's list is available
   and the per-profile conformance rules are not (standards §9, Q2). Until a conformance check exists,
   the server cannot claim `DLNA.ORG_PN` for anything. *Proposed decision:* ship the plain mode first.
3. **[OPEN] Which `DLNA.ORG_FLAGS` bits to set.** The bit layout is documented by the reference
   implementation (standards §6.4), but no baseline requirement fixes our value. *Proposed decision:*
   byte-based seek (bit 29) plus streaming transfer mode (bit 24) for audio and video, interactive
   (bit 23) for images. *Closes when:* the owner confirms.
4. **[OPEN] `searchable` on containers, the order of several `res` elements and `dc:date` precision.**
   The `searchable` attribute appears in the wild (standards §4.2) but the baseline does not mention
   it; a subtitle sidecar adds a second `res` (FUN-016, F-11); and the schema constraint on `dc:date`
   was not opened (standards §9, Q6), so this document emits a date only. [PROPOSED — not in the baseline] extend INT-060 with "containers shall carry `searchable`" once the client's
   `Search` capability is defined (F-03, INT-035). *Proposed decision:* the primary media resource
   comes first in the element order.
