# PhairDLNA — DLNA / UPnP AV MediaServer Standards Brief

**Status:** Requirements phase. No implementation implied.
**Scope:** What a conforming UPnP AV `MediaServer` (a.k.a. DLNA **DMS**) must implement so that DLNA DMRs and plain UPnP AV control points can browse, search, and pull media.
**Target:** pure-Rust, macOS-first, AGPL-3.0-or-later.

---

## 0. How to read this document

### 0.1 Evidence tags

Every substantive claim carries one of:

| Tag | Meaning |
|---|---|
| **[STANDARD]** | Normative text taken from a specification I actually opened. |
| **[OBSERVED]** | Documented behaviour of real clients/servers (implementation source, vendor doc, or field report). |
| **[ASSUMPTION]** | My inference. Not in any spec. Treat as a design hypothesis to be validated. |
| **[OPEN]** | Could not be resolved from the sources I reached. Recorded in §9. |

**UNVERIFIED** is used where a primary source could not be opened at all (see §0.2).

### 0.2 Fetch methodology and its limits (read before trusting every quote)

* `upnp.org` (Open Connectivity Foundation) **serves a JavaScript challenge to direct downloads from this environment**. `curl` of `https://upnp.org/specs/...pdf` returns a ~3 KB HTML interstitial, and `web_fetch` on those URLs fails. Verbatim spec text for the OCF documents in §1–§4, §7 was therefore obtained through the **search engine's full-text extraction of the same primary PDF URLs** (the returned `content` field is the extracted PDF body, and the returned `link` is the canonical `upnp.org` URL). Those PDFs are listed in **Sources** with that caveat. Treat section numbers as reliable (they come from the document body) but treat them as "as printed in the PDF", not as independently re-verified pagination.
* The IETF RFCs (6761, 6762, 9110, 9112) **were downloaded verbatim** with `curl` from `rfc-editor.org` and quoted exactly.
* **The DLNA Guidelines themselves (IEC 62481 series) are not freely fetchable.** `dlna.org/guidelines` timed out from this environment. DLNA-layer claims below are therefore taken from Microsoft's published `[MS-DLNHND]` / `[MS-UPMC]` extensions (which quote and cite DLNA guideline numbers), the DLNA 4.0 white paper, Windows certification docs, IEC preview PDFs, and one non-authoritative Scribd copy. All such claims are tagged **[STANDARD]** only where a citable published document states them, otherwise **[OBSERVED]** / **[OPEN]**.
* Sibling-agent artifacts already present in this workspace (`/workspace/research/raw/`: MiniDLNA `upnphttp.c` / `upnpsoap.c` / `clients.c`, ReadyMedia `upnpsoap.c` / `clients.c`, Gerbera supported-device list) are used as **OBSERVED** evidence of shipped server behaviour.

### 0.3 Corrections to the task brief (important)

1. **RFC 6761 is not mDNS.** RFC 6761 is *"Special-Use Domain Names"* (Cheshire/Krochmal, February 2013), which defines the IANA special-use domain registry procedure. **mDNS is RFC 6762**, *"Multicast DNS"*, same authors/date. I downloaded both. Grepping RFC 6762 for `ssdp|upnp` returns **zero hits** — mDNS never mentions SSDP or UPnP, which is itself the cleanest possible evidence that mDNS is a *name-resolution* mechanism and does not replace SSDP discovery. See §2.7.
2. **`sec:httpDLNA` and `sec:SMB` do not exist as far as I could determine.** Neither token appears in the DIDL-Lite XSD, in any UPnP CD/CM document, in the Microsoft DLNA extensions, or in any implementation I read. The real `sec:` namespace is **Samsung's** (`http://www.sec.co.kr/dlna`, occasionally `http://www.sec.co.kr/`) and carries `sec:CaptionInfo`, `sec:CaptionInfoEx`, `sec:dcmInfo`, `sec:ProductCap`. The DLNA namespace is `urn:schemas-dlna-org:metadata-1-0/`. The DLNA-specific content lives in the **4th field of `res@protocolInfo`** (`DLNA.ORG_PN`, `DLNA.ORG_OP`, `DLNA.ORG_CI`, `DLNA.ORG_FLAGS`, `DLNA.ORG_PS`) and in **HTTP response headers** (`transferMode.dlna.org`, `contentFeatures.dlna.org`). Logged as **[OPEN]** in §9.
3. **`RenderingControl` and `AVTransport` are MediaRenderer-side services.** A MediaServer normally exposes **ContentDirectory** + **ConnectionManager**; `AVTransport` on a DMS is a *virtual instance* created per connection, and `RenderingControl` is not part of a MediaServer's mandatory service set at all. The task lists them under "MediaServer protocol surface" — they belong in the renderer's surface (§1.4, §1.5). A DMS that wants to be a full-featured device may also expose `RenderingControl` (to set volume on itself is meaningless) — but a *control point* that wants to control a renderer's volume calls **RenderingControl on the renderer**.
4. **Several requested ContentDirectory actions are not in UPnP CD:1–:4.** `GetFreeSpace`, `GetUpdateCounter`, `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal`, `GetShareIndexInBrowse` are **not** in the CD action tables I retrieved (§1.2) and are implemented by **neither MiniDLNA nor ReadyMedia** (grepped the local sources: 0 hits each). `GetTransferTotal` is a *field of the `GetTransferProgress` action*, not an action. `GetShareIndexInBrowse` appears in the wild as a **Sonos/Samsung-style extension** (it surfaced in a Go "sonos" ContentDirectory client), not as UPnP CD. **Recommendation: do not implement them as UPnP CD actions; if implemented at all, do so as vendor-extension actions in a separate namespace.** See §8 and §9.

---

## 1. Protocol surface required for a UPnP AV MediaServer

### 1.1 Device and service inventory

**[STANDARD]** `MediaServer:1` mandates two services and one device type:

> DeviceType `MediaServer:1` — Root **YES**; `ContentDirectory:1.0` — **R** (serviceId `ContentDirectory`); `ConnectionManager:1.0` — **R** (serviceId `ConnectorManager`).
> "Each implementation of the MediaServer requires a Content Directory and ConnectionManager service."
> — *UPnP-av-MediaServer-v1-Device.pdf* ([upnp.org](https://upnp.org/specs/av/UPnP-av-MediaServer-v1-Device.pdf))

**[STANDARD]** `MediaServer:2` adds: *"Any instance of a MediaServer MUST have a ContentDirectory service and a ConnectionManager service. For a given instance (MediaServer), there MUST only be one instance of these services."* and the **backward-compatibility rule**: *"a MediaServer:2 implementation MUST respond to all SSDP queries that specify MediaServer:1 and MUST respond to all actions defined by the MediaServer:1"* ([UPnP-av-MediaServer-v2-Device-20060531.pdf](https://upnp.org/specs/av/UPnP-av-MediaServer-v2-Device-20060531.pdf)).

**[STANDARD]** AVTransport on a MediaServer is conditional: *"The MediaServer contains a ContentDirectory Service, a ConnectionManager Service, and an optional AVTransport Service (depending on the supported transfer protocols)"* ([AVArchitecture-v1](https://upnp.org/specs/av/UPnP-av-AVArchitecture-v1-20020625.pdf), [AVArchitecture-v2](https://upnp.org/specs/av/UPnP-av-AVArchitecture-v2.pdf)).

**⇒ Minimum conforming DMS surface for PhairDLNA:**

| Item | Type | Mandatory? |
|---|---|---|
| `urn:schemas-upnp-org:device:MediaServer:1` | device type | **Yes** |
| `urn:schemas-upnp-org:service:ContentDirectory:1` | service | **Yes** |
| `urn:schemas-upnp-org:service:ConnectionManager:1` | service | **Yes** |
| `urn:schemas-upnp-org:service:AVTransport:1` | service | Optional (virtual instances per connection) |
| `urn:schemas-upnp-org:service:RenderingControl:1` | service | No (renderer-side) |
| `urn:schemas-upnp-org:service:ScheduledRecording:1` | service | No (AV:2 feature) |

### 1.2 ContentDirectory — exact actions

**[STANDARD]** Required actions (CD:1, table *"Actions"* in [UPnP-av-ContentDirectory-v1-Service.pdf](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v1-Service.pdf) and CD:4 §2.5):

| Action | Req./Opt. | Arguments (in → out) |
|---|---|---|
| `GetSearchCapabilities()` | **R** | → `SearchCaps` (CSV) |
| `GetSortCapabilities()` | **R** | → `SortCaps` (CSV) |
| `GetSystemUpdateID()` | **R** | → `Id` (ui4) |
| `Browse()` | **R** | `ObjectID`, `BrowseFlag`, `Filter`, `StartingIndex`, `RequestedCount`, `SortCriteria` → `Result`, `NumberReturned`, `TotalMatches`, `UpdateID` |
| `Search()` | O (CD:2+) | `ContainerID`, `SearchCriteria`, `Filter`, `StartingIndex`, `RequestedCount`, `SortCriteria` → `Result`, `NumberReturned`, `TotalMatches`, `UpdateID` |
| `CreateObject()` | O | `ContainerID`, `Elements` → `ObjectID`, `Result` |
| `DestroyObject()` | O | `ObjectID` → — |
| `UpdateObject()` | O | `ObjectID`, `CurrentTagValue`, `NewTagValue` → — |
| `MoveObject()` | O | — |
| `ImportResource()` / `ExportResource()` | O | `SourceURI`, `DestinationURI` → `TransferID` |
| `DeleteResource()` | O | `ResourceURI` → — |
| `StopTransferResource()` | O (CD:4 "CA") | `TransferID` → — |
| `GetTransferProgress()` | O (CD:4 "CA") | `TransferID` → `TransferStatus`, `TransferLength`, **`TransferTotal`** |
| `CreateReference()` | O | `ContainerID`, `ObjectID` → `NewID` |
| `GetSortExtensionCapabilities()` | CR (CD:4) | → `SortExtensionCaps` |
| `GetFeatureList()` | **R** (CD:4) | → `FeatureList` |
| `GetServiceResetToken()` | **R** (CD:4) | → `Token` |

State variables (CD:4): `SearchCapabilities` (R), `SortCapabilities` (R), `SortExtensionCapabilities` (CR), `SystemUpdateID` (R), `ContainerUpdateIDs` (A), `ServiceResetToken` (R), `LastChange` (CR), `TransferIDs` (CR), `FeatureList` (R), `DeviceMode` (CR), `DeviceModeStatus` (CR), `PermissionsInfo` (CR) — [UPnP-av-ContentDirectory-v4-Service.pdf](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf).

**[STANDARD]** `Browse()` output semantics (CD:1 wording):

> *NumberReturned*: "If BrowseMetadata is specified in the BrowseFlags, then NumberReturned = 1".
> *TotalMatches*: "If BrowseMetadata is specified in the BrowseFlags then TotalMatches = 1, else if BrowseDirectChildren is specified in the BrowseFlags then TotalMatches = total number of objects in the container specified for the Browse() action (independent of the starting index specified by the StartingIndex argument). If the CDS cannot compute TotalMatches and NumberReturned is not equal to zero, then TotalMatches = 0. If the CDS cannot compute TotalMatches and NumberReturned is equal to zero, then the CDS should return an error code 720."
> *UpdateID*: "ContainerUpdateID … of the container being described if a container is specified in ObjectID. … If the ObjectID is zero, then the UpdateID returned is SystemUpdateID."
> — [UPnP-av-ContentDirectory-v1-Service.pdf](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v1-Service.pdf)

CD:4 changes the `UpdateID` semantics slightly: *"The value returned in the UpdateID argument shall be the SystemUpdateID state variable value at the time the Browse() response was generated."* ([CD:4](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf)). CD:4 also keeps the *error 720* rule and the *TotalMatches = 0* fallback.

**[OBSERVED]** `GetSearchCapabilities()` in the wild returns e.g. `dc:title,dc:creator,dc:date,upnp:class,res@size` (the example in CD:4 §2.5.1).

**[ASSUMPTION]** PhairDLNA should implement the **CD:1 required set** plus `Search` and `GetSystemUpdateID` (the latter two are cheap and universally called), and expose `SearchCapabilities` = `dc:title,dc:creator,dc:date,upnp:class,res@size`, `SortCapabilities` = `+dc:title,+dc:date,+dc:creator,+upnp:class` (the `+` prefix means ascending, `-` descending).

### 1.3 ConnectionManager — exact actions

**[STANDARD]** From [UPnP-av-ConnectionManager-v1-Service.pdf](https://upnp.org/specs/av/UPnP-av-ConnectionManager-v1-Service.pdf):

| Action | Args (in → out) |
|---|---|
| `GetProtocolInfo()` | → `Source`, `Sink` (both CSV of ProtocolInfo) |
| `PrepareForConnection()` | `RemoteProtocolInfo`, `PeerConnectionManager`, `PeerConnectionID`, `Direction` → `ConnectionID`, `AVTransportID`, `RcsID` |
| `ConnectionComplete()` | `ConnectionID` → — |
| `GetCurrentConnectionIDs()` | → `ConnectionIDs` (CSV) |
| `GetCurrentConnectionInfo()` | `ConnectionID` → `RcsID`, `AVTransportID`, `ProtocolInfo`, `PeerConnectionManager`, `PeerConnectionID`, `Direction`, `Status` |

State variables: `SourceProtocolInfo`, `SinkProtocolInfo`, `CurrentConnectionIDs` (all evented), plus `A_ARG_TYPE_ConnectionStatus` (allowed: `OK`, `ContentFormatMismatch`, `InsufficientBandwidth`, `UnreliableChannel`, `Unknown`), `A_ARG_TYPE_Direction` (`Input`/`Output`), `A_ARG_TYPE_ConnectionID` / `A_ARG_TYPE_AVTransportID` / `A_ARG_TYPE_RcsID` (`i4`).

**[STANDARD]** *"The ConnectionManager service defines the notion of 'Protocol Info' as information needed by a control point in order to determine (a certain level of) compatibility between the streaming mechanisms of two UPnP controlled devices."* The ProtocolInfo string is four colon-separated fields: `<transportProtocol>:<network>:<contentFormat>:<additionalInfo>`.

**[STANDARD]** The Annex A table *"Actions for ConnectionManager:1 Service"* lists for the **device**: `GetProtocolInfo()` **R**, `PrepareForConnection()` O, `ConnectionComplete()` O, `GetCurrentConnectionIDs()` **R**, `GetCurrentConnectionInfo()` **R** ([AnnexA-20101006.pdf](https://upnp.org/specs/av/UPnP-av-ConnectionManager-v1-Service-AnnexA-20101006.pdf)).

**[STANDARD]** `Direction` values are `Input`/`Output`; `PrepareForConnection` *"identifies the ConnectionManager service on the other side of the connection"* via `PeerConnectionManager` (format: *"the UDN of the connecting UPnP device followed by a slash ('/') and the service ID"*).

**[ASSUMPTION]** For an HTTP-push DMS, `PrepareForConnection` should return a `ConnectionID` and `AVTransportID` (virtual AVTransport instance) when the control point asks for `Direction=Output`; many real control points never call it for pure `http-get` transfers. MiniDLNA/ReadyMedia mark it optional in their SCPDs (comment `//OPTIONAL {"PrepareForConnection", ...}` in [readymedia-transcode upnpdescgen.c](https://github.com/JuanKman94/readymedia-transcode/blob/transcode/upnpdescgen.c)).

### 1.4 AVTransport — exact actions (renderer-side; virtual on a DMS)

**[STANDARD]** From [UPnP-av-AVTransport-v3-Service.pdf](https://upnp.org/specs/av/UPnP-av-AVTransport-v3-Service.pdf) and the AV:1 template:

| Action | Req./Opt. |
|---|---|
| `SetAVTransportURI()` | **R** |
| `SetNextAVTransportURI()` | O |
| `GetMediaInfo()` | **R** |
| `GetTransportInfo()` | **R** |
| `GetPositionInfo()` | **R** |
| `GetDeviceCapabilities()` | **R** |
| `GetTransportSettings()` | **R** |
| `Stop()` | **R** |
| `Play()` | **R** |
| `Pause()` | O |
| `Record()` | O |
| `Seek()` | **R** |
| `Next()` | **R** |
| `Previous()` | **R** |
| `SetPlayMode()` | O |
| `SetRecordQualityMode()` | O |
| `GetCurrentTransportActions()` | O |

`TransportState` allowed values: `STOPPED`, `PLAYING`, `PAUSED_PLAYBACK`, `RECORDING`, `TRANSITIONING`, `NO_MEDIA_PRESENT`.

**[STANDARD]** `CurrentTransportActions` *"shall contain a subset (including the empty set) of the following action names: 'Play', 'Stop', 'Pause', 'Seek', 'Next', 'Previous' and 'Record'"*; the state variable is *"conditionally required … if the AVTransport service implements the GetCurrentTransportActions() action, otherwise it is not allowed."*

### 1.5 RenderingControl — exact actions (renderer-side)

**[STANDARD]** `ListChannels`, `GetVolume`, `SetVolume`, `GetMute`, `SetMute` (plus `GetVolumeDB`, `SetVolumeDB`, `GetPresetNameList`, `SelectPreset`, `GetSnapshotTransfer`, `GetScreenSaverStatus`, `SetScreenSaverStatus`). Arguments are `InstanceID` + `Channel`; `DesiredVolume` is `ui2` (0–100).

**[STANDARD]** The ConnectionManager `PrepareForConnection` on the **renderer** returns an `RcsID` — the RenderingControl instance to use for volume control during that connection.

### 1.6 GENA eventing

**[STANDARD]** From [UPnP-arch-DeviceArchitecture-v1.1.pdf](https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.1.pdf) and [v2.0](https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0.pdf):

* **SUBSCRIBE**: `SUBSCRIBE *publisher* *path* HTTP/1.1` with `HOST`, `CALLBACK: <delivery URL>`, `NT: upnp:event`, `TIMEOUT: Second-<requested duration>` (Recommended). Response carries `SID: uuid:<subscription-UUID>` and `TIMEOUT: Second-<actual duration>` (REQUIRED; *"SHOULD be greater than or equal to 1800 seconds (30 minutes)"*).
* **UNSUBSCRIBE**: `UNSUBSCRIBE *publisher* *path* HTTP/1.1` with `HOST` and `SID`.
* **NOTIFY** (publisher → subscriber): `NOTIFY *delivery* *path* HTTP/1.1` with `HOST`, `CONTENT-TYPE: text/xml`, `CONTENT-LENGTH`, `NT: upnp:event`, `NTS: upnp:propchange`, `SID: uuid:<subscription-UUID>`, `SEQ: <event key>`.
* **SEQ**: *"Must be 0 for initial event message. Must be incremented by 1 for each event message sent to a particular subscriber."*
* **Initial event**: *"A special initial event message is sent when a subscriber first subscribes; this event message contains the names and values for all evented variables and allows the subscriber to initialize its model of the state of the service."*
* **Renewal**: re-SUBSCRIBE with the existing `SID` before `TIMEOUT` elapses.
* Body is a GENA property set: `<e:propertyset xmlns:e="urn:schemas-upnp-org:event-1-0">` containing `<e:property>` elements.

**[STANDARD]** In UPnP AV, the evented payload for AVTransport/RenderingControl is the **`LastChange`** variable holding a nested XML document (`<Event xmlns="urn:schemas-upnp-org:metadata-1-0/AVT/">` → `<InstanceID val="0">` → `<AVTransportURI val="…"/>`, `<CurrentTrackMetaData val="…"/>` …). Observed in the field (gmediarender log excerpt, [dietpi forum](https://dietpi.com/forum/t/going-round-in-circles-with-dlna-jellyfin-and-gmediarender/22649)).

**[STANDARD]** ContentDirectory's `LastChange` is **conditionally required** in CD:4; the `FeatureList`/`LastChange` XML schema for CDS LastChange is a separate spec.

**[ASSUMPTION]** PhairDLNA must implement GENA for at least `ConnectionManager` (`SourceProtocolInfo`, `SinkProtocolInfo`, `CurrentConnectionIDs`) and `ContentDirectory` (`SystemUpdateID`, `LastChange`). `ContentDirectory::LastChange` is *conditionally* required — but many DLNA DMRs never subscribe to CDS. **Implement GENA SUBSCRIBE/UNSUBSCRIBE/NOTIFY regardless**: the `eventSubURL` must exist and be functional, or DLNA-certified tools flag the device.

### 1.7 SOAP control

**[STANDARD]** SOAP 1.1 with `SOAPAction: "urn:schemas-upnp-org:service:ContentDirectory:1#Browse"`, `Content-Type: text/xml; charset="utf-8"`, `Content-Length` (never chunked request bodies) — see [audiosync DLNA_SPEC.md](https://github.com/insentodesu/audiosync/blob/main/docs/DLNA_SPEC.md) (secondary but explicit) and observed rclone/MiniDLNA traffic.

**[OBSERVED]** Real control points send `Content-Length` on SOAP POSTs and `Connection: close` is common on the server side.

---

## 2. SSDP

Primary source: [UPnP-arch-DeviceArchitecture-v1.1.pdf](https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.1.pdf); IPv6 + UDA 2.0 from [v2.0](https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0.pdf).

### 2.1 Addresses and ports

**[STANDARD]** IPv4 multicast group is **`239.255.255.250`, UDP port 1900** (*"Field value contains multicast address and port reserved for SSDP by IANA. MUST be 239.255.255.250:1900."*).
**[STANDARD]** IPv6: IANA registered SSDP as `FF0X::C`; *"A device advertising on the local link would use a scope of 2 and address [FF02::C]:1900. A site-scope advertisement on the home network would use scope 5 and specify address [FF05::C]:1900."* (UDA 2.0)
**[STANDARD]** UDA 2.0, IPv6 rules: *"Devices and control points SHALL use only the Link-Local unicast address as the source address and when specifying a literal IP address in LOCATION URLs in all multicast messages that are multicast to the Link-Local scope FF02::C for SSDP and FF02::130 for multicast eventing. Devices and control points SHALL listen to the Link-Local scope."*
**[STANDARD]** *"SSDP announcements SHALL be sent to [FF0X::C]:1900 (with 'X' being either 2 or 5 …)"*; control points listen on those addresses/ports.
**[STANDARD]** IPv4 multicast TTL *"SHOULD default to 2 and SHOULD be configurable"* (UDA 2.0, as quoted in [miniupnpd minissdp.c](https://github.com/miniupnp/miniupnp/blob/miniupnpd_2_3_11/miniupnpd/minissdp.c)); site-scope IPv6 hop limit *"SHOULD default to 10"*.

**[OPEN]** The task's literal `ssdp://239.255.255.250:1900` URI form did not appear in the UDA text I retrieved. The spec writes the endpoint as `239.255.255.250:1900` (HOST header). The `ssdp://` URI form appears in some ecosystem docs (e.g. `ssdp://` as a scheme prefix) but I could not tie it to normative UDA text. Treat the *endpoint* as normative, the *URI spelling* as cosmetic.

### 2.2 M-SEARCH (control point → server)

**[STANDARD]** Request format:

```
M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: <seconds>
ST: <search target>
USER-AGENT: OS/version UPnP/1.1 product/version
```

* **[STANDARD]** `MAN` value **must** be `"ssdp:discover"` (quoted).
* **[STANDARD]** `MX`: *"MUST be greater than or equal to 1 and SHOULD be less than 5 inclusive. Device responses SHOULD be delayed a random duration between 0 and this many seconds to balance load for the control point when it processes responses. This value MAY be increased if a large number of devices are expected to respond."* (UDA 1.1)
* **[STANDARD]** *"[Devices] MAY assume an MX field value less than that specified in the MX header field. If the MX header field specifies a field value greater than 5, the device SHOULD assume that it contained the value 5 or less."* (UDA 1.1, quoted by [Pelco's UPnP search doc](https://support.pelco.com/s/article/How-a-UPnP-Search-works-1538586696122) and [miniupnpd](https://github.com/miniupnp/miniupnp/blob/miniupnpd_2_3_11/miniupnpd/minissdp.c))
* **[STANDARD]** *"If a request does not contain an MX header field, the device MUST silently discard and ignore the search request."* (UDA 1.1, quoted in miniupnpd source.)
* **[STANDARD]** *"When a device receives a unicast M-SEARCH, it SHOULD respond within 1 second and it MAY respond sooner."*

### 2.3 ST matching semantics

**[STANDARD]** `ST` in a search request **MUST be one of**: `ssdp:all`, `upnp:rootdevice`, `uuid:<device-UUID>`, `urn:<domain>:device:<deviceType>:<version>`, `urn:<domain>:service:<serviceType>:<version>`.

**[STANDARD]** Response counts by ST (UDA 1.1):

| ST in M-SEARCH | Required responses |
|---|---|
| `ssdp:all` | *"Respond 3+2*d+*k* times for a root device with *d* embedded devices and *s* embedded services but only *k* distinct service types"* |
| `upnp:rootdevice` | *"Respond once for root device. Must be upnp:rootdevice."* |
| `uuid:<device-UUID>` | *"Respond once for each matching device, root or embedded."* |
| `urn:<domain>:device:<deviceType>:v` | *"Respond once for each matching device, root or embedded. Should specify the version of the device type contained in the M-SEARCH request."* |
| `urn:<domain>:service:<serviceType>:v` | *"Respond once for each matching service type. Should specify the version of the service type contained in the M-SEARCH request."* |

**[STANDARD]** *"Devices respond if the ST header field of the M-SEARCH request is 'ssdp:all', 'upnp:rootdevice', 'uuid:' followed by a UUID that exactly matches the one advertised by the device, or if the M-SEARCH request matches a device type or service type supported by the device."*

**⇒ For a MediaServer, `ssdp:all` therefore obliges a response for:** the root device, the `MediaServer:1` device type, the `ContentDirectory:1` service type, the `ConnectionManager:1` service type — and the root-device/`upnp:rootdevice` advertisement. Concretely, the standard MediaServer advertises **4–5 distinct targets**:

```
NOTIFY * HTTP/1.1
HOST: 239.255.255.250:1900
CACHE-CONTROL: max-age=1800
LOCATION: http://<ip>:<port>/description.xml
NT: <target>
NTS: ssdp:alive
SERVER: <OS>/<version> UPnP/1.1 <product>/<version>
USN: <usn>
```

with targets/USNs:

| NT | USN |
|---|---|
| `upnp:rootdevice` | `uuid:<UDN>::upnp:rootdevice` |
| `urn:schemas-upnp-org:device:MediaServer:1` | `uuid:<UDN>::urn:schemas-upnp-org:device:MediaServer:1` |
| `urn:schemas-upnp-org:service:ContentDirectory:1` | `uuid:<UDN>::urn:schemas-upnp-org:service:ContentDirectory:1` |
| `urn:schemas-upnp-org:service:ConnectionManager:1` | `uuid:<UDN>::urn:schemas-upnp-org:service:ConnectionManager:1` |
| `uuid:<UDN>` | `uuid:<UDN>` |

**[STANDARD]** USN forms (UDA 1.1): *"It MUST be one of the following forms. The prefix (before the double colon) MUST match the value of the UDN element in the device description."* — `uuid:<UUID>::upnp:rootdevice`, `uuid:<UUID>::urn:schemas-upnp-org:device:<deviceType>:<version>`, `uuid:<UUID>::urn:schemas-upnp-org:service:<serviceType>:<version>`, and the bare `uuid:<UUID>`.

### 2.4 M-SEARCH response

**[STANDARD]** Response is **unicast UDP to the source IP address and port of the requester**:

> *"To be found by a network search, a device MUST send a unicast UDP response to the source IP address and port that sent the request to the multicast address."*
> *"— [UDA 1.1](https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.1.pdf)"*

**Why unicast, technically (ASSUMPTION):** the SSDP group is a *discovery broadcast*; a unicast reply keeps the response off the LAN-wide multicast domain, lets the reply carry the responder's per-interface address, and prevents N devices × M listeners from producing a multicast storm. UDA 1.1 codifies it as a MUST.

**[STANDARD]** Response headers:

```
HTTP/1.1 200 OK
CACHE-CONTROL: max-age=<seconds>
DATE: <when response was generated>
EXT:
LOCATION: <URL for UPnP description for root device>
SERVER: OS/version UPnP/1.1 product/version
ST: <search target>
USN: <composite identifier>
```

* **[STANDARD]** `CACHE-CONTROL` *"MUST have the max-age directive ('max-age=') followed by an integer that specifies the number of seconds the advertisement is valid."* The max-age value *"SHOULD be greater than or equal to 1800 seconds (30 minutes)"*.
* **[STANDARD]** `LOCATION` *"MUST specify an address that is reachable on that interface"*; and *"The URL specified in the LOCATION header of the M-SEARCH response must be reachable by the control point to which the response is directed."*
* **[STANDARD]** `SERVER` format: `OS/version UPnP/1.1 product/version` (UDA 1.1) / `UPnP/2.0` (UDA 2.0).
* **[STANDARD]** `ST` in the response *"must be the same as for the NT header in NOTIFY messages with ssdp:alive"* for `ssdp:all`.
* **[STANDARD]** Multi-homed: *"Multi-homed devices MUST send the search response using the same UPnP-enabled interface on which the search request was received."*

**[ASSUMPTION]** Response delay: pick a uniform random delay in `[0, MX)` (UDA says "between 0 and this many seconds"). Cap at 4 s when `MX` is absent or > 5 (treat as 5).

### 2.5 NOTIFY advertisement

**[STANDARD]** `NOTIFY * HTTP/1.1` with `HOST: 239.255.255.250:1900`, `CACHE-CONTROL: max-age=…`, `LOCATION: …`, `NT: …`, `NTS: ssdp:alive`, `SERVER: …`, `USN: …`. `ssdp:byebye` is sent on withdrawal; UDA 2.0's example byebye carries only `HOST`, `NT`, `NTS: ssdp:byebye` (+ `USN`).

**[STANDARD]** Advertisement cadence: UDA states the advertisement is valid for `max-age` seconds and control points assume the device is gone afterwards. Common practice (and the value used by MiniDLNA/rclone/miniupnpd) is `max-age=1800` with re-advertisement at roughly `max-age/3…/4`.

**[OBSERVED]** MiniDLNA/rclone/ReadyMedia all use `CACHE-CONTROL: max-age=1800`; miniupnpd's comment: *"The cycle is anchored to the device's CACHE-CONTROL: max-age=1800 advertisement; sending one notification per max-age/4 (~7 minutes) is the conservative recommendation in the UPnP arch spec."* ([WaveFlow dlna.md](https://github.com/InstaZDLL/WaveFlow/blob/main/docs/features/dlna.md), [upnp-explorer ssdp cheat sheet](https://github.com/cvdlinden/upnp-explorer/blob/main/docs/ssdp-protocol-cheat-sheet.md)).

**[OBSERVED]** UDA 2.0 adds `BOOTID.UPNP.ORG` (incremented on each initial announce or update) and `OPT: "http://schemas.upnp.org/upnp/1/0/"; ns=01` / `01-NLS:` headers.

**[ASSUMPTION]** PhairDLNA should advertise on **every UPnP-enabled interface**, each with its own `LOCATION` (interface-local address), because the LOCATION must be reachable by the requesting control point.

### 2.6 IPv6

**[STANDARD]** Join `FF02::C` on UDP/1900 (and `FF05::C` for site scope), source from link-local addresses, and put a **link-local** literal address in `LOCATION` for link-scoped SSDP. Listen for GENA multicast events on `FF02::130`.

**[ASSUMPTION]** macOS specifics: the sandbox/CI environment may not have IPv6 link-local on the LAN interface. Implement IPv6 SSDP behind a feature flag; IPv4 is the interoperability-critical path.

### 2.7 Why mDNS does not replace SSDP for DLNA

**[STANDARD]** RFC 6762 (mDNS) *"provides the ability to perform DNS-like operations on the local link in the absence of any conventional Unicast DNS server. In addition, Multicast DNS designates a portion of the DNS namespace to be free for local use."* It resolves **names** to addresses/services.

**[STANDARD]** RFC 6762 §3: *"This document specifies that the DNS top-level domain '.local.' is a [special-use domain] … Any DNS query for a name ending in '.local.' MUST be sent to the [mDNS] multicast group"*, and *"names within this [domain] are link-local"*.

**[STANDARD]** RFC 6761 (*Special-Use Domain Names*) defines the **procedure and IANA registry** by which such names get reserved (and reserves `localhost.`, `invalid.`, `test.`, `example.*`).

**[STANDARD/negative evidence]** Grepping the full text of RFC 6762 for `ssdp` and `upnp` returns **no occurrences**. mDNS is a name-resolution protocol; SSDP is a *service-advertisement and capability-discovery* protocol that also carries the `LOCATION` pointer to the device description and the `USN`/`ST` identity model. DLNA's device classes are defined in terms of SSDP discovery + UPnP descriptions, not mDNS.

**⇒ Conclusion:** a DLNA DMS **must** implement SSDP. Adding mDNS (`.local.` name advertisement for the device) is a *nice-to-have* extra (AirPlay/Google Cast style), not a substitute.

---

## 3. Device description

### 3.1 Required XML structure

**[STANDARD]** From the MediaServer device templates ([v1](https://upnp.org/specs/av/UPnP-av-MediaServer-v1-Device.pdf), [v2](https://upnp.org/specs/av/UPnP-av-MediaServer-v2-Device-20060531.pdf), [v4-20101231](https://upnp.org/specs/av/UPnP-av-MediaServer-v4-Device-20101231.pdf)):

```xml
<?xml version="1.0"?>
<root xmlns="urn:schemas-upnp-org:device-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <device>
    <deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>
    <friendlyName>short user-friendly title</friendlyName>
    <manufacturer>manufacturer name</manufacturer>
    <manufacturerURL>URL to manufacturer site</manufacturerURL>
    <modelDescription>long user-friendly title</modelDescription>
    <modelName>model name</modelName>
    <modelNumber>model number</modelNumber>
    <modelURL>URL to model site</modelURL>
    <serialNumber>manufacturer's serial number</serialNumber>
    <UDN>uuid:UUID</UDN>
    <UPC>Universal Product Code</UPC>          <!-- optional -->
    <iconList>
      <icon><mimetype>image/png</mimetype><width>48</width><height>48</height>
            <depth>8</depth><url>icon.png</url></icon>
    </iconList>
    <serviceList>
      <service>
        <serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId>
        <SCPDURL>URL to service description</SCPDURL>
        <controlURL>URL for control</controlURL>
        <eventSubURL>URL for eventing</eventSubURL>
      </service>
      <service>
        <serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId>
        <SCPDURL>URL to service description</SCPDURL>
        <controlURL>URL for control</controlURL>
        <eventSubURL>URL for eventing</eventSubURL>
      </service>
    </serviceList>
    <presentationURL>URL for presentation</presentationURL>
  </device>
</root>
```

**[STANDARD]** The MediaServer:4 template is identical in shape but with `urn:schemas-upnp-org:device:MediaServer:4` and `urn:schemas-upnp-org:service:ContentDirectory:4`.

**[OBSERVED]** The `serviceId` for ConnectionManager is spelled `urn:upnp-org:serviceId:ConnectionManager` in every implementation I read (MiniDLNA, ReadyMedia, rclone, DLNA test tools) — note the spec template for MediaServer:1 renders it as `ConnectorManager` (an evident typo in the OCF template), while the OCF Annex and every implementation use `ConnectionManager`. Use `urn:upnp-org:serviceId:ConnectionManager`.

**[OBSERVED]** Windows' hosted-device model documents `controlURL` and `eventSubURL` as **required** and `URLBase` as forbidden; the UDA `URLBase` element is optional and, when present, supplies the base for relative URLs ([MS docs: Creating a Device Description](https://learn.microsoft.com/en-us/windows/win32/upnp/creating-a-device-description)).

### 3.2 URL rules

**[STANDARD]** The template comments `URLBase` as *"base URL for all relative URLs"*. Both absolute URLs and root-relative URLs (`/ctl/ContentDir`) are used in practice.

**[OBSERVED]** MiniDLNA/ReadyMedia/rclone use root-relative paths (`/ContentDirectory.xml`, `/ctl`, `/evt/...`) and **do not** emit `URLBase`; rclone's live description (captured in a [rclone forum log](https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480/3)) shows `SCPDURL` = `/static/ContentDirectory.xml`, `controlURL` = `/ctl`, `eventSubURL` = *(empty)*.

**[ASSUMPTION]** PhairDLNA: emit **absolute** `SCPDURL`/`controlURL`/`eventSubURL` (or root-relative plus a `URLBase`), and always emit a **non-empty `eventSubURL`** even if the GENA implementation is thin — some DLNA certification tools treat an empty `eventSubURL` as a missing capability.

**[OBSERVED]** `specVersion` should be `1.0` (UDA 1.0/1.1) — every real device description uses `<major>1</major><minor>0</minor>`.

### 3.3 The `Browse` response structure

**[STANDARD]** The SOAP response body is:

```xml
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:BrowseResponse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
      <Result>&lt;DIDL-Lite …&gt;…&lt;/DIDL-Lite&gt;</Result>
      <NumberReturned>3</NumberReturned>
      <TotalMatches>17</TotalMatches>
      <UpdateID>42</UpdateID>
    </u:BrowseResponse>
  </s:Body>
</s:Envelope>
```

* `Result` = the DIDL-Lite document, **XML-escaped** as text inside the element (double-escaped relative to the DIDL-Lite document itself).
* `NumberReturned`, `TotalMatches` = `ui4`.
* `UpdateID` = `ui4` (SystemUpdateID or ContainerUpdateID — §1.2).

**[OBSERVED]** Live example (rclone log, same forum thread): `<Result>&lt;DIDL-Lite xmlns:dc=&#34;…&#34; …&gt; … &lt;/DIDL-Lite&gt;</Result><NumberReturned>3</NumberReturned>` — note `&#34;` for `"` inside the escaped payload, i.e. the control point must unescape twice.

---

## 4. DIDL-Lite

### 4.1 Namespaces

| Prefix | URI | Source |
|---|---|---|
| (default) | `urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/` | **[STANDARD]** CD:4 "Default Namespaces for the AV Specifications": ContentDirectory → `didl-lite` |
| `dc` | `http://purl.org/dc/elements/1.1/` | **[STANDARD]** |
| `upnp` | `urn:schemas-upnp-org:metadata-1-0/upnp/` | **[STANDARD]** |
| `dlna` | `urn:schemas-dlna-org:metadata-1-0/` | **[OBSERVED]** DLNA metadata ns (used by DLNA, DLNA-compliant tools) |
| `sec` | `http://www.sec.co.kr/dlna` (also seen: `http://www.sec.co.kr/`) | **[OBSERVED]** Samsung extension ns; `sec:CaptionInfo`, `sec:CaptionInfoEx`, `sec:dcmInfo`, `sec:ProductCap` |

**[OBSERVED]** Namespace declarations must be **present** for every prefix used. ReadyMedia bug #316: it emitted `sec:` elements without declaring `xmlns:sec`, and *"the parser used in Cling is strict and rejects this as invalid XML"* ([sourceforge](https://sourceforge.net/p/minidlna/bugs/316/)). **⇒ Always declare every namespace you use.**

**[OBSERVED]** rclone emits `xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/"` and `xmlns:sec="http://www.sec.co.kr/"` on the DIDL-Lite root ([rclone dlna_util.go](https://github.com/rclone/rclone/blob/0c9826d3/cmd/serve/dlna/dlna_util.go)).

### 4.2 `<item>` and `<container>` attributes

**[STANDARD/secondary-schema]** The DIDL-Lite XSD (as published by Cling, quoting the CD spec) states:

* `<item>`: `id` (string, **required**), `parentID` (string, **required**), `restricted` (boolean, **required**), `refID` (string, optional). Text: *"Each item is additionally required to specify a value for the 'id' attribute. If the item is actually a reference to another item, a value for 'refID' is specified. Each item is also required to specify a value for the 'parentID attribute, as well as the 'restricted' attribute (true, false, 1, 0). When restricted='true', the ability to change or delete the item is restricted."*
* `<container>`: `id` (**required**), `parentID` (**required**), `restricted` (**required**), `childCount` (int, optional), `searchable` (boolean, optional).
* Source: [cling didl-lite-1.0.xsd](https://github.com/4thline/cling/blob/master/support/src/main/resources/org/fourthline/cling/support/contentdirectory/descriptor/didl-lite-1.0.xsd) (secondary but a faithful transcription of the CD:1 DIDL-Lite schema).

**[STANDARD]** Required child elements (per the CD:1 DIDL-Lite property table, quoted in [audiosync DLNA_SPEC.md](https://github.com/insentodesu/audiosync/blob/main/docs/DLNA_SPEC.md)): `id`, `parentID`, `title` (`dc:title`), `class` (`upnp:class`), `restricted`.

**[OBSERVED]** `searchable="0"` / `searchable="1"` appear on `<container>` in the wild (rclone emits `searchable="0"` on items; openHAB's Twonky capture shows `<container … restricted="1" childCount="5" searchable="1">`).

**[OPEN]** `writeStatus` — I could not open the CD:4 Appendix-B row that defines it. In the UPnP metadata model it appears as a **property** (in the `upnp:` namespace) rather than an XML attribute of `<item>`/`<container>`. **Do not emit `writeStatus` as an attribute until verified.** Logged in §9.

### 4.3 `upnp:class` values

**[STANDARD]** From the CD:4 class table ([UPnP-av-ContentDirectory-v4-Service.pdf](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf)) — all "A" (allowed):

```
object.item
object.item.imageItem            object.item.imageItem.photo
object.item.audioItem            object.item.audioItem.musicTrack
object.item.audioItem.audioBroadcast
object.item.audioItem.audioBook
object.item.videoItem            object.item.videoItem.movie
object.item.videoItem.videoBroadcast
object.item.videoItem.musicVideoClip
object.item.playlistItem         object.item.textItem
object.item.bookmarkItem         object.item.epgItem
object.item.epgItem.audioProgram
object.item.epgItem.videoProgram
object.container                 object.container.person
object.container.person.musicArtist
object.container.playlistContainer
object.container.album           object.container.album.musicAlbum
object.container.album.photoAlbum
object.container.genre           object.container.genre.musicGenre
object.container.genre.movieGenre
object.container.channelGroup    object.container.channelGroup.audioChannelGroup
object.container.channelGroup.videoChannelGroup
object.container.epgContainer
object.container.storageSystem    object.container.storageVolume
object.container.storageFolder
object.container.bookmarkFolder
Vendor-defined                    (X)
```

**⇒ Mapping for PhairDLNA's library:** audio → `object.item.audioItem.musicTrack`; video file → `object.item.videoItem.movie`; live/streamed video → `object.item.videoItem.videoBroadcast`; photo → `object.item.imageItem.photo`; folder → `object.container.storageFolder`; artist → `object.container.person` (or `object.container.person.musicArtist`).

**[OBSERVED]** rclone maps a plain video file to `object.item.videoItem` (not `.movie`); the DLNA `DIDL_V`/`DIDL_S` collection profiles require "Complies with the DIDL-Lite schema" and restrict allowed tags ([DLNA Part 2 copy](https://www.scribd.com/document/803046323/DLNA-Guidelines-June-2016-Part-2-Media-Format-Profiles), secondary).

### 4.4 `res` element and attributes

**[STANDARD]** `<res>` carries the resource URI as text and these attributes (CD:4 Appendix B table):

`res` (URI), `res@protocolInfo` (**required**), `res@importUri`, `res@size` (unsignedLong), `res@duration` (string), `res@protection` (string), `res@bitrate` (unsignedInt), `res@bitsPerSample`, `res@sampleFrequency`, `res@nrAudioChannels`, `res@resolution` (pattern `[0-9]+x[0-9]+`), `res@colorDepth`, `res@tspec`, `res@allowedUse`, `res@validityStart`, `res@validityEnd`, `res@remainingTime`, `res@usageInfo`, `res@rightsInfoURI`, `res@contentInfoURI`, `res@recordQuality`, `res@daylightSaving`, `res@framerate`.

**[STANDARD]** Duration format: *"H*:MM:SS.F*, or H*:MM:SS.F0/F1 … H* means any number of digits (including no digits) to indicate elapsed hours, MM exactly 2 digits (00–59), SS exactly 2 digits (00–59), F* any number of digits to indicate fractions of seconds."* (DIDL-Lite XSD doc, [cling](https://github.com/4thline/cling/blob/master/support/src/main/resources/org/fourthline/cling/support/contentdirectory/descriptor/didl-lite-1.0.xsd)).

**[STANDARD]** *"The 'bitrate' attribute — The bitrate in bytes/second of the resource."* Same source. **[OBSERVED]** Some servers emit bits/s; `bitrate="705625"` next to `sampleFrequency="2822400"` suggests a units mismatch in the wild ([supersonic issue #931](https://github.com/supersonic-app/supersonic/issues/931)). **Use bytes/second** per the schema; it is also what DLNA expects.

**[STANDARD]** `res@protocolInfo` is `<transportProtocol>:<network>:<contentFormat>:<additionalInfo>`, e.g. `http-get:*:video/mp4:*`.

### 4.5 `upnp:albumArtURI`, `dc:*`, `upnp:artist/genre/album/date`

**[STANDARD]** `upnp:albumArtURI` is an optional UPnP-metadata element containing the album-art URL.

**[OBSERVED]** DLNA convention adds a `dlna:profileID` attribute to the album-art URI, e.g. `<upnp:albumArtURI dlna:profileID="JPEG_TN" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/">…</upnp:albumArtURI>` ([MiniDLNA upnpsoap.c](https://github.com/keenetic/minidlna/blob/master/upnpsoap.c); [Gerbera issue #2894](https://github.com/gerbera/gerbera/issues/2894)). **[STANDARD, secondary]** Microsoft's WMC-compat guidance says: *"The dlna:profileID and xmlns:dlna attributes MUST NOT be included in the upnp:albumArtURI nodes"* ([MS-UPMC](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/c12f5530-fa2d-4d9b-a960-18f4e9678100)) — i.e. Windows Media Center chokes on them. **⇒ Recommendation: emit a plain `upnp:albumArtURI` with no `dlna:` attributes.**

**[STANDARD/secondary]** `[MS-DLNHND]` quotes the DLNA property vocabulary; the common ones observed in real DIDL-Lite:

* `dc:title` — human-readable name. **Required.**
* `dc:creator` — creator/artist (Dublin Core).
* `upnp:artist` — artist.
* `upnp:album` — album.
* `upnp:genre` — genre.
* `upnp:originalTrackNumber` — track number.
* `dc:date` — date. **[OBSERVED]** Real-world values are inconsistent: `2024-05-28T02:28:47` (QNAP), `2022-05-26T16:22:39+0000` (Gerbera), `1996-01-01` (Mediatomb/Gerbera), `2012-01-01` (Twonky). **Emit ISO-8601 `YYYY-MM-DD` or `YYYY-MM-DDThh:mm:ss`.**

**[OPEN]** Exact type/format constraints on `dc:date`, `upnp:artist`, `upnp:genre`, `upnp:album` in CD:4 Appendix B were not opened directly. Treat as free-form UTF-8 strings; do not invent a format. Logged in §9.

### 4.6 Escaping and XML entities

**[STANDARD]** XML 1.0: *"The ampersand character (&) and the left angle bracket (<) may appear in their literal form only when used as markup delimiters … If they are needed elsewhere, they must be escaped using either numeric character references or the strings '&amp;' and '&lt;' … The right angle bracket (>) may be represented using the string '&gt;' … the double-quote (") … '&quot;'"* ([W3C XML 1.0](https://www.w3.org/TR/1998/REC-xml-19980210)).

**[ASSUMPTION / implementation rule]**

1. Escape element text: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`.
2. Escape attribute values: `&` → `&amp;`, `<` → `&lt;`, `"` → `&quot;`, `'` → `&apos;`.
3. Because the DIDL-Lite document is carried as **text inside `<Result>`**, escape the DIDL-Lite document **a second time** when serialising the SOAP body (`<` → `&lt;`, `&` → `&amp;`, `"` → `&quot;`). A control point must unescape twice.

**[OBSERVED]** Real-world evidence of the double-escaping: rclone's live response contains `&lt;` and `&#34;`; Sonos's `LastChange` embeds the DIDL-Lite again (`val="&lt;DIDL-Lite …"`).

**[OBSERVED]** Control points unescape defensively: `clement87/sonos` replaces `&lt;`, `&gt;`, `&quot;`, `&amp;` with their literals when extracting metadata ([sonos.class.php](https://github.com/Clement87/sonos/blob/master/sonos.class.php)).

**[OBSERVED]** Some renderers (Samsung) are reportedly unhappy with `&#34;` — a Go implementation patches `&#34;` back to `"` before sending `SetAVTransportURI` ([dlna-home soapbuilders.go](https://github.com/Tzxhy/dlna-home/blob/master/soapcalls/soapbuilders.go)). **[ASSUMPTION]** Emit `"` inside element text unescaped where legal, and prefer `&quot;` only inside attributes — do not emit numeric character references for `"` in text.

**[OBSERVED]** Control points truncate/limit: DLNA's client implementation guide asks clients to support *"DLNA element values and attribute values … that are a maximum size of 8192 bytes each, in their XML-escaped form, encoded in UTF-8"* ([Comcast DLNA CIG](https://media2.comcast.net/anon.comcastonline2/support/help/faqs/boxee/Comcast-SP-HDuDTA20-DLNA-CIG-I01Public-130314Issued.pdf), secondary). **⇒ Keep individual metadata values well under 8 KB.**

---

## 5. HTTP delivery semantics for a MediaServer

Primary: RFC 9110 (downloaded verbatim), RFC 9112 (downloaded verbatim).

### 5.1 Methods and status codes

**[STANDARD]** RFC 9110 §14.2: *"The 'Range' header field on a GET request modifies the method semantics to request transfer of only one or more subranges of the selected representation data."* — **range handling is defined for GET only**: *"A server MUST ignore a Range header field received with a request method that is unrecognized or for which range handling is not defined."*

**[STANDARD]** RFC 9110 §14.2: *"If all of the preconditions are true … and that ranges-specifier is satisfiable … the server SHOULD send a 206 (Partial Content) response with content containing one or more partial representations."* and *"… and either the range-unit is not supported for that target resource or the ranges-specifier is unsatisfiable … the server SHOULD send a 416 (Range Not Satisfiable) response."*

**[STANDARD]** RFC 9110 §15.5.17: *"A server that generates a 416 response to a byte-range request SHOULD generate a Content-Range header field specifying the current length of the selected representation"*, e.g. `Content-Range: bytes */47022`. Also: *"Because servers are free to ignore Range, many implementations will respond with the entire selected representation in a 200 (OK) response … clients cannot depend on receiving a 416 (Range Not Satisfiable) response even when it is most appropriate."*

**[STANDARD]** RFC 9110 §14.4 (`Content-Range`): `Content-Range = range-unit SP ( range-resp / unsatisfied-range )`, `range-resp = incl-range "/" ( complete-length / "*" )`, `incl-range = first-pos "-" last-pos`, `unsatisfied-range = "*/" complete-length`.

**[STANDARD]** RFC 9110 §14.3 (`Accept-Ranges`): *"a server that supports byte-range requests … can send the field `Accept-Ranges: bytes`"*. *"A client MAY generate range requests regardless of having received an Accept-Ranges field."*

**[OBSERVED]** MiniDLNA sends `Accept-Ranges: bytes` on media responses and implements **only single-range** `Range: bytes=start-end` (parsed by hand in `upnphttp.c`), answering **416** for unsatisfiable ranges (`Send416()` in the local `minidlna_upnphttp.c`, lines 535–545, 2054–2087) and `206` + `Content-Range: bytes a-b/total` otherwise. It does **not** implement `multipart/byteranges`.

**[STANDARD]** RFC 9110 §14.2: *"A server that supports range requests MAY ignore or reject a Range header field that contains an invalid ranges-specifier, a ranges-specifier with more than two overlapping ranges, or a set of many small ranges that are not listed in ascending order, since these are indications of either a broken client or a deliberate denial-of-service attack."*

**⇒ Implementation rule (NORMATIVE + ASSUMPTION):** support **one** byte range; for multiple ranges either serve the first satisfiable range as a single `206`, or ignore `Range` entirely and return `200`. Do **not** implement `multipart/byteranges`. Reject/ignore absurd range sets (RFC 9110 §17.15 DoS guidance).

### 5.2 Headers to emit on a media response

| Header | Value | Tag |
|---|---|---|
| `Content-Type` | The MIME type that matches `res@protocolInfo` field 3 | **[STANDARD]** RFC 9110 §8.3 |
| `Content-Length` | Byte length of the served body | **[STANDARD]** RFC 9110 §8.6 |
| `Accept-Ranges` | `bytes` | **[STANDARD]** RFC 9110 §14.3 |
| `Content-Range` | `bytes <first>-<last>/<total>` on 206; `bytes */<total>` on 416 | **[STANDARD]** RFC 9110 §14.4, §15.5.17 |
| `Connection` | `close` or keep-alive | **[STANDARD]** HTTP/1.1 default is persistent |
| `transferMode.dlna.org` | `Streaming` \| `Interactive` \| `Background` | **[OBSERVED]** DLNA; see §6 |
| `contentFeatures.dlna.org` | `DLNA.ORG_PN=…;DLNA.ORG_OP=…;DLNA.ORG_CI=…;DLNA.ORG_FLAGS=…` | **[OBSERVED]** DLNA; see §6 |

**[OBSERVED]** MiniDLNA emits `transferMode.dlna.org` and `contentFeatures.dlna.org` on every media response; for images it uses `Interactive` + `DLNA.ORG_PN=JPEG_TN`/`JPEG_SM`, and it **rejects** `transferMode: Streaming` for images with a 406 (local `minidlna_upnphttp.c`, `Send406`).

### 5.3 Content-Length and chunked transfer

**[STANDARD]** RFC 9110 §8.6: *"The 'Content-Length' header field indicates the associated content length or, in the case of a request, the size of the message body."*
**[STANDARD]** RFC 9112 §6.1: *"A recipient MUST be able to parse the chunked transfer coding … the sender MUST apply chunked as the final transfer coding to ensure … the recipient knows the message length."*

**[OBSERVED]** Every media server I read (MiniDLNA, ReadyMedia, rclone, Plex) sends `Content-Length` and **never** uses chunked for media bodies; the `Content-Length` is always known because files are seekable.

**[OPEN]** Whether a DLNA-certified renderer *refuses* a chunked body or a body without `Content-Length` is **not publicly documented**. The AudioSync DLNA spec author reached the same conclusion: *"Unknown: whether renderers that carry the DLNA logo refuse a stream without DLNA-specific response headers, a chunked body, or a body without Content-Length"* ([audiosync DLNA_SPEC.md](https://github.com/insentodesu/audiosync/blob/main/docs/DLNA_SPEC.md), §10 "Gaps").

**⇒ Implementation rule (ASSUMPTION, low risk):** always send `Content-Length` for media bodies; do not use chunked for media; chunked is acceptable for SOAP/description responses only if a `Content-Length` is also known (it is, for us — just send it).

### 5.4 Connection handling / reuse

**[STANDARD]** HTTP/1.1 persistent connections are the default; a `Connection: close` header forces close after the response (RFC 9110 §9.6 / RFC 9112 §9.6).

**[OBSERVED]** MiniDLNA and rclone both send `Connection: close` on every response; real control points (Windows Media Center, Samsung, Sony) work fine with that.

**[ASSUMPTION]** PhairDLNA should implement keep-alive properly (HTTP/1.1 default) because renderers frequently pipeline a range request after a HEAD; but correctness must not depend on it. Ensure: HEAD returns the same headers (including `Content-Length` and `Accept-Ranges`) with an empty body; GET with `Range` never returns a body longer than requested; every response sets `Content-Length` or `Connection: close`.

---

## 6. The DLNA-specific layer

### 6.1 Device classes

**[STANDARD, secondary]** DLNA defines **DMC** (controller), **DMP** (player), **DMR** (renderer) and **DMS** (server) ([Windows Embedded Compact 7 DLNA guidance](https://download.microsoft.com/download/2/4/A/24A36661-A629-4CE6-A615-6B2910A1367A/Support%20for%20DLNA%20Media%20Devices%20in%20Windows%20Embedded%20Compact%207.pdf); [MS-DLNHND](https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D-140515.pdf)). A DMS *"is a UPnP Device that implements the UPnP MediaServer Device type"*; a DMR *"is a UPnP Device that implements the UPnP MediaRenderer Device type."*

### 6.2 What a DMR actually requires from a DMS

**[STANDARD, secondary]** Windows "Device.Media Requirements" (the DLNA certification requirements mirrored by Microsoft):

* *"A DMR must declare these Profile IDs in the list of supported profiles exposed in the SinkProtocolInfo state variable available as part of the Connection Manager Service (CMS)."*
* PROT-01: *"A DMR that exposes in SinkProtocolInfo a protocolInfo value with a MIME Type and a DLNA Profile ID must be able to play content encoded according to the DLNA specifications for the Profile ID."*
* PROT-05: *"A DMR that exposes in SinkProtocolInfo a protocolInfo value with a MIME Type but without a DLNA Profile ID must be able to play content as specified in Table PROT."*
* *"The DLNA certification for an AV DMR must include the Image Class, the Audio Class, and the A/V Class."*
* — [Device.Media Requirements](https://learn.microsoft.com/sv-se/previous-versions/windows/hardware/cert-program/devicemedia-requirements)

**⇒ DMR requirements translate into DMS duties:**

1. The DMS's `res@protocolInfo` must carry a MIME type the DMR advertises in its `Sink` list (the control point matches them).
2. If the DMS advertises `DLNA.ORG_PN=<profile>`, the content must actually conform to that DLNA Media Format Profile.
3. If the DMS does **not** advertise a `DLNA.ORG_PN`, the DMR must still be able to play the MIME type per Table PROT — i.e. the DMS's MIME type must be honest.

**[STANDARD, secondary]** `[MS-DLNHND]`: *"The DLNA Guidelines require that all content be available for streaming using HTTP. Optionally, content can also be available for streaming using RTSP. HTTP server implementations (that is, the DMS role) MUST support version 1.1 of HTTP, as specified in [DLNA] guideline 7.4.23. HTTP client implementations (that is, the DMP and DMR roles) SHOULD use HTTP version 1.1 but MAY use HTTP version 1.0, as specified in [DLNA] guideline 7.4.42."*

**[STANDARD, secondary]** `[MS-DLNHND]`: *"The DMS MUST process Browse and Search UPnP actions according to the MediaServer Requirements table in [DLNA] (the table that starts with guideline 7.3.55 and ends with guideline 7.3.80)."* and *"The DMS MUST support streaming via HTTP and SHOULD support streaming via RTSP."*

### 6.3 DLNA Media Profile basics

**[STANDARD, secondary]** DLNA 4.0 white paper: *"In DLNA 4.0, every media resource exposed to media rendering devices by a Digital Media Server (DMS) must be available in a mandatory Media Format Profile."* ([DLNA 4.0 White Paper](https://static1.squarespace.com/static/54f08a57e4b0e91e31e8c76c/t/57dc19f1e58c62e05fed8606/1474042355530/DLNA+4+0+White+Paper.pdf))

**[OBSERVED]** The canonical DLNA Media Profile ID list (from `libdlna`, the reference DLNA implementation) includes, per class:

* Image: `JPEG_SM`, `JPEG_MED`, `JPEG_LRG`, `PNG_*`
* Audio: `MP3`, `LPCM`, `AC3`, `AMR`, `ATRAC3`, `MPEG4`, `WMA`
* AV: `MPEG1`, `MPEG2`, `MPEG4_PART2`, `MPEG4_PART10` (AVC), `WMV9`
* Collection: `DIDL_S`, `DIDL_V`
* — [libdlna dlna.h](https://github.com/da2ce7/libdlna/blob/master/include/dlna/dlna.h)

**[ASSUMPTION]** For PhairDLNA, the minimum viable DLNA profile set for a v1 release: **MPEG-4 Part 2 / AVC in MP4, MP3, JPEG**. These are the formats a modern DMR is guaranteed to handle and the ones every reference server advertises.

### 6.4 The DLNA signalling that actually matters

**[STANDARD, secondary]** The **4th field of `protocolInfo`** is the DLNA extension point: *"The concept of the '4th field of protocolInfo' is widely used throughout the DLNA Guidelines (see [DLNA] guideline 7.3.26) and is used as a mechanism to state which media formats are supported, and if features like seeking are supported."* ([MS-DLNHND](https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D-140515.pdf))

Tokens (as implemented by libdlna / VLC / MiniDLNA / rclone):

| Token | Meaning |
|---|---|
| `DLNA.ORG_PN=<profile>` | DLNA Media Format Profile ID (e.g. `MP3`, `AVC_MP4_HP_HD_AAC`, `JPEG_TN`) |
| `DLNA.ORG_OP=01` | operation: **0x01 = Range (byte) seek supported**, 0x10 = TimeSeek |
| `DLNA.ORG_CI=0\|1` | conversion: 0 = not transcoded, 1 = transcoded |
| `DLNA.ORG_FLAGS=<32 hex>` | binary capability flags (bit 30 = time-based seek, bit 29 = byte-based seek, bit 28 = playcontainer, bit 24 = streaming transfer mode, bit 23 = interactive, bit 22 = background, bit 21 = connection stalling, bit 20 = DLNA v1.5) |
| `DLNA.ORG_PS=1,2` | server-side play speed |

Sources: [libdlna dlna.h](https://github.com/da2ce7/libdlna/blob/master/include/dlna/dlna.h), [VLC dlna patch](https://mailman.videolan.org/pipermail/vlc-devel/2018-December/122081.html), [rclone forum log](https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480/3).

**[OBSERVED]** DLNA-specific **HTTP request headers** a client may send, which a server may act on: `transferMode.dlna.org` (`Streaming`/`Interactive`/`Background`), `getcontentFeatures.dlna.org`, `getAvailableSeekRange.dlna.org`, `TimeSeekRange.dlna.org`, `realTimeInfo.dlna.org`, `getCaptionInfo.sec`, `getCaptionInfoEx.sec`, `getMediaInfo.sec` — parsed in MiniDLNA's `upnphttp.c` (local copy) and [Fossies minidlna-1.3.3/upnphttp.c](https://fossies.org/linux/privat/minidlna-1.3.3/upnphttp.c?M=435).

**[OBSERVED]** DLNA-specific **HTTP response headers** a server should emit: `transferMode.dlna.org`, `contentFeatures.dlna.org`, `realTimeInfo.dlna.org` (`DLNA.ORG_TLAG=*`), `ifoFileURI.dlna.org`, `availableSeekRange.dlna.org` / `X-AvailableSeekRange`.

**[STANDARD, secondary]** `[MS-DLNHND]`: *"If the HTTP response status code is 200 and the DMS supports the DLNA Seek Media Operation using the TimeSeekRange.dlna.org HTTP header for the content that the client is requesting, then the DMS SHOULD include the X-AvailableSeekRange header in the response."*

### 6.5 The `sec:` namespace — and the two names that do not exist

* `xmlns:sec="http://www.sec.co.kr/dlna"` is a **Samsung** namespace used for `sec:CaptionInfo`, `sec:CaptionInfoEx`, `sec:dcmInfo`, `sec:ProductCap`, `sec:X_ProductCap` — observed in rclone's description, MediaShare, Samsung AllShare, and MiniDLNA's `SEC_NAMESPACE` macro. **[OBSERVED]**
* `xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/"` is the DLNA metadata namespace (`dlna:profileID` on album art; `dlna:lifetime` in DLNA collection profiles). **[OBSERVED]**
* **`sec:httpDLNA` / `sec:SMB` / `httpNoDLNA`: NOT FOUND** in any primary or reputable secondary source reached. See §0.3 and §9. **Do not emit these.** If they came from an internal design note, they need re-derivation.

### 6.6 A DLNA-capable server must still serve non-DLNA UPnP clients

**[STANDARD, secondary]** `[MS-DLNHND]` documents the Windows Media Center compatibility flags, and in that context: *"in response to the browse or search action the 4th_field token of the protocolInfo attribute of every res XML tag SHOULD be set to '*'"*, and a flag `WMC_COMPAT_EXCLUDE_DLNA_1_5` that *"causes a DMS to exclude all DLNA-defined attributes and parameters in the DIDL-Lite responses"* (removing `DLNA.ORG_PN`, `DLNA.ORG_OP`, `DLNA.ORG_PS`, `DLNA.ORG_CI`, `DLNA.ORG_FLAGS` and any `http-get` `res` elements when `WMC_COMPAT_EXCLUDE_HTTP` is set) ([MS-DLNHND](https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D-140515.pdf), [MS-UPMC](https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D-210625-diff.pdf)).

**⇒ Design conclusion (NORMATIVE + ASSUMPTION):** the DLNA layer must be *additive and separable*. A single code path must be able to emit:
* plain UPnP DIDL-Lite (`http-get:*:video/mp4:*`), and
* DLNA-flavoured DIDL-Lite (`http-get:*:video/mp4:DLNA.ORG_PN=…;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=…`).

Never let DLNA-only tokens break a plain UPnP client. Conversely, never let a plain-UPnP-only build break a DLNA client.

---

## 7. Version negotiation

### 7.1 ContentDirectory 1 / 2 / 3 / 4

| Version | Adds | Advertise? |
|---|---|---|
| **CD:1** (AV:1, 2002) | `Browse`, `GetSearchCapabilities`, `GetSortCapabilities`, `GetSystemUpdateID` (all required); `Search` optional in later CD but core to AV:1-era servers | **Yes — primary** |
| **CD:2** (AV:2, 2006-05-31) | `Search`, `CreateObject`, `DestroyObject`, `UpdateObject`, `MoveObject`, `ImportResource`, `ExportResource`, `DeleteResource`, `StopTransferResource`, `GetTransferProgress`, `CreateReference` — plus `MediaServer:2` and the AV:2 `ScheduledRecording` service | Optional |
| **CD:3** (AV:3) | "Tracking Changes Option", "Foreign Meta data", "FreeFormQuery (XML based Search)"; `GetSortExtensionCapabilities` | Optional |
| **CD:4** (AV:4, 2013-03-31) | "Richer Content Description (ResExt)", "Enriched Content Navigation (Content Segmentation)", "Content Privacy", "Resource management", "Exclusive ownership", "Time Shift Support"; plus `GetFeatureList`, `GetServiceResetToken`, `GetSortExtensionCapabilities`, `DeviceMode`/`DeviceModeStatus`, `PermissionsInfo`, `ServiceResetToken`, `TransferIDs` | Optional |

Sources: [UPnP AV tutorial, July 2014](https://upnp.org/resources/documents/UPnP_AV_tutorial_July2014.pdf) (version→feature table); [UPnPAV4Overview_March2012.pdf](https://upnp.org/resources/documents/UPnPAV4Overview_March2012.pdf); [CD:4 action table](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf).

**[STANDARD]** AV versions are cumulative: *"Each new version is fully backward compatible with a previous version"* (UPnP AV tutorial).

**[STANDARD]** CD:4 `UpdateID` semantics differ from CD:1 (SystemUpdateID vs ContainerUpdateID) — §1.2.

### 7.2 AV:1 vs AV:4 differences that touch a MediaServer

| Area | AV:1 | AV:4 |
|---|---|---|
| Services | CDS, CMS, optional AVT | CDS, CMS, optional AVT, **RCS enhancements**, SRS (AV:2), DeviceProtection (for privacy) |
| CDS actions | 4 required + `Search` | + `CreateObject`, `DestroyObject`, `UpdateObject`, `MoveObject`, `Import/ExportResource`, `DeleteResource`, `StopTransferResource`, `GetTransferProgress`, `CreateReference`, `GetFeatureList`, `GetServiceResetToken`, `GetSortExtensionCapabilities`, device-mode actions, permissions actions |
| Metadata | `res` + basic `dc:`/`upnp:` properties | `upnp:resExt` (multi-stream, `segmentInfo`, `clockSync`, `DRMInfo`, `uniqueContentIdentification`), `upnp:class` extended set, `upnp:albumArtURI` |
| Navigation | flat `Browse` | "DVD-like" navigation, content segmentation, `FreeFormQuery` (AV:3), `GetObjectIDfromPath`-style path addressing (AV:4-era "object path" work) |
| Sync | none | `CLOCKSYNC` feature via `GetFeatureList`, `upnp:resExt::clockSync` |
| Privacy | none | `objectOwner`/`inclusionControl` + DeviceProtection service, `ExclusiveOwnership` mode |
| Renderer matching | MIME matching | "Renderer Content Matching (with DRM)" in CMS |

Sources: [UPnPAV4Overview](https://upnp.org/resources/documents/UPnPAV4Overview_March2012.pdf), [UPnP AV tutorial](https://upnp.org/resources/documents/UPnP_AV_tutorial_July2014.pdf), [CD:4](https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf).

**[ASSUMPTION — advertisement strategy]** Advertise **exactly one** `ContentDirectory` service instance whose `serviceType` is `urn:schemas-upnp-org:service:ContentDirectory:1`, and implement the CD:1 required actions with CD:2-compatible superset behaviour where free (e.g. `Search`). Rationale:

* **[OBSERVED]** Every real DMS I read ships `ContentDirectory:1` (MiniDLNA, ReadyMedia, rclone, Plex, DLNA test tools).
* **[STANDARD]** MediaServer:2 *"MUST respond to all SSDP queries that specify MediaServer:1 and MUST respond to all actions defined by the MediaServer:1"* — i.e. compatibility is expressed by *behaviour*, not by advertising a newer version.
* **[ASSUMPTION]** A control point that sees `ContentDirectory:4` may use CD:4-only actions (`GetFeatureList`, `FreeFormQuery`, device-mode actions) and then fail against a partially implemented service. Advertising `:1` keeps every legacy DMR on a well-trodden path.

**⇒ Also advertise `MediaServer:1`** as the device type, and (optionally) *also* answer `ST: urn:schemas-upnp-org:device:MediaServer:2|4` if the implementation really supports the extra actions. **[ASSUMPTION]** Do not advertise `:4` until `GetFeatureList`, `GetServiceResetToken`, `GetSortExtensionCapabilities` and the device-mode/permissions actions are implemented (or are at least spec-compliant "unsupported" errors).

### 7.3 ConnectionManager / AVTransport / RenderingControl versions

**[OBSERVED]** All real devices ship `ConnectionManager:1`, `AVTransport:1`, `RenderingControl:1`. `ConnectionManager:2/3` exist (with `GetFeatureList` for clock sync) but are effectively unused. **[ASSUMPTION]** Advertise `:1`.

---

## 8. MUST/SHOULD → concrete implementation obligations

Legend: **NORMATIVE** = from a specification; **OBSERVED** = real-world client/server behaviour; **ASSUMPTION** = my design inference.

### 8.1 Discovery (SSDP)

| # | Obligation | Tag |
|---|---|---|
| D1 | Join `239.255.255.250:1900` on every UPnP-enabled interface and answer `M-SEARCH` with `MAN: "ssdp:discover"`. | NORMATIVE |
| D2 | `MX` handling: use `min(MX, 5)` as the window; delay the reply by a uniform random time in `[0, MX)`. | NORMATIVE |
| D3 | If `MX` is absent → silently ignore the request (no reply, no error). | NORMATIVE |
| D4 | Reply **unicast** to the source IP:port of the multicast request. | NORMATIVE |
| D5 | Reply within `MX` seconds; for a *unicast* M-SEARCH reply within 1 s. | NORMATIVE |
| D6 | Response must carry `CACHE-CONTROL: max-age=`, `DATE`, `EXT`, `LOCATION`, `SERVER`, `ST`, `USN`. | NORMATIVE |
| D7 | `max-age` ≥ 1800 s; re-advertise `ssdp:alive` every `max-age/3…/4`. | NORMATIVE (max-age value) + ASSUMPTION (cadence) |
| D8 | `SERVER: <OS>/<ver> UPnP/1.1 PhairDLNA/<ver>` (UDA 2.0 → `UPnP/2.0`). | NORMATIVE |
| D9 | `USN` prefix must equal the `UDN` value; use `uuid:<UDN>::upnp:rootdevice`, `…::urn:schemas-upnp-org:device:MediaServer:1`, `…::urn:schemas-upnp-org:service:ContentDirectory:1`, `…::urn:schemas-upnp-org:service:ConnectionManager:1`, and bare `uuid:<UDN>`. | NORMATIVE |
| D10 | `ST` matching table (`ssdp:all` → 4–5 responses; `upnp:rootdevice` → 1; `uuid:` → per matching device; `urn:…:device:…` / `urn:…:service:…` → per match). | NORMATIVE |
| D11 | Send `NTS: ssdp:byebye` for each advertised NT on shutdown. | NORMATIVE |
| D12 | `LOCATION` must be reachable **on the interface the request arrived on**; multi-homed devices MUST reply on the receiving interface. | NORMATIVE |
| D13 | IPv6: join `FF02::C:1900` (link-local) and `FF05::C:1900` (site); use link-local source addresses and link-local literals in `LOCATION` for link-scoped SSDP. | NORMATIVE (UDA 2.0) + ASSUMPTION (macOS feasibility) |
| D14 | IPv4 multicast TTL default 2. | NORMATIVE |
| D15 | Also answer `ST: urn:schemas-upnp-org:device:MediaServer:1` (and `:2`/`:4` only if truly implemented) for forward compatibility. | NORMATIVE (MediaServer:2 rule) + ASSUMPTION |
| D16 | Do **not** rely on mDNS for discovery; SSDP is the DLNA discovery mechanism. | NORMATIVE (by absence) + ASSUMPTION |

### 8.2 Device description

| # | Obligation | Tag |
|---|---|---|
| P1 | Serve `description.xml` at a stable URL referenced by `LOCATION`; `Content-Type: text/xml; charset="utf-8"`. | NORMATIVE (UDA) + OBSERVED (rclone logs) |
| P2 | `<root xmlns="urn:schemas-upnp-org:device-1-0">`, `specVersion` 1.0, one `<device>`. | NORMATIVE |
| P3 | Include `deviceType`, `friendlyName`, `manufacturer`, `modelName`, `UDN` (as `uuid:…`), `serviceList` with `serviceType`/`serviceId`/`SCPDURL`/`controlURL`/`eventSubURL` for **both** ContentDirectory:1 and ConnectionManager:1. | NORMATIVE |
| P4 | `UDN` must be stable across restarts and unique per device; the `USN` prefix must equal it. | NORMATIVE |
| P5 | `serviceId` values: `urn:upnp-org:serviceId:ContentDirectory`, `urn:upnp-org:serviceId:ConnectionManager`. | OBSERVED |
| P6 | `controlURL`/`eventSubURL` present and non-empty. | OBSERVED / ASSUMPTION |
| P7 | URLs may be absolute or relative; if relative, resolution is against the description URL (or `URLBase` when present). | NORMATIVE (`URLBase` template) + ASSUMPTION (resolution base) |
| P8 | Serve a valid **SCPD** for each service (action list, argument list, state variables) — control points parse it. | NORMATIVE (UDA) + OBSERVED |
| P9 | Advertise `deviceType urn:schemas-upnp-org:device:MediaServer:1`. | NORMATIVE |

### 8.3 ContentDirectory

| # | Obligation | Tag |
|---|---|---|
| C1 | Implement `Browse`, `GetSearchCapabilities`, `GetSortCapabilities`, `GetSystemUpdateID` (all required). | NORMATIVE |
| C2 | `Browse` returns `Result`, `NumberReturned`, `TotalMatches`, `UpdateID`; `NumberReturned`=1/`TotalMatches`=1 for `BrowseMetadata`; `TotalMatches`=total children for `BrowseDirectChildren`. | NORMATIVE |
| C3 | If `TotalMatches` cannot be computed: return `TotalMatches=0` with `NumberReturned>0`, or SOAP error **720** when nothing matched. | NORMATIVE |
| C4 | `UpdateID` = container's `ContainerUpdateID`, or `SystemUpdateID` when `ObjectID` is `0` (CD:1) / always `SystemUpdateID` (CD:4). Pick one and document it; recommend **CD:1 semantics** while advertising CD:1. | NORMATIVE (both) + ASSUMPTION (choice) |
| C5 | `BrowseMetadata` → `StartingIndex` must be 0; `RequestedCount=0` means "all". | NORMATIVE |
| C6 | `Filter` must accept `*` and named properties; implement `*` correctly (clients send it constantly). | NORMATIVE + OBSERVED |
| C7 | `SortCriteria`: accept `""` and `+prop`/`-prop`; only advertise in `GetSortCapabilities` what you actually sort by. | NORMATIVE + OBSERVED |
| C8 | Implement `Search` (CD:2+) with `SearchCriteria` in the `dc:title contains "x"` form. | NORMATIVE (CD:2, O in CD:1) + ASSUMPTION (worth it) |
| C9 | `Result` must be a **DIDL-Lite document**, XML-escaped once more as SOAP text. | NORMATIVE |
| C10 | Implement `GetSearchCapabilities` returning a CSV of property names. | NORMATIVE |
| C11 | `SystemUpdateID` must be monotonic and change when the tree changes. | NORMATIVE + ASSUMPTION (semantics) |
| C12 | Do **not** implement `GetFreeSpace`, `GetUpdateCounter`, `GetObjectIDfromPath`, `GetParentID`, `GetTransferTotal`, `GetShareIndexInBrowse` as UPnP CD actions — they are not in CD:1–:4. | NORMATIVE (absence) + OBSERVED (absent in MiniDLNA/ReadyMedia) |
| C13 | If a specific client is known to need one of those, expose it as a **vendor action** in the `urn:schemas-upnp-org:service:ContentDirectory:1` namespace but document it as non-standard. | ASSUMPTION |

### 8.4 ConnectionManager

| # | Obligation | Tag |
|---|---|---|
| M1 | Implement `GetProtocolInfo` returning `Source` and `Sink` as CSV of 4-field protocolInfo strings. | NORMATIVE |
| M2 | `Source` MUST list the MIME types the DMS can serve; format `http-get:*:<mime>:<4th field>`. | NORMATIVE + ASSUMPTION (the `Source` semantics come from the spec text: "protocols this ConnectionManager supports for 'sourcing' (sending) data, in its current state") |
| M3 | `Sink` may be empty for a pure server; renderers fill it with what they can play. | NORMATIVE (state variable is generic) + ASSUMPTION |
| M4 | Implement `GetCurrentConnectionIDs` and `GetCurrentConnectionInfo`; maintain a connection table (even if always empty for pure HTTP push). | NORMATIVE |
| M5 | Implement `PrepareForConnection`/`ConnectionComplete` if you expose AVTransport virtual instances; otherwise mark them optional in the SCPD. | NORMATIVE (Annex A: optional) + OBSERVED (MiniDLNA/ReadyMedia mark optional) |
| M6 | `Direction` values `Input`/`Output`; `Status` values `OK`/`ContentFormatMismatch`/`InsufficientBandwidth`/`UnreliableChannel`/`Unknown`. | NORMATIVE |
| M7 | `SourceProtocolInfo`/`SinkProtocolInfo`/`CurrentConnectionIDs` are evented variables → GENA support required if you subscribe. | NORMATIVE |

### 8.5 DIDL-Lite

| # | Obligation | Tag |
|---|---|---|
| L1 | Every `<item>`/`<container>` carries `id`, `parentID`, `restricted`; `<container>` may carry `childCount`, `searchable`; `<item>` may carry `refID` for reference items. | NORMATIVE (schema) |
| L2 | Every item carries `dc:title` and `upnp:class`. | NORMATIVE |
| L3 | `upnp:class` from the CD:4 allowed list; use `object.item.audioItem.musicTrack`, `object.item.videoItem.movie`, `object.item.videoItem.videoBroadcast`, `object.item.imageItem.photo`, `object.container.storageFolder`, `object.container.person`. | NORMATIVE |
| L4 | `upnp:albumArtURI` = absolute URL to cover art; **do not** put `dlna:profileID` on it. | NORMATIVE (allowed) + OBSERVED (WMC conflict) |
| L5 | `<res protocolInfo="http-get:*:<mime>:<4th>">` with the absolute media URL as text. | NORMATIVE |
| L6 | Populate `res@size`, `res@duration` (`H*:MM:SS.F*`), `res@bitrate` (**bytes/s**), `res@sampleFrequency`, `res@nrAudioChannels`, `res@resolution` (`WxH`), `res@bitsPerSample` where known. | NORMATIVE (attribute existence) + ASSUMPTION (which to populate) |
| L7 | Declare every namespace prefix used (never emit `sec:` without `xmlns:sec`). | OBSERVED (Cling strictness) |
| L8 | XML-escape `& < >` in text and `& < > " '` in attributes; escape the whole DIDL-Lite again into `<Result>`. | NORMATIVE (XML 1.0) + OBSERVED |
| L9 | Keep each element/attribute value ≲ 8 KB (XML-escaped UTF-8). | OBSERVED (DLNA CIG) |
| L10 | `dc:date` as ISO-8601 date or date-time. | OBSERVED + ASSUMPTION |
| L11 | Use `dc:creator` **and** `upnp:artist` for artists; `upnp:album` for the album; `upnp:genre` for genre. | OBSERVED + ASSUMPTION |

### 8.6 HTTP delivery

| # | Obligation | Tag |
|---|---|---|
| H1 | Implement `GET` and `HEAD` for media; `HEAD` returns identical headers, empty body. | NORMATIVE (HTTP) + OBSERVED |
| H2 | `200` with full body when no `Range`; `206` + `Content-Range: bytes a-b/total` for a single byte range. | NORMATIVE |
| H3 | Unsatisfiable range → `416` with `Content-Range: bytes */total`. | NORMATIVE (SHOULD) |
| H4 | Multiple ranges → serve one range, or ignore `Range` and return `200`. **Do not** implement `multipart/byteranges`. | NORMATIVE (MAY ignore) + ASSUMPTION |
| H5 | Send `Accept-Ranges: bytes` on media responses. | NORMATIVE |
| H6 | Always send `Content-Length`; do not chunk media bodies. | NORMATIVE (HTTP) + ASSUMPTION (DLNA caution) |
| H7 | `Content-Type` MUST equal the MIME in `res@protocolInfo` field 3. | NORMATIVE (consistency) + ASSUMPTION |
| H8 | Ignore `Range` on non-GET methods; ignore unknown range units. | NORMATIVE |
| H9 | Support HTTP/1.1 persistent connections correctly; `Connection: close` is acceptable. | NORMATIVE + OBSERVED |
| H10 | Respect `If-Range` semantics: if the condition fails, ignore `Range` and return `200`. | NORMATIVE (RFC 9110 §13.1.5) |
| H11 | Reject path traversal / wide links: the media URL must resolve inside the served roots. | ASSUMPTION (MiniDLNA has an explicit "Rejecting wide link" check) |
| H12 | Serve a `404` for unknown media IDs, `405` for unsupported methods, `400` for malformed `Range`. | ASSUMPTION |

### 8.7 GENA eventing

| # | Obligation | Tag |
|---|---|---|
| E1 | Implement `SUBSCRIBE` (with `CALLBACK`, `NT: upnp:event`, `TIMEOUT: Second-N`), `UNSUBSCRIBE` (with `SID`), and `NOTIFY` (with `NT`, `NTS: upnp:propchange`, `SID`, `SEQ`, `CONTENT-TYPE: text/xml`, `Content-Length`). | NORMATIVE |
| E2 | `SID` must be `uuid:`-prefixed and globally unique; respond with `TIMEOUT: Second-<actual>` ≥ 1800 s. | NORMATIVE |
| E3 | Send the **initial** event (`SEQ: 0`) with all evented variables right after accepting a subscription. | NORMATIVE |
| E4 | Increment `SEQ` by 1 per subsequent event to that subscriber; wrap 2^32−1 → 1. | NORMATIVE |
| E5 | Deliver `LastChange` as a nested XML document for AVTransport/RenderingControl; CDS `LastChange` per the CDS LastChange schema. | NORMATIVE |
| E6 | Support subscription renewal (re-SUBSCRIBE with `SID`) and expiry. | NORMATIVE |
| E7 | Validate the `CALLBACK` URL (loopback/link-local only) to avoid SSRF via the GENA callback — PhairPlay already models this as `GenaCallbackPolicy`. | ASSUMPTION (project-internal consistency) |

### 8.8 DLNA layer

| # | Obligation | Tag |
|---|---|---|
| N1 | Serve media over **HTTP/1.1** (the DMS role MUST support HTTP 1.1 per DLNA guideline 7.4.23 as quoted by MS-DLNHND). | NORMATIVE |
| N2 | Advertise an honest MIME type per item; if you advertise `DLNA.ORG_PN=X`, the content must conform to DLNA Media Format Profile X. | NORMATIVE (PROT-01/PROT-05) |
| N3 | Put `DLNA.ORG_PN`/`DLNA.ORG_OP=01`/`DLNA.ORG_CI=0`/`DLNA.ORG_FLAGS` in the 4th protocolInfo field for DLNA mode; `DLNA.ORG_OP=01` advertises byte-range seek. | OBSERVED |
| N4 | Emit `transferMode.dlna.org` (`Streaming` for AV, `Interactive` for images) and `contentFeatures.dlna.org` on media responses. | OBSERVED |
| N5 | Return `406` if the client requests a `transferMode.dlna.org` the server does not support for that content. | OBSERVED (MiniDLNA) |
| N6 | Handle `getcontentFeatures.dlna.org: 1` (reply with `contentFeatures.dlna.org`); reject other values with `400`. | OBSERVED (dlna-microstack) |
| N7 | Handle `TimeSeekRange.dlna.org` / `getAvailableSeekRange.dlna.org` / `realTimeInfo.dlna.org` request headers. | OBSERVED |
| N8 | Provide a DLNA Media Profile set covering at least MP4 (MPEG-4 Part 2 / AVC), MP3, JPEG. | ASSUMPTION (from libdlna profile list) |
| N9 | Keep DLNA signalling **additive**: plain UPnP clients must keep working; provide the ability to emit plain `http-get:*:<mime>:*`. | NORMATIVE (MS-DLNHND WMC flags) + ASSUMPTION |
| N10 | Do not emit `sec:httpDLNA`, `sec:SMB`, `httpNoDLNA`, `dlna:profileID` on album art, or undeclared namespaces. | OBSERVED / NORMATIVE (WMC) |

---

## 9. Open questions (must be closed before implementation sign-off)

| # | Question | Why it matters | Status |
|---|---|---|---|
| Q1 | Does a DLNA-certified DMR reject a **chunked** media body, or one without `Content-Length`? | Decides whether chunked can be used for transcoded/live streams. | OPEN — the AudioSync spec author flags the identical gap; DLNA guideline text is not public. |
| Q2 | Exact DLNA **Media Profile ID** list and per-profile conformance rules. | Needed for `DLNA.ORG_PN` values we emit. | OPEN — only the `libdlna` list (secondary) was obtained. |
| Q3 | Exact normative wording of DLNA guideline **7.3.55–7.3.80** (MediaServer Browse/Search requirements table). | The table a DMS must satisfy. | OPEN — only quoted at second hand by MS-DLNHND. |
| Q4 | `sec:httpDLNA` / `sec:SMB` / `httpNoDLNA` — real names, or a design-note error? | Would be emitted into DIDL-Lite if they exist. | OPEN — not found anywhere. |
| Q5 | `writeStatus` — attribute or `upnp:` property? Allowed values? | Would be emitted on every item. | OPEN — the CD:4 Appendix-B row was not opened. |
| Q6 | Exact type/format constraints for `dc:date`, `upnp:artist`, `upnp:genre`, `upnp:album` in CD:4 Appendix B. | Metadata correctness. | OPEN — table not opened. |
| Q7 | Does `ContentDirectory:4` require `LastChange` to be evented for a plain MediaServer? (It is marked CR.) | Whether GENA on CDS is mandatory. | PARTLY OPEN — CD:4 marks `LastChange` CR; the "CR" condition text was not opened. |
| Q8 | Behaviour of `GetShareIndexInBrowse` / `GetFreeSpace` — which clients need them and under which namespace? | Possible vendor-extension work. | OPEN — only a Sonos-style Go client was seen to use `GetShareIndexInBrowse`. |
| Q9 | `ssdp://239.255.255.250:1900` — is this a normative URI form anywhere? | Cosmetic only. | OPEN. |
| Q10 | macOS-specific: IPv6 link-local on the LAN interface in CI/sandbox; `IP_ADD_MEMBERSHIP` per-interface behaviour; `SO_REUSEPORT`/port-1900 sharing with other daemons on the same host. | Build/testability. | OPEN (engineering, not standards). |
| Q11 | Whether `ContentDirectory:1` vs `:4` `UpdateID` semantics cause client-visible differences in practice. | Cache-invalidation correctness. | ASSUMPTION that CD:1 semantics are safe for all observed clients. |
| Q12 | DLNA `DIDL_V` / `DIDL_S` collection-profile constraints if we ever publish playlists as `text/xml`. | Only if playlist export is in scope. | OPEN (out of scope for v1). |

---

## 10. Sources

### 10.1 Primary specifications — downloaded verbatim, quoted exactly

| # | URL | What it contributed |
|---|---|---|
| 1 | https://www.rfc-editor.org/rfc/rfc6761.txt | **Special-Use Domain Names** — the domain-reservation registry procedure; used to correct the brief's "RFC 6761 (mDNS)". |
| 2 | https://www.rfc-editor.org/rfc/rfc6762.txt | **Multicast DNS** — abstract, `.local.` semantics (§3); negative evidence: contains no mention of SSDP or UPnP. |
| 3 | https://www.rfc-editor.org/rfc/rfc9110.txt | HTTP semantics: §8.3 Content-Type, §8.6 Content-Length, §13.1.5 If-Range, §14.1–14.4 Range/Accept-Ranges/Content-Range, §14.6 multipart/byteranges, §15.5.17 416, §17.15 range DoS. |
| 4 | https://www.rfc-editor.org/rfc/rfc9112.txt | Transfer-Encoding / chunked rules (§6.1) and connection framing. |
| 5 | https://www.w3.org/TR/1998/REC-xml-19980210 | XML 1.0 entity rules (`&amp; &lt; &gt; &quot; &apos;`) used for DIDL-Lite escaping. |

### 10.2 Primary OCF/UPnP specifications — text extracted from these exact PDF URLs

> Caveat repeated from §0.2: `upnp.org` served a JavaScript challenge to direct download from this environment. The text below was obtained via the search engine's full-text extraction of the identical canonical PDF URLs. Treat as primary but re-verify any single quoted sentence before it becomes a hard-coded requirement.

| # | URL | What it contributed |
|---|---|---|
| 6 | https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.1.pdf | SSDP: multicast address/port, `M-SEARCH` format, `MX` rules, ST-matching table, unicast-response MUST, `CACHE-CONTROL`/`SERVER`/`USN`/`LOCATION` rules, GENA SUBSCRIBE/UNSUBSCRIBE/NOTIFY, `SEQ`, initial event, `TIMEOUT` ≥ 1800. |
| 7 | https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.0.pdf | SSDP header semantics cross-check (`ssdp:all` response counts, `max-age` ≥ 1800, SERVER format). |
| 8 | https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0.pdf | IPv6 SSDP: `FF02::C`/`FF05::C`, link-local source+LOCATION rule, `FF02::130` multicast eventing, `BOOTID.UPNP.ORG`, `OPT`/`01-NLS`, `ssdp:byebye`/`ssdp:update`. |
| 9 | https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0-20140901.pdf | Same as #8 (dated variant) — cross-check of IPv6 rules. |
| 10 | https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0-20150220.pdf | Same as #8 (dated variant) — cross-check of NOTIFY/ssdp:byebye syntax. |
| 11 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v1-Service.pdf | CD:1 action/argument table, `Browse` output semantics, error 720, `TotalMatches`/`UpdateID` rules, `SearchCapabilities`/`SortCapabilities`. |
| 12 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v2-Service.pdf | CD:2 `Browse` output wording and search behaviour. |
| 13 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service.pdf | CD:4 state-variable table, full action table (R/O + control-point R/O), `upnp:class` allowed-value table, `res@*` property table, `GetSearchCapabilities`/`GetSortCapabilities`/`GetSystemUpdateID` text, DIDL-Lite root-element grammar, default-namespace table. |
| 14 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service-20130331.pdf | CD:4 dated edition (2013-03-31) — cross-check of the action tables and `CLOCKSYNC`/`DeviceMode` feature text. |
| 15 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service-20101231.pdf | CD:4 earlier edition — cross-check of action R/O table. |
| 16 | https://upnp.org/specs/av/UPnP-av-ConnectionManager-v1-Service.pdf | ConnectionManager:1 actions, arguments, `SourceProtocolInfo`/`SinkProtocolInfo`, `ProtocolInfo` definition, `PeerConnectionManager` format, `PrepareForConnection` semantics. |
| 17 | https://upnp.org/specs/av/UPnP-av-ConnectionManager-v1-Service-AnnexA-20101006.pdf | Annex A R/O table for ConnectionManager:1 actions (device vs control point). |
| 18 | https://upnp.org/specs/av/UPnP-av-MediaServer-v1-Device.pdf | MediaServer:1 required service table, device description template, `serviceId` strings. |
| 19 | https://upnp.org/specs/av/UPnP-av-MediaServer-v2-Device-20060531.pdf | MediaServer:2 "one instance each" rule and the MediaServer:1 backward-compatibility MUST. |
| 20 | https://upnp.org/specs/av/UPnP-av-MediaServer-v4-Device-20101231.pdf | MediaServer:4 template with `ContentDirectory:4` and the full element list. |
| 21 | https://upnp.org/specs/av/UPnP-av-AVTransport-v3-Service.pdf | AVTransport:3 action R/O table, `TransportState` values, `CurrentTransportActions` rule. |
| 22 | https://upnp.org/specs/av/UPnP-av-AVArchitecture-v1-20020625.pdf | AV:1 architecture: MediaServer = CDS + CMS + optional AVT; `Browse`/`Search`/`PrepareForConnection` flow. |
| 23 | https://upnp.org/specs/av/UPnP-av-AVArchitecture-v2.pdf | AV:2 architecture and `GetFeatureList` / clock-sync architecture. |
| 24 | https://upnp.org/resources/documents/UPnP_AV_tutorial_July2014.pdf | AV version→feature table (AV2/AV3/AV4 deltas), `Browse` argument summary, DLNA 4th-field tokens, `X_GetDLNAUploadProfiles`. |
| 25 | https://upnp.org/resources/documents/UPnP_UDA_tutorial_July2014.pdf | UDA tutorial: SSDP NOTIFY/M-SEARCH message shapes, GENA header table, `SEQ` semantics. |
| 26 | https://upnp.org/resources/documents/UPnPAV4Overview_March2012.pdf | UPnP AV:4 feature list (resExt, segmentation, privacy, resource control, sync, renderer content matching). |

### 10.3 DLNA-layer and ecosystem sources (secondary / vendor documentation)

| # | URL | What it contributed | Status |
|---|---|---|---|
| 27 | https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D-140515.pdf | Microsoft's published DLNA extensions: "4th field of protocolInfo" (DLNA guideline 7.3.26), DMS MUST support HTTP/1.1 (7.4.23), client SHOULD use HTTP/1.1 (7.4.42), DMS MUST process Browse/Search per DLNA guidelines 7.3.55–7.3.80, WMC 4th-field `*` guidance, `X-AvailableSeekRange` / `TimeSeekRange.dlna.org` behaviour. | Secondary, quotes DLNA guideline numbers |
| 28 | https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D-210625-diff.pdf | `WMC_COMPAT_EXCLUDE_DLNA_1_5` / `WMC_COMPAT_EXCLUDE_HTTP` semantics; the rule that `dlna:profileID` MUST NOT appear in `upnp:albumArtURI`. | Secondary |
| 29 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/c12f5530-fa2d-4d9b-a960-18f4e9678100 | Same WMC-compat rules in HTML form. | Secondary |
| 30 | https://learn.microsoft.com/sv-se/previous-versions/windows/hardware/cert-program/devicemedia-requirements | DLNA DMR certification requirements PROT-01/PROT-05, `SinkProtocolInfo` Profile-ID declaration, AV DMR class requirements. | Secondary |
| 31 | https://static1.squarespace.com/static/54f08a57e4b0e91e31e8c76c/t/57dc19f1e58c62e05fed8606/1474042355530/DLNA+4+0+White+Paper.pdf | DLNA 4.0: "every media resource exposed … by a DMS must be available in a mandatory Media Format Profile"; DMR/DMP role definitions. | Vendor white paper |
| 32 | https://github.com/da2ce7/libdlna/blob/master/include/dlna/dlna.h | Reference DLNA implementation: Media Profile IDs per class, `DLNA.ORG_FLAGS` bit layout, `DLNA.ORG_OP` values, `dlna_write_protocol_info()`. | Open-source reference impl |
| 33 | https://cdn.standards.iteh.ai/samples/23044/1882f21ecfbc492285b08628036a085b/IEC-62481-4-2017.pdf | IEC 62481-4 preview: DTCP-IP DIS is optional for a UPnP AV MediaServer; DTCP flags ride in the 4th protocolInfo field. | Standard preview |
| 34 | https://www.scribd.com/document/803046323/DLNA-Guidelines-June-2016-Part-2-Media-Format-Profiles | DLNA Part 2 Media Format Profiles: `DIDL_S`/`DIDL_V` collection profiles, "Complies with the DIDL-Lite schema", `dlna:` namespace usage. | **Non-authoritative copy** — treat with care |
| 35 | https://github.com/4thline/cling/blob/master/support/src/main/resources/org/fourthline/cling/support/contentdirectory/descriptor/didl-lite-1.0.xsd | The DIDL-Lite XSD transcribed from the CD spec: required attributes on `item`/`container`, the full `res` attribute list, duration format, bitrate units. | Schema transcription |
| 36 | https://github.com/StevenLooman/home-assistant-dlna-dmr/blob/master/home_assistant_dlna_dmr/dlna_dmr.py | Real DMR-side implementation: `Accept-Ranges: bytes`, `transferMode.dlna.org`, `contentFeatures.dlna.org`, DIDL-Lite shape for `SetAVTransportURI`. | Open-source impl |
| 37 | https://github.com/rclone/rclone/blob/0c9826d3/cmd/serve/dlna/dlna_util.go | Real DMS: DIDL-Lite namespace set (`dc`, `upnp`, default DIDL-Lite, `dlna`, `sec`). | Open-source impl |
| 38 | https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480/3 | Captured live wire trace of a real Samsung control point: description XML, `Browse` SOAP request/response, escaped `Result`, `Connection: close`. | Field report |
| 39 | https://github.com/insentodesu/audiosync/blob/main/docs/DLNA_SPEC.md | Independent DLNA-sender spec: SOAP envelope shape, `Content-Length` (never chunked request bodies), protocolInfo 4-field matching, DIDL-Lite in `CurrentURIMetaData`, and an explicit "Gaps" section flagging the chunked/Content-Length uncertainty (Q1). | Open-source spec |
| 40 | https://github.com/seattlerb/upnp-contentdirectory/blob/master/lib/UPnP/service/content_directory.rb | Reference CD:1/2 implementation: full action list, state variables, `BrowseFlags` values. | Open-source impl |
| 41 | https://github.com/seattlerb/upnp-connectionmanager/blob/master/lib/UPnP/service/connection_manager.rb | Reference ConnectionManager:1 implementation: action/argument direction table. | Open-source impl |
| 42 | https://github.com/JuanKman94/readymedia-transcode/blob/transcode/upnpdescgen.c | ReadyMedia's generated device description: service types/ids/URLs, `PrepareForConnection` marked `//OPTIONAL`. | Open-source impl |
| 43 | https://github.com/keenetic/minidlna/blob/master/upnpsoap.c | MiniDLNA: `dlna:profileID` on album art, DLNA namespace macro. | Open-source impl |
| 44 | https://sourceforge.net/p/minidlna/bugs/316/ | The "undeclared `sec:` namespace breaks strict Cling parsers" bug. | Bug report |
| 45 | https://fossies.org/linux/privat/minidlna-1.3.3/upnphttp.c?M=435 | MiniDLNA 1.3.3 HTTP layer: `transferMode.dlna.org`/`contentFeatures.dlna.org` parsing, 406 on unsupported transfer mode, `Content-Length` handling. | Open-source impl |
| 46 | https://github.com/miniupnp/miniupnp/blob/miniupnpd_2_3_11/miniupnpd/minissdp.c | miniupnpd: verbatim UDA 1.1 quotes (MX > 5 → assume ≤ 5; missing MX → ignore request; multicast TTL default 2; IPv6 scope rules). | Open-source impl quoting the spec |
| 47 | https://support.pelco.com/s/article/How-a-UPnP-Search-works-1538586696122 | Verbatim UDA 1.1 quotes for MX semantics, unicast M-SEARCH 1-second rule, unicast-response MUST. | Vendor doc quoting UDA |
| 48 | https://sming.readthedocs.io/en/latest/_inc/Sming/Libraries/SSDP/index.html | SSDP ST-matching table cross-check (ssdp:all / upnp:rootdevice / uuid: / urn:device / urn:service). | Open-source impl |
| 49 | https://github.com/cvdlinden/upnp-explorer/blob/main/docs/ssdp-protocol-cheat-sheet.md | Practical SSDP wire shapes, `max-age=1800` convention. | Open-source impl |
| 50 | https://github.com/InstaZDLL/WaveFlow/blob/main/docs/features/dlna.md | Independent DLNA DMS: `transferMode.dlna.org: Streaming` + `contentFeatures.dlna.org` required for a DLNA scrubber; DIDL-Lite field set. | Open-source impl |
| 51 | https://github.com/supersonic-app/supersonic/issues/931 | Field report on `res@bitrate` units ambiguity; the canonical DIDL-Lite field set. | Issue tracker |
| 52 | https://github.com/gerbera/gerbera/issues/2894 | Real DMS DIDL-Lite with `upnp:albumArtURI dlna:profileID="JPEG_TN"`, `DLNA.ORG_PN=MP3`, `DLNA.ORG_FLAGS`. | Issue tracker |
| 53 | https://github.com/gerbera/gerbera/issues/2017 | Gerbera's `sec:` namespace usage and Samsung subtitle workarounds. | Issue tracker |
| 54 | https://github.com/Tzxhy/dlna-home/blob/master/soapcalls/soapbuilders.go | Control point behaviour: it *rewrites* `&#34;` → `"` and `&amp;` → `&` before sending `SetAVTransportURI` (Samsung quirk). | Open-source impl |
| 55 | https://github.com/Clement87/sonos/blob/master/sonos.class.php | Control point double-unescaping of the DIDL-Lite inside `Result`/`LastChange`. | Open-source impl |
| 56 | https://emby.media/learn/articles/playback-streaming/casting/dlna/Playback-DLNA.html | Plain-language role/protocolInfo/DIDL-Lite summary (corroboration only). | Vendor doc |
| 57 | https://media2.comcast.net/anon.comcastonline2/support/help/faqs/boxee/Comcast-SP-HDuDTA20-DLNA-CIG-I01Public-130314Issued.pdf | DLNA client guide: 8192-byte limit for DIDL-Lite element/attribute values. | Vendor doc |
| 58 | https://github.com/0xfab-ri/dlna-microstack/blob/master/include/dlna/DlnaHttpServer.h | DLNA HTTP server rules: `RANGE` → `CONTENT-RANGE`; unsupported `transferMode.dlna.org` → 406; `getcontentFeatures.dlna.org` → 400 when the value ≠ 1. | Open-source impl |
| 59 | https://github.com/StevenLooman/home-assistant-dlna-dmr (repo root) | DMR-side `protocolInfo` handling for `CurrentURIMetaData`. | Open-source impl |

### 10.4 In-workspace OBSERVED artifacts (fetched by a sibling agent; read locally)

| # | Path | What it contributed |
|---|---|---|
| 60 | `/workspace/research/raw/minidlna_upnphttp.c` | Real HTTP delivery behaviour: single-range parser, `206`/`416` handling, `Content-Range` formatting, `Accept-Ranges: bytes`, `Content-Length` always present, DLNA header emission, 406 on bad transfer mode. |
| 61 | `/workspace/research/raw/minidlna_upnpsoap.c` | Real ContentDirectory/ConnectionManager SCPD generation and DIDL-Lite emission; used to confirm which CD actions exist in the wild. |
| 62 | `/workspace/research/raw/minidlna_clients.c` | The real-world DMR fingerprint list (Xbox 360, PS3, Samsung CDE/Q/QN/BD, AllShare, Cling, Roku…) and the flags that change behaviour (`FLAG_DLNA`, `FLAG_SKIP_DLNA_PN`, `FLAG_NO_RESIZE`, `FLAG_CONVERT_MS`, `FLAG_CAPTION_RES`) — direct evidence that DMRs differ in what they accept. |
| 63 | `/workspace/research/raw/t_iAbadia_ReadyMedia_master_upnpsoap.c` | ReadyMedia's SOAP dispatcher: confirms CD:1 action set actually implemented, and that the requested "extension" actions are absent. |
| 64 | `/workspace/research/raw/gerbera_supported.txt` | The Gerbera "Supported Devices" list of DMRs and their quirk flags (Samsung DCM10/CAPTION_PROTOCOL/SAMSUNG_BOOKMARK_MSEC, etc.). |
| 65 | `/workspace/research/raw/PhairPlay/AGENTS.md` | The sibling project's renderer-side component map (`SsdpAdvertiser`, `SsdpResponder`, `DlnaDescriptionServer`, `GenaEventManager`, `GenaCallbackPolicy`, `DlnaProtocolInfo`), used for terminology consistency and to avoid duplicating the GENA SSRF guard. |

### 10.5 Sources deliberately **not** usable

* `https://upnp.org/specs/**.pdf` — direct download blocked by a JavaScript challenge from this environment (HTTP 200 with a 3 KB HTML interstitial). Text obtained via the search index; flagged throughout.
* `https://www.dlna.org/guidelines` — connection timed out; the DLNA Guidelines (IEC 62481) full text is not openly fetchable. All DLNA-guideline citations are therefore second-hand (Microsoft's extensions, which quote guideline numbers).
* `https://web.archive.org/web/...` — TLS connection failure from this environment.
* `https://r.jina.ai/...` — requires authentication (401).
