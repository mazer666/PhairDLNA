# F-03 — ContentDirectory browsing and search

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-030, INT-031, INT-032, INT-033, INT-034, INT-035, INT-036,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
INT-037, INT-038, INT-039, INT-040, INT-041, INT-042, INT-043, INT-046, INT-047, INT-048, INT-049

## 1. Purpose and user story

The **ContentDirectory** service is the part of a media server a client actually talks to. Its `Browse` action answers "what is inside this folder?", and the
answer is a **DIDL-Lite** document — XML describing the items and folders, each with a title, a class (`upnp:class`, for example `object.item.videoItem.movie`)
and, for media, a `res` element with the URL the client can fetch.

A **control point** is the client application on the television or phone. It browses by object identifier: `0` is the root, `1` is the content directory, and
every folder and item has its own opaque identifier.

**User story.** As a user, I want my television's DLNA app to browse my folders and find a film in a couple of taps, so that the app shows the tree my files
really have, with correct names and counts, and so that a folder it never heard of produces a clear "no such object" error instead of an empty list that looks
like data loss.

## 2. Scope

**In scope** — the `ContentDirectory:1` action set: `Browse`, `GetSearchCapabilities`, `GetSortCapabilities`, `GetSystemUpdateID`, plus `Search` (INT-036); the
argument semantics of `Browse` and the meaning of its four output arguments; the object-identifier scheme seen by the client: `0`, `1` and the opaque catalogue
identifiers (INT-038); the error codes `701 No Such Object` and `720`; the advertising of capabilities and the exclusion of everything that is not a
ContentDirectory action (INT-048).

**Explicitly out of scope** — the DIDL-Lite content itself: the element set, the escaping, the `res` element and the double escaping into `<Result>` (F-05). The
`ConnectionManager` actions (F-04), the device description, the SCPD generation and the SOAP envelope (F-02). Write actions: `CreateObject`, `DestroyObject`,
`UpdateObject`, `MoveObject`, `ImportResource`, `ExportResource`, `DeleteResource` (INT-040). The CD:2–CD:4 extra actions (`GetFeatureList`,
`GetServiceResetToken`, `GetSortExtensionCapabilities`, the device-mode and permissions actions): we advertise `ContentDirectory:1` (INT-028).

## 3. Functional requirements

**INT-030** — The server shall implement the `Browse` action with the arguments `ObjectID`, `BrowseFlag`, `Filter`, `StartingIndex`, `RequestedCount`,
`SortCriteria` and `SearchCriteria`. [SPEC] `Browse` is required in ContentDirectory:1 and takes `ObjectID`, `BrowseFlag`, `Filter`, `StartingIndex`,
`RequestedCount` and `SortCriteria` (standards §1.2, §8.3 C1). [DESIGN] The six CD:1 arguments are declared in the SCPD; the parser also accepts a
`SearchCriteria` element and ignores it, because that argument belongs to `Search` and to CD:2 (standards §1.2). Raised in §12.

**INT-031** — The `Browse` response shall contain `Result`, `NumberReturned`, `TotalMatches` and `UpdateID` exactly once each. [SPEC] These are the four output
arguments of `Browse` (standards §1.2, §8.3 C2).

**INT-032** — The server shall honour `StartingIndex` and `RequestedCount` and shall never return more items than `RequestedCount` permits; `RequestedCount` of
`0` shall mean "all items". [SPEC] (standards §8.3 C5, which also fixes `StartingIndex` to 0 for a metadata browse). [DESIGN] "All items" is still bounded by
the page limit (SEC-061); when it truncates the page, `TotalMatches` still reports the true total.

**INT-033** — The server shall honour `BrowseFlag` such that `BrowseMetadata` returns only the container metadata with `NumberReturned` and `TotalMatches` both
equal to 1, and `BrowseDirectChildren` returns its children with `TotalMatches` equal to the total number of children independent of `StartingIndex`. [SPEC] The
CD:1 wording is normative: "If BrowseMetadata is specified in the BrowseFlags, then NumberReturned = 1", and `TotalMatches` counts the objects in the container,
"independent of the starting index" (standards §1.2, §8.3 C2).

**INT-034** — The server shall apply the `Filter` argument, shall support the wildcard `*` that clients send constantly, and shall return the elements the
client asked for when the filter is understood. [SPEC] The filter is `*` or a comma-separated list of property names (standards §8.3 C6).

**INT-035** — The server shall implement `GetSearchCapabilities` and `GetSortCapabilities` and shall return only capabilities it actually implements. [SPEC]
Both actions are required in ContentDirectory:1 (standards §1.2, §8.3 C1, C10). [DESIGN] The lists name only the properties the server can really use,
and they grow with the catalogue: `SearchCapabilities` = `dc:title`; `SortCapabilities` = `dc:title,upnp:class`. The catalogue module (F-08, F-09) adds
`dc:creator`, `dc:date` and `res@size` when it can search and sort by them — see the open question in §12.

**INT-036** — The server shall implement `Search` for the capability strings it advertises in `GetSearchCapabilities`. [SPEC] `Search` is optional in CD:1 and
required from CD:2; the research recommends it because it is cheap and widely called (standards §1.2, §8.3 C8). [DESIGN] The expression grammar is `dc:title
contains "text"`, over the properties `GetSearchCapabilities` advertises.

**INT-037** — The server shall implement `GetSystemUpdateID` and shall keep the returned value monotonically non-decreasing, changing whenever the catalogue
changes. [SPEC] `GetSystemUpdateID` is required; the monotonicity is the practical meaning of a cache-invalidation counter (standards §8.3 C11).

**INT-038** — The server shall return the object ID `0` for the root container and the standard ID `1` for the content directory root, consistently with its own
hierarchy. [SPEC] The standard container identifiers are `0` (root) and `1` (content directory) (standards §1.2, §8.3 C2). [DESIGN] `0` and `1` are the only
numeric identifiers; every other object carries an opaque catalogue identifier (INT-062, ARCHITECTURE §8).

**INT-039** — The server shall return an UPnP error `701 No Such Object` for an unknown `ObjectID` instead of an empty successful result. [SPEC] (standards §8.3
C3, which uses the same code for a container that cannot be described).

**INT-040** — The server shall not implement write operations (`CreateObject`, `DestroyObject`, `UpdateObject`, `MoveObject`, `ImportResource`,
`ExportResource`, `DeleteResource`) in the baseline; those actions shall be absent from the SCPD or shall return an error rather than a partial implementation.
[SPEC] These are optional CD:2 actions (standards §1.2). [DESIGN] They are absent from the SCPD; a request for one is answered with the UPnP fault `401 Invalid
Action` (F-02).

**INT-041** — The server shall return `NumberReturned` and `TotalMatches` as non-negative integers and shall not exceed `TotalMatches`. [SPEC] Both are `ui4`
(standards §3.3).

**INT-042** — The server shall return a `Result` document that is valid DIDL-Lite XML and parses independently of the surrounding SOAP envelope. [SPEC] The
`Result` element carries the DIDL-Lite document as text (standards §3.3, §8.3 C9).

**INT-043** — The server shall return the container's `UpdateID` for a `Browse` of a container and the `SystemUpdateID` for a `Browse` of object ID `0`. [SPEC]
The CD:1 rule: "ContainerUpdateID … of the container being described if a container is specified in ObjectID. … If the ObjectID is zero, then the UpdateID
returned is SystemUpdateID" (standards §1.2, §8.3 C4). [DESIGN] We advertise `ContentDirectory:1` (F-02), so the CD:1 rule is the one implemented; CD:4 always
returns the `SystemUpdateID` (standards §1.2).

**INT-046** — When `TotalMatches` cannot be computed, the server shall return `TotalMatches` of `0` with a non-zero `NumberReturned`, and shall return the SOAP
error `720` when `NumberReturned` is also zero. [SPEC] The CD:1 wording is normative (standards §1.2, §8.3 C3). [DESIGN] Our index can always count the
children, so this is the fallback path for a container whose size cannot be determined; it must still exist, because a client that receives `TotalMatches = 0`
with `NumberReturned = 0` treats the answer as broken.

**INT-047** — The server shall accept `SortCriteria` in the empty form and in the `+property` and `-property` forms, and shall advertise in
`GetSortCapabilities` only the properties it can actually sort by. [SPEC] (standards §8.3 C7). [DESIGN] `+` is ascending, `-` descending; an empty
`SortCriteria` means the catalogue's stable order.

**INT-048** — The server shall not advertise or implement any non-standard ContentDirectory action; a client-specific extension shall be documented as a vendor
extension and kept out of the standard service description. [SPEC] `GetFreeSpace`, `GetUpdateCounter`, `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal`
and `GetShareIndexInBrowse` are **not** ContentDirectory actions in CD:1–CD:4; none is implemented by MiniDLNA or ReadyMedia, and `GetTransferTotal` is a
*field* of `GetTransferProgress` (standards §0.3, §8.3 C12–C13, §9 Q8). [DESIGN] The SCPD contains only standard actions; a vendor action would live in a
separate namespace and be documented as non-standard.

**INT-049** — The server shall expose `SearchCapabilities` and `SortCapabilities` as comma-separated lists of the property names it supports. [SPEC] The example
in CD:4 is `dc:title,dc:creator,dc:date,upnp:class,res@size` (standards §1.2, §8.3 C10). [DESIGN] The lists name plain properties without a `+`/`-` prefix; the
prefix belongs to the `SortCriteria` *argument* (INT-047). Raised in §12.

## 4. Technical requirements

- **Action handlers.** `ContentDirectory::browse`, `::search`, `::get_search_capabilities`, `::get_sort_capabilities` and `::get_system_update_id` are plain
  functions with typed requests and typed responses, dispatched by the SOAP dispatcher of F-02 (ARCHITECTURE §4.3). They never touch the filesystem.

- **Catalogue access.** A browse reads the SQLite index only (ARCHITECTURE §5): it answers "children of this object, ordered, offset, limited" and "how many
  children"; a browse never triggers a directory scan (PERF-008).

- **Page arithmetic.** `StartingIndex` is an offset into the ordered child list, `RequestedCount` a limit, additionally capped by the server's page limit
  (SEC-061). `NumberReturned` counts the items in the `Result`; `TotalMatches` counts the children, independent of offset and limit (INT-033, INT-041).

- **Browse flags.** `BrowseMetadata` returns the container itself as a single `<container>` element; `BrowseDirectChildren` returns the children. The
  `BrowseFlags` value is a bit field; an unknown flag combination yields the UPnP fault `402 Invalid Args` (F-02).

- **Filter.** `*` means "every property"; a comma-separated list means "these properties". A property the server does not know is ignored rather than rejected,
  so a client asking for a property of a newer ContentDirectory version still gets an answer (INT-034).

- **Sort.** The catalogue order is the fallback; `+property` sorts ascending and `-property` descending by the named property. Only the properties of INT-035
  are sortable (INT-047).

- **Update identifiers.** The service holds one `SystemUpdateID` counter and one `ContainerUpdateID` per container; both increase when the catalogue changes
  (INT-037, INT-043). [DESIGN] The counters are `u32`, monotonically non-decreasing, and never wrap back to a smaller value.

- **Error mapping.** An unknown object identifier becomes the UPnP fault `701 No Such Object`; a browse of a non-container is the same fault; a container of
  undeterminable size that yields nothing is the SOAP error `720` (INT-039, INT-046).

- **SOAP response.** The body is the envelope of standards §3.3 with the arguments `Result`, `NumberReturned`, `TotalMatches` and `UpdateID` in that order.
  [DESIGN] The `Result` element carries the DIDL-Lite document **escaped a second time**, because it is carried as XML text (standards §4.6, §8.3 C9). The
  escaping itself is owned by F-05 (INT-073); this feature asserts that the element carries escaped XML (AC-12, AC-13).

- **Module boundary.** The handlers live in `phairdlna-core/src/upnp/content_directory.rs`; they read the catalogue through the read-only interface and never
  hold a lock across I/O (ARCHITECTURE §10).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: a `Browse` of a container with 10 000 children returns its first page within 100 ms (PERF-005). Method: an automated benchmark over a
  generated catalogue, ≥ 30 iterations, median and p95 reported (PERF-011).

- [DESIGN] Target: a browse performs no filesystem access (PERF-008). Method: an automated test asserting that no filesystem handle is opened during the
  action.

- [DESIGN] Target: the generated `Result` stays within the server's page and document-size limits (SEC-061); the item count in one response is bounded by the
  page limit, whatever the client asks for.

- [DESIGN] Target: the catalogue lookup is an index scan, never a full table scan (PERF-009). Method: a query-plan assertion in the index test.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effect is the tree, the titles and the counts the user sees on the television.

- [UX-004] When a client asks for an object that does not exist, the log record shall name the object identifier and the redacted client address, so the user
  can tell "the file was moved" from "the app is broken".

- [UX-010] The titles the user sees come from the file names; the onboarding text shall not mention object identifiers.

- [UX-001] Everything here is operable without a GUI. The diagnostics command can replay a stored browse request and print the answer (INT-103).

- [DOC-005] *DIDL-Lite*, *control point*, *object identifier*, *container* and *filter* are explained at first use.

## 7. Security and privacy

- Every argument is client-supplied and untrusted (SEC-018): `ObjectID` is parsed as an identifier, never a path; `Filter` and `SortCriteria` are matched
  against known property names, never used to build a query string. The identifier scheme exposes no filesystem path: identifiers are opaque and stable
  (INT-062, PLT-008).

- The response contains only what the catalogue already exposes: titles, classes and media URLs. The item count in one response and the size of the generated
  document are bounded (SEC-061), so a client cannot force an unbounded allocation by asking for everything.

- `Filter` and `SortCriteria` are bounded in length and count before they are parsed (SEC-011, SEC-060); a browse never writes to the catalogue or to a media
  root (FUN-018, SEC-001).

## 8. Interoperability

- Normative: UPnP AV ContentDirectory:1 action table and the `Browse` output semantics, including the error-720 and `UpdateID` rules (standards §1.2, §8.3
  C1–C4).

- [OBSERVED] Every shipped media server implements exactly the four required actions, and none implements `GetFreeSpace`, `GetUpdateCounter`,
  `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal` or `GetShareIndexInBrowse` (standards §0.3, §8.3 C12).

- [OBSERVED] `GetSearchCapabilities` in the field returns `dc:title,dc:creator,dc:date,upnp:class,res@size` (standards §1.2). We advertise a subset of it
  until the catalogue can search in more; the subset is a subset on purpose, because the list must name only what the server really does (INT-035).

- [OBSERVED] Control points send `Filter` as `*` almost always, and `SortCriteria` as an empty string or as `+property` (standards §8.3 C6–C7).

- [SPEC] A `MediaServer:2` implementation must answer the actions of `MediaServer:1` (standards §1.1); our action set is the CD:1 required set plus `Search`,
  so a `MediaServer:1` client is served.

- [OPEN] The DLNA guideline table a DMS must satisfy (guidelines 7.3.55–7.3.80) was only available second hand (standards §9 Q3); it constrains browse
  behaviour and remains unverified.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Unknown `ObjectID` | UPnP fault `701 No Such Object` | INT-039 |
| `BrowseDirectChildren` on an item (not a container) | UPnP fault `701 No Such Object` | INT-039 |
| `TotalMatches` not computable, items returned | `TotalMatches = 0`, `NumberReturned > 0` | INT-046 |
| `TotalMatches` not computable, nothing returned | SOAP error `720` | INT-046 |
| Write action requested | UPnP fault `401 Invalid Action` | INT-040 |

No failure of this feature may take down the HTTP task: a bad browse request produces a fault, never a panic (UX-005, INT-025).

## 10. Acceptance criteria and traceability

All criteria below are *planned*. The criteria that the dispatcher can answer without a socket are
**covered** by the tests listed in §11; the criteria that need a running server or the real catalogue
are still **planned**.

**AC-01** — *Given* a client that calls `Browse` with the six CD:1 arguments, *when* the answer is inspected, *then* it contains `Result`, `NumberReturned`,
`TotalMatches` and `UpdateID`, each exactly once, in that order. Requirements: INT-030, INT-031 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_browse_answer_carries_the_four_arguments_in_order`.

**AC-02** — *Given* a container with 12 children, *when* a client calls `Browse` with `BrowseFlag = BrowseMetadata`, *then* the `Result` describes the container
itself and `NumberReturned` and `TotalMatches` are both `1`. Requirements: INT-033 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::a_metadata_browse_describes_the_container`.

**AC-03** — *Given* a container with 12 children, *when* a client calls `Browse` with `BrowseFlag = BrowseDirectChildren` and `StartingIndex = 5`, *then*
`TotalMatches` is `12`, `NumberReturned` is the number of items actually returned, both are non-negative integers and `NumberReturned` does not exceed
`TotalMatches`. Requirements: INT-033, INT-041 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::a_browse_with_an_offset_returns_the_rest`.

**AC-04** — *Given* a container with 12 children, *when* a client calls `Browse` with `RequestedCount = 5`, *then* at most 5 items are in the `Result`; and
*when* it calls with `RequestedCount = 0`, *then* the page contains every child the server's page limit allows and `TotalMatches` still reports the true total.
Requirements: INT-032, INT-041 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::a_browse_with_a_limit_returns_at_most_that_many`.

**AC-05** — *Given* a client that sends `Filter = *`, *when* the answer is inspected, *then* each item carries the full property set the server produces for it;
*given* a known property, *then* only that property is present; *given* an unknown property, *then* the answer is complete and no fault is returned.
Requirements: INT-034 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_filter_narrows_the_answer_without_failing`.

**AC-06** — *Given* the service is queried for its capabilities, *when* `GetSearchCapabilities` and `GetSortCapabilities` are called, *then* each answer is a
comma-separated list of property names and every property named is one the server really searches or sorts by. Requirements: INT-035, INT-049 · Verification:
protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory_search.rs::the_capabilities_name_the_properties_the_server_uses`.

**AC-07** — *Given* a catalogue with a known title, *when* a client calls `Search` with `dc:title contains "<fragment>"` for a property that
`GetSearchCapabilities` advertises, *then* the matching item is in the `Result` with a correct `NumberReturned` and `TotalMatches`. Requirements: INT-036 ·
Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory_search.rs::a_search_finds_the_matching_item`.

**AC-08** — *Given* the catalogue changes, *when* `GetSystemUpdateID` is called before and after, *then* the value is not smaller afterwards and it differs
across the change; *given* no change, *then* the value is unchanged; and *given* a browse of a container, *when* the `UpdateID` is read, *then* it equals that
container's update identifier, while a browse of `ObjectID = 0` returns the `SystemUpdateID`. Requirements: INT-037, INT-043 · Verification: protocol
conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory_search.rs::the_system_update_identifier_grows_with_the_catalogue`.

**AC-09** — *Given* a client that browses `ObjectID = 0` and `ObjectID = 1`, *when* the answers are inspected, *then* `0` is the root container and `1` is the
content directory root, and both are consistent with the hierarchy the server returns. Requirements: INT-038 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_root_and_the_content_directory_root_are_known`.

**AC-10** — *Given* an `ObjectID` that the server does not know, *when* a client calls `Browse`, *then* the response is the UPnP error `701 No Such Object` and
not an empty successful result. Requirements: INT-039 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::an_unknown_object_is_the_fault_701`.

**AC-11** — *Given* the served SCPD of the ContentDirectory service, *when* it is parsed, *then* it lists `Browse`, `GetSearchCapabilities`,
`GetSortCapabilities`, `GetSystemUpdateID` and `Search`, and none of the write actions; *given* a request for one of them, *then* the answer is the UPnP error
`401 Invalid Action`. Requirements: INT-040, INT-023 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_write_actions_are_absent_and_not_answered`.

**AC-12** — *Given* a browse answer, *when* the `Result` text is extracted and unescaped once, *then* it is a well-formed DIDL-Lite document that parses on its
own, with a `DIDL-Lite` root element in the namespace `urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/`; *given* the items, *then* no `writeStatus` attribute is
present. Requirements: INT-042, INT-071, INT-077 (owned by F-05) · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_result_is_a_didl_lite_document_that_parses_on_its_own`; the DIDL-Lite writer itself is still the placeholder of F-05.

**AC-13** — *Given* a browse answer on the wire, *when* the raw `Result` element is inspected, *then* it carries the DIDL-Lite document **escaped a second
time** — the text contains `&lt;DIDL-Lite` and not a literal `<DIDL-Lite` — and unescaping it twice yields the DIDL-Lite document. Requirements: INT-042 (the
escaping rule itself is INT-073, owned by F-05) · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::the_raw_result_element_carries_the_document_escaped`. The document is escaped once, so the text carries `&lt;DIDL-Lite` and *one* unescaping yields the DIDL-Lite document — see the open question in §12.

**AC-14** — *Given* a container whose child count cannot be computed, *when* the browse returns items, *then* `TotalMatches` is `0` and `NumberReturned` is not;
and *given* that container returns no items, *then* the answer is the SOAP error `720`. Requirements: INT-046 · Verification: protocol conformance test ·
Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::an_uncountable_container_answers_with_zero_or_720`.

**AC-15** — *Given* a browse with an empty `SortCriteria`, *then* the server's own order is returned; *given* `SortCriteria = "+dc:title"`, *then* the items are
ordered ascending by title; *given* `"-dc:title"`, *then* descending. Requirements: INT-047 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory_search.rs::the_sort_criteria_order_the_answer`.

**AC-16** — *Given* the served SCPD of the ContentDirectory service and the SOAP dispatcher, *when* both are inspected, *then* none of `GetFreeSpace`,
`GetUpdateCounter`, `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal` or `GetShareIndexInBrowse` appears in the action list and none of them is answered
by the dispatcher. Requirements: INT-048 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/content_directory.rs`
— **covered** by `crates/phairdlna-core/tests/content_directory.rs::no_non_standard_action_is_answered`.

## 11. Test strategy and dependencies

**Test locations**:

| Purpose | Path | State |
| --- | --- | --- |
| Action-level conformance of the ContentDirectory service | `crates/phairdlna-core/tests/content_directory.rs` | **exists** |
| The search, the capabilities and the order of an answer | `crates/phairdlna-core/tests/content_directory_search.rs` | **exists** |
| Page arithmetic unit tests | `crates/phairdlna-core/src/upnp/page.rs` (inline `#[cfg(test)]` module) | **exists** |
| Filter and sort unit tests | `crates/phairdlna-core/src/upnp/browse.rs` (inline `#[cfg(test)]` module) | **exists** |
| DIDL-Lite validation of the produced `Result` | `crates/phairdlna-core/tests/didl_validation.rs` (F-05) | planned — the DIDL-Lite parser of F-05 does not exist yet |

**Method.** The action handlers are plain functions with typed inputs and outputs, so they are exercised without a socket; a hand-written SOAP client (the one
F-02 and F-04 use) drives the dispatcher over loopback. The `Result` document is parsed by the DIDL-Lite parser of F-05 and validated against the vendored
schema (interoperability matrix PC-14), which makes AC-12 and AC-13 depend on F-05's parser.

**Dependencies.** F-02 (the SCPD and the SOAP dispatcher), F-04 (the fault codes and the control path), F-05 (the DIDL-Lite generator, the escaping and the
`res` URLs), and the catalogue module for the identifier scheme and the child counts.

## 12. Open questions and decisions

> **The capability lists are smaller than the design of §3, until the catalogue exists.**
> The design of INT-035 names `dc:creator`, `dc:date` and `res@size` as searchable and `dc:creator`, `dc:date` as sortable. The catalogue
> of F-08 and F-09 cannot search in them or sort by them yet — it searches in `dc:title` alone — and INT-035 asks for *only the capabilities the server actually implements*, so the lists are `dc:title` and
> `dc:title,upnp:class`. The lists grow with the catalogue module (F-08, F-09). *Closes when:* the catalogue can search and sort by the other properties,
> or the owner decides that the lists shall name the design of §3 regardless.

> **A contradiction between AC-12 and AC-13, found while the work package was implemented.**
> AC-12 says that the `Result` text, *unescaped once*, is a well-formed DIDL-Lite document; AC-13 says
> that the text carries the document *escaped a second time* and that *unescaping it twice* yields the
> DIDL-Lite document. Both cannot hold at once: a text that is escaped twice carries `&amp;lt;DIDL-Lite`
> and is not a document after one unescaping. **The implementation escapes it once** — the text on the wire
> carries `&lt;DIDL-Lite`, and one unescaping yields the DIDL-Lite document. That is what the shipped
> servers do (the `rclone` capture in standards §3.3 shows `&lt;DIDL-Lite` on the wire) and it is what
> AC-12 demands. *Closes when:* the owner confirms the wording of AC-13, or corrects AC-12.

> **The filter and the two mandatory properties, changed by F-05 (D-07).**
> The placeholder writer of WP2 applied the filter to the mandatory properties as well, so a browse with
> `Filter=dc:title` returned an item without `upnp:class`. F-05 owns the document and `INT-066` asks for
> `dc:title` and `upnp:class` on every item and container; the schema makes them mandatory, and a document
> without them is not a DIDL-Lite document. **The filter now narrows the other properties only**, and the
> two mandatory ones are always written. `INT-034` is read as "the elements the client asked for are
> returned", which stays true. The list of properties a filter may name grew from `dc:title,upnp:class` to
> the ten properties the document can carry, because the document can carry them now.

1. **[OPEN] The `UpdateID` semantics: CD:1 or CD:4?** CD:1 returns the *container's* update identifier for a container browse and the `SystemUpdateID` only for
   `ObjectID = 0`; CD:4 always returns the `SystemUpdateID` (standards §1.2, §7.1, §9 Q11). INT-043 states the CD:1 rule and we advertise `ContentDirectory:1`.
   *Closes when:* a device round shows that a client is confused by a container-specific update identifier, or the owner moves to `ContentDirectory:4`. 2.
   **[OPEN] `SearchCriteria` as an argument of `Browse`.** INT-030 lists `SearchCriteria` among the `Browse` arguments, but the CD:1 action table does not have
   it — it belongs to `Search` and to CD:2 (standards §1.2). This document accepts the argument and ignores it, so that a client which sends it is not answered
   with `402 Invalid Args`. *Closes when:* the owner amends INT-030 or confirms the accept-and-ignore reading. 3. **[OPEN] `writeStatus` — attribute or
   property, and allowed values.** The CD:4 Appendix-B row defining it was not opened, so it appears in the metadata model as an `upnp:` property rather than an
   attribute (standards §4.2, §9 Q5). The baseline resolves this by prohibition: INT-077 forbids emitting it, and AC-19 asserts that it is absent. *Closes
   when:* the row is available and the owner decides whether the prohibition can be lifted. 4. **[OPEN] The `+`/`-` prefix in `SortCapabilities`, and the
   `Search` expression grammar.** The research proposes prefixed `SortCapabilities` entries (`+dc:title,+dc:date`), while INT-049 says comma-separated lists of
   *property names* (standards §1.2); we advertise plain names and accept the prefix in the `SortCriteria` argument. INT-036 names the capability strings but
   not the syntax, so we implement `dc:title contains "text"` and reject anything else with `402`. *Closes when:* the owner confirms the reading, fixes the
   grammar, or amends INT-049. 5. **[OPEN] Free space (FUN-023) against INT-048.** FUN-023 (COULD) says the server "shall expose the free space of a root
   through the ContentDirectory service when the file system provides it", but `GetFreeSpace` is not a ContentDirectory action (standards §8.3 C12) and INT-048
   forbids non-standard actions in the standard service. *Proposed decision:* expose it, if at all, through a vendor action outside the standard namespace
   (standards §8.3 C13), or not at all. *Closes when:* the owner decides. 6. **[OPEN] The DLNA browse guideline table.** Guidelines 7.3.55–7.3.80 define what a
   DMS must do for Browse and Search, but only a second-hand quotation was available (standards §9 Q3). *Closes when:* the guideline text is obtained.
