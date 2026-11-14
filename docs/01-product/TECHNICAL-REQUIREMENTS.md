# Technical requirements — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Architect
Scope: the **normative technical baseline**. This document *defines* the requirement IDs.
`REQUIREMENTS-INDEX.md` is the cross-cutting *view* of the same IDs (priority, feature,
milestone, verification, status). Feature specifications in `../03-features/` elaborate
these IDs with acceptance criteria; they do not redefine them.

Product-level commitments (`PRD-*`) are defined in `PRODUCT-REQUIREMENTS.md`.

**Reading rule:** a requirement is binding when its ID appears here. Anything written
only in a feature specification is a *proposal* until it is given an ID here.

---

## 1. Protocol layer — SSDP discovery

Normative source: UPnP Device Architecture 1.1 / 2.0. Verified obligations are marked
**[N]** in `../05-quality/INTEROPERABILITY-MATRIX.md`; the research that produced them is
recorded in the sources list of that document.

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-001 | The server shall answer `M-SEARCH` requests on UDP 1900 within at most 1 second and shall answer with `HTTP/1.1 200 OK` and the headers `ST`, `USN`, `LOCATION`, `CACHE-CONTROL: max-age=1800`, `SERVER`, `DATE` and `EXT`. | MUST |
| INT-002 | The server shall match `ST` against `ssdp:all`, `upnp:rootdevice`, the advertised MediaServer device type and the advertised service types, and shall ignore an `ST` it does not advertise. | MUST |
| INT-003 | The server shall send its SSDP response unicast to the address and port of the requester. | MUST |
| INT-004 | The server shall send periodic `NOTIFY` advertisements with `NT: upnp:rootdevice` and `NTS: ssdp:alive`, re-advertising at an interval no longer than a third of the advertised `max-age`, and shall send `ssdp:byebye` for each advertised `NT` on graceful shutdown. | MUST |
| INT-005 | The `USN` header shall have the form `uuid:<UDN>::<target>` where `<target>` is `upnp:rootdevice`, `urn:schemas-upnp-org:device:MediaServer:1`, `urn:schemas-upnp-org:service:ContentDirectory:1`, `urn:schemas-upnp-org:service:ConnectionManager:1`, or the bare `uuid:<UDN>`. | MUST |
| INT-006 | The `LOCATION` header shall contain an absolute HTTP URL of the device description document that is reachable **on the network interface the request arrived on**. | MUST |
| INT-007 | The server shall bind SSDP to the IPv4 multicast group `239.255.255.250:1900` and shall support the IPv6 link-local group `FF02::C:1900`. | MUST |
| INT-008 | The server shall honour the `MX` header of an `M-SEARCH` by choosing a uniformly random delay inside `[0, min(MX, 5))` seconds, and shall ignore an `M-SEARCH` that carries no `MX` header. | MUST |
| INT-009 | The server shall select the sending interface by the route of the incoming request and shall never answer a request that arrived on an interface excluded by configuration. | MUST |
| INT-010 | The server shall ignore malformed `M-SEARCH` datagrams without logging them at error level and without affecting the next valid request. | MUST |
| INT-011 | The server shall rate-limit discovery responses per source address so that a single host cannot force an unbounded response rate. | SHOULD |
| INT-012 | The server shall expose the configured `friendlyName`, `manufacturer`, `modelName` and `UDN` in the SSDP response headers consistently with the device description. | MUST |
| INT-013 | The server shall answer `M-SEARCH` only when the `MAN` header contains the quoted token `ssdp:discover`, and shall not echo the header otherwise. | SHOULD |
| INT-014 | The server shall use a stable `UDN` across restarts for the same installation unless the user resets the identity explicitly, and the `USN` prefix shall equal that `UDN`. | MUST |
| INT-015 | The server shall provide a CLI switch to print the exact SSDP headers it would send, so that interoperability problems can be diagnosed without packet capture. | SHOULD |
| INT-016 | The server shall set the multicast time-to-live of its advertisements to 2, so that they do not leave the local link. | MUST |
| INT-017 | The server shall answer `M-SEARCH` for the device type `urn:schemas-upnp-org:device:MediaServer:1` in addition to the root device and the service types. | MUST |
| INT-018 | The server shall not rely on mDNS for discovery; SSDP is the DLNA discovery mechanism and no mDNS advertisement shall be sent. | MUST |
| INT-019 | The server shall expose the version of the UPnP architecture it implements in the `SERVER` header, in the form `<OS>/<version> UPnP/1.1 PhairDLNA/<version>`. | MUST |

## 2. Protocol layer — device and service description

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-020 | The server shall serve an XML device description at a stable URL path that contains a root device with `deviceType` `urn:schemas-upnp-org:device:MediaServer:1`, a `UDN`, `friendlyName`, `manufacturer`, `modelName`, `modelNumber` and `modelURL`. | MUST |
| INT-021 | The device description shall list the `ContentDirectory:1` and `ConnectionManager:1` services, each with `serviceType`, `serviceId`, `SCPDURL`, `controlURL` and a non-empty `eventSubURL`; `AVTransport:1` may be listed as an optional service. | MUST |
| INT-022 | All URLs inside the device description shall be absolute HTTP URLs or root-relative paths that resolve correctly from the request host. | MUST |
| INT-023 | The server shall serve a valid SCPD document for every advertised service, describing every action with its arguments and every state variable with its data type and allowed value range. | MUST |
| INT-024 | The XML served by the server shall be well-formed, shall declare its encoding, and shall not contain a DOCTYPE declaration. | MUST |
| INT-025 | The server shall answer an unknown SOAP action with an HTTP fault using the UPnP error codes `401 Invalid Action` or `402 Invalid Args` as appropriate, and shall not crash. | MUST |
| INT-026 | The server shall respond to `GET` on the device description and on SCPD documents with `Content-Type: text/xml; charset="utf-8"` and a correct `Content-Length`. | MUST |
| INT-027 | The server shall not include a client-controllable value in the XML output without XML escaping. | MUST |
| INT-028 | The server shall advertise `ContentDirectory:1` in addition to any newer ContentDirectory version, so that legacy clients can be served. | MUST |
| INT-029 | The device description shall reference the `ConnectionManager` service even when no transcoding is available, because clients use it to learn the transfer protocols. | MUST |
| INT-044 | The device description shall use the service identifiers `urn:upnp-org:serviceId:ContentDirectory` and `urn:upnp-org:serviceId:ConnectionManager`, and shall declare `specVersion` 1.0. | MUST |
| INT-045 | The device description shall contain exactly one `device` element and shall not contain a `URLBase` element. | MUST |

## 3. Protocol layer — ContentDirectory

A conforming `MediaServer:1` device has exactly two mandatory services; the
`ContentDirectory` service must implement `Browse`, `GetSearchCapabilities`,
`GetSortCapabilities` and `GetSystemUpdateID`. Actions such as `GetFreeSpace`,
`GetUpdateCounter`, `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal` and
`GetShareIndexInBrowse` are **not** part of ContentDirectory 1–4 and shall not be
advertised as standard actions.

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-030 | The server shall implement the `Browse` action with the arguments `ObjectID`, `BrowseFlag`, `Filter`, `StartingIndex`, `RequestedCount`, `SortCriteria` and `SearchCriteria`. | MUST |
| INT-031 | The `Browse` response shall contain `Result`, `NumberReturned`, `TotalMatches` and `UpdateID` exactly once each. | MUST |
| INT-032 | The server shall honour `StartingIndex` and `RequestedCount` and shall never return more items than `RequestedCount` permits; `RequestedCount` of `0` shall mean "all items". | MUST |
| INT-033 | The server shall honour `BrowseFlag` such that `BrowseMetadata` returns only the container metadata with `NumberReturned` and `TotalMatches` both equal to 1, and `BrowseDirectChildren` returns its children with `TotalMatches` equal to the total number of children independent of `StartingIndex`. | MUST |
| INT-034 | The server shall apply the `Filter` argument, shall support the wildcard `*` that clients send constantly, and shall return the elements the client asked for when the filter is understood. | MUST |
| INT-035 | The server shall implement `GetSearchCapabilities` and `GetSortCapabilities` and shall return only capabilities it actually implements. | MUST |
| INT-036 | The server shall implement `Search` for the capability strings it advertises in `GetSearchCapabilities`. | SHOULD |
| INT-037 | The server shall implement `GetSystemUpdateID` and shall keep the returned value monotonically non-decreasing, changing whenever the catalogue changes. | MUST |
| INT-038 | The server shall return the object ID `0` for the root container and the standard ID `1` for the content directory root, consistently with its own hierarchy. | MUST |
| INT-039 | The server shall return an UPnP error `701 No Such Object` for an unknown `ObjectID` instead of an empty successful result. | MUST |
| INT-040 | The server shall not implement write operations (`CreateObject`, `DestroyObject`, `UpdateObject`, `MoveObject`, `ImportResource`, `ExportResource`, `DeleteResource`) in the baseline; those actions shall be absent from the SCPD or shall return an error rather than a partial implementation. | MUST |
| INT-041 | The server shall return `NumberReturned` and `TotalMatches` as non-negative integers and shall not exceed `TotalMatches`. | MUST |
| INT-042 | The server shall return a `Result` document that is valid DIDL-Lite XML and parses independently of the surrounding SOAP envelope. | MUST |
| INT-043 | The server shall return the container's `UpdateID` for a `Browse` of a container and the `SystemUpdateID` for a `Browse` of object ID `0`. | SHOULD |
| INT-046 | When `TotalMatches` cannot be computed, the server shall return `TotalMatches` of `0` with a non-zero `NumberReturned`, and shall return the SOAP error `720` when `NumberReturned` is also zero. | MUST |
| INT-047 | The server shall accept `SortCriteria` in the empty form and in the `+property` and `-property` forms, and shall advertise in `GetSortCapabilities` only the properties it can actually sort by. | MUST |
| INT-048 | The server shall not advertise or implement any non-standard ContentDirectory action; a client-specific extension shall be documented as a vendor extension and kept out of the standard service description. | MUST |
| INT-049 | The server shall expose `SearchCapabilities` and `SortCapabilities` as comma-separated lists of the property names it supports. | MUST |

## 4. Protocol layer — ConnectionManager and transfer protocols

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-050 | The server shall implement `GetProtocolInfo` and return at least the `http-get` transfer protocol with the MIME types it can actually serve. | MUST |
| INT-051 | The `GetProtocolInfo` result shall not advertise a container or MIME type that the server cannot deliver. | MUST |
| INT-052 | The server shall implement `GetCurrentConnectionIDs` and return an empty list when no transfer is active. | MUST |
| INT-053 | The server shall implement `PrepareForConnection`, `ConnectionComplete` and `GetCurrentConnectionInfo` in a way that never leaves a connection in a permanently "pending" state. | MUST |
| INT-054 | The server shall return the same `Source` value in `GetCurrentConnectionInfo` as the `res` URI it serves for the same object. | MUST |
| INT-055 | The server shall advertise the `http-get` protocol with the `*` wildcard for the additional header field only when it does not depend on request-specific headers. | SHOULD |
| INT-056 | The server shall not implement the `http-subscribe` transfer protocol in the baseline. | MUST |
| INT-057 | The server shall reject every GENA `SUBSCRIBE` request unless the user has explicitly enabled eventing, and shall reject a callback URL that is not a validated address on the local network. | MUST |
| INT-058 | The server shall never implement the UPnP Internet Gateway Device actions (`AddPortMapping`, `DeletePortMapping`, `GetExternalIPAddress`, `NewInternalClient`) and shall answer them with an error. | MUST |
| INT-059 | The server shall not implement a network self-update mechanism; updates shall be installed by the user through their platform's package manager. | MUST |

## 5. Protocol layer — DIDL-Lite metadata

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-060 | Every `<item>` and `<container>` shall carry the attributes `id`, `parentID` and `restricted`; containers shall additionally carry `childCount`. | MUST |
| INT-061 | The server shall use the UPnP object classes `object.item.audioItem.musicTrack`, `object.item.videoItem.movie`, `object.item.videoItem.videoBroadcast`, `object.item.imageItem.photo` and `object.container.storageFolder` as appropriate. | MUST |
| INT-062 | The `id` attribute shall be a stable, opaque identifier for the object that does not change between restarts and does not expose an absolute filesystem path. | MUST |
| INT-063 | The `res` element shall carry `protocolInfo`, `duration` (for timed media, formatted `H*:MM:SS.F*`), `size` and, for images, `resolution` formatted as `<width>x<height>`. | MUST |
| INT-064 | The `protocolInfo` string shall have the form `http-get:*:<mime>:<additional>` and shall name a MIME type the server actually serves. | MUST |
| INT-065 | The server shall XML-escape all title, artist, album, genre and description values: `&`, `<` and `>` in element text; `&`, `<`, `>`, `"` and `'` in attribute values. | MUST |
| INT-066 | The server shall include `dc:title` and `upnp:class` for every item and container. | MUST |
| INT-067 | The server shall include `upnp:albumArtURI` for audio items for which an artwork file exists, using an absolute URL the client can fetch over HTTP and without a `dlna:profileID` attribute. | SHOULD |
| INT-068 | The server shall include `dc:date` when the file system provides a modification date, formatted as an ISO 8601 date or date-time. | SHOULD |
| INT-069 | The server shall include `upnp:artist`, `upnp:album` and `upnp:genre` for audio items when the tags provide them. | SHOULD |
| INT-070 | The server shall include `upnp:originalTrackNumber` when the audio tags provide it. | COULD |
| INT-071 | The server shall produce a `DIDL-Lite` document whose root element is `DIDL-Lite` in the namespace `urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/`, and shall declare every namespace prefix it uses. | MUST |
| INT-072 | The server shall truncate a single metadata value to a documented maximum length so that no element or attribute value exceeds 8 KB in its XML-escaped UTF-8 form. | MUST |
| INT-073 | The server shall escape the complete DIDL-Lite document a second time when embedding it into the `Result` element of the SOAP body, because the document is carried as XML text. | MUST |
| INT-074 | The server shall report `res@bitrate` in bytes per second, as the metadata schema defines it. | MUST |
| INT-075 | The server shall emit `res@sampleFrequency`, `res@nrAudioChannels` and `res@bitsPerSample` when the media inspection provides them. | SHOULD |
| INT-076 | The server shall use `dc:creator` in addition to `upnp:artist` for the artist of an audio item. | SHOULD |
| INT-077 | The server shall not emit the `writeStatus` attribute, because its definition could not be verified against a primary specification. | MUST |
| INT-078 | The DLNA signalling in the fourth `protocolInfo` field shall be additive: the server shall be able to emit plain `http-get:*:<mime>:*` as well as a DLNA-flavoured string, and plain UPnP clients shall keep working in both modes. | MUST |
| INT-079 | The server shall emit `DLNA.ORG_PN` for a media resource only when the content actually conforms to that DLNA Media Format Profile, and shall emit `DLNA.ORG_OP=01` only when byte-range seeking is supported. | MUST |

## 6. Protocol layer — HTTP media delivery

Normative source: RFC 9110 / RFC 9112. The server serves **HTTP/1.1**, which the DLNA
guidelines require of a media server.

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-080 | The media server shall answer `GET` and `HEAD` requests for media objects with `HTTP/1.1 200 OK`, the correct `Content-Type`, and a `Content-Length` equal to the file size. | MUST |
| INT-081 | The media server shall support a single byte range and answer with `HTTP/1.1 206 Partial Content`, a correct `Content-Range: bytes <first>-<last>/<total>` and a `Content-Length` equal to the range length. | MUST |
| INT-082 | The media server shall answer a range request whose start is beyond the end of the file with `HTTP/1.1 416 Range Not Satisfiable` and a `Content-Range: bytes */<size>` header. | MUST |
| INT-083 | The media server shall answer a syntactically invalid or unsatisfiable `Range` header with `HTTP/1.1 416` rather than with a partial content response, and shall never answer with a malformed body. | MUST |
| INT-084 | The media server shall advertise `Accept-Ranges: bytes` on media responses. | MUST |
| INT-085 | The media server shall not implement `multipart/byteranges`; for a multi-range request it shall serve the first satisfiable range as a single `206` or ignore the `Range` header and answer `200`. | MUST |
| INT-086 | The media server shall answer a `HEAD` request with the same headers as the corresponding `GET`, including `Content-Length` and `Accept-Ranges`, and without a body. | MUST |
| INT-087 | The media server shall use the extension of the file, refined by the sniffed content, to determine the `Content-Type`; when both are unknown it shall fall back to `application/octet-stream`. | MUST |
| INT-088 | The media server shall not use chunked transfer encoding for media objects and shall always send a `Content-Length`. | MUST |
| INT-089 | The media server shall close a connection that a client abandoned mid-transfer without leaking the transfer slot. | MUST |
| INT-090 | The media server shall serve the media URI that it advertised in the DIDL-Lite `res` element, byte-for-byte, for the lifetime of the catalogue entry. | MUST |
| INT-091 | The media server shall keep the file open for the duration of the transfer when possible, and shall reopen it if it was removed and restored. | SHOULD |
| INT-092 | The media server shall ignore a `Range` header on a method other than `GET`, and shall ignore a range unit it does not support. | MUST |
| INT-093 | The media server shall honour `If-Range`: when its condition does not hold, it shall ignore the `Range` header and answer `200` with the full representation. | MUST |
| INT-094 | The media server shall answer `404` for an unknown media identifier, `405` for an unsupported method and `400` for a malformed request line or header block. | MUST |
| INT-095 | The media server shall support HTTP/1.1 persistent connections, and every response shall carry either a `Content-Length` or `Connection: close`. | MUST |
| INT-096 | The media server shall emit the DLNA response headers `transferMode.dlna.org` and `contentFeatures.dlna.org` on media responses, and shall answer `406` when the client requests a transfer mode the server does not support for that content. | SHOULD |
| INT-097 | The media server shall answer a request carrying `getcontentFeatures.dlna.org: 1` with the `contentFeatures.dlna.org` header and shall reject any other value of that header with `400`. | SHOULD |
| INT-098 | The media server shall serve the media URL only when it resolves inside one of the configured roots. | MUST |
| INT-099 | The media server shall emit `realTimeInfo.dlna.org` with `DLNA.ORG_TLAG=*` on media responses. | COULD |

## 7. Client compatibility

| ID | Requirement | Priority |
| --- | --- | --- |
| INT-100 | The server shall maintain a declarative profile of known client quirks, each with the client family, the observed symptom, the workaround and the evidence that justified it. | MUST |
| INT-101 | A quirk workaround shall never violate a normative requirement of the UPnP or DLNA specifications for other clients. | MUST |
| INT-102 | The server shall record every device-test result — device model, firmware version, client application, media format and observed outcome — in the interoperability matrix. | MUST |
| INT-103 | The server shall provide a diagnostics command that replays a stored `M-SEARCH` request and prints the response it would produce. | SHOULD |
| INT-104 | The server shall allow the user to disable a quirk workaround globally without editing source code. | SHOULD |

---

## 8. Library and media management

| ID | Requirement | Priority |
| --- | --- | --- |
| FUN-001 | The user shall be able to configure one or more media roots; each root shall have a name shown to clients and an absolute filesystem path. | MUST |
| FUN-002 | The server shall serve the union of all configured roots as one catalogue. | MUST |
| FUN-003 | The server shall not follow a symbolic link that points outside its root unless the user explicitly allows it. | MUST |
| FUN-004 | The server shall ignore hidden files and macOS resource-fork sidecar files (`._*`) when building the catalogue. | MUST |
| FUN-005 | The server shall index audio, video, image and playlist files by extension and by sniffed content type. | MUST |
| FUN-006 | The server shall re-scan a root incrementally and detect added, changed and removed files without a full re-read of unchanged files. | MUST |
| FUN-007 | The server shall detect a root that is unmounted, inaccessible or replaced by a different volume, and shall report it in diagnostics instead of failing the whole catalogue. | MUST |
| FUN-008 | The server shall keep the catalogue usable while a rescan runs; a browse during a scan shall return the previously indexed content. | MUST |
| FUN-009 | The server shall store its catalogue in a local SQLite database that can be deleted and rebuilt without data loss for the user's media. | MUST |
| FUN-010 | The server shall recover from a corrupted index database by rebuilding it automatically, and shall report that it did so. | MUST |
| FUN-011 | The server shall assign stable object identifiers that do not change when the catalogue is rebuilt. | MUST |
| FUN-012 | The server shall extract duration, dimensions, bitrate and codec information for media files where it can do so without executing an external program. | SHOULD |
| FUN-013 | The server shall extract embedded artwork from audio files and use a sidecar image when one exists. | SHOULD |
| FUN-014 | The server shall generate thumbnails for image files and video posters on demand and cache them in a disposable cache. | SHOULD |
| FUN-015 | The server shall support playlists in M3U, M3U8 and PLS format, resolving entries relative to the playlist file. | SHOULD |
| FUN-016 | The server shall expose a subtitle sidecar file (`.srt`, `.vtt`, `.sub`, `.ass`) as an additional `res` element where the client profile supports it. | SHOULD |
| FUN-017 | The server shall support media stored on removable volumes and network shares, and shall recover when such a volume reappears. | MUST |
| FUN-018 | The server shall never write to, rename or delete a file inside a media root, with the single exception of its own cache directory when the cache is configured inside the root. | MUST |
| FUN-019 | The server shall keep the raw path exactly as the operating system reported it for every file operation, and shall never re-encode or re-normalise a path before using it. | MUST |
| FUN-020 | The server shall compute a separate identity key for indexing and de-duplication by applying Unicode NFC and a per-component case fold to the path, scoped by the identity of the volume it lives on; it shall determine uniqueness from the platform's stable file identity rather than from the path. | MUST |
| FUN-021 | The server shall provide a way to exclude subdirectories of a root. | SHOULD |
| FUN-022 | The server shall support multiple roots that point to the same underlying volume without producing duplicate catalogue entries. | SHOULD |
| FUN-023 | The server shall expose the free space of a root through the ContentDirectory service when the file system provides it. | COULD |
| FUN-024 | The server shall allow the user to configure a scan schedule (interval, or manual only). | SHOULD |
| FUN-025 | The server shall make a newly added file visible in the catalogue within 30 seconds on a file system that delivers reliable change events, and shall document the fallback behaviour when it does not. | SHOULD |

## 9. Playback and streaming

| ID | Requirement | Priority |
| --- | --- | --- |
| FUN-040 | The server shall serve media by byte range from the original file without transcoding whenever the client accepts the format. | MUST |
| FUN-041 | The server shall report the container and codec of a file in a way a client can use to decide whether it can play it. | MUST |
| FUN-042 | The server shall support multiple simultaneous direct-play streams, bounded by a configurable limit. | MUST |
| FUN-043 | The server shall bound the number of concurrent transfers and shall refuse further transfers with a clear error rather than exhausting resources. | MUST |
| FUN-044 | The server shall apply a per-stream bandwidth limit when configured, and shall document that the limit applies to the whole transfer, not to a rate. | SHOULD |
| FUN-045 | The server shall report the position and duration of the active transfer through the AVTransport service when that service is implemented. | SHOULD |
| FUN-046 | The server shall cancel a transfer cleanly when the client closes the connection. | MUST |
| FUN-047 | The server shall implement optional transcoding only when the user has explicitly enabled it and selected a target format. | COULD |
| FUN-048 | The transcoder, if enabled, shall run in a separate process with an explicit resource budget, a queue bound and a cancellation path. | COULD |
| FUN-049 | The server shall never silently fall back to transcoding; a file that cannot be played directly shall be reported as incompatible. | MUST |
| FUN-050 | The server shall keep the buffer size small and bounded so that a slow client cannot consume unbounded memory. | MUST |

## 10. Configuration

| ID | Requirement | Priority |
| --- | --- | --- |
| CFG-001 | The server shall read its configuration from a single, versioned configuration file whose path is given by an environment variable or a command-line switch. | MUST |
| CFG-002 | The configuration schema shall be typed and validated at start-up; an invalid value shall abort start-up with a message naming the field, the file, the expected type and the accepted values. | MUST |
| CFG-003 | The configuration file shall carry a `schema_version`, and the server shall migrate an older known schema version automatically and warn about it. | MUST |
| CFG-004 | The server shall provide documented defaults for every setting such that an empty configuration file yields a working, secure server. | MUST |
| CFG-005 | The server shall refuse to start when the configuration would expose an administration interface to a non-loopback address, unless the user explicitly overrides it. | MUST |
| CFG-006 | The server shall support selecting the network interfaces used for discovery and for media delivery. | MUST |
| CFG-007 | The server shall support a per-root setting for symbolic link following, disabled by default. | MUST |
| CFG-008 | The server shall support resource limits: maximum concurrent streams, maximum cache size, maximum metadata extraction time, maximum index workers. | SHOULD |
| CFG-009 | The server shall support an optional access-control list of client addresses allowed to browse or to stream. | SHOULD |
| CFG-010 | The server shall support a "trusted client" list whose quirks are handled by a specific device profile. | SHOULD |
| CFG-011 | The server shall provide a command that validates a configuration file without starting the server. | MUST |
| CFG-012 | The server shall provide a command that exports the effective configuration and one that imports it, preserving the schema version. | SHOULD |
| CFG-013 | Unknown configuration keys shall be reported as errors rather than ignored, so that a typo never silently disables a security setting. | MUST |
| CFG-014 | Secrets (such as an administration token) shall be readable from a file with owner-only permissions and shall never appear in the configuration file itself. | MUST |
| CFG-015 | The configuration file shall never contain a value that is expanded from an environment variable without an explicit opt-in, to keep the file's meaning unambiguous. | SHOULD |

## 11. Command line and service operation

| ID | Requirement | Priority |
| --- | --- | --- |
| OPS-001 | The server shall provide a command that runs the daemon in the foreground with structured log output on standard error. | MUST |
| OPS-002 | The server shall provide a command that reports the current status: version, uptime, configured roots, catalogue size, active transfers and last scan result. | MUST |
| OPS-003 | The server shall provide a command that scans a root on demand and reports progress and errors. | MUST |
| OPS-004 | The server shall provide a command that lists the known network interfaces and which ones would be used for discovery. | MUST |
| OPS-005 | The server shall provide a command that prints the device description and the SSDP headers it would advertise, for debugging. | SHOULD |
| OPS-006 | The server shall write logs with a timestamp, a level, a component and a structured message. | MUST |
| OPS-007 | Log output shall be redactable: file system paths, client addresses and identifiers shall be replaceable by a stable pseudonym when redaction is enabled. | MUST |
| OPS-008 | The server shall shut down gracefully on `SIGINT` and `SIGTERM`, sending `ssdp:byebye` and closing open transfers within a bounded time. | MUST |
| OPS-009 | The server shall support running as a background service on macOS (launchd), Linux (systemd user service) and Windows (service). | SHOULD |
| OPS-010 | The server shall support a configuration file that sets the service identity, so that a second instance can run next to the first. | SHOULD |
| OPS-011 | The command line shall provide `--help` and `--version` output that names the version, the licence and the documentation location. | MUST |
| OPS-012 | The server shall expose its version in the `SERVER` header and in the device description. | MUST |
| OPS-013 | The server shall write a structured diagnostic report (anonymised) that a user can attach to a bug report. | SHOULD |
| OPS-014 | The command line shall return a distinct exit code for configuration errors, runtime failures and "nothing to do", so that scripts can react. | SHOULD |
| OPS-015 | The server shall never require root privileges to serve media from the user's own directories. | MUST |
| OPS-016 | The server shall expose an unauthenticated route that serves the corresponding source of the exact running build, so that a network user can obtain it at no charge as the licence requires. | MUST |
| OPS-017 | The server shall report the exact source revision it was built from in its status output and in the source offer. | MUST |

## 12. Local administration

| ID | Requirement | Priority |
| --- | --- | --- |
| OPS-020 | The administration surface shall bind to the loopback interface only by default and shall refuse to start when configured otherwise without an explicit override. | MUST |
| OPS-021 | The administration surface shall require a per-installation random token for every state-changing request. | MUST |
| OPS-022 | The administration surface shall validate the `Host` header against the loopback names to resist DNS-rebinding attacks. | MUST |
| OPS-023 | State-changing administration requests shall require a CSRF token bound to the session. | MUST |
| OPS-024 | The administration surface shall not expose media file contents, only metadata and control actions. | MUST |
| OPS-025 | The administration surface shall be optional; the server shall run completely without it. | MUST |
| OPS-026 | The administration surface shall expose a clear "what is this, who can see it, how to turn it off" explanation in the first-run output. | MUST |
| OPS-027 | The administration surface shall apply the same rate limiting and request-size limits as the media surface. | MUST |

## 13. Security and privacy

### 13.1 Filesystem

| ID | Requirement | Priority |
| --- | --- | --- |
| SEC-001 | The server shall resolve every media path canonically and verify that it is inside an allowed root before opening it. | MUST |
| SEC-002 | The server shall reject a path containing `..` segments, encoded separators (`%2f`, `%5c`), or a null byte, before canonicalisation. | MUST |
| SEC-003 | The server shall resolve a symbolic link and re-check containment after resolution, to resist a link that changes between check and use. | MUST |
| SEC-004 | The server shall open files in a way that does not follow a final symbolic link when the file is outside the root. | MUST |
| SEC-005 | The server shall never construct a shell command from user, client or file metadata input. | MUST |
| SEC-006 | The server shall not follow a symbolic link that points outside a root, unless the user explicitly enabled it for that root, and it shall log the attempt. | MUST |
| SEC-007 | The server shall treat all file names and metadata as untrusted input, including their length and encoding. | MUST |
| SEC-008 | The server shall open media files read-only, without requesting write permission on the containing directory. | MUST |

### 13.2 Network and parsing

| ID | Requirement | Priority |
| --- | --- | --- |
| SEC-010 | The XML parser shall not process a DTD, shall not resolve external entities and shall bound the nesting depth and the total document size. | MUST |
| SEC-011 | The server shall bound the size of every accepted HTTP request line, header block and body. | MUST |
| SEC-012 | The server shall bound the number of concurrent connections and the number of concurrent transfers. | MUST |
| SEC-013 | The server shall answer an unauthenticated or malformed request with a generic error that reveals no internal path, version detail or stack trace. | MUST |
| SEC-014 | The server shall not follow a redirect to a non-local address, and shall reject a callback URL that is not a validated local address. | MUST |
| SEC-015 | The server shall validate and normalise every URL path segment it accepts, and shall decode percent-encoding exactly once. | MUST |
| SEC-016 | The server shall enforce a read and write timeout on every connection. | MUST |
| SEC-017 | The server shall never bind a non-loopback port other than the media port without an explicit configuration entry. | MUST |
| SEC-018 | The server shall treat an SSDP or SOAP peer as untrusted and shall perform no action on its behalf other than answering it. | MUST |

### 13.3 Privacy

| ID | Requirement | Priority |
| --- | --- | --- |
| SEC-090 | The product shall contain no telemetry, no analytics and no crash reporting to any third party. | MUST |
| SEC-091 | The product shall make no outbound network connection other than the SSDP multicast announcements and responses, and the media connections it serves, unless the user explicitly starts a feature that requires internet access. | MUST |
| SEC-092 | The product shall not attempt to configure the user's router, and shall not speak the UPnP Internet Gateway Device protocol. | MUST |
| SEC-093 | The product shall store no personal data beyond local configuration, the catalogue index and local logs. | MUST |
| SEC-094 | Logs and diagnostics shall be redactable, and the redaction shall apply to file system paths, client addresses and device identifiers. | MUST |
| SEC-095 | The catalogue database shall be a local file with user-only permissions, and the server shall not create it inside a shared or world-readable location by default. | MUST |
| SEC-096 | The product shall ship no third-party service, no advertising identifier and no device fingerprint. | MUST |

### 13.4 Resource limits

| ID | Requirement | Priority |
| --- | --- | --- |
| SEC-060 | Every parser in the server shall have an explicit input size bound and a documented failure behaviour. | MUST |
| SEC-061 | The server shall bound the number of entries in a single DIDL-Lite response and the size of the generated XML document. | MUST |
| SEC-062 | The server shall run metadata extraction with a time limit per file and shall abandon a file that exceeds it. | MUST |
| SEC-063 | The server shall bound the number of open file descriptors used by transfers and shall release them on cancellation. | MUST |
| SEC-064 | The server shall bound the size of its on-disk cache and shall evict entries when the limit is exceeded. | MUST |
| SEC-065 | The server shall never allocate memory proportional to a client-supplied length field without validating it against the actual file size. | MUST |
| SEC-066 | The server shall parse media metadata and generate thumbnails in a separate, sandboxed process with no network access, a bounded memory and CPU budget and a time limit, and shall quarantine a file whose parsing fails. | MUST |
| SEC-067 | The server shall bound the declared dimensions of an image before allocating any buffer for it, so that a file declaring enormous dimensions cannot exhaust memory. | MUST |
| SEC-068 | The server shall treat playlist entries as untrusted paths, shall resolve and contain them, and shall refuse entries that use a network scheme. | MUST |
| SEC-069 | The server shall not resolve or follow a macOS Finder alias as if it were a symbolic link; aliases shall be treated according to the same containment rules as symbolic links. | MUST |
| SEC-070 | The server shall offer the corresponding source of the running build, including its dependency tree and build recipe, over HTTP without authentication, as required by the AGPL network-use clause. | MUST |
| SEC-071 | The server shall accept only a `Host` header that matches a configured local host name, so that a DNS-rebinding attack from a web page cannot reach the server. | MUST |
| SEC-072 | The server shall require a custom, non-standard request header on every state-changing administration request, in addition to the CSRF token. | MUST |

## 14. Performance and resources

| ID | Requirement | Priority |
| --- | --- | --- |
| PERF-001 | The headless server shall keep its resident memory below 50 MB while idle with 10 000 indexed objects. | SHOULD |
| PERF-002 | The headless server shall reach the "ready to answer discovery" state within 2 seconds of start-up on the reference hardware. | SHOULD |
| PERF-003 | An incremental rescan of a tree with 10 000 unchanged files shall complete without re-reading file contents. | MUST |
| PERF-004 | The server shall index at least 200 media files per second on the reference hardware when metadata extraction is disabled. | SHOULD |
| PERF-005 | A `Browse` request over a container with 10 000 children shall return its first page within 100 ms. | MUST |
| PERF-006 | The server shall sustain 10 simultaneous direct-play streams without unbounded memory growth. | SHOULD |
| PERF-007 | Memory growth per active stream shall be bounded and documented. | MUST |
| PERF-008 | The server shall not perform a full directory scan on a browse request. | MUST |
| PERF-009 | The catalogue database shall support 100 000 objects with an index scan that stays within a documented budget. | SHOULD |
| PERF-010 | The reference hardware for all performance measurements shall be documented, including model, CPU, RAM, storage type and file system. | MUST |
| PERF-011 | Every performance claim shall report the method, the hardware, the number of iterations and the statistic used (median and 95th percentile). | MUST |
| PERF-012 | The server shall degrade gracefully when a catalogue is larger than the tested envelope: it shall remain responsive and shall not exhaust memory. | MUST |
| PERF-013 | The CPU usage of an idle server shall be negligible (below 1 % averaged over one minute on the reference hardware). | SHOULD |
| PERF-014 | The server shall survive a scan of a root that contains a broken or slow network share without blocking browse requests. | MUST |
| PERF-015 | A client shall discover the server within 10 seconds at the 95th percentile over 30 trials on a documented single-subnet LAN. | SHOULD |

## 15. Usability and accessibility

| ID | Requirement | Priority |
| --- | --- | --- |
| UX-001 | The server shall be fully operable without any graphical interface. | MUST |
| UX-002 | A first run shall require no more than one action (choosing a folder) before the server is usable on the network. | SHOULD |
| UX-003 | The first-run output shall name the device name, the address it is reachable at and how to verify it from another device. | MUST |
| UX-004 | Every error message shall state what happened, why it happened and what the user can do next. | MUST |
| UX-005 | The server shall never abort with a panic message that lacks context; unexpected failures shall produce a structured internal error and a readable one-line summary. | MUST |
| UX-006 | The command line shall provide `--help` for every subcommand. | MUST |
| UX-007 | The macOS application shall be accessible with the keyboard alone and shall expose the status of the server as text, not only as an icon. | MUST |
| UX-008 | The macOS application shall respect the system accessibility settings for contrast, motion and text size. | SHOULD |
| UX-009 | The macOS application shall show an accessible status item for assistive technologies. | MUST |
| UX-010 | The onboarding shall use plain language and avoid protocol jargon in the user-facing text. | MUST |
| UX-011 | Diagnostics shall be copyable in one action and shall not contain personal data unless the user enables redaction off deliberately. | SHOULD |
| UX-012 | The server shall tell the user which client is currently connected and what it is streaming. | SHOULD |
| UX-013 | Long-running operations shall report progress and allow cancellation. | MUST |
| UX-014 | The user interface shall present the server state, the scan progress, the shared folders and the last observed client activity in one view. | SHOULD |
| UX-015 | The product shall provide a simple presentation mode and an advanced presentation mode over one configuration schema; switching between them shall never reset a setting, restart the server or interrupt a transfer. | SHOULD |
| UX-016 | A safety-critical warning raised in the advanced mode shall remain visible in the simple mode. | MUST |
| UX-017 | The user interface shall be operable with the keyboard alone, shall expose accessible labels and a deterministic focus order, and shall respect the system's text size and contrast settings. | MUST |
| UX-018 | The user interface shall expose the status of the server as text for assistive technologies, not only as an icon. | MUST |
| UX-019 | The user shall be able to start and stop the server, pause and resume indexing, and remove a shared folder without touching the source files. | MUST |
| UX-020 | All user-facing strings shall be prepared for translation, and the set of supported locales shall be decided before the first release. | SHOULD |
| MB-001 | The server shall be able to run headless independently of the user interface; the interface shall reconnect and reflect the actual server state rather than a cached assumption. | MUST |
| MB-002 | The user interface shall acknowledge every action visibly, report progress and explain errors, and shall never block its event loop with scanning, metadata or network input/output. | MUST |
| MB-003 | The user interface shall support keyboard navigation, assistive-technology labels, scalable text, sufficient contrast and the reduced-motion preference. | MUST |
| MB-004 | The user interface shall open no popover or settings view that starts an index or a network scan it does not need. | MUST |
| MB-005 | The user interface shall communicate with the server only over the local administration surface by default, and shall use the authenticated and authorised model defined for that surface. | MUST |
| MB-006 | The menu bar status shall remain legible in light and dark appearance, on a small display and when the menu bar is crowded, and shall not rely on colour alone. | SHOULD |
| MB-007 | The first-run assistant shall offer an optional verification step that shows whether a television can discover and play the server, without assuming that a television is present. | SHOULD |
| MB-008 | Uninstalling and stopping shall be safe and obvious, and the auto-start setting shall be visible; no hidden persistent helper shall remain. | MUST |
| MB-009 | The menu bar application shall ship as a bundle with the metadata the operating system requires for its local-network and folder-access permissions. | MUST |

## 16. Platform and portability

| ID | Requirement | Priority |
| --- | --- | --- |
| PLT-001 | The core crate shall not depend on any OS-specific crate and shall compile on macOS, Linux and Windows from the same source. | MUST |
| PLT-002 | The project shall provide a build matrix covering macOS (arm64, x86_64), Linux (x86_64) and Windows (x86_64). | MUST |
| PLT-003 | All OS-specific behaviour shall be reachable through a documented trait with a default implementation for unsupported platforms. | MUST |
| PLT-004 | The server shall support macOS Apple Silicon natively without a Rosetta translation requirement. | MUST |
| PLT-005 | The server shall detect the case sensitivity, the Unicode normalisation behaviour and the volume identity of each media root instead of assuming them. | MUST |
| PLT-006 | The server shall run correctly when the user's home directory or a media root contains spaces, non-ASCII characters or emoji. | MUST |
| PLT-007 | Windows path handling shall support long paths and shall reject path traversal attempts using both separator characters. | MUST |
| PLT-008 | The media URI scheme shall not expose absolute file system paths to clients. | MUST |
| PLT-009 | The build shall not require a C toolchain beyond what the platform already provides, and any native dependency shall be documented. | SHOULD |
| PLT-010 | The server shall behave identically on all platforms with respect to protocol behaviour, ordering and identifiers. | MUST |
| PLT-011 | The server shall treat the mount, unmount and remount of a volume as a first-class event and shall re-key the affected catalogue entries when a volume reappears under a different identity. | MUST |
| PLT-012 | The server shall treat a permission error on a subtree as a quarantined subtree that needs consent, and shall continue indexing the rest of the root. | MUST |
| PLT-013 | On macOS the always-on server shall run as a system-context launchd daemon and the user interface as a per-user launchd agent, because the operating system's local-network permission exempts daemons but not agents. | MUST |
| PLT-014 | The macOS application shall be built as a universal binary for Apple Silicon and Intel. | MUST |
| PLT-015 | The Windows installation shall declare long-path support in the executable manifest and shall document that the system-wide `LongPathsEnabled` setting is also required. | MUST |
| PLT-016 | The Linux distribution shall provide native packages and shall not use a filesystem-sandboxed package format as its primary distribution channel, because such a sandbox blocks access to arbitrary media folders. | MUST |
| PLT-017 | The server shall obtain the user's consent for folders that the operating system protects (macOS privacy prompts, Linux polkit, Windows elevation), and shall explain the denial instead of failing silently. | MUST |
| PLT-018 | The server shall not require elevated privileges to serve media from the user's own directories. | MUST |

## 17. Adaptive resource management

| ID | Requirement | Priority |
| --- | --- | --- |
| RES-001 | The server shall arbitrate streaming, discovery, metadata reading, indexing, artwork generation, optional transcoding and maintenance through one scheduler with explicit priority classes; streaming correctness and control-plane responsiveness shall take precedence over background indexing. | MUST |
| RES-002 | Every worker pool, queue, cache and connection count shall have a documented bound, a cancellation policy and a defined overload behaviour. | MUST |
| RES-003 | Indexing shall yield when streaming latency, CPU, disk input/output or memory pressure crosses a documented threshold, and shall use hysteresis so that it does not oscillate. | SHOULD |
| RES-004 | A slow client shall not be mistaken for server overload, and the server shall not treat one client's slowness as a reason to degrade another client's stream. | MUST |
| RES-005 | The resource policy shall live in the shared core and not in a user interface; platform adapters shall supply operating-system hints, not policy decisions. | MUST |
| RES-006 | Local resource metrics shall be opt-in to view and export, shall be privacy-preserving and shall never be transmitted off the machine. | MUST |
| RES-007 | When resources are insufficient the server shall fail gracefully with a meaningful explanation; it shall never silently drop an active stream, corrupt state or alter a source file. | MUST |
| RES-008 | On a battery-powered or thermally constrained machine the server may reduce background work, but it shall never silently change which media is exposed and never disable an active playback. | SHOULD |
| RES-009 | Start-up, scanning, sleep/wake and shutdown shall release workers and file handles predictably. | MUST |

## 17. Reliability

| ID | Requirement | Priority |
| --- | --- | --- |
| REL-001 | An abrupt termination of the server shall never modify a media original; after a restart the catalogue shall be consistent or safely rebuildable. | MUST |
| REL-002 | A media root that is offline shall be reported as unavailable and shall not be silently removed from the configuration or the catalogue. | MUST |
| REL-003 | A network interface change, a sleep/wake cycle and a client disconnect shall not leak sockets or tasks, and the server shall recover within a documented interval. | MUST |
| REL-004 | The server shall survive a seven-day soak on a defined mixed workload without a crash, an unbounded memory growth or an unbounded descriptor growth. | MUST |
| REL-005 | A configuration change shall be validated, applied atomically and recoverable; an invalid configuration shall never silently change the network exposure. | MUST |
| REL-006 | Active transfers shall survive a change of the presentation mode in the user interface, and the headless service shall remain independent of the life cycle of any user interface. | MUST |

## 18. Service lifecycle and network recovery

| ID | Requirement | Priority |
| --- | --- | --- |
| LIFE-001 | The server shall expose the distinct states *Starting*, *Ready*, *Degraded*, *Suspended*, *Recovering*, *Stopping*, *Stopped* and *Failed*, and every transition shall carry a reason, a timestamp and a user-facing explanation. | MUST |
| LIFE-002 | The server shall handle a sleep/wake cycle without spinning, leaking resources or reporting a "ready" state it does not have; on wake it shall refresh the active interfaces, rebind its listeners as required and re-advertise only on permitted interfaces. | MUST |
| LIFE-003 | The server shall handle a change of interface, address, subnet or route, and shall never widen its network exposure beyond the configured policy. | MUST |
| LIFE-004 | When a configured interface disappears the server shall enter the *Degraded* or *Recovering* state and shall not automatically expose itself on another interface. | MUST |
| LIFE-005 | The server shall detect the loss and the return of a network or removable volume, shall preserve the configured roots and the index, and shall mark the affected entries as unavailable rather than deleting them. | MUST |
| LIFE-006 | A transfer interrupted by a real sleep or a real network loss shall end in a way that is truthful for the protocol; the server shall not promise a transparent continuation that the client cannot achieve. | MUST |
| LIFE-007 | The server shall retry discovery and rebinding with a bounded backoff and jitter, and shall not produce a multicast storm or a tight retry loop. | MUST |
| LIFE-008 | The server shall recover from an abnormal termination with an atomic state and either a consistent catalogue or a safe repair, and shall not require a user interface to be running. | MUST |
| LIFE-009 | The administration surface and the user interface shall reflect the effective interface, address, listener state and the last recovery event, with the technical detail available on demand. | MUST |
| LIFE-010 | The installation of the login-item service and the permissions it needs shall be explicit and reversible, and the server shall not install a hidden privileged helper or bypass the firewall. | MUST |
| LIFE-011 | The server shall keep a local, redacted timeline of network changes and recovery events that the user may export after reviewing it. | SHOULD |

## 19. Documentation and developer experience

| ID | Requirement | Priority |
| --- | --- | --- |
| DOC-001 | Every feature shall have a specification with the ten mandatory sections and stable requirement IDs. | MUST |
| DOC-002 | Every performance, compatibility or security claim in the documentation shall link to an evidence record. | MUST |
| DOC-003 | `WHERE_TO_CHANGE.md` shall list every source path a newcomer may need to change and shall stay in sync with the tree. | MUST |
| DOC-004 | Every public module shall have a doc comment stating what it is, why it exists and a usage example. | MUST |
| DOC-005 | The documentation shall be readable by a programming beginner: acronyms explained, terms defined, examples concrete. | MUST |
| DOC-006 | The documentation shall contain no duplicated normative statement; the canonical location shall be linked from everywhere else. | MUST |
| DOC-007 | Superseded baselines shall be preserved unchanged in the archive with a manifest and a digest. | MUST |
| DOC-008 | Every document shall carry a status, an owner role, a version and a date. | MUST |
| DOC-009 | The getting-started guide shall contain commands that a reader can execute verbatim. | MUST |
| DOC-010 | The repository shall contain a single navigation path from README to the relevant feature specification and source file. | MUST |
| DOC-011 | Broken internal links shall be detected by an automated gate. | MUST |
| DOC-012 | Contribution guidance shall state the standards, the gates and the expectations for evidence. | MUST |

## 20. Supply chain, licensing and provenance

| ID | Requirement | Priority |
| --- | --- | --- |
| SUP-001 | The repository shall contain the complete, unmodified AGPL-3.0 licence text and the licence metadata shall match it. | MUST |
| SUP-002 | Every direct dependency shall be recorded with its licence and its purpose before it is used. | MUST |
| SUP-003 | The licence check shall fail the build if a dependency uses a licence that is incompatible with AGPL-3.0-or-later. | MUST |
| SUP-004 | The vulnerability check shall run over the dependency graph and its result shall be recorded. | MUST |
| SUP-005 | Release artefacts shall be accompanied by a software bill of materials and a checksum. | MUST |
| SUP-006 | Any dependency that requires a native toolchain or ships a licence with distribution obligations shall be approved by the owner before it is adopted. | MUST |
| SUP-007 | The project shall not vendor third-party source code into the repository without a record of its licence and origin. | MUST |
| SUP-008 | The core shall carry no `unsafe` code unless an ADR approves it with a written safety argument. | MUST |
| SUP-009 | The macOS distribution shall carry a defined code-signing and notarisation plan before the first stable release, and the plan shall be recorded here. | MUST |

---

## 21. Requirement-level priorities

- **MUST** — required for correctness, safety or the promise of the product. A milestone
  with an open `MUST` cannot be accepted.
- **SHOULD** — strongly desirable; tracked, and may delay a milestone but does not
  invalidate it.
- **COULD** — optional enhancement; explicitly outside the current baseline.

The mapping from every ID to feature, milestone, verification method and status is in
`REQUIREMENTS-INDEX.md`.
