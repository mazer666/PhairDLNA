# F-05 — DIDL-Lite metadata generation

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-060, INT-061, INT-062, INT-063, INT-064, INT-065, INT-066, INT-067, INT-068,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
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
  public and is vendored for offline validation (qa-performance §1.3, item 4) — the vendoring is
  still to be done; a planned gate validates
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
  no DOCTYPE and no external entity (INT-024, SEC-010), and every metadata value of the document is
  bounded (INT-072, SEC-061), so a huge tag cannot exhaust memory. The identifier of an object and
  the address of the server are not cut: they are produced by the catalogue and by the server, they
  carry no data of the file, and cutting them would break the address (F-05 §9).

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
| A value of the catalogue that is not a metadata value (the identifier, the address) | not truncated: the identifier is produced by the catalogue and the address by the server, and cutting them would break the address | INT-062, INT-090 |
| The DLNA profile of a file cannot be determined | the plain `*` fourth field is written | INT-078, INT-079 |

A metadata problem never fails the whole browse: one broken file must not cost the user the other 99 files in
the folder (F-03).

## 10. Acceptance criteria and traceability

All criteria below are **implemented**; every one of them has a test. The *location* of a
criterion names the file that proves it, and the file exists today.

**AC-01** — *Given* a generated document, *when* it is parsed, *then* the root element is `DIDL-Lite`
in the namespace `urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/`, every prefix the document uses is
declared on it, and each item and container carries `id`, `parentID` and `restricted` while each
container also carries `childCount`. Requirements: INT-071, INT-060 · Verification: protocol
conformance test · Location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-02** — *Given* a music file, a film, an image and a folder, *when* their items are rendered, *then* their
`upnp:class` values are `object.item.audioItem.musicTrack`, `object.item.videoItem.movie`,
`object.item.imageItem.photo` and `object.container.storageFolder`, and the `id` attribute of one object is
identical before and after a restart and contains no path separator. Requirements: INT-061, INT-062 ·
Verification: protocol conformance test · Location: `crates/phairdlna-core/tests/didl_lite.rs`
*Partly implemented:* the class values and the path-freeness of the identifier are proven; the
identifier is produced by the catalogue, and its stability across a restart is F-08's job (the hash of
ARCHITECTURE §8).

**AC-03** — *Given* an item with a known duration, an image with known dimensions and an audio file with a known
bitrate, *when* the `res` element is rendered, *then* it carries a `protocolInfo` with four colon-separated
fields whose third is a MIME type the HTTP layer serves for the same object, a `size`, a `duration` matching
`H*:MM:SS.F*`, a `resolution` matching `<width>x<height>` for the image and a `bitrate` in bytes per second.
Requirements: INT-063, INT-064, INT-074, INT-051 · Verification: protocol conformance test · Location:
`crates/phairdlna-core/tests/didl_resource.rs`

**AC-04** — *Given* a title containing `&`, `<`, `>`, `"` and `'`, *when* it is rendered as element
text and as an attribute value, *then* `&`, `<` and `>` are escaped in the text, all five characters
are escaped in the attribute, and the document still parses; and *when* a generated document is
embedded into the `Result` element of a SOAP response and the response is parsed and unescaped twice,
*then* the recovered document equals the original document. Requirements: INT-065, INT-073, INT-042 ·
Verification: protocol conformance test · Location: `crates/phairdlna-core/tests/didl_escaping.rs`

**AC-05** — *Given* a title of 20 000 characters containing characters that expand when escaped,
*when* the document is rendered, *then* the escaped value is at most 8192 bytes long and ends with
the truncation marker. Requirements: INT-072 · Verification: automated test · Location:
`crates/phairdlna-core/tests/didl_lite.rs`

**AC-06** — *Given* an audio item with artwork, *when* the item is rendered, *then*
`upnp:albumArtURI` holds an absolute HTTP URL of this server and carries no `dlna:profileID`
attribute and no `xmlns:dlna` declaration. Requirements: INT-067 · Verification: protocol
conformance test · Location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-07** — *Given* a file whose modification time is known and an audio item whose tags provide an artist and a
track number, *when* the item is rendered, *then* `dc:date` matches `YYYY-MM-DD` or `YYYY-MM-DDThh:mm:ss`, and
`upnp:artist`, `dc:creator` and `upnp:originalTrackNumber` carry the tagged values. Requirements: INT-068,
INT-069, INT-070, INT-076 · Verification: protocol conformance test · Location:
`crates/phairdlna-core/tests/didl_lite.rs`

**AC-08** — *Given* the plain mode and the DLNA mode, *when* the same item is rendered twice,
*then* the plain document contains `http-get:*:<mime>:*` and the DLNA document contains the
`DLNA.ORG_PN`, `DLNA.ORG_OP`, `DLNA.ORG_CI` and `DLNA.ORG_FLAGS` tokens, and both parse.
Requirements: INT-078, INT-055, INT-079 · Verification: protocol conformance test · Location:
`crates/phairdlna-core/tests/didl_resource.rs`

**AC-09** — *Given* a file whose content does not conform to any known DLNA Media Format Profile, *when* the
item is rendered, *then* no `DLNA.ORG_PN` token is emitted, `DLNA.ORG_OP=01` is emitted only when byte-range
seeking is available, and no attribute named `writeStatus` appears anywhere. Requirements: INT-079, INT-077 ·
Verification: protocol conformance test · Location: `crates/phairdlna-core/tests/didl_resource.rs`

## 11. Test strategy and dependencies

**Test locations.** The three rows below exist today; the three rows after them are still planned.

| Purpose | Path | State |
| --- | --- | --- |
| Document shape, class mapping, the length policy, the artwork and the metadata | `crates/phairdlna-core/tests/didl_lite.rs` | **exists** |
| The attributes of the resource and the two modes of the `protocolInfo` | `crates/phairdlna-core/tests/didl_resource.rs` | **exists** |
| Escaping round-trip and the property tests | `crates/phairdlna-core/tests/didl_escaping.rs` | **exists** |
| Writer, truncation, namespace, protocol and mapping unit tests | `crates/phairdlna-core/src/didl/` (inline `#[cfg(test)]` modules) | **exists** |
| Shape snapshots and schema validation | `crates/phairdlna-core/tests/snapshots/` (`insta`); vendored `qa/xsd/didl-lite.xsd` | planned |
| Fuzzing of the parser | `fuzz/fuzz_targets/fuzz_didl_xml.rs`, corpus `qa/corpus/didl/*.xml` (qa-performance §4.2) | planned |
| Device behaviour | `qa/device-matrix.toml` (qa-performance §3.2) | planned |

**Method.** The document is produced from an in-memory object model, so the module is testable without a
catalogue, a filesystem or a socket. The tests read the document with the XML reader of the project
(`crates/phairdlna-core/src/upnp/xml_reader.rs`), not with a string match: the reader resolves the
namespaces and decodes the entities, so a test that uses it proves that a client can read the document.
The escaping tests are property-based: generate arbitrary strings, render them, parse them back and
compare (qa-performance §2.6).

**The property test without a dependency.** The project has no property-testing dependency, and adding one
(`proptest`) would cost a licence assessment and a change of the supply-chain policy (qa-performance §2.6).
The property test therefore uses a small deterministic generator inside the test file: the same run produces
the same values, and a failure can be reproduced exactly. *This is a deviation from qa-performance §2.6 and
needs the owner's confirmation.* The XSD gate proves schema-legality and is still planned.

**Dependencies.** F-03 (the `Browse` action that consumes the document), F-06 (the media URI and its MIME type),
F-07 (the client profile that selects the mode), F-09 (the metadata), F-16 (the parsing and size limits).

## 12. Open questions and decisions

### 12.1 Decisions taken by the implementation (WP3)

These decisions are implemented and covered by a test. The ones marked **[owner decision]** change a
documented behaviour and are reported to the owner with this work package.

**D-01 — The truncation policy (INT-072).** A value is cut so that its XML-escaped UTF-8 form is at most
8192 bytes. The cut happens on the raw value and on a character boundary, so an entity is never cut in half,
and the value ends with `…` (U+2026). The escaped form of the cut value therefore never exceeds the budget.
*Implemented in `crates/phairdlna-core/src/didl/value.rs`. **[owner decision]** the number and the marker
are still open in question 1 below.*

**D-02 — `writeStatus` is never written (INT-077).** The attribute is not part of the model at all, so it
cannot be written by accident. *Implemented.*

**D-03 — The plain mode is the default (INT-078, INT-079).** The server starts in the plain mode and writes
`http-get:*:<mime>:*`. `DLNA.ORG_PN` is written only when the catalogue knows that the content conforms to a
media format profile; `DLNA.ORG_OP=01` only when the server really serves byte ranges; `DLNA.ORG_CI=0`
always in the DLNA mode. The mode is chosen by the dispatcher, not by the file, because the mode belongs to
the client profile (F-07). *Implemented.*

**D-04 — The `dlna` namespace is never declared.** The DLNA signalling lives in the fourth field of
`res@protocolInfo`, which is an attribute value and not a namespaced element, and `dlna:profileID` is
forbidden on `upnp:albumArtURI` (`INT-067`). A prefix that is used is declared, and nothing else is
(`INT-071`). *Implemented.*

**D-05 — The artwork address is built by the server (`INT-067).** The catalogue carries the opaque
identifier of the artwork, not an address; the address is built from the base URL of the server, so that a
tag file cannot make the client fetch a third-party address. The path is `/art/<id>`; the HTTP layer has
to serve exactly that address, as it has to serve `/media/<id>` (`INT-090`). **[owner decision]** the path
is a proposal, because no requirement names it.

**D-06 — `restricted` is an attribute, not an element (`INT-060).** The placeholder writer of WP2 wrote a
`<restricted>` child element; the schema has no such element, and the attribute is what `INT-060` asks for.
The document now carries `restricted="1"` and nothing else.

**D-07 — The two mandatory properties survive every filter (`INT-066`, `INT-034`).** `dc:title` and
`upnp:class` are written on every item and container, whatever the client asked for: the schema makes them
mandatory, and a document without them is not a DIDL-Lite document. The filter narrows the other
properties. `INT-034` asks for "the elements the client asked for"; that is satisfied, because the asked-for
elements are returned. **[owner decision]** this changes the behaviour of the WP2 placeholder, which dropped
`upnp:class` when the filter did not name it.

**D-08 — The class of a file the catalogue cannot describe.** The five classes of `INT-061` are used for the
five kinds the catalogue knows. A file of an unknown kind gets the generic class `object.item`, which the
schema allows. **[owner decision]** the requirement names five classes and does not say what an unknown
kind maps to.

**D-09 — The document carries no XML declaration.** The document is carried as the text of another document,
and a declaration is only allowed at the start of a document. The client receives the root element.

**D-10 — The dispatcher takes the base address and the policy.** `Dispatcher::new` takes the catalogue
*and* the base address of the server, because the document builds its links from it, and it starts in
the plain mode. The new constructor `Dispatcher::with_policy` takes the mode as its third argument,
and `browse` takes the base address and the policy. The signature of `Dispatcher::new` and of `browse`
therefore changed with WP3. Nothing outside the crate uses them yet, and the server (F-13) is the first
consumer.

### 12.2 Open questions

1. **[OPEN] The truncation policy and `writeStatus` (INT-072, INT-077).** INT-072 asks for "a
   documented maximum length"; this document proposes 8192 bytes of escaped UTF-8, truncation on the
   raw value and a trailing `…`. INT-077 forbids `writeStatus` because the research could not verify
   its definition (standards §9, Q5). *Implemented as D-01 and D-02; closes when:* the owner approves the
   number and the marker, and when the CD:4 Appendix-B row is available.
2. **[OPEN] The DLNA Media Format Profile list.** Only the reference implementation's list is available
   and the per-profile conformance rules are not (standards §9, Q2). Until a conformance check exists,
   the server cannot claim `DLNA.ORG_PN` for anything, so the catalogue leaves the profile empty and the
   server stays in the plain mode (D-03). *Closes when:* the conformance rules exist.
3. **[OPEN] Which `DLNA.ORG_FLAGS` bits to set.** The bit layout is documented by the reference
   implementation (standards §6.4), but no baseline requirement fixes our value. *Proposed decision:*
   byte-based seek (bit 29) plus streaming transfer mode (bit 24) for audio and video, interactive
   (bit 23) for images. *Closes when:* the owner confirms. The bits and the format of the token are
   implemented (`DlnaFlags`), but no policy sets them yet.
4. **[OPEN] `searchable` on containers, the order of several `res` elements and `dc:date` precision.**
   The `searchable` attribute appears in the wild (standards §4.2) but the baseline does not mention
   it; a subtitle sidecar adds a second `res` (FUN-016, F-11); and the schema constraint on `dc:date`
   was not opened (standards §9, Q6), so this document emits a date only. [PROPOSED — not in the baseline] extend INT-060 with "containers shall carry `searchable`" once the client's
   `Search` capability is defined (F-03, INT-035). *Proposed decision:* the primary media resource
   comes first in the element order. *Partly implemented:* the containers carry `searchable="1"`, because
   `Search` exists (F-03, INT-035) and the property of the catalogue is a search of the title; the
   baseline still does not name the attribute, so the owner has to confirm it.
5. **[OPEN] The `ORG_FLAGS` token layout.** The research says only "<32 hex>" (standards §6.4). The
   reference implementation writes the significant digits first and pads the rest with zeros
   (`01700000000000000000000000000000`); this implementation writes the same way. A client that parses the
   token as a plain hexadecimal number would read a different value. *Closes when:* the owner confirms the
   layout or a capture of a real client settles it.
6. **[OPEN] The filter and the mandatory properties.** `INT-034` and `INT-066` pull in two directions, and
   D-07 resolves them in favour of the schema. *Closes when:* the owner confirms D-07.
