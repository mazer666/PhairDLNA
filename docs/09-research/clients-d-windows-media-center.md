# Windows Media Center (WMC) and Windows Media Player / "Media Player" as UPnP/DLNA Control Points
Status: **Research input — evidence, not norm** · Owner role: Interoperability Specialist · Version/date: 1.0 / 2026-11-14
> Committed so that every citation in the feature specifications resolves. Where this document and a requirement disagree, the requirement wins.

Research subject for **PhairDLNA** — a new pure-Rust DLNA/UPnP AV MediaServer.

**Scope note / honesty statement up front.** Windows Media Center itself is a *Digital Media Player* (DMP)
in Microsoft's DLNA profile, and Microsoft documents the DMP generically — there is **no separate
"WMC protocol"**. Almost everything Microsoft publishes about the WMC/WMP control point is published
under the names `[MS-DLNHND]` (Digital Media Player / Digital Media Server roles) and `[MS-UPMC]`
(Microsoft Compatibility Extension Flags). Windows Media Player 12 on Windows 7 and Windows 10/11
send byte-for-byte the *same* UA shape and the *same* SOAP shapes as the Windows 7 "Media Center"
app, because they share the same DMP stack (`WMPNSS`). Consequently almost every "WMC quirk" below is
either (a) a Microsoft-documented DMP requirement, or (b) an observed Windows-family control point
behaviour that is documented in server projects' issue trackers rather than by Microsoft.

**Label legend:** `STANDARD` = stated in a Microsoft Open Specification (primary source).
`OBSERVED` = directly witnessed in a primary source I fetched (server log, issue tracker, or source code).
`ASSUMPTION` = inferred, or only seen in a search snippet / behind a bot-block that I could not fetch.
`OPEN QUESTION` = not established by any source I could fetch.

---

### Windows Media Center (WMC)

#### What the client sends

- **STANDARD — WMC is the "Digital Media Player (DMP)" role of the Microsoft DLNA profile.**
  "DMP implementations also use the UPnP MediaServer V1.0 Device Control Protocol with the extensions
  defined in [DLNA] to request a list of multimedia content that is available for streaming from the server."
  — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/d40517ad-2d0a-4397-a46b-db7ba281a837

- **OBSERVED — SSDP M-SEARCH: the Windows control point searches for the *ContentDirectory service*,
  not (only) for the device.** A real miniDLNA debug log from a Windows 7 box:
  `minissdp.c:683: info: SSDP M-SEARCH from 192.168.1.200:65020 ST: urn:schemas-upnp-org:service:ContentDirectory:1, MX: 3, MAN: "ssdp:discover"`
  — https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4
  Note the low `MX: 3`. The same log also shows `ST: uuid:DIRECTV2PC-...` lines from other clients, so
  `MediaServer:1` / `MediaRenderer:1` M-SEARCHes come from the device-announcement side, not from WMC.
  **OPEN QUESTION** — I found **no** primary source showing WMC/WMP issuing `ST: urn:schemas-upnp-org:device:MediaServer:1`
  or `MediaRenderer:1`; the only device-level ST I can source for the Windows family is the one above.

- **OBSERVED — HTTP User-Agent, Windows 7 (the WMC/WMP12 era):**
  `User-Agent: Microsoft-Windows/6.1 UPnP/1.0 Windows-Media-Player/12.0.7601.17514 DLNADOC/1.50 (MS-DeviceCaps/1024)`
  Used for `GET /rootDesc.xml`, `GET /ContentDir.xml` and `POST /ctl/ContentDir`.
  — https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4

- **OBSERVED — the icon fetch uses a *different* UA:**
  `User-Agent: Windows-Media-Player-DMS/12.0.7601.17514` (same page, same log).

- **OBSERVED — HTTP User-Agent, Windows 10 (WMP 12):**
  `Microsoft-Windows/10.0 UPnP/1.0 Windows-Media-Player/12.0.19041.1320 DLNADOC/1.50 (MS-DeviceCaps/1024)`
  — https://github.com/gerbera/gerbera/issues/2215

- **OBSERVED — Windows 10 shell / Explorer as a DLNA control point uses a *third* UA:**
  `Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50`
  Gerbera's client-detection table carries this as a captured, commented UA and maps it to the generic
  "Standard UPnP" profile with **no** quirks enabled:
  `// User-Agent(actionReq): Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50`
  — https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc (also asserted in
  https://github.com/gerbera/gerbera/blob/master/test/util/test_upnp_clients.cc)

- **OBSERVED — WMC's own HTTP stack identifies itself as `Media Center PC 6.0`** inside an IE-style UA:
  `User-Agent: Mozilla/4.0 (compatible; MSIE 7.0; Windows NT 6.1; WOW64; Trident/7.0; SLCC2; .NET CLR 2.0.50727; .NET CLR 3.5.30729; .NET CLR 3.0.30729; Media Center PC 6.0; .NET4.0C; .NET4.0E; CMDTDF; MS-RTC LM 8)`
  with `Accept: application/x-ms-application, image/jpeg, application/xaml+xml, image/gif, image/pjpeg, application/x-ms-xbap, ...`.
  — https://docs.netscaler.com/us/en/citrix-adc/current-release/appexpert/rewrite/content-length-header-behaviour.html
  **Caveat:** this example is a *web page* request (EHTP shell), not a UPnP SOAP request. Do **not** use
  this string for UPnP client detection.

- **STANDARD — the Microsoft extension to the UPnP HTTP User-Agent is the `devicecaps` token:**
  ```
  User-Agent = "User-Agent: " user-agent-data
  user-agent-data = *( SP product )   ; RFC2616 section 3.8
                     SP DLNA-CP-version ; DLNA guideline 7.2.32
                     devicecaps
  devicecaps    = SP "(MS-DeviceCaps/" 1*10DIGIT ")"
  ```
  "The numerical value in the devicecaps token is a bitwise-OR combination of the compatibility flags
  defined in [MS-UPMC] section 2.2.1."
  — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/b85654cb-9685-49a0-9a8e-cd6e767b8df3
  and https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/6cf02c0e-7952-40de-b70d-33b02b0372b8

- **STANDARD — the numeric value the Windows DMP is supposed to put in `devicecaps` is `1024`:**
  "All UPnP Browse and search action invocations performed by the DMP, SHOULD include a User-Agent header
  as specified in section 2.2.4.1. The numerical value in the devicecaps token in the User-Agent header
  SHOULD be 1024."
  — [MS-DLNHND] §3.2.4.1, https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf
  `1024` = `%x400` = **`WMC_COMPAT_DO_NOT_LIMIT_RESPONSE_SIZE`** — i.e. the Windows control point
  explicitly asks the server *not* to apply the 200 kB SOAP cap. (Microsoft's own example DDD value is
  `94` = EXCLUDE_RTSP|EXCLUDE_DLNA|EXCLUDE_DLNA_1_5|EXCLUDE_PCMPARAMS|INCLUDE_RTSP_FOR_VIDEO —
  https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/6cf02c0e-7952-40de-b70d-33b02b0372b8)

- **OPEN QUESTION — the "0100-1999" style header.** There is **no** `0100-1999` literal anywhere in
  [MS-UPMC] or [MS-DLNHND] (I grepped the full extracted text of both PDFs; the only `1999` hit is
  "RFC2616, June 1999"). The closest real constructs are (a) the `devicecaps` token above, whose value
  is a 1–10 decimal digit number (observed in the wild: `1024` for Windows 7 and Windows 10 WMP, and
  `1024` for Samsung AllShare-on-PC — `DLNADOC/1.50 SEC_HHP_[PC]LPC001/1.0  MS-DeviceCaps/1024`,
  https://github.com/necropotame/readymedia/blob/master/clients.c), and (b) the `microsoft:X_DeviceCaps`
  decimal element in the DDD (example: `94`). If "0100-1999" refers to a header range, **I could not
  establish it** and it must not be implemented on my say-so.

- **STANDARD — how the DMP decides to use `Search` at all:**
  "The DMP SHOULD determine if the UPnP ContentDirectory service on the DMS supports the search action,
  by examining the Service Description Document of the ContentDirectory service… If the DMS supports the
  search action, then the search action SHOULD be used to search the DMS for music and video content,
  as well as pictures. If the DMS does not support the search action, then the DMP MUST use the Browse
  action instead."
  — [MS-DLNHND] §3.2.4.1, https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf
  ⇒ **The server must advertise `Search` in its ContentDirectory SCPD if it wants WMC/WMP to use it.**

- **STANDARD — the DMP uses a magic `ObjectID` to find playlists when the server looks Microsoft-made:**
  "If the value of the Microsoft-CDS variable is True, then the DMP SHOULD search for playlist containers
  by invoking the Browse action and specifying "13" as the value for the ObjectID parameter."
  `Microsoft-CDS` is set True when "the manufacturer XML tag in the UPnP Device Description Document of
  the DMS contains the word "Microsoft"".
  Server-side mirror rule: "If the value of the ObjectID parameter in a Browse action is 13, then the DMS
  SHOULD treat this as Browse action for a ContentDirectory service container that contains all playlist
  containers." ([MS-DLNHND] §3.4.5.1). ⇒ **A server must handle `Browse(ObjectID="13")` and return
  `object.container.playlistContainer` objects.**

- **OBSERVED — eventing:** the Windows client subscribes to the ContentDirectory event channel with
  `NT: upnp:event`, `Timeout: Second-1800`, `Callback: <http://192.168.1.200:2869/upnp/eventing/izfroyrldf>`
  — https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4
  (Note the callback port `2869`.)

- **OBSERVED — SearchCriteria actually sent by the Windows family.** Windows Media Player sends the
  search criterion **`@refID exists false`** (i.e. "everything that is not a reference item"), which
  gerbera did not support and had to implement:
  — https://github.com/gerbera/gerbera/issues/1407 ("Windows Media Player does not display files")
  **ASSUMPTION** — the *full* Windows search string
  `upnp:class derivedfrom "object.item.videoItem" and @refID exists false` is visible only in search
  snippets for a Cloudflare-protected SourceForge miniDLNA thread
  (https://sourceforge.net/p/minidlna/discussion/879957/thread/fa4b2f6d/ — **I could not fetch this page**,
  it returns a Cloudflare "Just a moment..." interstitial).
  **OBSERVED** — the *identical* combined string
  `upnp:class derivedfrom "object.item.videoItem" and @refID exists false` **is** confirmed for an LG
  WebOS TV client, fetched at
  https://sites.google.com/site/tweakradje/devices/minidlna-readymedia
  ⇒ treat the combined Windows string as *highly likely but unverified for WMC*.

- **OBSERVED — SortCriteria actually sent by the Windows family.** WMP sends `+dc:title` and
  `+upnp:originalTrackNumber,+dc:title`:
  ```
  warning: Unknown sort key 'dc:title' in '+dc:title',
  warning: Unknown sort key 'dc:title' in '+upnp:originalTrackNumber,+dc:title',
  ```
  — https://github.com/gerbera/gerbera/issues/2215
  ⇒ **The server must accept `dc:title` and `upnp:originalTrackNumber` as sort keys, and must not choke
  on a `SortCriteria` it does not know.**
  **ASSUMPTION** — `RequestedCount=200`, `StartingIndex=0` and `Filter=*` for Windows search requests
  (seen in the unfetchable SourceForge snippet only; the confirmed LG log shows `Count: 30`,
  `StartingIndex: 30` and a long attribute filter list).

- **STANDARD — the DMP also browses folders using the Microsoft media property `microsoft:folderPath`:**
  `microsoft:folderPath` is defined in [MS-UPMC] §2.1.7 and is a "property of the media item"; it is
  listed as an *MMPE* (Media Property Extension) and therefore is legal in DIDL-Lite.
  — https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D.pdf
  **ASSUMPTION** — that WMP *searches* on `dc:title,microsoft:folderPath` (only in the unfetchable SF snippet).

#### What it chokes on

- **STANDARD — the 204,800-byte (200 kB) SOAP response cap.** [MS-DLNHND] §3.4.5.1:
  "If the value of the `WMC_COMPAT_DO_NOT_LIMIT_RESPONSE_SIZE` bit in the CompatFlags variable is 0, then
  the size of the SOAP response to the Browse or Search action SHOULD NOT exceed 204,800 bytes."
  Footnote 15 of the same document: "Windows Vista does not limit the size of the SOAP response."
  — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/28e72c9b-225b-42ec-8891-bc7f7e220e8e
  and https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf
  ⇒ **A server that does not implement the `MS-DeviceCaps` negotiation will silently truncate WMC's
  Browse/Search responses** — the classic "the folder list stops halfway / shows no files" symptom.

- **STANDARD — the complete `WMC_COMPAT_*` flag set (15 flags).** The authoritative list and the exact
  value of each flag, from [MS-UPMC] §2.2.1 (index at
  https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/cb53cb44-4965-4056-b626-2b9526657499 ;
  full text at https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D.pdf):

  | Flag | Value | What it forces the SERVER to do |
  |---|---|---|
  | `WMC_COMPAT_EXCLUDE_HTTP` | `%x1` | Drop every `res` / `upnp:albumArtURI` whose `protocolInfo` starts with `http-get`. Affects Browse, Search, GetProtocolInfo. **MUST NOT** be combined with EXCLUDE_RTSP. |
  | `WMC_COMPAT_EXCLUDE_RTSP` | `%x2` | Drop every `res` whose `protocolInfo` starts with `rtsp-rtp-udp` (ASF/MP3/LPCM over RTSP). **MUST NOT** be combined with EXCLUDE_HTTP or INCLUDE_RTSP_FOR_VIDEO. |
  | `WMC_COMPAT_EXCLUDE_DLNA` | `%x4` | Remove **all** DLNA attributes/params: `dlna:profileID`, `xmlns:dlna` on `upnp:albumArtURI`, and `DLNA.ORG_PN`, `DLNA.ORG_OP`, `DLNA.ORG_PS`, `DLNA.ORG_CI`, `DLNA.ORG_FLAGS`, `DLNA.ORG_MAXSP` from `protocolInfo`. Also: GetProtocolInfo MUST NOT announce DLNA profiles. Reason given: *"Because some devices fall down if the string DLNA exists in the protocol response…"* |
  | `WMC_COMPAT_EXCLUDE_DLNA_1_5` | `%x8` | Map DLNA profile `MP3X`→`MP3`, `WMVSPLL_BASE`/`WMVSPML_BASE`→`WMVMED_BASE`; omit any `WMDRM_*` profile ID; **"When this flag is used, all URLs MUST have a file extension, such as ".mp3" for MP3 content and ".pcm" for LPCM content."** Also: `desc` XML tags are escaped / not escaped depending on the variant. |
  | `WMC_COMPAT_EXCLUDE_PCMPARAMS` | `%x10` | For `audio/L16` and `audio/L8`, `protocolInfo` MUST NOT carry `rate=` / `channels=` parameters. |
  | `WMC_COMPAT_EXCLUDE_WMDRMND` | `%x20` | Do not emit `res` tags whose URL needs WMDRM-ND. |
  | `WMC_COMPAT_INCLUDE_RTSP_FOR_VIDEO` | `%x40` | Emit RTSP `res` URLs for video **even if** EXCLUDE_RTSP / EXCLUDE_DLNA_1_5 are set. |
  | `WMC_COMPAT_EXCLUDE_WMALOSSLESS_NONTRANSCODED` | `%x80` | Do not emit `res` tags for non-transcoded WMA Lossless. |
  | `WMC_COMPAT_EXCLUDE_SEARCH` | `%x100` | **ContentDirectory:Search is not supported**; `GetSearchCapabilities` MUST return no capabilities. |
  | `WMC_COMPAT_DO_NOT_LIMIT_RESPONSE_SIZE` | `%x400` | DIDL-Lite responses are not limited to 200 kB. |
  | `WMC_COMPAT_EXCLUDE_VIDEO_TRANSCODING` | `%x800` | Do not offer transcoded video `res` tags. |
  | `WMC_COMPAT_PLAYLIST_FAKECHILDCOUNT` | `%x1000` | Return `childCount="1"` for playlist containers (perf optimisation). |
  | `WMC_COMPAT_EXCLUDE_NONPCM_AUDIO_TRANSCODING` | `%x2000` | Do not offer transcoded audio `res` tags, except LPCM. |
  | `WMC_COMPAT_EXCLUDE_TRANSCODING_TO_MPEG2` | `%x4000` | Do not offer MPEG-2-transcoded video `res` tags. |
  | `WMC_COMPAT_EXCLUDE_RES_FILTERING` | `%x8000` | **All** applicable transcoding `res` elements MUST be present in every DIDL-Lite response. |

  Per-flag Learn pages verified fetchable:
  - `WMC_COMPAT_EXCLUDE_DLNA` — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/c12f5530-fa2d-4d9b-a960-18f4e9678100
  - `WMC_COMPAT_EXCLUDE_DLNA_1_5` — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/0941c244-0d05-4d7d-9a61-745af114d5cf
  - `WMC_COMPAT_EXCLUDE_RTSP` — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/1f78a997-225c-4598-a5a7-66f749240adc
  - `X_DeviceCaps` example / flag decode — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/6cf02c0e-7952-40de-b70d-33b02b0372b8

- **STANDARD — the *server-side decision algorithm* for these flags** ([MS-DLNHND] §3.4.5.1,
  https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/28e72c9b-225b-42ec-8891-bc7f7e220e8e):
  1. Default: `WMC_COMPAT_EXCLUDE_DLNA_1_5 = 1`, **all other bits 0**.
  2. If the control point's DDD says manufacturer "Microsoft"/"Microsoft Corporation" and modelName
     "Windows Media Player" → SHOULD set `EXCLUDE_DLNA_1_5` **and** `INCLUDE_RTSP_FOR_VIDEO`.
  3. If the CP's DDD was not found → MUST set `INCLUDE_RTSP_FOR_VIDEO`.
  4. Parse the UA per §2.2.4.1. `DLNA-CP-version` token `1.00` → MUST set `EXCLUDE_RTSP`.
     `1.50` → MUST clear `EXCLUDE_DLNA_1_5`. First char 2–9 → MUST clear `EXCLUDE_DLNA_1_5`.
  5. `devicecaps` token present → SHOULD set CompatFlags to that number (MAY ignore it).
  6. No UA / no devicecaps / ignored → use `microsoft:X_DeviceCaps` from the CP's DDD.
  7. Cascade rules: `EXCLUDE_DLNA=1` ⇒ `EXCLUDE_DLNA_1_5=1`.
     `EXCLUDE_DLNA_1_5=1` ⇒ `EXCLUDE_RTSP=1` **and** `DO_NOT_LIMIT_RESPONSE_SIZE=1`.
     `EXCLUDE_HTTP=1 && EXCLUDE_RTSP=1` ⇒ `EXCLUDE_HTTP=0`.
     `EXCLUDE_RES_FILTERING=1` ⇒ clear EXCLUDE_WMALOSSLESS_NONTRANSCODED / EXCLUDE_VIDEO_TRANSCODING /
     EXCLUDE_NONPCM_AUDIO_TRANSCODING / EXCLUDE_TRANSCODING_TO_MPEG2.
  8. `EXCLUDE_DLNA=1` ⇒ the **4th field of every `protocolInfo`** SHOULD be `"*"`.
  9. `EXCLUDE_DLNA_1_5=1` ⇒ drop `pn=` values starting with `WMDRM_`; `MP3X`→`MP3`;
     `WMVSPLL_BASE`/`WMVSPML_BASE`→`WMVMED_BASE`; escape (or not) nested `desc` XML tags.
  10. `EXCLUDE_RTSP=0` ⇒ the response SHOULD contain **RTSP URLs** for ASF, MP3 and LPCM content.
     `INCLUDE_RTSP_FOR_VIDEO=1` ⇒ SHOULD contain RTSP URLs for ASF video.
  ⇒ **Practical consequence for PhairDLNA:** implement at minimum the parsing of the
  `(MS-DeviceCaps/N)` token, honour `EXCLUDE_DLNA` (strip the literal string "DLNA"), honour
  `EXCLUDE_RTSP`/`INCLUDE_RTSP_FOR_VIDEO` (RTSP is optional — a pure-HTTP server may simply have no
  RTSP URLs to exclude, which is compliant), honour `EXCLUDE_SEARCH` (return empty
  `GetSearchCapabilities`), and honour `DO_NOT_LIMIT_RESPONSE_SIZE` (do not cap at 204,800 bytes).

- **STANDARD — `ContentDirectory` service version.** [MS-DLNHND] and [MS-UPMC] reference only
  **[UPNPCDS1] — "ContentDirectory:1 Service Template Version 1.01"** as the normative service
  template, and the DMP is specified against `ContentDirectory` service semantics of that version.
  — https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D.pdf
  **OBSERVED** — every Windows control-point trace I fetched uses
  `urn:schemas-upnp-org:service:ContentDirectory:1` exclusively, for `Browse`, `Search`,
  `GetSearchCapabilities`, `GetSortCapabilities` and `GetSystemUpdateID`
  (https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4 ;
  https://github.com/gerbera/gerbera/issues/2215).
  **OPEN QUESTION** — I found **no** source showing WMC probing `ContentDirectory:2` (or :3/:4). Treat
  `ContentDirectory:1` as the required service and do not assume :2 support is probed.

- **STANDARD — WMC/WMP DO support `Search`.** See the §3.2.4.1 rule above plus the observed
  `ContentDirectory:1#Search` SOAPActions with `SearchCapabilities` in the SCPD
  (https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4).
  ⇒ A server that omits `Search` from its SCPD makes WMC fall back to `Browse` (which is legal but
  produces a different, flatter tree).

- **STANDARD — `upnp:class` requirements.** Microsoft's own spec never pins a class value for video;
  the DMP *searches* by `derivedfrom`, i.e. class **prefix** matching. The `derivedfrom` semantics
  ("true for all objects whose class name begins with 'object.item.'") are defined in the UPnP
  ContentDirectory:1 template, https://upnp.org/specs/av/UPnP-av-ContentDirectory-v1-Service.pdf
  (mirrored at http://rfc.nop.hu/upnp/ContentDirectory1.0.pdf).
  **OPEN QUESTION** — I found **no** primary source showing WMC *requiring*
  `object.item.videoItem.movie` specifically. The observed Windows search criterion uses the base class
  `object.item.videoItem`. For PhairDLNA: emit the most specific class you can
  (`object.item.videoItem.movie`) — it satisfies both `derivedfrom "object.item.videoItem"` and
  `derivedfrom "object.item"` — and never emit `object.item.videoItem.movie` as the *only* answer to a
  class query, because `derivedfrom` matches by prefix.

- **STANDARD — `protocolInfo`.** [MS-UPMC] §2.2.1.3/§2.2.1.4 and [MS-DLNHND] §3.4.5.1 require the
  4th field to be `"*"` when `EXCLUDE_DLNA` is set, and to contain `pn=`/`op=`/`ci=`/`flags=` DLNA
  parameters otherwise. The DLNA guidelines (not free, referenced as `[DLNA]`) define
  `protocolInfo="http-get:*:video/mp4:*"` as the mandatory baseline form.
  **OPEN QUESTION** — I found **no** primary source proving (a) that WMC does *exact, case-sensitive*
  string matching on `protocolInfo`, or (b) that WMC *requires* the DLNA profile suffix
  (`DLNA.ORG_PN=...`) on every item. The one hard, citable statement is Microsoft's own:
  *"Because some devices fall down if the string DLNA exists in the protocol response…"* — which is
  about the **presence of the substring "DLNA"**, not about matching. Safe engineering rule: emit
  `http-get:*:<mime>:*` by default, and only add DLNA 4th-field parameters when the client did not
  ask you to strip them.

- **STANDARD — `res@duration`, `dc:date`, XML escaping.** [MS-UPMC] §2.1 specifies the MMPE
  properties (`microsoft:year`, `microsoft:folderPath`, …) and states the general DIDL-Lite rules:
  "The XML tags MUST follow the syntax rules for XML defined in [XML10]. Unless otherwise specified, any
  characters in the value of an XML tag MUST be escaped, using the XML escaping mechanism defined in
  [XML10] section 2.4." ([MS-UPMC] §2.1). `dc:date` is a plain UPnP/DIDL-Lite property; Microsoft's
  `microsoft:year` exists *because* `dc:date` carries the whole date
  (https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D.pdf).
  **OPEN QUESTION** — no source found stating that WMC *fails* without `dc:date` or without
  `res@duration`. Gerbera nevertheless exposes a `SIMPLE_DATE` client quirk ("Return `dc:date` without
  timezone as Zulu time") — https://docs.gerbera.io/en/latest/supported-devices.html — which suggests
  date formatting does matter to *some* DLNA clients, but no WMC-specific statement exists.

- **OPEN QUESTION — `sec:httpDLNA` / `GetFeatureList`.** The `sec:` (SEC) namespace, `sec:httpDLNA*`
  feature lists and the `GetFeatureList` action are **DLNA/SEC-company extensions, not Microsoft
  specifications**, and I found no Microsoft primary source requiring them from WMC. They must not be
  treated as a WMC requirement.

- **OPEN QUESTION — HTTP/1.0 vs 1.1, chunked transfer-encoding, `Content-Length` mandatory-ness,
  206 vs 200 on range requests, suffix ranges `bytes=-N`, `416`.** Microsoft's DLNA extension
  documents ([MS-DLNHND]) defer all of this to the (paywalled) DLNA guidelines:
  "The DMS MUST process requests to stream content using HTTP according to the rules specified in
  [DLNA] guideline 7.4.1." ([MS-DLNHND] §3.4.5.2). The only Microsoft-specific streaming requirement I
  could source is:
  - **STANDARD** — "If the HTTP response status code is 200 and the DMS supports the DLNA Seek Media
    Operation using the TimeSeekRange.dlna.org HTTP header … then the DMS SHOULD include the
    `X-AvailableSeekRange` header in the response" ([MS-DLNHND] §3.4.5.2 + §2.2.4.2,
    https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf).
  - **STANDARD** — "The client SHOULD parse the `X-AvailableSeekRange` header, if it is present… If the
    `X-AvailableSeekRange` header is present, but is syntactically incorrect, then it MUST be ignored."
    ([MS-DLNHND] §3.1.5.1)
  - **OBSERVED (server behaviour, not a WMC rule)** — ReadyMedia returns **206 + `Content-Range`** when
    a `Range:` header is present and **400** for an out-of-bounds start, **416** for an end past EOF:
    https://github.com/necropotame/readymedia/blob/master/upnphttp.c
    (`start_dlna_header(&str, (h->reqflags & FLAG_RANGE ? 206 : 200), …)`).
  - **OBSERVED (server behaviour)** — ReadyMedia's `Range` parser does **not** support suffix ranges:
    `h->req_RangeStart = strtoll(p+6, &colon, 10);` on `bytes=-500` yields `-500`, which then trips
    `if ((h->req_RangeStart > h->req_RangeEnd) || (h->req_RangeStart < 0)) { … Send400(h); }`
    (same file, `ParseHttpHeaders`). ⇒ **If PhairDLNA wants to be safe with Windows-family clients,
    implement suffix ranges (`bytes=-N`) and answer 206/`Content-Range`, and never 400 a legal Range.**
  - **OPEN QUESTION** — no source found proving WMC rejects chunked responses or requires
    `Content-Length` on media GETs. Note that Windows' own HTTP stack historically breaks when
    `Content-Length` is mangled (the NetScaler "Content-Length header renaming" page above documents
    a WMC-family client breaking on a corrupted `Content-Length` header) — that is weak evidence that
    this client family parses `Content-Length` strictly.

#### Documented failure symptoms

- **OBSERVED — "Windows Media Player does not display files" (files visible to the server, invisible to the client).**
  Symptom (Windows 7 64-bit Pro, WMP): the UPnP server is discovered, but **no files are listed**.
  Root cause: WMP issued the search criterion **`@refID exists false`**, and gerbera did not
  implement the `exists` operator over `@refID`; gerbera also imported its library "in chunnks of 200",
  and WMP builds its own cache from the results.
  — https://github.com/gerbera/gerbera/issues/1407 (fixed by https://github.com/gerbera/gerbera/pull/1408)
  ⇒ **Server-side lesson: if you do not implement the `exists` operator on `@refID`, WMP shows an empty library.**

- **OBSERVED — WMP's `SortCriteria` breaks naive servers.**
  ```
  warning: Unknown sort key 'dc:title' in '+dc:title',
  warning: Unknown sort key 'dc:title' in '+upnp:originalTrackNumber,+dc:title',
  error:   no such table: INNER
  error:   [sqlite_database.cc:242] select(): prematurely shutting down.
  ```
  — https://github.com/gerbera/gerbera/issues/2215
  ⇒ **Server-side lesson: never let an unknown sort key reach the SQL layer; ignore it and sort by
  something safe.**

- **OBSERVED — WMC indexing is slow and partial.**
  "Generating the index took an excessive amount of time. The index was generated only in parts rather
  than all at once." (Windows Media Center, against a simpleDLNA server with a very large music library.)
  WMP in the same report: "The displayed list constantly jumps everywhere", with the reporter concluding
  this is how Windows indexes the data.
  — https://github.com/nmaier/simpleDLNA/issues/24
  ⇒ **Server-side lesson: the Windows client builds its own index; large libraries need paging and a
  server that answers every `Search` quickly, or the UI will look half-populated.**

- **OBSERVED — the 200 kB cap manifests as "the library stops part-way".** The cap is a normative
  Microsoft SHOULD for exactly the case where the client did *not* send
  `(MS-DeviceCaps/1024)`; [MS-DLNHND] §3.4.5.1 + footnote 15.
  — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/28e72c9b-225b-42ec-8891-bc7f7e220e8e
  ⇒ **Server-side lesson: parse `MS-DeviceCaps` and, when it is 1024 (or has bit `%x400`), do not cap
  the SOAP response at 204,800 bytes.**

- **OBSERVED — "no folders" in the Windows DLNA client.** A Microsoft Q&A thread where the Windows 10
  DLNA client shows the shared folders but no files, and the accepted explanation is the WMP cache
  directory plus library/permission state:
  — https://learn.microsoft.com/en-us/answers/questions/2109766/win-10-dnla-clients-folders-showing-but-not-files
  ⇒ This is a *client-side* symptom with a *client-side* fix; it is listed here so PhairDLNA's
  maintainers do not chase it as a server bug.

- **OBSERVED — Microsoft itself documents that "Windows Media Player downloads all item info and builds up cache"**
  as a client characteristic, listed by gerbera under its "Microsoft" device section:
  - Windows Explorer (Browse in directory and play with selectable player)
  - Windows MediaPlayer 12 (downloads all item info and builds up cache)
  — https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst and
    https://docs.gerbera.io/en/latest/supported-devices.html
  ⇒ **The Windows client is a *pull-everything* client: expect many `Search`/`Browse` calls and an
  `ObjectID=13` playlist browse, and make them fast.**

- **OPEN QUESTION — "cannot seek" / "stops after N seconds" / "shows files but no playback"** as
  *WMC-specific* reports. I found no server-tracker issue that isolates these to WMC. The closest
  real reports are generic DLNA clients (e.g. Serviio: "Some shows would stop in the middle… Serviio
  log just shows that the receiving device stopped playing" —
  https://forum.serviio.org/viewtopic.php?f=11&t=30766) and are **not** WMC-specific.

#### Server-side workarounds observed in existing implementations

**ReadyMedia / miniDLNA** (`https://github.com/necropotame/readymedia`, formerly
`https://github.com/samacleese/minidlna`):

- **OBSERVED — there is NO `Windows Media Center` client entry.** `git log -S "Media Center"` over the
  full history of the ReadyMedia/miniDLNA tree returns **nothing**. The Microsoft-family entries are:
  - `clients.c` — `EMediaRoom` ("MS MediaRoom") matched on UA `Microsoft-IPTV-Client`;
    `FLAG_MS_PFS` on `EXbox` ("Xbox/"), `ENetgearEVA2000`, `ERokuSoundBridge`, `EMediaRoom`, `ELifeTab`;
    and a generic `{ 0, FLAG_DLNA | FLAG_MIME_AVI_AVI, "Windows", "FDSSDP", EUserAgent }` entry.
    — https://github.com/necropotame/readymedia/blob/master/clients.c
    (`NEWS` for 1.1.6: "Add AllShare and Windows client detection." — https://github.com/necropotame/readymedia/blob/master/NEWS)
  - `clients.h` — `#define FLAG_MS_PFS 0x00000080 /* Microsoft PlaysForSure client */`
    — https://github.com/necropotame/readymedia/blob/master/clients.h

- **OBSERVED — the PlaysForSure bitrate workaround** (this is the classic Microsoft-family quirk):
  ```c
  if( bitrate && (args->filter & FILTER_RES_DURATION) ) { ... }
  if(args->flags & FLAG_MS_PFS)
          br /= 8;      /* bitrate is reported in bits/s by the DB, Microsoft wants kbit/s */
  ```
  — `upnpsoap.c` (around the `res` attribute emitter) —
  https://github.com/necropotame/readymedia/blob/master/upnpsoap.c
  (`NEWS` 1.0.22: "Fix bitrate displayed on Microsoft PFS devices." — https://github.com/necropotame/readymedia/blob/master/NEWS)

- **OBSERVED — PlaysForSure album-art handling:**
  - In `upnpsoap.c`, a video's album art is **not** offered as a second `<res>` for PFS clients:
    ```c
    if( *mime == 'v' && (passed_args->filter & FILTER_RES) && !(passed_args->flags & FLAG_MS_PFS) ) { … }
    ```
  - And PFS image items get a synthetic `<upnp:album>` when there is no album:
    ```c
    if( (passed_args->flags & FLAG_MS_PFS) && *mime == 'i' ) {
            if( passed_args->client == EMediaRoom && !album )
                    ret = strcatf(str, "&lt;upnp:album&gt;%s&lt;/upnp:album&gt;", "[No Keywords]");
    ```
  - https://github.com/necropotame/readymedia/blob/master/upnpsoap.c
  ⇒ **Server-side lesson: the Microsoft family wants a single video `res` plus an `upnp:albumArtURI`,
  and it wants an `<upnp:album>` to be present even when empty.**

- **OBSERVED — the PFS `?albumArt=true` URL convention** is honoured in `upnphttp.c`:
  ```c
  if( cflags & FLAG_MS_PFS )
  {
          if( strstr(object, "?albumArt=true") )
          { /* serve the album art straight from the DB, else 404 */ }
  }
  ```
  — https://github.com/necropotame/readymedia/blob/master/upnphttp.c

- **OBSERVED — ReadyMedia's magic containers are flagged `FLAG_MS_PFS`** so the Microsoft family gets
  its own container tree — `containers.c` lines ~103–114 —
  https://github.com/necropotame/readymedia/blob/master/containers.c

- **OBSERVED — ReadyMedia hard-codes the Microsoft-compatible model name** so that Windows' own DMP
  treats it as a Microsoft DMS:
  ```c
  snprintf(modelname, MODELNAME_MAX_LEN, "Windows Media Connect compatible (%s)", val);
  ```
  — `minidlna.c` — https://github.com/necropotame/readymedia/blob/master/minidlna.c
  ⇒ **Important for PhairDLNA:** this is a *deliberate* trick. Microsoft's own DMP sets
  `Microsoft-CDS = True` when "the manufacturer XML tag … contains the word 'Microsoft'"
  ([MS-DLNHND] §3.2.7.1), which switches playlist lookup from `Search` to `Browse(ObjectID="13")`.
  Naming yourself "Windows Media Connect compatible" therefore changes the Windows client's whole
  browsing algorithm. That is a **behavioural fork**, not a compatibility win — decide it consciously.

- **OBSERVED — ReadyMedia implements `Search` with the full Windows search grammar**, including
  `@refID`, `@parentID`, `@id` as searchable properties and the `exists` operator, and translates
  `upnp:class derivedfrom "object.item.videoItem" and @refID exists false` into
  `o.CLASS like "item.videoItem%" and REF_ID is NULL` —
  `upnpsoap.c` — https://github.com/necropotame/readymedia/blob/master/upnpsoap.c
  (the `FILTER_REFID` handling and the `Translated SearchCriteria` debug line are visible in the
  server logs quoted above).

- **OBSERVED — ReadyMedia does NOT implement `MS-DeviceCaps`/`CompatFlags` at all.** `grep -rn
  "MS-DeviceCaps\|DeviceCaps\|WMC_COMPAT"` over the whole tree returns only the `AllShare` comment
  that quotes a *client's* UA. ⇒ ReadyMedia ignores the `devicecaps` token and never applies
  `WMC_COMPAT_EXCLUDE_DLNA` — it just always emits DLNA parameters. It gets away with it because the
  Windows clients it was tested against did not object.

**Gerbera** (`https://github.com/gerbera/gerbera`):

- **OBSERVED — PR #1408 "Add support for Windows Media Player"** is the single most WMC/WMP-relevant
  change in any open-source DLNA server I found. It:
  1. Added the Microsoft search properties to the SQL emitter:
     ```cpp
     const static std::map<std::string, std::string> propertyLowerStatement = {
         { "metadata", "{0}{1}{0}.{0}property_name{0}='{3}' AND LOWER({0}{1}{0}.{0}property_value{0})" },
         { "@refID",   "LOWER({0}{2}{0}.{0}ref_id{0})" },
         { "@parentID","LOWER({{0}{2}{0}.{0}parent_id{0})" },
         { "@id",      "LOWER({{0}{2}{0}.{0}id{0})" },
     };
     ```
  2. Added the `exists` / `@exists` and `contains` / `derivedfrom` operators:
     ```cpp
     { "exists",    "({3} IS {5} AND {0}{2}{0}.{0}upnp_class{0} IS NOT NULL)" },
     { "@exists",   "({3} IS {5})" },
     { "derivedfrom", "(LOWER({0}{2}{0}.{0}upnp_class{0}) LIKE LOWER('{5}%'))" },
     ```
  3. Documented the Microsoft clients in `doc/supported-devices.rst`:
     ```
     Microsoft
     ~~~~~~~~~
     -  Windows Explorer (Browse in directory and play with selectable player)
     -  Windows MediaPlayer 12 (downloads all item info and builds up cache)
     ```
  — https://github.com/gerbera/gerbera/pull/1408/files
    (diff: https://patch-diff.githubusercontent.com/raw/gerbera/gerbera/pull/1408.diff ;
     files touched: `src/database/search_handler.cc`, `doc/supported-devices.rst`)
  ⇒ **Server-side lesson: implement `@refID`, `@parentID`, `@id` as searchable properties and the
  `exists` operator, or WMP shows an empty library.**

- **OBSERVED — gerbera's client table gives the Windows family NO special quirks.** The Windows 10
  shell UA `Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50` maps to the "Standard UPnP"
  profile with `Quirk::None`:
  ```cpp
  // User-Agent(actionReq): Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50
  { "Standard UPnP", DEFAULT_CLIENT_GROUP, ClientType::StandardUPnP,
    ClientConfig::getFlags({ Quirk::None }), ClientMatchType::UserAgent, "UPnP/1.0" },
  ```
  — https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc
  (asserted in https://github.com/gerbera/gerbera/blob/master/test/util/test_upnp_clients.cc)

- **OBSERVED — gerbera's `upnp:class` constants** include both the base and the specific classes:
  `#define UPNP_CLASS_VIDEO_ITEM "object.item.videoItem"` and
  `#define UPNP_CLASS_VIDEO_MOVIE "object.item.videoItem.movie"` —
  https://github.com/gerbera/gerbera/blob/master/src/upnp/upnp_common.h
  ⇒ **Gerbera's choice (emit the specific class, search works either way because `derivedfrom` is a
  prefix match) is the safe default for PhairDLNA.**

**Other servers:**

- **OBSERVED — *no* open-source DLNA server implements `MS-DeviceCaps` / `WMC_COMPAT_*`.**
  A Sourcegraph code search for `WMC_COMPAT` and `MS-DeviceCaps` across all of GitHub returns **0
  matches**; `DeviceCaps` only matches unrelated projects (e.g. `crates/sdroxide-types/src/caps.rs`).
  (`https://sourcegraph.com/search?q=context:global+WMC_COMPAT` — searched 2026-11-14.)
  ⇒ **PhairDLNA can be first-class here at low cost: implement the `MS-DeviceCaps` parsing and the
  15 `WMC_COMPAT_*` flags.**
- **OBSERVED — closed-source servers (Plex, Serviio, Twonky, Emby, UMS) document no WMC-specific
  behaviour at all.** A cross-server survey of DLNA Search support lists all of them as "closed source,
  unverified" and records for "Windows Media sharing" only: *"Responses capped at 204,800 bytes unless
  the client sets a compatibility flag ✔ Microsoft spec MS-DLNHND"* —
  https://github.com/peter--s/dlna-dms-music-assistant-notes/blob/main/README.md
- **OBSERVED — Serviio's own log format shows the shape of a Windows-style `Browse` call**
  (filter listing `res@resolution,res@protocolInfo,res@size,res@duration,res@bitrate,…,dc:creator,dc:date,…,upnp:albumArtURI,…`,
  `startIndex = 0, count = 1`) though that particular client was logged as
  "Unrecognized device (profile 1) / Generic DLNA profile", i.e. **not** positively identified as
  Windows — https://forum.serviio.org/viewtopic.php?f=27&t=9634

---

### Windows Media Player / "Media Player" (Windows 10/11)

**Honesty note.** There is **no Microsoft Open Specification that describes "Windows Media Player" or
the Windows 10/11 "Media Player" app as a control point.** Everything Microsoft publishes is the
generic DMP role in [MS-DLNHND], which WMP12/WMC implement. WMP is therefore barely documented as a
distinct client, and in practice it **uses the same code path as WMC** — the same UA family, the same
`devicecaps` token, the same `Search`/`Browse`/`ObjectID=13` behaviour. The differences below are
mostly *version* differences, not *architecture* differences.

**What the client sends**

- **OBSERVED — Windows 7 / Windows 10 WMP 12 UA (the canonical Windows control-point UA):**
  `Microsoft-Windows/6.1 UPnP/1.0 Windows-Media-Player/12.0.7601.17514 DLNADOC/1.50 (MS-DeviceCaps/1024)`
  — https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4
- **OBSERVED — Windows 10 WMP 12 UA:**
  `Microsoft-Windows/10.0 UPnP/1.0 Windows-Media-Player/12.0.19041.1320 DLNADOC/1.50 (MS-DeviceCaps/1024)`
  — https://github.com/gerbera/gerbera/issues/2215
  ⇒ **The `devicecaps` value is `1024` on both Windows 7 and Windows 10 — i.e. the Windows control
  point asks the server not to apply the 200 kB cap, consistently, since at least 2011.**
- **OBSERVED — Windows 10 shell / Explorer / "Play on" UA:**
  `Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50` (no `devicecaps` token) —
  https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc
  ⇒ **A server must handle *both* UA shapes: with and without `(MS-DeviceCaps/N)`.**
- **STANDARD — the `DLNADOC/1.50` token is the DLNA-CP-version token of [MS-DLNHND] §2.2.4.1**, and
  per §3.4.5.1 a `dlna-version` of `1.50` MUST cause the server to **clear**
  `WMC_COMPAT_EXCLUDE_DLNA_1_5` — i.e. the *older* Windows behaviour (strip DLNA 1.5 stuff) applies to
  `1.00` clients, while `1.50` clients get the full DLNA 1.5 metadata.
  — https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/28e72c9b-225b-42ec-8891-bc7f7e220e8e
- **STANDARD — the same `Search`/`Browse`/`ObjectID=13` rules as WMC** (§3.2.4.1, §3.2.7.1 above) apply
  verbatim to WMP — https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf
- **OBSERVED — WMP's `GetSortCapabilities` result is echoed back verbatim by the server**, and WMP
  then asks for `SortCriteria: +dc:title` / `+upnp:originalTrackNumber,+dc:title` —
  https://github.com/gerbera/gerbera/issues/2215
  ⇒ **Advertise `dc:title` and `upnp:originalTrackNumber` in `GetSortCapabilities`; otherwise WMP
  will send you sort keys you never advertised.**
- **STANDARD — a Windows client that does *not* send `devicecaps` gets the 200 kB cap applied**
  (Windows Vista behaviour per the [MS-DLNHND] footnote 15) —
  https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/ee214164-6314-4430-9ba4-fe5260aab99f

**What it chokes on**

- **OBSERVED — "Windows Media Player does not display files"** when the server cannot answer the
  `@refID exists false` search criterion, or when the library is still being imported —
  https://github.com/gerbera/gerbera/issues/1407
- **OBSERVED — WMP's sort keys can crash the server's SQL layer** if the server does not validate
  them — https://github.com/gerbera/gerbera/issues/2215
- **OBSERVED — WMP builds a full local cache** ("downloads all item info and builds up cache"), so it
  issues a *large* number of `Search`/`Browse` calls and is sensitive to slow responses —
  https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst
- **OBSERVED — the Windows 10/11 DLNA *client* shows folders but no files** for reasons that are
  client-side (WMP cache corruption, library permissions) rather than protocol-level —
  https://learn.microsoft.com/en-us/answers/questions/2109766/win-10-dnla-clients-folders-showing-but-not-files
- **OPEN QUESTION — the Windows 11 "Media Player" app (the new UWP app that replaced WMP 12) as a
  DLNA control point.** I found **no** primary source describing its UPnP behaviour, UA, or whether it
  even implements a control-point role. It may be that the "Media Player" app in Win11 has **no**
  DLNA control-point role at all, and the DLNA browsing in Explorer / "Media Player" continues to be
  handled by the legacy `WMPNetworkSharing` / `System` UPnP stack that also produced the
  `Microsoft-DLNA DLNADOC/1.50` UA. **Treat Windows 11 Media Player as UNVERIFIED.**

**Documented failure symptoms**

- **OBSERVED — Windows 10 DLNA client: "folders showing but not files"** (Microsoft Q&A) —
  https://learn.microsoft.com/en-us/answers/questions/2109766/win-10-dnla-clients-folders-showing-but-not-files
  and the British mirror https://learn.microsoft.com/en-gb/answers/questions/2199901/win-10-dnla-clients-folders-showing-but-not-files
- **OBSERVED — WMP's index/cache behaviour** (gerbera's "Microsoft" device page) —
  https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst
- **OPEN QUESTION — no WMP-specific "cannot seek" / "stops after N seconds" report was found.**

**Server-side workarounds observed in existing implementations**

- **ReadyMedia** — the Microsoft-family workarounds listed above (PlaysForSure bitrate ÷8, single video
  `res` + `upnp:albumArtURI`, synthetic `<upnp:album>[No Keywords]</upnp:album>` for `EMediaRoom`,
  `?albumArt=true` handling, magic containers flagged `FLAG_MS_PFS`, and the
  "Windows Media Connect compatible (…)" model name).
  — https://github.com/necropotame/readymedia/blob/master/upnpsoap.c ,
    https://github.com/necropotame/readymedia/blob/master/upnphttp.c ,
    https://github.com/necropotame/readymedia/blob/master/clients.c ,
    https://github.com/necropotame/readymedia/blob/master/clients.h ,
    https://github.com/necropotame/readymedia/blob/master/containers.c ,
    https://github.com/necropotame/readymedia/blob/master/minidlna.c
- **Gerbera** — PR #1408 (Microsoft search properties + `exists` operator + the "Microsoft" device
  documentation), and the `Microsoft-DLNA` UA → "Standard UPnP" mapping in `client_manager.cc`.
  — https://github.com/gerbera/gerbera/pull/1408/files ,
    https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc
- **Vuio (Rust DLNA server)** — the closest existing Rust precedent; its client detection table has
  **no Microsoft/WMC entry**, only `X-AV-Client-Info` sniffing for Sony/Samsung/LG/Panasonic/Xbox/
  PlayStation and a generic `"dlnadoc/1.50" → Standard` fallback:
  — https://github.com/vuiodev/vuio/blob/main/crates/vuio-core/src/web/client.rs
  ⇒ **PhairDLNA's Windows support would be a differentiator in the Rust ecosystem.**
- **OBSERVED — no server anywhere implements `WMC_COMPAT_*`** (Sourcegraph global code search, 0 hits
  for `WMC_COMPAT` and `MS-DeviceCaps`).

---

## Sources

| # | URL | What it contributed |
|---|---|---|
| 1 | https://winprotocoldoc.z19.web.core.windows.net/MS-UPMC/%5BMS-UPMC%5D.pdf | [MS-UPMC] v20240423, full text. The complete set of 15 `WMC_COMPAT_*` flags with values and server obligations (§2.2.1.1–§2.2.1.15); the `microsoft:X_DeviceCaps` DDD element; the MMPE properties (`microsoft:year`, `microsoft:folderPath`, …); the XML-escaping rule for DIDL-Lite tags. |
| 2 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/cb53cb44-4965-4056-b626-2b9526657499 | [MS-UPMC] index page listing all 15 `WMC_COMPAT_*` flag names — confirms the flag set is complete and not extended beyond v10. |
| 3 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/6cf02c0e-7952-40de-b70d-33b02b0372b8 | `microsoft:X_DeviceCaps` example (`94`) and how to decode the decimal value into flags. |
| 4 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/c12f5530-fa2d-4d9b-a960-18f4e9678100 | `WMC_COMPAT_EXCLUDE_DLNA` (§2.2.1.3) — including the "because some devices fall down if the string DLNA exists in the protocol response" rationale and the list of DLNA.ORG_* params to strip. |
| 5 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/0941c244-0d05-4d7d-9a61-745af114d5cf | `WMC_COMPAT_EXCLUDE_DLNA_1_5` (§2.2.1.4) — profile remapping (MP3X→MP3, WMVSPLL_BASE→WMVMED_BASE), `WMDRM_*` omission, and the "all URLs MUST have a file extension" rule. |
| 6 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-upmc/1f78a997-225c-4598-a5a7-66f749240adc | `WMC_COMPAT_EXCLUDE_RTSP` (§2.2.1.2) — the RTSP `res` exclusion and the mutually-exclusive-flag rules. |
| 7 | https://winprotocoldoc.z19.web.core.windows.net/MS-DLNHND/%5BMS-DLNHND%5D.pdf | [MS-DLNHND] v20240423, full text. §2.2.4.1 User-Agent ABNF with the `devicecaps` token; §3.2.4.1 the DMP's Search-vs-Browse decision, `Microsoft-CDS`, `ObjectID=13`, and the "devicecaps SHOULD be 1024" rule; §3.2.7.1 the `manufacturer contains "Microsoft"` trigger; §3.4.5.1 the full server-side CompatFlags algorithm; §3.4.5.2 the `X-AvailableSeekRange` requirement; §3.1.5.1 client-side handling of that header. |
| 8 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/28e72c9b-225b-42ec-8891-bc7f7e220e8e | [MS-DLNHND] §3.4.5.1 "Invoking a UPnP Browse or Search Action" — the normative CompatFlags cascade and the 204,800-byte cap. |
| 9 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/b85654cb-9685-49a0-9a8e-cd6e767b8df3 | [MS-DLNHND] §2.2.4.1 User-Agent — the exact ABNF for the `(MS-DeviceCaps/N)` token. |
| 10 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/d40517ad-2d0a-4397-a46b-db7ba281a837 | [MS-DLNHND] §1.3 Overview — establishes the DMP role and its use of UPnP MediaServer V1.0 + DLNA extensions. |
| 11 | https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dlnhnd/ee214164-6314-4430-9ba4-fe5260aab99f | [MS-DLNHND] Appendix A Product Behavior — "Windows Vista does not limit the size of the SOAP response"; Vista ignores `devicecaps`. |
| 12 | https://www.truenas.com/community/threads/setting-up-upnp-with-minidlna-on-the-freenas-nightlies-post-09-25-2011.2615/page-4 | **Real server log** of the Windows 7 control point: UA `Microsoft-Windows/6.1 UPnP/1.0 Windows-Media-Player/12.0.7601.17514 DLNADOC/1.50 (MS-DeviceCaps/1024)`, UA `Windows-Media-Player-DMS/12.0.7601.17514`, SSDP `ST: urn:schemas-upnp-org:service:ContentDirectory:1, MX: 3`, `SUBSCRIBE /evt/ContentDir` with `NT: upnp:event` / `Timeout: Second-1800` / callback port 2869. |
| 13 | https://github.com/gerbera/gerbera/issues/1407 | "Windows Media Player does not display files" — WMP sends the search criterion `@refID exists false`; gerbera's chunked import + missing `exists` support produced an empty library. |
| 14 | https://github.com/gerbera/gerbera/pull/1408/files | The actual server-side fix: `@refID`/`@parentID`/`@id` search properties, the `exists`/`@exists`/`contains`/`derivedfrom` operators in `src/database/search_handler.cc`, and the "Microsoft" section in `doc/supported-devices.rst`. |
| 15 | https://github.com/gerbera/gerbera/issues/2215 | Windows 10 WMP UA `Microsoft-Windows/10.0 UPnP/1.0 Windows-Media-Player/12.0.19041.1320 DLNADOC/1.50 (MS-DeviceCaps/1024)`; WMP `SortCriteria` `+dc:title` and `+upnp:originalTrackNumber,+dc:title`; resulting "Unknown sort key 'dc:title'" warnings and `no such table: INNER` / "prematurely shutting down" SQL failures. |
| 16 | https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc | Captured Windows 10 shell UA `Microsoft-Windows/10.0 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50` and its mapping to the generic "Standard UPnP" profile with `Quirk::None`. |
| 17 | https://github.com/gerbera/gerbera/blob/master/src/upnp/upnp_common.h | Gerbera's `upnp:class` constants, incl. both `object.item.videoItem` and `object.item.videoItem.movie`. |
| 18 | https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst | The "Microsoft" device section: "Windows Explorer (Browse in directory and play with selectable player)" / "Windows MediaPlayer 12 (downloads all item info and builds up cache)". |
| 19 | https://docs.gerbera.io/en/latest/supported-devices.html | Same device list, plus the full client-quirk catalogue (`FORCE_SORT_CRITERIA_TITLE`, `SIMPLE_DATE`, `ASCIIXML`, `STRICTXML`, `NO_SEC_NAMESPACE`, …). |
| 20 | https://github.com/necropotame/readymedia/blob/master/upnpsoap.c | ReadyMedia's Microsoft PlaysForSure workarounds: bitrate ÷ 8 for `FLAG_MS_PFS`; video album art not offered as a `<res>` for PFS clients; synthetic `<upnp:album>[No Keywords]</upnp:album>` for `EMediaRoom`; `@refID`/`@parentID`/`@id` search support and the `exists` translation (`o.CLASS like "item.videoItem%" and REF_ID is NULL`). |
| 21 | https://github.com/necropotame/readymedia/blob/master/upnphttp.c | ReadyMedia's HTTP layer: `Range` parsing (`strtoll`, no suffix-range support → 400), 206 + `Content-Range` on range requests, 416 on out-of-bounds end, and the `?albumArt=true` PFS convention. |
| 22 | https://github.com/necropotame/readymedia/blob/master/clients.c | ReadyMedia's client table: `EMediaRoom` (`Microsoft-IPTV-Client`), the `Windows`/`FDSSDP` entry, and the `AllShare` comment quoting `DLNADOC/1.50 SEC_HHP_[PC]LPC001/1.0  MS-DeviceCaps/1024`. |
| 23 | https://github.com/necropotame/readymedia/blob/master/clients.h | `#define FLAG_MS_PFS 0x00000080 /* Microsoft PlaysForSure client */`. |
| 24 | https://github.com/necropotame/readymedia/blob/master/minidlna.c | `"Windows Media Connect compatible (%s)"` model name — the deliberate "pretend to be Microsoft" trick. |
| 25 | https://github.com/necropotame/readymedia/blob/master/NEWS | "Add AllShare and Windows client detection." (1.1.6), "Improve Windows 7 interoperability." (1.0.19), "Fix bitrate displayed on Microsoft PFS devices." (1.0.22). |
| 26 | https://github.com/nmaier/simpleDLNA/issues/24 | "Windows Media Player & Center issues" — WMC indexing "took an excessive amount of time… generated only in parts rather than all at once"; WMP's list "constantly jumps everywhere". |
| 27 | https://sites.google.com/site/tweakradje/devices/minidlna-readymedia | A verified miniDLNA debug log showing the search shape used by a DLNA client: `SearchCriteria: upnp:class derivedfrom "object.item.videoItem" and @refID exists false`, `Filter: res@resolution,…`, `Count: 30`, `StartingIndex: 30` (LG WebOS TV — used to corroborate the *pattern*, not the Windows attribution). |
| 28 | https://docs.netscaler.com/us/en/citrix-adc/current-release/appexpert/rewrite/content-length-header-behaviour.html | The `Media Center PC 6.0` WMC HTTP stack UA (IE7 shell) and the documented failure mode when `Content-Length` is mangled on a Microsoft-family client. |
| 29 | https://learn.microsoft.com/en-us/answers/questions/2109766/win-10-dnla-clients-folders-showing-but-not-files | Windows 10 DLNA client: "folders showing but not files" — client-side cache/library cause, not a protocol quirk. |
| 30 | https://forum.serviio.org/viewtopic.php?f=27&t=9634 | A real `ContentDirectory:1#Browse` trace showing the long `Filter` attribute list a DLNA control point sends (client logged as "Unrecognized device / Generic DLNA profile", so **not** positively Windows). |
| 31 | https://github.com/vuiodev/vuio/blob/main/crates/vuio-core/src/web/client.rs | The closest existing Rust DLNA-server precedent: its client-detection table has **no** Microsoft/WMC entry; generic `"dlnadoc/1.50" → Standard` fallback only. |
| 32 | https://github.com/peter--s/dlna-dms-music-assistant-notes/blob/main/README.md | Cross-server survey of DLNA Search support; records that all major servers are closed-source/unverified and that only the Microsoft spec documents the 204,800-byte response cap. |
| 33 | https://sourceforge.net/p/minidlna/discussion/879957/thread/fa4b2f6d/ | **COULD NOT BE FETCHED** (Cloudflare interstitial). Search snippets from this page carry the Windows 7 WMP `Search` body (`RequestedCount 200`, `StartingIndex 0`, `SortCriteria +dc:title`, `Filter *`) and the `upnp:class derivedfrom "object.item.videoItem" and @refID exists false` criterion. Everything drawn from this page in this document is marked ASSUMPTION. |
| 34 | https://upnp.org/specs/av/UPnP-av-ContentDirectory-v1-Service.pdf (mirror: http://rfc.nop.hu/upnp/ContentDirectory1.0.pdf) | The ContentDirectory:1 `derivedfrom` / `exists` operator semantics that the Windows search criteria rely on. |
| 35 | `https://sourcegraph.com/search?q=context:global+WMC_COMPAT` and `…/MS-DeviceCaps` | Global code search showing that **no** open-source DLNA server implements the Microsoft compatibility-extension flags. |

---

## Practical checklist for PhairDLNA (derived from the above)

1. Parse `User-Agent` for the token `(MS-DeviceCaps/N)` and for `DLNADOC/1.50` / `DLNADOC/1.00`.
   Implement the §3.4.5.1 cascade exactly. Set `CompatFlags` from `devicecaps` when present.
2. Never cap a Browse/Search SOAP response at 204,800 bytes when `MS-DeviceCaps` has bit `%x400`
   (Windows always sends 1024). Cap it otherwise, or not at all.
3. Implement `Search` with `derivedfrom` on `upnp:class` (prefix match) and the `exists` operator on
   `@refID` — otherwise WMP/WMC show an **empty library**.
4. Advertise `GetSortCapabilities` including `dc:title` and `upnp:originalTrackNumber`; ignore unknown
   sort keys rather than passing them to the storage layer.
5. Support `Browse(ObjectID="13")` returning `object.container.playlistContainer` objects.
6. Emit `upnp:class` as specific as possible (`object.item.videoItem.movie`), and always emit a
   `dc:date`; escape XML per XML 1.0 §2.4.
7. Serve `res` with an explicit `Content-Length`, honour `Range` (including suffix `bytes=-N`),
   answer 206 + `Content-Range` (and 416 for unsatisfiable ranges), and emit
   `X-AvailableSeekRange` when you support `TimeSeekRange.dlna.org` seeking.
8. Decide consciously whether to impersonate Microsoft in the DDD `manufacturer`/`modelName`
   (it flips the Windows client into `Microsoft-CDS` mode and `Browse(ObjectID=13)` playlist mode).
9. Consider implementing the full `WMC_COMPAT_*` set — no open-source server does today.
