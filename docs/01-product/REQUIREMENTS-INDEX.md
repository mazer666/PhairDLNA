# Requirements index — PhairDLNA

Status: **Approved (v1.1 accessibility update, 2026-11-18; v1.0 baseline preserved)** · Owner role: Product Owner

This is the **cross-cutting view** of every requirement. The normative text of each
requirement is defined exactly once — in `TECHNICAL-REQUIREMENTS.md` (technical
domains) or `PRODUCT-REQUIREMENTS.md` (product commitments). This file adds the
short title, priority, owning feature, milestone, verification method and status,
and it is **generated** from those definitions by `scripts/generate_requirement_views.py`.
Do not edit it by hand; edit the definition and re-run the script.

**Priority vocabulary.** `MUST` = required for correctness, safety or the promise of
the product; a milestone with an open `MUST` cannot be accepted. `SHOULD` = strongly
desirable, tracked, may delay a milestone. `COULD` = optional enhancement, outside
the current baseline. Defined in
[../00-governance/REQUIREMENTS-METHOD.md](../00-governance/REQUIREMENTS-METHOD.md) section 3.


**Status vocabulary.** `Baseline` = part of the approved requirement baseline; no
implementation status is claimed. `Scheduled` = assigned to a milestone.
`Implemented` = code exists.
`Verified` = evidence recorded.


Requirement method, ID rules and the traceability chain are defined in
[../00-governance/REQUIREMENTS-METHOD.md](../00-governance/REQUIREMENTS-METHOD.md).


---


## PRD — Product commitments

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `PRD-001` | Be public and licensed AGPL-3.0-or-later, with the complete unmodified licence text shipped in… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-002` | Be free of charge, with no advertising, no telemetry, no analytics and no vendor account. | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-003` | Normal operation shall not require an internet connection; no component shall contact a… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-004` | Never expose the host to the public internet automatically; no router configuration (port… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-005` | Be implemented in Rust and shall be free of platform-specific code; platform integration shall… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-006` | Provide the complete product function; no capability shall require the graphical application to… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-007` | Media files shall be opened read-only by default; the server shall never modify | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-008` | Run on macOS (Apple Silicon and Intel) as the first supported platform | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-009` | Use sensible, safe defaults: a new installation shall serve the LAN without configuration,… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-010` | Remain usable end-to-end for a person who has never used a DLNA server | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-011` | Documentation shall be part of the product: every feature shall have a specification | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-012` | Every claim about performance, compatibility or security shall be backed by a recorded… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-013` | Not contain advertising, "recommended" content, telemetry endpoints or promotional network… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-014` | Expose its own status, diagnostics and configuration locally to the person who installed it,… | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |
| `PRD-015` | Direct play shall be the default and preferred playback path; transcoding shall be optional | MUST | `cross-cutting` | M0 | Manual acceptance protocol | Baseline |

## FUN — Functional behaviour

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `FUN-001` | Be able to configure one or more media roots; each root shall have a name shown to clients and… | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-002` | Serve the union of all configured roots as one catalogue. | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-003` | Not follow a symbolic link that points outside its root unless the user explicitly allows it. | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-004` | Ignore hidden files and macOS resource-fork sidecar files (`._*`) when building the catalogue. | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-005` | Index audio, video, image and playlist files by extension and by sniffed content type. | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-006` | Re-scan a root incrementally and detect added | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-007` | Detect a root that is unmounted, inaccessible or replaced by a different volume, and shall… | MUST | `F-08` | M1 | Automated test | Verified |
| `FUN-008` | Keep the catalogue usable while a rescan runs; a browse during a scan shall return the… | MUST | `F-09` | M2 | Automated test | Verified |
| `FUN-009` | Store its catalogue in a local SQLite database that can be deleted and rebuilt without data… | MUST | `F-09` | M2 | Automated test | Verified |
| `FUN-010` | Recover from a corrupted index database by rebuilding it automatically | MUST | `F-09` | M2 | Automated test | Verified |
| `FUN-011` | Assign stable object identifiers that do not change when the catalogue is rebuilt. | MUST | `F-09` | M1 | Automated test | Verified |
| `FUN-012` | Extract duration, dimensions, bitrate and codec information for media files where it can do so… | SHOULD | `F-09` | M1 | Automated test | Baseline |
| `FUN-013` | Extract embedded artwork from audio files and use a sidecar image when one exists. | SHOULD | `F-10` | M2 | Automated test | Implemented |
| `FUN-014` | Generate thumbnails for image files and video posters on demand and cache them in a disposable… | SHOULD | `F-10` | M2 | Automated test | Implemented |
| `FUN-015` | Support playlists in M3U, M3U8 and PLS format, resolving entries relative to the playlist file. | SHOULD | `F-11` | M2 | Automated test | Verified |
| `FUN-016` | Expose a subtitle sidecar file (`.srt`, `.vtt`, `.sub`, `.ass`) as an additional `res` element… | SHOULD | `F-11` | M2 | Automated test | Verified |
| `FUN-017` | Support media stored on removable volumes and network shares | MUST | `F-08` | M2 | Automated test | Implemented |
| `FUN-018` | Never write to, rename or delete a file inside a media root, with the single exception of its… | MUST | `F-08` | M2 | Automated test | Implemented |
| `FUN-019` | Keep the raw path exactly as the operating system reported it for every file operation | MUST | `F-08` | M2 | Automated test | Verified |
| `FUN-020` | Compute a separate identity key for indexing and de-duplication by applying Unicode NFC and a… | MUST | `F-11` | M2 | Automated test | Verified |
| `FUN-021` | Provide a way to exclude subdirectories of a root. | SHOULD | `F-11` | M2 | Protocol conformance + device test | Implemented |
| `FUN-022` | Support multiple roots that point to the same underlying volume without producing duplicate… | SHOULD | `F-11` | M2 | Protocol conformance + device test | Implemented |
| `FUN-023` | Expose the free space of a root through the ContentDirectory service when the file system… | COULD | `F-08` | M2 | Automated test | Baseline |
| `FUN-024` | Allow the user to configure a scan schedule (interval | SHOULD | `F-08` | M2 | Automated test | Baseline |
| `FUN-025` | Make a newly added file visible in the catalogue within 30 seconds on a file system that… | SHOULD | `F-09` | M1 | Automated test | Verified |
| `FUN-026` | Offer a subtitle sidecar as an additional resource with the MIME type that matches its format | MUST | `F-11` | M2 | Automated test | Verified |
| `FUN-027` | Deliver a subtitle file in a character set the client can read | MUST | `F-11` | M2 | Automated test | Implemented |
| `FUN-028` | Serve a file larger than 4 GiB with correct byte offsets across the 4 GiB boundary and at the… | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-029` | Expose each additional audio or subtitle track of a media item as its own resource | MUST | `F-11` | M2 | Automated test | Implemented |
| `FUN-040` | Serve media by byte range from the original file without transcoding whenever the client… | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-041` | Report the container and codec of a file in a way a client can use to decide whether it can… | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-042` | Support multiple simultaneous direct-play streams | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-043` | Bound the number of concurrent transfers and shall refuse further transfers with a clear error… | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-044` | Apply a per-stream bandwidth limit when configured | SHOULD | `F-06` | M1 | Automated test | Baseline |
| `FUN-045` | Report the position and duration of the active transfer through the AVTransport service when… | SHOULD | `F-06` | M1 | Automated test | Baseline |
| `FUN-046` | Cancel a transfer cleanly when the client closes the connection. | MUST | `F-06` | M1 | Automated test | Verified |
| `FUN-047` | Implement optional transcoding only when the user has explicitly enabled it and selected a… | COULD | `F-17` | M5 | Automated test | Baseline |
| `FUN-048` | The transcoder, if enabled, shall run in a separate process with an explicit resource budget, a… | COULD | `F-17` | M5 | Automated test | Baseline |
| `FUN-049` | Never silently fall back to transcoding; a file that cannot be played directly shall be… | MUST | `F-17` | M5 | Automated test | Baseline |
| `FUN-050` | Keep the buffer size small and bounded so that a slow client cannot consume unbounded memory. | MUST | `F-06` | M1 | Automated test | Verified |

## INT — Interoperability (DLNA / UPnP)

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `INT-001` | Answer `M-SEARCH` requests on UDP 1900 within at most 1 second and shall answer with `HTTP/1.1… | MUST | `F-01` | M1 | Protocol conformance + device test | Verified |
| `INT-002` | Match `ST` against `ssdp:all`, `upnp:rootdevice`, the advertised MediaServer device type and… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-003` | Send its SSDP response unicast to the address and port of the requester. | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-004` | Send periodic `NOTIFY` advertisements with `NT: upnp:rootdevice` and `NTS: ssdp:alive` | MUST | `F-01` | M1 | Protocol conformance + device test | Verified |
| `INT-005` | The `USN` header shall have the form `uuid:<UDN>::<target>` where `<target>` is… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-006` | The `LOCATION` header shall contain an absolute HTTP URL of the device description document… | MUST | `F-01` | M1 | Protocol conformance + device test | Verified |
| `INT-007` | Bind SSDP to the IPv4 multicast group `239.255.255.250:1900` and shall support the IPv6… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-008` | Honour the `MX` header of an `M-SEARCH` by choosing a uniformly random delay inside `[0 | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-009` | Select the sending interface by the route of the incoming request and shall never answer a… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-010` | Ignore malformed `M-SEARCH` datagrams without logging them at error level and without affecting… | MUST | `F-01` | M1 | Protocol conformance + device test | Verified |
| `INT-011` | Rate-limit discovery responses per source address so that a single host cannot force an… | SHOULD | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-012` | Expose the configured `friendlyName`, `manufacturer`, `modelName` and `UDN` in the SSDP… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-013` | Answer `M-SEARCH` only when the `MAN` header contains the quoted token `ssdp:discover` | SHOULD | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-014` | Use a stable `UDN` across restarts for the same installation unless the user resets the… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-015` | Provide a CLI switch to print the exact SSDP headers it would send | SHOULD | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-016` | Set the multicast time-to-live of its advertisements to 2 | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-017` | Answer `M-SEARCH` for the device type `urn:schemas-upnp-org:device:MediaServer:1` in addition… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-018` | Not rely on mDNS for discovery; SSDP is the DLNA discovery mechanism and no mDNS advertisement… | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-019` | Expose the version of the UPnP architecture it implements in the `SERVER` header | MUST | `F-01` | M1 | Protocol conformance test | Verified |
| `INT-020` | Serve an XML device description at a stable URL path that contains a root device with… | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-021` | List the `ContentDirectory:1` and `ConnectionManager:1` services | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-022` | All URLs inside the device description shall be absolute HTTP URLs or root-relative paths that… | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-023` | Serve a valid SCPD document for every advertised service | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-024` | The XML served by the server shall be well-formed | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-025` | Answer an unknown SOAP action with an HTTP fault using the UPnP error codes `401 Invalid… | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-026` | Respond to `GET` on the device description and on SCPD documents with `Content-Type: text/xml;… | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-027` | Not include a client-controllable value in the XML output without XML escaping. | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-028` | Advertise `ContentDirectory:1` in addition to any newer ContentDirectory version | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-029` | Reference the `ConnectionManager` service even when no transcoding is available | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-030` | Implement the `Browse` action with the arguments `ObjectID` | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-031` | The `Browse` response shall contain `Result`, `NumberReturned`, `TotalMatches` and `UpdateID`… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-032` | Honour `StartingIndex` and `RequestedCount` and shall never return more items than… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-033` | Honour `BrowseFlag` such that `BrowseMetadata` returns only the container metadata with… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-034` | Apply the `Filter` argument, shall support the wildcard `*` that clients send constantly, and… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-035` | Implement `GetSearchCapabilities` and `GetSortCapabilities` and shall return only capabilities… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-036` | Implement `Search` for the capability strings it advertises in `GetSearchCapabilities`. | SHOULD | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-037` | Implement `GetSystemUpdateID` and shall keep the returned value monotonically non-decreasing | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-038` | Return the object ID `0` for the root container and the standard ID `1` for the content… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-039` | Return an UPnP error `701 No Such Object` for an unknown `ObjectID` instead of an empty… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-040` | Not implement write operations (`CreateObject` | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-041` | Return `NumberReturned` and `TotalMatches` as non-negative integers and shall not exceed… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-042` | Return a `Result` document that is valid DIDL-Lite XML and parses independently of the… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-043` | Return the container's `UpdateID` for a `Browse` of a container and the `SystemUpdateID` for a… | SHOULD | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-044` | Use the service identifiers `urn:upnp-org:serviceId:ContentDirectory` and… | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-045` | Contain exactly one `device` element and shall not contain a `URLBase` element. | MUST | `F-02` | M1 | Protocol conformance test | Verified |
| `INT-046` | When `TotalMatches` cannot be computed, the server shall return `TotalMatches` of `0` with a… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-047` | Accept `SortCriteria` in the empty form and in the `+property` and `-property` forms | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-048` | Not advertise or implement any non-standard ContentDirectory action; a client-specific… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-049` | Expose `SearchCapabilities` and `SortCapabilities` as comma-separated lists of the property… | MUST | `F-03` | M1 | Protocol conformance test | Verified |
| `INT-050` | Implement `GetProtocolInfo` and return at least the `http-get` transfer protocol with the MIME… | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-051` | The `GetProtocolInfo` result shall not advertise a container or MIME type that the server… | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-052` | Implement `GetCurrentConnectionIDs` and return an empty list when no transfer is active. | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-053` | Implement `PrepareForConnection`, `ConnectionComplete` and `GetCurrentConnectionInfo` in a way… | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-054` | Return the same `Source` value in `GetCurrentConnectionInfo` as the `res` URI it serves for the… | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-055` | Advertise the `http-get` protocol with the `*` wildcard for the additional header field only… | SHOULD | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-056` | Not implement the `http-subscribe` transfer protocol in the baseline. | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-057` | Reject every GENA `SUBSCRIBE` request unless the user has explicitly enabled eventing | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-058` | Never implement the UPnP Internet Gateway Device actions (`AddPortMapping` | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-059` | Not implement a network self-update mechanism; updates shall be installed by the user through… | MUST | `F-04` | M1 | Protocol conformance test | Verified |
| `INT-060` | Every `<item>` and `<container>` shall carry the attributes `id` | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-061` | Use the UPnP object classes `object.item.audioItem.musicTrack` | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-062` | The `id` attribute shall be a stable, opaque identifier for the object that does not change… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-063` | The `res` element shall carry `protocolInfo`, `duration` (for timed media, formatted… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-064` | The `protocolInfo` string shall have the form `http-get:*:<mime>:<additional>` and shall name a… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-065` | XML-escape all title, artist, album, genre and description values: `&`, `<` and `>` in element… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-066` | Include `dc:title` and `upnp:class` for every item and container. | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-067` | Include `upnp:albumArtURI` for audio items for which an artwork file exists | SHOULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-068` | Include `dc:date` when the file system provides a modification date | SHOULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-069` | Include `upnp:artist`, `upnp:album` and `upnp:genre` for audio items when the tags provide them. | SHOULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-070` | Include `upnp:originalTrackNumber` when the audio tags provide it. | COULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-071` | Produce a `DIDL-Lite` document whose root element is `DIDL-Lite` in the namespace… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-072` | Truncate a single metadata value to a documented maximum length so that no element or attribute… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-073` | Escape the complete DIDL-Lite document a second time when embedding it into the `Result`… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-074` | Report `res@bitrate` in bytes per second, as the metadata schema defines it. | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-075` | Emit `res@sampleFrequency`, `res@nrAudioChannels` and `res@bitsPerSample` when the media… | SHOULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-076` | Use `dc:creator` in addition to `upnp:artist` for the artist of an audio item. | SHOULD | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-077` | Not emit the `writeStatus` attribute, because its definition could not be verified against a… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-078` | The DLNA signalling in the fourth `protocolInfo` field shall be additive: the server shall be… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-079` | Emit `DLNA.ORG_PN` for a media resource only when the content actually conforms to that DLNA… | MUST | `F-05` | M1 | Protocol conformance test | Verified |
| `INT-080` | Answer `GET` and `HEAD` requests for media objects with `HTTP/1.1 200 OK` | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-081` | Support a single byte range and answer with `HTTP/1.1 206 Partial Content` | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-082` | Answer a range request whose start is beyond the end of the file with `HTTP/1.1 416 Range Not… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-083` | Answer a syntactically invalid or unsatisfiable `Range` header with `HTTP/1.1 416` rather than… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-084` | Advertise `Accept-Ranges: bytes` on media responses. | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-085` | Not implement `multipart/byteranges`; for a multi-range request it shall serve the first… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-086` | Answer a `HEAD` request with the same headers as the corresponding `GET` | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-087` | Use the extension of the file, refined by the sniffed content, to determine the `Content-Type`;… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-088` | Not use chunked transfer encoding for media objects and shall always send a `Content-Length`. | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-089` | Close a connection that a client abandoned mid-transfer without leaking the transfer slot. | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-090` | Serve the media URI that it advertised in the DIDL-Lite `res` element | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-091` | Keep the file open for the duration of the transfer when possible | SHOULD | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-092` | Ignore a `Range` header on a method other than `GET` | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-093` | Honour `If-Range`: when its condition does not hold | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-094` | Answer `404` for an unknown media identifier, `405` for an unsupported method and `400` for a… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-095` | Support HTTP/1.1 persistent connections, and every response shall carry either a… | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-096` | Emit the DLNA response headers `transferMode.dlna.org` and `contentFeatures.dlna.org` on media… | SHOULD | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-097` | Answer a request carrying `getcontentFeatures.dlna.org: 1` with the `contentFeatures.dlna.org`… | SHOULD | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-098` | Serve the media URL only when it resolves inside one of the configured roots. | MUST | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-099` | Emit `realTimeInfo.dlna.org` with `DLNA.ORG_TLAG=*` on media responses. | COULD | `F-06` | M1 | Protocol conformance test | Verified |
| `INT-100` | Maintain a declarative profile of known client quirks | MUST | `F-07` | M4 | Protocol conformance + device test | Verified |
| `INT-101` | A quirk workaround shall never violate a normative requirement of the UPnP or DLNA… | MUST | `F-07` | M4 | Protocol conformance test | Verified |
| `INT-102` | Record every device-test result — device model | MUST | `F-07` | M4 | Protocol conformance + device test | Verified |
| `INT-103` | Provide a diagnostics command that replays a stored `M-SEARCH` request and prints the response… | SHOULD | `F-07` | M4 | Protocol conformance + device test | Verified |
| `INT-104` | Allow the user to disable a quirk workaround globally without editing source code. | SHOULD | `F-07` | M4 | Protocol conformance test | Verified |
| `INT-105` | Give each additional resource of an item a `protocolInfo` that names its own MIME type | MUST | `F-11` | M2 | Protocol conformance test | Verified |

## SEC — Security and privacy

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `SEC-001` | Resolve every media path canonically and verify that it is inside an allowed root before… | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-002` | Reject a path containing `..` segments, encoded separators (`%2f`, `%5c`), or a null byte,… | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-003` | Resolve a symbolic link and re-check containment after resolution | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-004` | Open files in a way that does not follow a final symbolic link when the file is outside the… | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-005` | Never construct a shell command from user, client or file metadata input. | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-006` | Not follow a symbolic link that points outside a root | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-007` | Treat all file names and metadata as untrusted input | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-008` | Open media files read-only, without requesting write permission on the containing directory. | MUST | `F-08` | M1 | Negative security test | Verified |
| `SEC-010` | Not process a DTD, shall not resolve external entities and shall bound the nesting depth and… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-011` | Bound the size of every accepted HTTP request line | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-012` | Bound the number of concurrent connections and the number of concurrent transfers. | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-013` | Answer an unauthenticated or malformed request with a generic error that reveals no internal… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-014` | Not follow a redirect to a non-local address, and shall reject a callback URL that is not a… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-015` | Validate and normalise every URL path segment it accepts | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-016` | Enforce a read and write timeout on every connection. | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-017` | Never bind a non-loopback port other than the media port without an explicit configuration… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-018` | Treat an SSDP or SOAP peer as untrusted and shall perform no action on its behalf other than… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-060` | Every parser in the server shall have an explicit input size bound and a documented failure… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-061` | Bound the number of entries in a single DIDL-Lite response and the size of the generated XML… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-062` | Run metadata extraction with a time limit per file and shall abandon a file that exceeds it. | MUST | `F-16` | M1 | Negative security test | Baseline |
| `SEC-063` | Bound the number of open file descriptors used by transfers and shall release them on… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-064` | Bound the size of its on-disk cache and shall evict entries when the limit is exceeded. | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-065` | Never allocate memory proportional to a client-supplied length field without validating it… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-066` | Parse media metadata and generate thumbnails in a separate | MUST | `F-16` | M1 | Negative security test | Baseline |
| `SEC-067` | Bound the declared dimensions of an image before allocating any buffer for it | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-068` | Treat playlist entries as untrusted paths, shall resolve and contain them, and shall refuse… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-069` | Not resolve or follow a macOS Finder alias as if it were a symbolic link; aliases shall be… | MUST | `F-16` | M1 | Negative security test | Baseline |
| `SEC-070` | Offer the corresponding source of the running build | MUST | `F-16` | M1 | Negative security test | Baseline |
| `SEC-071` | Accept only a `Host` header that matches a configured local host name | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-072` | Require a custom, non-standard request header on every state-changing administration request,… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-090` | Contain no telemetry, no analytics and no crash reporting to any third party. | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-091` | Make no outbound network connection other than the SSDP multicast announcements and responses | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-092` | Not attempt to configure the user's router, and shall not speak the UPnP Internet Gateway… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-093` | Store no personal data beyond local configuration | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-094` | Logs and diagnostics shall be redactable, and the redaction shall apply to file system paths,… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-095` | Be a local file with user-only permissions, and the server shall not create it inside a shared… | MUST | `F-16` | M1 | Negative security test | Verified |
| `SEC-096` | Ship no third-party service, no advertising identifier and no device fingerprint. | MUST | `F-16` | M1 | Negative security test | Verified |

## PERF — Performance and resources

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `PERF-001` | Keep its resident memory below 50 MB while idle with 10 000 indexed objects. | SHOULD | `F-18` | M1 | Benchmark | Baseline |
| `PERF-002` | Reach the "ready to answer discovery" state within 2 seconds of start-up on the reference… | SHOULD | `F-18` | M1 | Benchmark | Baseline |
| `PERF-003` | An incremental rescan of a tree with 10 000 unchanged files shall complete without re-reading… | MUST | `F-09` | M1 | Benchmark | Baseline |
| `PERF-004` | Index at least 200 media files per second on the reference hardware when metadata extraction is… | SHOULD | `F-09` | M1 | Benchmark | Baseline |
| `PERF-005` | A `Browse` request over a container with 10 000 children shall return its first page within 100… | MUST | `F-09` | M1 | Benchmark | Baseline |
| `PERF-006` | Sustain 10 simultaneous direct-play streams without unbounded memory growth. | SHOULD | `F-18` | M1 | Benchmark | Baseline |
| `PERF-007` | Memory growth per active stream shall be bounded and documented. | MUST | `F-18` | M1 | Benchmark | Baseline |
| `PERF-008` | Not perform a full directory scan on a browse request. | MUST | `F-09` | M1 | Benchmark | Baseline |
| `PERF-009` | Support 100 000 objects with an index scan that stays within a documented budget. | SHOULD | `F-09` | M1 | Benchmark | Baseline |
| `PERF-010` | The reference hardware for all performance measurements shall be documented | MUST | `F-18` | M1 | Benchmark | Baseline |
| `PERF-011` | Every performance claim shall report the method | MUST | `F-18` | M1 | Benchmark | Baseline |
| `PERF-012` | Degrade gracefully when a catalogue is larger than the tested envelope: it shall remain… | MUST | `F-09` | M1 | Benchmark | Baseline |
| `PERF-013` | The CPU usage of an idle server shall be negligible (below 1 % averaged over one minute on the… | SHOULD | `F-18` | M1 | Benchmark | Baseline |
| `PERF-014` | Survive a scan of a root that contains a broken or slow network share without blocking browse… | MUST | `F-18` | M1 | Benchmark | Baseline |
| `PERF-015` | A client shall discover the server within 10 seconds at the 95th percentile over 30 trials on a… | SHOULD | `F-18` | M1 | Benchmark | Baseline |

## UX — Usability and accessibility

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `UX-001` | Be fully operable without any graphical interface. | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-002` | A first run shall require no more than one action (choosing a folder) before the server is… | SHOULD | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-003` | The first-run output shall name the device name | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-004` | Every error message shall state what happened | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-005` | Never abort with a panic message that lacks context; unexpected failures shall produce a… | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-006` | Provide `--help` for every subcommand. | MUST | `F-15` | M2 | Manual usability protocol | Implemented |
| `UX-007` | Be accessible with the keyboard alone and shall expose the status of the server as text | MUST | `F-15` | M2 | Manual usability protocol | Implemented |
| `UX-008` | Respect the system accessibility settings for contrast and motion and let users enlarge app… | SHOULD | `F-15` | M2 | Manual usability protocol | Scheduled |
| `UX-009` | Show an accessible status item for assistive technologies. | MUST | `F-15` | M2 | Manual usability protocol | Implemented |
| `UX-010` | Use plain language and avoid protocol jargon in the user-facing text. | MUST | `F-15` | M2 | Manual usability protocol | Implemented |
| `UX-011` | Diagnostics shall be copyable in one action and shall not contain personal data unless the user… | SHOULD | `F-14` | M2 | Manual usability protocol | Implemented |
| `UX-012` | Tell the user which client is currently connected and what it is streaming. | SHOULD | `F-14` | M2 | Manual usability protocol | Implemented |
| `UX-013` | Long-running operations shall report progress and allow cancellation. | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-014` | Present the server state, the scan progress, the shared folders and the last observed client… | SHOULD | `F-15` | M2 | Manual usability protocol | Implemented |
| `UX-015` | Provide a simple presentation mode and an advanced presentation mode over one configuration… | SHOULD | `F-15` | M2 | Manual usability protocol | Verified |
| `UX-016` | A safety-critical warning raised in the advanced mode shall remain visible in the simple mode. | MUST | `F-15` | M2 | Manual usability protocol | Verified |
| `UX-017` | Be operable with the keyboard alone, shall expose accessible labels and a deterministic focus… | MUST | `F-15` | M2 | Manual usability protocol | Scheduled |
| `UX-018` | Expose the status of the server as text for assistive technologies | MUST | `F-15` | M2 | Manual usability protocol | Verified |
| `UX-019` | Be able to start and stop the server, pause and resume indexing, and remove a shared folder… | MUST | `F-15` | M2 | Manual usability protocol | Baseline |
| `UX-020` | All user-facing strings shall be prepared for translation | SHOULD | `F-15` | M2 | Manual usability protocol | Verified |

## OPS — Operations and administration

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `OPS-001` | Provide a command that runs the daemon in the foreground with structured log output on standard… | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-002` | Provide a command that reports the current status: version | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-003` | Provide a command that scans a root on demand and reports progress and errors. | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-004` | Provide a command that lists the known network interfaces and which ones would be used for… | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-005` | Provide a command that prints the device description and the SSDP headers it would advertise | SHOULD | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-006` | Write logs with a timestamp, a level, a component and a structured message. | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-007` | Log output shall be redactable: file system paths | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-008` | Shut down gracefully on `SIGINT` and `SIGTERM` | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-009` | Support running as a background service on macOS (launchd) | SHOULD | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-010` | Support a configuration file that sets the service identity | SHOULD | `F-13` | M1 | Automated CLI test | Baseline |
| `OPS-011` | Provide `--help` and `--version` output that names the version | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-012` | Expose its version in the `SERVER` header and in the device description. | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-013` | Write a structured diagnostic report (anonymised) that a user can attach to a bug report. | SHOULD | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-014` | Return a distinct exit code for configuration errors | SHOULD | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-015` | Never require root privileges to serve media from the user's own directories. | MUST | `F-13` | M1 | Automated CLI test | Verified |
| `OPS-016` | Expose an unauthenticated route that serves the corresponding source of the exact running build | MUST | `F-14` | M2 | Automated CLI test | Baseline |
| `OPS-017` | Report the exact source revision it was built from in its status output and in the source offer. | MUST | `F-14` | M2 | Automated CLI test | Implemented |
| `OPS-020` | Bind to the loopback interface only by default and shall refuse to start when configured… | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-021` | Require a per-installation random token for every state-changing request. | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-022` | Validate the `Host` header against the loopback names to resist DNS-rebinding attacks. | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-023` | State-changing administration requests shall require a CSRF token bound to the session. | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-024` | Not expose media file contents, only metadata and control actions. | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-025` | Be optional; the server shall run completely without it. | MUST | `F-14` | M2 | Automated CLI test | Verified |
| `OPS-026` | Expose a clear "what is this, who can see it, how to turn it off" explanation in the first-run… | MUST | `F-14` | M2 | Automated CLI test | Implemented |
| `OPS-027` | Apply the same rate limiting and request-size limits as the media surface. | MUST | `F-14` | M2 | Automated CLI test | Implemented |

## CFG — Configuration

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `CFG-001` | Read its configuration from a single, versioned configuration file whose path is given by an… | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-002` | Be typed and validated at start-up; an invalid value shall abort start-up with a message naming… | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-003` | Carry a `schema_version`, and the server shall migrate an older known schema version… | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-004` | Provide documented defaults for every setting such that an empty configuration file yields a… | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-005` | Refuse to start when the configuration would expose an administration interface to a… | MUST | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-006` | Support selecting the network interfaces used for discovery and for media delivery. | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-007` | Support a per-root setting for symbolic link following | MUST | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-008` | Support resource limits: maximum concurrent streams | SHOULD | `F-12` | M2 | Automated configuration test | Implemented |
| `CFG-009` | Support an optional access-control list of client addresses allowed to browse or to stream. | SHOULD | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-010` | Support a "trusted client" list whose quirks are handled by a specific device profile. | SHOULD | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-011` | Provide a command that validates a configuration file without starting the server. | MUST | `F-12` | M2 | Automated configuration test | Baseline |
| `CFG-012` | Provide a command that exports the effective configuration and one that imports it | SHOULD | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-013` | Unknown configuration keys shall be reported as errors rather than ignored | MUST | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-014` | Secrets (such as an administration token) shall be readable from a file with owner-only… | MUST | `F-12` | M2 | Automated configuration test | Verified |
| `CFG-015` | Never contain a value that is expanded from an environment variable without an explicit opt-in | SHOULD | `F-12` | M2 | Automated configuration test | Baseline |

## PLT — Platform and portability

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `PLT-001` | Not depend on any OS-specific crate and shall compile on macOS | MUST | `F-22` | M0 | Build matrix | Verified |
| `PLT-002` | Provide a build matrix covering macOS (arm64, x86_64), Linux (x86_64) and Windows (x86_64). | MUST | `F-22` | M0 | Build matrix | Verified |
| `PLT-003` | All OS-specific behaviour shall be reachable through a documented trait with a default… | MUST | `F-22` | M0 | Build matrix | Verified |
| `PLT-004` | Support macOS Apple Silicon natively without a Rosetta translation requirement. | MUST | `F-15` | M0 | Build matrix | Verified |
| `PLT-005` | Detect the case sensitivity, the Unicode normalisation behaviour and the volume identity of… | MUST | `F-22` | M0 | Build matrix | Baseline |
| `PLT-006` | Run correctly when the user's home directory or a media root contains spaces | MUST | `F-22` | M0 | Build matrix | Baseline |
| `PLT-007` | Windows path handling shall support long paths and shall reject path traversal attempts using… | MUST | `F-22` | M0 | Build matrix | Verified |
| `PLT-008` | Not expose absolute file system paths to clients. | MUST | `F-22` | M0 | Build matrix | Baseline |
| `PLT-009` | Not require a C toolchain beyond what the platform already provides | SHOULD | `F-22` | M0 | Build matrix | Baseline |
| `PLT-010` | Behave identically on all platforms with respect to protocol behaviour | MUST | `F-22` | M0 | Build matrix | Verified |
| `PLT-011` | Treat the mount, unmount and remount of a volume as a first-class event and shall re-key the… | MUST | `F-22` | M2 | Build matrix | Implemented |
| `PLT-012` | Treat a permission error on a subtree as a quarantined subtree that needs consent | MUST | `F-22` | M3 | Build matrix | Baseline |
| `PLT-013` | On macOS the always-on server shall run as a system-context launchd daemon and the user… | MUST | `F-22` | M3 | Build matrix | Not implemented |
| `PLT-014` | Be built as a universal binary for Apple Silicon and Intel. | MUST | `F-22` | M3 | Build matrix | Verified |
| `PLT-015` | Declare long-path support in the executable manifest and shall document that the system-wide… | MUST | `F-22` | M3 | Build matrix | Verified |
| `PLT-016` | Provide native packages and shall not use a filesystem-sandboxed package format as its primary… | MUST | `F-22` | M3 | Build matrix | Verified |
| `PLT-017` | Obtain the user's consent for folders that the operating system protects (macOS privacy prompts | MUST | `F-22` | M3 | Build matrix | Baseline |
| `PLT-018` | Not require elevated privileges to serve media from the user's own directories. | MUST | `F-22` | M3 | Build matrix | Baseline |

## DOC — Documentation and developer experience

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `DOC-001` | Every feature shall have a specification with the ten mandatory sections and stable requirement… | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-002` | Every performance, compatibility or security claim in the documentation shall link to an… | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-003` | `WHERE_TO_CHANGE.md` shall list every source path a newcomer may need to change and shall stay… | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-004` | Every public module shall have a doc comment stating what it is | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-005` | Be readable by a programming beginner: acronyms explained | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-006` | Contain no duplicated normative statement; the canonical location shall be linked from… | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-007` | Superseded baselines shall be preserved unchanged in the archive with a manifest and a digest. | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-008` | Every document shall carry a status, an owner role, a version and a date. | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-009` | The getting-started guide shall contain commands that a reader can execute verbatim. | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-010` | Contain a single navigation path from README to the relevant feature specification and source… | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-011` | Broken internal links shall be detected by an automated gate. | MUST | `F-19` | M0 | Documentation gate | Baseline |
| `DOC-012` | Contribution guidance shall state the standards | MUST | `F-19` | M0 | Documentation gate | Baseline |

## SUP — Supply chain and licensing

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `SUP-001` | Contain the complete, unmodified AGPL-3.0 licence text and the licence metadata shall match it. | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-002` | Every direct dependency shall be recorded with its licence and its purpose before it is used. | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-003` | Fail the build if a dependency uses a licence that is incompatible with AGPL-3.0-or-later. | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-004` | Run over the dependency graph and its result shall be recorded. | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-005` | Release artefacts shall be accompanied by a software bill of materials and a checksum. | MUST | `F-19` | M0 | Licence and advisory gate | Verified |
| `SUP-006` | Any dependency that requires a native toolchain or ships a licence with distribution… | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-007` | Not vendor third-party source code into the repository without a record of its licence and… | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-008` | Carry no `unsafe` code unless an ADR approves it with a written safety argument. | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |
| `SUP-009` | Carry a defined code-signing and notarisation plan before the first stable release | MUST | `F-19` | M0 | Licence and advisory gate | Baseline |

## REL — Reliability

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `REL-001` | An abrupt termination of the server shall never modify a media original; after a restart the… | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |
| `REL-002` | A media root that is offline shall be reported as unavailable and shall not be silently removed… | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |
| `REL-003` | A network interface change, a sleep/wake cycle and a client disconnect shall not leak sockets… | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |
| `REL-004` | Survive a seven-day soak on a defined mixed workload without a crash | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |
| `REL-005` | A configuration change shall be validated, applied atomically and recoverable; an invalid… | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |
| `REL-006` | Active transfers shall survive a change of the presentation mode in the user interface | MUST | `F-18` | M4 | Fault-injection and soak test | Verified |

## RES — Adaptive resource management

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `RES-001` | Arbitrate streaming, discovery, metadata reading, indexing, artwork generation, optional… | MUST | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-002` | Every worker pool, queue, cache and connection count shall have a documented bound, a… | MUST | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-003` | Indexing shall yield when streaming latency, CPU, disk input/output or memory pressure crosses… | SHOULD | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-004` | A slow client shall not be mistaken for server overload | MUST | `F-18` | M2 | Resource-pressure test | Verified |
| `RES-005` | Live in the shared core and not in a user interface; platform adapters shall supply… | MUST | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-006` | Local resource metrics shall be opt-in to view and export | MUST | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-007` | When resources are insufficient the server shall fail gracefully with a meaningful explanation;… | MUST | `F-18` | M2 | Resource-pressure test | Implemented |
| `RES-008` | On a battery-powered or thermally constrained machine the server may reduce background work | SHOULD | `F-18` | M2 | Resource-pressure test | Verified |
| `RES-009` | Start-up, scanning, sleep/wake and shutdown shall release workers and file handles predictably. | MUST | `F-18` | M2 | Resource-pressure test | Implemented |

## LIFE — Service lifecycle and network recovery

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `LIFE-001` | Expose the distinct states *Starting*, *Ready*, *Degraded*, *Suspended*, *Recovering*,… | MUST | `F-13` | M2 | Lifecycle and recovery test | Implemented |
| `LIFE-002` | Handle a sleep/wake cycle without spinning, leaking resources or reporting a "ready" state it… | MUST | `F-13` | M2 | Lifecycle and recovery test | Scheduled |
| `LIFE-003` | Handle a change of interface, address, subnet or route, and shall never widen its network… | MUST | `F-13` | M2 | Lifecycle and recovery test | Scheduled |
| `LIFE-004` | When a configured interface disappears the server shall enter the *Degraded* or *Recovering*… | MUST | `F-13` | M2 | Lifecycle and recovery test | Scheduled |
| `LIFE-005` | Detect the loss and the return of a network or removable volume | MUST | `F-13` | M2 | Lifecycle and recovery test | Baseline |
| `LIFE-006` | A transfer interrupted by a real sleep or a real network loss shall end in a way that is… | MUST | `F-13` | M2 | Lifecycle and recovery test | Baseline |
| `LIFE-007` | Retry discovery and rebinding with a bounded backoff and jitter | MUST | `F-13` | M2 | Lifecycle and recovery test | Scheduled |
| `LIFE-008` | Recover from an abnormal termination with an atomic state and either a consistent catalogue or… | MUST | `F-13` | M2 | Lifecycle and recovery test | Baseline |
| `LIFE-009` | The administration surface and the user interface shall reflect the effective interface | MUST | `F-13` | M2 | Lifecycle and recovery test | Baseline |
| `LIFE-010` | The installation of the login-item service and the permissions it needs shall be explicit and… | MUST | `F-15` | M2 | Lifecycle and recovery test | Verified |
| `LIFE-011` | Keep a local, redacted timeline of network changes and recovery events that the user may export… | SHOULD | `F-13` | M2 | Lifecycle and recovery test | Implemented |

## MB — macOS menu bar application

| ID | Requirement | Priority | Feature | Milestone | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `MB-001` | Be able to run headless independently of the user interface; the interface shall reconnect and… | MUST | `F-15` | M2 | Manual macOS usability protocol | Verified |
| `MB-002` | Acknowledge every action visibly, report progress and explain errors, and shall never block its… | MUST | `F-15` | M2 | Manual macOS usability protocol | Implemented |
| `MB-003` | Support keyboard navigation, assistive-technology labels, scalable text, sufficient contrast… | MUST | `F-15` | M2 | Manual macOS usability protocol | Baseline |
| `MB-004` | Open no popover or settings view that starts an index or a network scan it does not need. | MUST | `F-15` | M2 | Manual macOS usability protocol | Verified |
| `MB-005` | Communicate with the server only over the local administration surface by default | MUST | `F-15` | M2 | Manual macOS usability protocol | Verified |
| `MB-006` | Remain legible in light and dark appearance, on a small display and when the menu bar is… | SHOULD | `F-15` | M2 | Manual macOS usability protocol | Implemented |
| `MB-007` | The first-run assistant shall offer an optional verification step that shows whether a… | SHOULD | `F-15` | M2 | Manual macOS usability protocol | Baseline |
| `MB-008` | Uninstalling and stopping shall be safe and obvious | MUST | `F-15` | M2 | Manual macOS usability protocol | Baseline |
| `MB-009` | Ship as a bundle with the metadata the operating system requires for its local-network and… | MUST | `F-15` | M2 | Manual macOS usability protocol | Implemented |

---


**Totals:** 346 requirements — 5 COULD, 284 MUST, 57 SHOULD.


## Coverage check

A requirement is only *planned* when it has all of: an ID, a definition, a priority,
an owning feature, a milestone and a verification method. A requirement missing any
of these stays `Draft` and is not part of the baseline.


## Revision history

| Version/date | Change |
| --- | --- |
| 1.1 / 2026-11-18 | Owner direction DEC-077 revises UX-008 and UX-017 for app-owned text scaling through 200% and reflow without clipping; the v1.0 baseline is preserved in `docs/99-archive/2026-11-18-larger-text-accessibility/`. |
