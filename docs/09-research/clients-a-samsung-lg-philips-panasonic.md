# DLNA Client Behaviour: Samsung, LG, Philips, Panasonic

Primary-source research notes for the PhairDLNA project. Every claim below is tagged:

- **STANDARD** — behaviour mandated by the UPnP/DLNA specifications (or is the norm for all DLNA clients).
- **OBSERVED** — behaviour actually seen in server logs / bug reports / vendor docs that were fetched (URL inline).
- **ASSUMPTION** — plausible but only supported by a search snippet, an unverified page, or inference from adjacent evidence.
- **OPEN QUESTION** — something the client very likely does but for which I could not find a primary source.

Nothing here was tested locally. All quoted logs and words come from the linked pages/files.

---

## Quick orientation: service versions and SCIDs (all four families)

- **STANDARD** — A DLNA DMS advertises `urn:schemas-upnp-org:service:ContentDirectory:1` and `urn:schemas-upnp-org:service:ConnectionManager:1`; ReadyMedia does exactly that, plus the MS extension `urn:microsoft.com:serviceId:X_MS_MediaReceiverRegistrar` — https://raw.githubusercontent.com/necropotame/readymedia/master/upnpdescgen.c (`{"/serviceType", "urn:schemas-upnp-org:service:ContentDirectory:1"}`, `{"/serviceId", "urn:upnp-org:serviceId:ContentDirectory"}`).
- **OBSERVED** — Every Samsung/LG request captured in this research used **ContentDirectory:1** (`urn:schemas-upnp-org:service:ContentDirectory:1#Browse`, `#GetSearchCapabilities`, `#X_GetFeatureList`). No client in this research ever probed ContentDirectory:2. — https://forum.serviio.org/viewtopic.php?f=10&t=30057 , https://sourceforge.net/p/minidlna/bugs/311/
- **OBSERVED** — Samsung's `X_GetFeatureList` is a **vendor action carried on ContentDirectory:1**, not the standard `GetFeatureList` of ContentDirectory:2. A ReadyMedia user noted exactly this: *"There's another bug here. The namespace is wrong, it shows version 1 but GetFeatureList wasn't added to the UPnP standard until ContentDirectory:2 (current standard is ContentDirectory:4)"* — https://sourceforge.net/p/minidlna/bugs/210/
- **OBSERVED** — Samsung fetches **three** service descriptions before browsing: `/serviceDescription/ConnectionManager`, `/serviceDescription/ContentDirectory` **and** `/serviceDescription/X_MS_MediaReceiverRegistrar` — https://forum.serviio.org/viewtopic.php?f=10&t=30057
- **OPEN QUESTION** — Whether any of these TVs will work against a server that publishes only ContentDirectory:1 and *omits* `X_MS_MediaReceiverRegistrar`. The Samsung client above explicitly asked for it, so omitting it is at least risky.

---

### Samsung Smart TVs (Linux-based 2013/2014 and earlier, and Tizen-based 2015+)

#### **What the client sends**

- **STANDARD** — SSDP `M-SEARCH * HTTP/1.1` with `ST: urn:schemas-upnp-org:device:MediaServer:1`, `MAN: "ssdp:discover"`, `MX:`.
- **OBSERVED** — A Samsung 6 Series TV sends **two** M-SEARCHes: one for `urn:schemas-upnp-org:device:MediaServer:1` and one for `urn:schemas-upnp-org:device:RemoteUIServerDevice:1` — https://forum.serviio.org/viewtopic.php?f=10&t=30057
  - `Received a valid M-SEARCH message for search target urn:schemas-upnp-org:device:MediaServer:1 from address /192.168.0.50:3942`
  - `Received a valid M-SEARCH message for search target urn:schemas-upnp-org:device:RemoteUIServerDevice:1 from address /192.168.0.50:3942`
- **ASSUMPTION** — A ReadyMedia log captured on a Raspberry Pi forum shows the same ST for a 2012 Samsung (`SEC_HHP_[TV]UE32ES6300/1.0 DLNADOC/1.50`): `SSDP M-SEARCH from 192.168.178.20:24234 ST: urn:schemas-upnp-org:device:MediaServer:1, MX: 10, MAN: "ssdp:discover"` — https://forums.raspberrypi.com/viewtopic.php?p=224748 (page could not be fetched directly; snippet only).

**User-Agent strings actually observed (server logs):**

| UA string | TV family | Source |
|---|---|---|
| `DLNADOC/1.50 SEC_HHP_[TV] Samsung 6 Series (55)/1.0 UPnP/1.0` | 2019-era "6 Series" | https://forum.serviio.org/viewtopic.php?f=10&t=30057 |
| `DLNADOC/1.50 SEC_HHP_55" OLED/1.0 UPnP/1.0` | 2023/24 QN55S90D (Tizen) | https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480 |
| `SEC_HHP_[TV]UE40ES8000/1.0 DLNADOC/1.50` | 2012 ES8000 (Linux) | https://sourceforge.net/p/minidlna/bugs/311/ |
| `SEC_HHP_[TV]UE40D7000/1.0 DLNADOC/1.50` (and `SEC_HHP_ Family TV/1.0`) | 2011 D7000 / "Family TV" | https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c |
| `DLNADOC/1.50 SEC_HHP_[TV] UE65JU7000/1.0 UPnP/1.0` | 2015 JU7000 | https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c |
| `DLNADOC/1.50 SEC_HHP_[TV] Samsung Q7 Series (49)/1.0` | 2017 Q series | ReadyMedia 1.3.3 `clients.c` — https://downloads.sourceforge.net/project/minidlna/minidlna/1.3.3/minidlna-1.3.3.tar.gz |
| `DLNADOC/1.50 SEC_HHP_Samsung QN90AA 50 TV/1.0` | 2021 Neo QLED | ReadyMedia 1.3.3 `clients.c` — https://downloads.sourceforge.net/project/minidlna/minidlna/1.3.3/minidlna-1.3.3.tar.gz |
| `SamsungWiselinkPro/1.0` | Series A (2009/2010) | https://sourceforge.net/p/minidlna/bugs/74/ |
| `SEC_HHP_BD-D5100/1.0` | Samsung Blu-ray | https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c |
| `SHP, UPnP/1.0, Samsung UPnP SDK/1.0` | Tizen OS (as reported in a Gerbera client list) | https://github.com/gerbera/gerbera/issues/3459 |
| `Samsung/1.0 UPnP/1.0` | Tizen OS (matched by Kodi's Samsung quirk) | https://github.com/xbmc/xbmc/commit/7f5c291395d1245a2c10b776cc784083b21e11e4 |
| `DLNADOC/1.50 SEC_HHP_[PC]LPC001/1.0  MS-DeviceCaps/1024` | **AllShare on PC — not a TV** | https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c |

- **OBSERVED** — Samsung's *browser/app* UA is a completely different family and should not be matched against DLNA traffic: `Mozilla/5.0 (SMART-TV; LINUX; Tizen 3.0) AppleWebKit/538.1 (KHTML, like Gecko) Version/3.0 TV Safari/538.1` (2017) … `Mozilla/5.0 (SmartHub; SMART-TV; U; Linux/SmartTV; Maple2012) …` (2012) — https://developer.samsung.com/smarttv/develop/guides/fundamentals/retrieving-platform-information.html

**SOAP / browse behaviour:**

- **OBSERVED** — Sequence on a 2012 ES8000: `X_GetFeatureList` (twice) then `Browse` with an **empty ObjectID**:
  ```xml
  <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
    <ObjectID></ObjectID><BrowseFlag>BrowseDirectChildren</BrowseFlag><Filter>*</Filter>
    <StartingIndex>0</StartingIndex><RequestedCount>30</RequestedCount><SortCriteria></SortCriteria>
  </u:Browse>
  ```
  → server answered `UPnPError 402: Invalid Args` and the TV displayed an error. — https://sourceforge.net/p/minidlna/bugs/311/
- **OBSERVED** — A 2023/24 Tizen TV (QN55S90DAGXZD) browses with **`BrowseMetadata` and `RequestedCount` 0**, repeatedly, and only ever gets the container back:
  ```xml
  <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
    <ObjectID>0</ObjectID><BrowseFlag>BrowseMetadata</BrowseFlag><Filter>*</Filter>
    <StartingIndex>0</StartingIndex><RequestedCount>0</RequestedCount><SortCriteria></SortCriteria>
  </u:Browse>
  ```
  The reporter's summary: *"Apparently, each one is using different `BrowseFlags`. VLC uses the `BrowseDirectChildren` with 5000 `RequestedCount` and this works correctly. The Samsung TV tries to browse using the `BrowseMetadata` with 0 `RequestedCount` and this is not resulting in the correct list of files."* — https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480
- **OBSERVED** — Filter used by the Samsung client in the 2012-era log is literally `*` — https://sourceforge.net/p/minidlna/bugs/311/ and https://sourceforge.net/p/minidlna/bugs/165/
- **OBSERVED** — When the user changes the sort order in the Samsung UI, the client sends **`RequestedCount` = -1** with `StartingIndex` 0. — https://sourceforge.net/p/minidlna/bugs/134/
- **OBSERVED** — Samsung does **not** use `SortCriteria` for sorting; it uses dedicated ObjectIDs. A user who sniffed Samsung's own AllShare server reported: *"Samsung uses spesial OpjectID's for sorting: V_T is Title, V_L is LatestDate...... These objects have children V_T_0002, V_T_0002_02 ...."* and for music `A_L` (Latest). — https://sourceforge.net/p/minidlna/bugs/134/ and https://sourceforge.net/p/minidlna/bugs/125/
- **OBSERVED** — When fetching icons the Samsung client uses **HTTP/1.0** and DLNA request headers:
  ```
  GET /icons/lrg.png HTTP/1.0
  HOST: x.x.x.x:2869
  ACCEPT-LANGUAGE: en-us
  getcontentFeatures.dlna.org:1
  transferMode.dlna.org:Interactive
  ```
  — https://sourceforge.net/p/minidlna/bugs/311/
- **OBSERVED** — For media, the Samsung client sends a **`HEAD` request over HTTP/1.0** and asks for subtitles with a vendor header:
  ```
  HEAD /media/object_id/24/res_id/0/group/default/ext/file.und.mp4 HTTP/1.0
  getCaptionInfo.sec: 1
  ```
  — https://github.com/gerbera/gerbera/issues/3926
- **OBSERVED** — Samsung uses the vendor bookmark action `X_SetBookmark` with `ObjectID` + `PosSecond`. The unit of `PosSecond` differs by generation: seconds on C/D/E/F/J, **milliseconds** on Q/QN. — ReadyMedia 1.3.3 `upnpsoap.c` (`if( h->req_client && (h->req_client->type->flags & FLAG_CONVERT_MS) ) { sec /= 1000; }`) — https://downloads.sourceforge.net/project/minidlna/minidlna/1.3.3/minidlna-1.3.3.tar.gz — and the client-table comment *"Samsung Series [Q] TVs work with milliseconds for bookmarks"* in the same file.
- **OPEN QUESTION** — Whether any Samsung TV ever issues a ContentDirectory **`Search`** action (and with which `SearchCriteria`). I found no captured Samsung `Search` request. (The `upnp:class derivedfrom "object.item.videoItem"` + `dc:title contains "..."` syntax appears in a third-party DLNA *client* library, not in Samsung traffic: https://github.com/yellobyte/SoapESP32 )

#### **What it chokes on**

- **OBSERVED** — **Returning more items than `RequestedCount` puts the TV into an error state and stops browsing.** Root-cause analysis on a real 2012 Samsung: *"TV goes into error state and stop browsing when receives dlna response with `NumberReturned ` > `RequestedCount`."* — https://github.com/jellyfin/jellyfin/issues/2335
- **OBSERVED** — **Vendor `upnp:class` values break it.** A Samsung 2017 TV shows empty leaves and *"Trying to navigate to those leaves will crash the TV dlna client."* The Kodi maintainer's conclusion: *"the samsung app just supports basic types as shown in the spec … Kodi uses a few other types to represent seasons/shows/episodes such as `object.container.album.videoAlbum.videoBroadcastShow` or `object.container.album.videoAlbum.videoBroadcastSeason`."* — https://github.com/xbmc/xbmc/issues/22828
- **OBSERVED** — **It does not parse a full ISO-8601 `dc:date`, but does accept `yyyy-MM-dd`.** Field report: *"My TV does not recognize content dates in full ISO 8601 format, but recognizes short dates with 'yyyy-MM-dd' format … Format docs shows that yyyy-MM-dd allowed … so reducing the accuracy to a day seems to me quite appropriate for the sake of compatibility."* — https://github.com/jellyfin/jellyfin/issues/2335
- **OBSERVED** — **Titles that already contain the file extension produce duplicate entries.** rclone's server carries the fix and the reason: `// Remove file extension from title to prevent Samsung TVs from duplicating it` / `// File type is already provided via MIME type in protocolInfo` — https://raw.githubusercontent.com/rclone/rclone/master/cmd/serve/dlna/cds.go
- **OBSERVED** — **Chunked transfer-encoding over HTTP/1.0 is rejected.** Gerbera's PR states the Samsung TV could not play transcoded resources because libupnp answers `406 Not Acceptable` for HTTP/1.0 + chunked: `if (using_virtual_dir && UpnpFileInfo_get_FileLength(finfo) == UPNP_USING_CHUNKED) { /* Chunked encoding is only supported by HTTP 1.1 clients */ if (resp_major == 1 && resp_minor == 1) {...} else { err_code = HTTP_NOT_ACCEPTABLE; }`. The PR notes *"the Samsung TV cited in the PR"* could not play, and that `curl -0` (HTTP/1.0) reproduced it. — https://github.com/gerbera/gerbera/pull/3954
- **OBSERVED** — **It expects a "wrong" MIME type for Matroska.** ReadyMedia: `/* From what I read, Samsung TV's expect a [wrong] MIME type of x-mkv. */` → `video/x-matroska` is rewritten to `video/x-mkv`. Also `video/quicktime` → `video/mp4`, and on Series A `video/x-msvideo` → `video/mpeg`. — https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c and https://raw.githubusercontent.com/necropotame/readymedia/master/upnphttp.c
- **OBSERVED** — **It needs the vendor `sec:dcmInfo` bookmark element, but `Filter=*` does not imply it.** A ReadyMedia user's diagnosis: *"The problem seems to be that the ES7000 sends a Browser with Filter: \*. On that case, set_filter_flags returns all flags but the Vendor-specific ones (we need FILTER_SEC_DCM_INFO)."* — https://sourceforge.net/p/minidlna/bugs/165/
- **OBSERVED** — **Advertising the DCM10 capability breaks some models.** ReadyMedia commit: *"Advertising the DCM10 vendor-specific feature to Samsung Series C and D (at least) players causes them to always browse ContainerID 1, no matter which section is chosen from their GUI."* And from the F-series bug: *"for Samsung F series TVs only genRootDesc() is working OK"* → the server must not send the DCM10 root description to F-series. — https://github.com/Darkvater/minidlna/commit/e3a53fc8a3b78bdb30dfe6b84bd8c41239ef470c and https://sourceforge.net/p/minidlna/bugs/237/
- **OBSERVED** — **`RequestedCount` = -1 (sent when the sort menu is used) produces `UPnPError 701: No such object`** and the TV's sort menu does nothing. — https://sourceforge.net/p/minidlna/bugs/134/
- **OBSERVED** — **Tizen TVs may not display the server at all.** A 2023 QN85C with Tizen OS never listed a Gerbera server although SSDP traffic looked fine; the only thing that made it appear was renaming the TV from `75" QLED TV` to `QN85C` and changing the server's `ModelName`/`ModelDescription`, then a power cycle. — https://github.com/gerbera/gerbera/issues/3459
  - Corroborated on Samsung's own community forum for a 2024 Q80D and a QN95: the fix reported by users was to **rename the TV to a plain alphanumeric name** (`SamsungTV`) — *"It worked !! After god knows how much time with Samsung Tech Support and Synology NAS support. Just removing the blank character and renaming it SamsungTV."* — https://eu.community.samsung.com/t5/tv/nas-won-t-show-on-tv/td-p/11370913
- **OBSERVED** — **External subtitles: legacy Samsung used a custom method, Series J no longer supports it.** ReadyMedia commit: *"Older Samsung devices used a custom method for describing subtitles, but apparently Series J no longer supports it. The good thing is, they now support the more standard method … So let's just expose both methods to the client."* — https://github.com/Darkvater/minidlna/commit/4ec6cd005358f3474ce9caec4f0954b4756570ed
- **OBSERVED** — **Tizen Samsung TVs do not display external SRT subtitles at all** with Gerbera, while an old Linux-based Samsung J5000 does: *"subtitles is working in VLC and old samsung TV J5000 running legacy linux but it is NOT working on new samsung TV running tizen."* — https://github.com/gerbera/gerbera/issues/3152
  - Related, from a newer Gerbera bug: even though the DIDL carries `<sec:CaptionInfoEx protocolInfo="http-get:*:application/x-srt:*" sec:type="srt">…</sec:CaptionInfoEx>`, *"the response to the Samsung HEAD request does not contain a CaptionInfo.sec header"* — https://github.com/gerbera/gerbera/issues/3926
- **OBSERVED** — **Files > 2 GB may be reported as 2 GB** by some firmware: *"I tried to share a big mkv file through miniDLNA. The file was 4.7 GB. While my samsung TV decoded it correctly, it states that it is only 2GB."* (other C-series firmware did not show the bug) — https://sourceforge.net/p/minidlna/bugs/44/
- **OBSERVED** — **The FF/REW remote keys are a client-side limitation, not a server one.** ReadyMedia maintainer: *"Samsung's DLNA playback only allows you to use the left and right arrow keys to FF/REW. So I think this is just a TV limitation."* and *"The server cannot influence the behavior of the client buttons, so there is nothing to do here."* — https://sourceforge.net/p/minidlna/bugs/79/ and https://sourceforge.net/p/minidlna/bugs/111/
- **OBSERVED** — **Series A AVI playback is broken by a MIME trick.** Bisected to the commit *"Make a distinction between Samsung A series and later models. We need to trick A series only into playing AVI files."*, which broke seeking in AVI on a later B-series TV (`USER-AGENT: SamsungWiselinkPro/1.0`). — https://sourceforge.net/p/minidlna/bugs/74/ and https://github.com/Darkvater/minidlna/commit/0cf2e5731a4f212558e56d59c91eb12c9e85b856
- **OPEN QUESTION** — Exact tolerance for `res/@duration` format on Samsung. ReadyMedia emits `duration="0:22:38.272"` (H:MM:SS.mmm) in the DIDL it shows to Samsung; I found no Samsung bug report proving that format is required, nor one proving ISO-8601 durations are accepted.

#### **Documented failure symptoms** (users' own words)

- *"The TV displays an error"* and the server logs `Returning UPnPError 402: Invalid Args` — after the client sent `Browse` with an empty `<ObjectID></ObjectID>`. — https://sourceforge.net/p/minidlna/bugs/311/
- *"the TV displays an error and the following is logged … Returning UPnPError 402: Invalid Args"* — https://sourceforge.net/p/minidlna/bugs/311/
- *"I am only getting empty folders"* (older Samsung Smart TV, Jellyfin) — https://github.com/jellyfin/jellyfin/issues/2335
- *"the folders show up empty"* after 10.5.0 — https://github.com/jellyfin/jellyfin/issues/2335
- *"TV goes into error state and stop browsing when receives dlna response with `NumberReturned ` > `RequestedCount`"* — https://github.com/jellyfin/jellyfin/issues/2335
- *"Only Videos jumps straight to the full item-list, which then causes the Samsung client to crash after a period (presumably because it's trying to display every item at once, rather than paging)."* — https://sourceforge.net/p/minidlna/bugs/311/
- *"but all of those entries are empty, when selected"* … *"Trying to navigate to those leaves will crash the TV dlna client."* — https://github.com/xbmc/xbmc/issues/22828
- *"Video Library -> Files is empty"* — https://github.com/xbmc/xbmc/issues/22828
- *"my TV kept on saying 'unknown file format' whatever video I tried to see"* (Samsung UE40D5000, regression) — https://sourceforge.net/p/minidlna/bugs/190/
- *"Error message on TV: Not possible to access to the device. Check it and retry."* with `UPnPError 701: No such object` (Samsung D6xxx) — https://sourceforge.net/p/minidlna/bugs/157/
- *"TV set Samsung 40D6530 cant see video files, but other mp3, Jpg is ok. Message on tv - cant see and open files from source."* — https://sourceforge.net/p/minidlna/bugs/78/
- *"the TV displays an error"* / *"Only music files are listed on a Samsung D5100 blu-ray reader, regardless if I'm under Videos, Music or Pictures menu"* (DCM10 regression) — https://sourceforge.net/p/minidlna/bugs/231/
- *"it returns error on the TV(UE40D6100) when selecting Music. Videos and pictures work OK."* — https://sourceforge.net/p/minidlna/bugs/125/
- *"Serve DLNA works on everything except on my Samsung TV"* (2023/24 Tizen; TV never shows the files) — https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480
- *"My Samsung TV can't show NAS"* / *"The new TV won't do this"* (2024 Q80D) — https://eu.community.samsung.com/t5/tv/nas-won-t-show-on-tv/td-p/11370913
- *"my TV keeps on saying unknown file format"*, *"can't see and open files from source"* — https://sourceforge.net/p/minidlna/bugs/78/

#### **Server-side workarounds observed in existing implementations**

**ReadyMedia / MiniDLNA** (source: https://github.com/necropotame/readymedia — files `clients.c`, `clients.h`, `upnpsoap.c`, `upnphttp.c`, `upnpdescgen.c`, `containers.c`. Note this mirror is a 1.2.x-era snapshot; the Series Q/QN entries and `FLAG_CONVERT_MS` quoted below are from the 1.3.3 release tarball, https://downloads.sourceforge.net/project/minidlna/minidlna/1.3.3/minidlna-1.3.3.tar.gz):

- **Client detection table** with per-family flags (`clients.c`):
  - `SEC_HHP_` → `ESamsungSeriesCDE` with `FLAG_SAMSUNG | FLAG_DLNA | FLAG_NO_RESIZE | FLAG_SAMSUNG_DCM10` (1.2.x mirror) — 1.3.3 additionally sets `FLAG_CAPTION_RES` for CDEFJ.
  - `SEC_HHP_[TV] Samsung Q` and `SEC_HHP_Samsung QN` → additionally `FLAG_CONVERT_MS` (*"Samsung Series [Q] TVs work with milliseconds for bookmarks"*) — ReadyMedia 1.3.3 `clients.c`.
  - `SamsungWiselinkPro` → Series A; `Samsung DTV DMR` → Series B; `SEC_HHP_BD` → Samsung BDP.
  - `SEC_HHP_[PC]` → deliberately **no** Samsung flags: *"This is AllShare running on a PC. We don't want to respond with Samsung capabilities, or Windows (and AllShare) might get grumpy."*
  - Series CDE **BDP must be a separate entry from the TV**: *"Samsung Series [CDE] BDPs and TVs must be separated, or some of our advertised extra features trigger a folder browsing bug on BDPs."*
- **Samsung "magic containers"** (`containers.c`): ObjectIDs `I`/`A`/`V` are mapped to the image/music/video roots **only for `FLAG_SAMSUNG_DCM10` clients**:
  ```c
  /* Samsung DCM10 containers for Series E(?) */
  { NULL, "I", &image_id, …, FLAG_SAMSUNG_DCM10 },
  { NULL, "A", &music_id, …, FLAG_SAMSUNG_DCM10 },
  { NULL, "V", &video_id, …, FLAG_SAMSUNG_DCM10 },
  ```
- **`X_GetFeatureList` implementation** (`upnpsoap.c`): returns `samsung.com_BASICVIEW` feature with container ids `1`/`2`/`3` normally, but **`A`/`V`/`I` for DCM10 clients**:
  ```c
  else if (h->req_client && (h->req_client->type->flags & FLAG_SAMSUNG_DCM10)) { audio = "A"; video = "V"; image = "I"; }
  ```
- **Samsung-specific root device description** (`upnpdescgen.c` → `genRootDescSamsung`, served only to DCM10 clients in `upnphttp.c`):
  ```c
  samsungRootDesc[8].eltname  = "/sec:ProductCap";
  samsungRootDesc[8].data     = "smi,DCM10,getMediaInfo.sec,getCaptionInfo.sec";
  samsungRootDesc[12].eltname = "/sec:X_ProductCap";
  samsungRootDesc[12].data    = "smi,DCM10,getMediaInfo.sec,getCaptionInfo.sec";
  ```
- **`X_SetBookmark`** implemented, with ms↔s conversion for Q/QN (`FLAG_CONVERT_MS`), and bookmark echoed back in `sec:dcmInfo`:
  `<sec:dcmInfo>CREATIONDATE=0,FOLDER=%s,BM=%d</sec:dcmInfo>` (and `upnp:lastPlaybackPosition`, `upnp:playbackCount`).
- **Empty `ObjectID` accepted only for Samsung** — the patch proposed in the bug thread and later made upstream is exactly:
  ```c
  if( !ObjectID && !(ObjectID = GetValueFromNameValueList(&data, "ContainerID")) )
  {
      if (h->req_client && (h->req_client->type->flags & FLAG_SAMSUNG_DCM10)) { ObjectID = "0"; }
      else { SoapError(h, 402, "Invalid Args"); goto browse_error; }
  }
  ```
  — https://sourceforge.net/p/minidlna/bugs/311/
- **MIME rewriting for Samsung**: `video/x-matroska` → `video/x-mkv`; `video/quicktime` → `video/mp4`; Series A `video/x-msvideo` → `video/mpeg`.
- **`transferMode.dlna.org: Interactive` tolerated for non-image files**: *"Samsung TVs (well, at least the A950) do this for some reason, and I don't see them fixing this bug any time soon."* — otherwise `Send406`. (`upnphttp.c`)
- **`CaptionInfo.sec` response header** emitted when the client sent `getCaptionInfo.sec`:
  `strcatf(&str, "CaptionInfo.sec: http://%s:%d/Captions/%lld.srt\r\n", …)` (`upnphttp.c`)
- **HTTP/1.0 handling**: image responses over HTTP/1.0 are sent unchunked with `Content-Length` instead of `Transfer-Encoding: chunked` (`upnphttp.c`).
- **Commits that document the intent**:
  - `fc6b47f` *"upnpsoap: fix incorrectly formed Samsung GetFeatureList reply … otherwise the renderer will get stuck in an endless loop"* — https://github.com/Darkvater/minidlna/commit/fc6b47fe6aca39699668cbba74d110b988270abc (bug: https://sourceforge.net/p/minidlna/bugs/210/)
  - `ce16739` *"samsung: fix root_container setting — Pretend that we don't support SamsungGetFeatureList if we have a custom root container specified, or else the Samsung client will jump straight to our normal A/V/P sections."* — https://github.com/Darkvater/minidlna/commit/ce16739213cfe4e652b5655606937bf8041a339c
  - `c1bb37a` *"clients: Re-enable Samsung DCM10 flag … For future reference, the DCM10 flag enables (at least) video bookmark support and Samsung 'BasicView' support; so selecting, for example, Videos from the Samsung UI can send you straight to the Videos section of the media library."* — https://github.com/Darkvater/minidlna/commit/c1bb37a38754e0fed6a55a2ebf204c68c3d2135b
  - `4ec6cd0` *"Fix external subtitles on Samsung Series J."* — https://github.com/Darkvater/minidlna/commit/4ec6cd005358f3474ce9caec4f0954b4756570ed
  - `1e23a1e` *"Send a custom root description to enable the subtitle menu on some Samsung TV's."* — https://github.com/Darkvater/minidlna/commit/1e23a1e8e9aba1001cd116e89b034575b305b6dd
  - `d5b77cb` *"Add bookmark support for Samsung TVs."* / `126f0f5` *"Samsung bookmark - go back to the beginning of the movie if less than 30 seconds was played."* / `cae3e88` *"Samsung bookmarking: fix for magic containers."*
  - `345ecb9` *"Fix SRT caption support with the latest Samsung Series D firmware."*

**Gerbera** (https://github.com/gerbera/gerbera):

- Quirks `SAMSUNG`, `SAMSUNG_FEATURES`, `SAMSUNG_HIDE_DYNAMIC`, `DCM10`, `CAPTION_PROTOCOL`, `SAMSUNG_BOOKMARK_SEC`, `SAMSUNG_BOOKMARK_MSEC`; documented semantics: *"SAMSUNG_HIDE_DYNAMIC: hides dynamic folders at the top level because some Samsung devices cannot browse them."* — https://docs.gerbera.io/en/stable/supported-devices.html
- `src/upnp/quirks.cc` implements `getSamsungFeatureList` (the `samsung.com_BASICVIEW` feature with container ids `A`/`V`/`I`/`P`), `getSamsungFeatureRoot` (maps ObjectIDs `A`,`A_D`,`A_T`,`V`,`V_D`,`V_T`,`I`,`I_D`,`I_T`,`P`,`P_D`,`P_T` to the corresponding upnp:class — the `V_T`/`A_L` style sort IDs are visible here in the `_T` = "Title" variants), `getSamsungObjectIDfromIndex`, `getSamsungIndexfromRID`, and `save/restoreSamsungBookMarkedPosition` — https://github.com/gerbera/gerbera/blob/master/src/upnp/quirks.cc
- `addCaptionInfo()` adds the `CaptionInfo.sec` header **only** when the client sent `getCaptionInfo.sec: 1` and the item is a video — https://github.com/gerbera/gerbera/blob/master/src/upnp/quirks.cc
- `src/request_handler/device_description_handler.cc` emits `dlna:X_DLNADOC M-DMS-1.50` for Samsung and, for `DCM10` clients, `sec:ProductCap` / `sec:X_ProductCap` = `smi,DCM10,getMediaInfo.sec,getCaptionInfo.sec` — https://github.com/gerbera/gerbera/blob/master/src/request_handler/device_description_handler.cc
- PR #3701 *"improve Samsung TVs quirks"* maps its flags onto MiniDLNA's (`FLAG_CAPTION_RES`→`Quirk::CaptionProtocol`, `FLAG_CONVERT_MS`→`Quirk::SamsungBookmarkMilliSeconds`) and adds Series A/B/QN — https://github.com/gerbera/gerbera/pull/3701
- PR #3954 changed transcoded responses from `UPNP_USING_CHUNKED` to `UPNP_UNTIL_CLOSE` so HTTP/1.0 clients (the Samsung TV) are not answered with 406 — https://github.com/gerbera/gerbera/pull/3954

**Jellyfin DLNA plugin** (https://github.com/jellyfin/jellyfin-plugin-dlna):

- `samsung.com_BASICVIEW` feature list with **container id `"0"`** for all three media types — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/ContentDirectory/ControlHandler.cs
- `dc:date` written as **`yyyy-MM-dd` only** — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Didl/DidlBuilder.cs (`item.PremiereDate.Value.ToString("yyyy-MM-dd", …)`)
- Root detection accepts `""`, `"0"` **and `"1"`**: `// Samsung sometimes uses 1 as root` — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Didl/DidlBuilder.cs
- Samsung device profile: match on User-Agent substring `SEC`, add `xmlns:sec="http://www.sec.co.kr/"` to the DIDL root, `EnableAlbumArtInDidl=true`, `EnableSingleAlbumArtLimit=true`, `TimelineOffsetSeconds=0` — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Samsung%20Smart%20TV.xml

**rclone `serve dlna`** (https://github.com/rclone/rclone):

- `// Samsung TVs sometimes send empty ObjectID, default to root container` → `if browse.ObjectID == "" { browse.ObjectID = "0" }` — https://raw.githubusercontent.com/rclone/rclone/master/cmd/serve/dlna/cds.go
- `// Remove file extension from title to prevent Samsung TVs from duplicating it`
- Implements `X_GetFeatureList` (`samsung.com_BASICVIEW`) and stubs `X_SetBookmark`
- Emits `sec:ProductCap`/`sec:X_ProductCap` = `smi,DCM10,getMediaInfo.sec,getCaptionInfo.sec` in the root description

**Kodi's UPnP server** (https://github.com/xbmc/xbmc):

- User-Agent `"Samsung/1.0 UPnP/1.0"` → `ECLIENTQUIRKS_ONLYSTORAGEFOLDER | ECLIENTQUIRKS_BASICVIDEOCLASS` — i.e. expose only plain storage folders and plain `object.item.videoItem` — https://github.com/xbmc/xbmc/commit/7f5c291395d1245a2c10b776cc784083b21e11e4
- `virtualpath://upnproot` must be reported as a folder, or the Samsung client shows empty entries — https://github.com/xbmc/xbmc/commit/0a7eaf90eb1b7acc3d7636894503cc7e0070240e
- Object IDs were switched to base64 to avoid parser problems — https://github.com/xbmc/xbmc/issues/22828

---

### LG webOS TVs (and older NetCast)

#### **What the client sends**

- **OBSERVED** — NetCast / older LG DLNA UA: `Linux/2.6.35 UPnP/1.0 DLNADOC/1.50 INTEL_NMPR/2.0 LGE_DLNA_SDK/1.6.0` (NetCast), `Linux/2.6.31-1.0 UPnP/1.0 DLNADOC/1.50 INTEL_NMPR/2.0 LGE_DLNA_SDK/1.5.0` (LG), `Linux/2.6.28.9 UPnP/1.0 DLNADOC/1.50 INTEL_NMPR/2.0 LGE_DLNA_SDK/1.5.0` (LG LW5590) — https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c and https://emby.media/community/topic/23009-lg-tv-lw5590-dlna-connection-failed-to-htpc/
- **OBSERVED** — An LG TV's first Browse is `BrowseMetadata` with `RequestedCount` **0**, then `BrowseDirectChildren` with `RequestedCount` **16**:
  ```xml
  <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
    <ObjectID>0</ObjectID><BrowseFlag>BrowseMetadata</BrowseFlag>
    <Filter>@id,@parentID,@restricted,@childCount,dc:title,dc:creator, upnp:artist,upnp:class,dc:date,upnp:album…</Filter>
    <StartingIndex>0</StartingIndex><RequestedCount>0</RequestedCount><SortCriteria></SortCriteria>
  </u:Browse>
  ```
  — https://emby.media/community/topic/23009-lg-tv-lw5590-dlna-connection-failed-to-htpc/
- **OBSERVED** — A webOS LG TV sends an explicit attribute filter and a large page size:
  ```xml
  <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
    <ObjectID>68</ObjectID><BrowseFlag>BrowseDirectChildren</BrowseFlag>
    <Filter>res@resolution,res@nrAudioChannels,res@sampleFrequency,res@bitrate,dc:creator,
            res@dlna:cleartextSize,dc:date,upnp:genre,res,res@duration,res@size,upnp:albumArtURI,
            upnp:originalTrackNumber,upnp:album,upnp:artist,upnp:author</Filter>
    <StartingIndex>0</StartingIndex><RequestedCount>30</RequestedCount><SortCriteria/>
  </u:Browse>
  ```
  — https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1960
- **OBSERVED** — Other captured webOS LG `RequestedCount` values: **99** (https://emby.media/community/topic/17224-dlna-profile-for-lg-webos-tv/) and **30** (https://emby.media/community/topic/123549-playback-error-lg-tv-livetv/). So the LG client does not use a single fixed page size.
- **OBSERVED** — **The TV browses the whole server by itself.** Server maintainer: *"Ah yes, this is a known problem with LG TVs, they like to browse all of the server folders by themselves. Unfortunately we have no way to know whether they are legitimately browsing (because a person is doing it), or if the TV is doing it automatically. I don't know why they do it."* — https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1960
- **OBSERVED** — The same behaviour seen as a network flood from an LG 42LM620T (2012): *"LG TV makes requests for whole content of the server every time it is been switched on"* — https://sourceforge.net/p/minidlna/bugs/211/
- **OPEN QUESTION** — Whether LG webOS ever sends `SearchCriteria` (ContentDirectory `Search`). I found only `Browse` traffic from LG.
- **OPEN QUESTION** — LG webOS sends `SortCriteria` empty in every captured sample, but ReadyMedia implemented a workaround for an "LG TV ordering bug", which implies some LG firmware *does* send a sort order. See below.

#### **What it chokes on**

- **OBSERVED** — **Seeking requires three DLNA HTTP response headers.** The fix reported for an LG OLED65C8 (webOS 05.10.40) was to add, on the media response:
  ```
  contentFeatures.dlna.org: DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000
  realTimeInfo.dlna.org: DLNA.ORG_TLAG=*
  transferMode.dlna.org: Streaming
  ```
  *"Almost everything worked out of the box with my TV (LG OLED65C8, WebOS 05.10.40), but seeking within videos didn't (I use only direct play, no transcoding)."* — https://github.com/gerbera/gerbera/issues/839
- **OBSERVED** — **Thumbnails larger than 160 px are not picked up by the LG webOS TV.** *"Also, thumbnails larger than 160 pixels are not picked up by the TV, as is already mentioned in Gerbera docs."* — https://github.com/gerbera/gerbera/issues/839
- **OBSERVED** — **Thumbnail `res/@protocolInfo` must advertise `DLNA.ORG_CI=1` plus `DLNA.ORG_FLAGS`.** A WebOS 5 NanoCell displayed thumbnails from a QNAP DLNA server but not from Gerbera. QNAP sent:
  ```xml
  <res protocolInfo="http-get:*:image/jpeg:DLNA.ORG_PN=JPEG_TN;DLNA.ORG_CI=1;DLNA.ORG_FLAGS=00900000000000000000000000000000">
  ```
  while Gerbera 1.9.2 sent:
  ```xml
  <res protocolInfo="http-get:*:image/jpeg:DLNA.ORG_PN=JPEG_TN;DLNA.ORG_OP=01;DLNA.ORG_CI=0" resolution="128x53">
  ```
  Upgrading Gerbera to 1.10 fixed it. — https://github.com/gerbera/gerbera/issues/2557
- **OBSERVED** — **LG paginates with a mismatched page size, so the server must impose its own ordering.** ReadyMedia commit `b94f04d` (*"upnpsoap: Work around LG TV ordering bug."*) documents it precisely:
  > *"My LG TV (42LE5500) retrieves items from BrowseDirectChildren in a weird way. The TV can only display up to 20 items per page, but it sets RequestedCount to 54. StartingIndex is correctly set to 0, 20, 40, … etc. When minidlna responds with the 54 entries, the TV sorts folders (containers) first, before items, then displays the entries, limited to 20 per page. In my setup, I have ~23 folders whose names are similar to the files, so that when sorting by name, the folders are mixed with the files, thereby resulting in **duplicate folders appearing on all 3 pages and many missing files**. The solution is to make sure minidlna orders items the same way that the TV expects them…"*
  — https://github.com/Darkvater/minidlna/commit/b94f04d6ef5b9532088fadb9a874d5317d519495
- **OBSERVED** — **Subtitles only appear if the title contains a dot** (older LG). ReadyMedia: `/* LG hack: subtitles won't get used unless dc:title contains a dot. */` → the server appends `.` to the title — https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c
  - **OBSERVED** — This was later found to be LG-2012-specific: *"LG 2012 series does not need '.' in title anymore to be able to show captions"* — https://github.com/Darkvater/minidlna/commit/533907539357df45d1923638d11192b98f7bf280
- **OBSERVED** — **An LG TV contacting ReadyMedia could crash the server** (NULL MIME dereference) on an LG 50UM-series TV: *"The process frequently aborted with a segmentation fault … The issue is caused by the fact that the pointer mime is NULL, which is never checked."* — https://sourceforge.net/p/minidlna/bugs/324/
- **OPEN QUESTION** — Whether LG webOS is sensitive to `upnp:class` values beyond the plain ones, and whether it cares about `dc:date` format. No primary source found.

#### **Documented failure symptoms**

- *"the TV can only display up to 20 items per page, but it sets RequestedCount to 54"* → *"duplicate folders appearing on all 3 pages and many missing files"* — https://github.com/Darkvater/minidlna/commit/b94f04d6ef5b9532088fadb9a874d5317d519495
- *"LG TV makes requests for whole content of the server every time it is been switched on"* (network flooded for *"10 minutes to several hours"*) — https://sourceforge.net/p/minidlna/bugs/211/
- *"seeking within videos didn't (I use only direct play, no transcoding)"* (LG OLED65C8) — https://github.com/gerbera/gerbera/issues/839
- *"the WebOS 5 NanoCell did not [show thumbnails]"* although *"the TV itself had successfully displayed thumbnails from QNAP"* — https://github.com/gerbera/gerbera/issues/2557
- *"my player still doesn't see the subtitle track"* (LG HX806PH, external .srt) — https://sourceforge.net/p/minidlna/bugs/181/
- *"The process frequently aborted with a segmentation fault"* when an LG TV contacted the server — https://sourceforge.net/p/minidlna/bugs/324/

#### **Server-side workarounds observed in existing implementations**

- **ReadyMedia**: `clients.c` entries `ELGNetCastDevice` (`LGE_DLNA_SDK/1.6.0`) and `ELGDevice` (`LGE_DLNA_SDK`) with `FLAG_DLNA | FLAG_CAPTION_RES | FLAG_MIME_FLAC_FLAC` — https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c
  - **Ordering workaround** in `upnpsoap.c`: `/* LG TV ordering bug */ else if( args.client == ELGDevice ) ret = xasprintf(&orderBy, "order by o.CLASS, d.TITLE");` — the client's `SortCriteria` is ignored for LG. — https://github.com/Darkvater/minidlna/blob/bc7ef770e0a3d9f20570fa04aaefadf483361466/upnpsoap.c
  - **Caption workaround**: `/* LG hack: subtitles won't get used unless dc:title contains a dot. */ else if( passed_args->client == ELGDevice && (passed_args->flags & FLAG_HAS_CAPTIONS) ) { ret = asprintf(&alt_title, "%s.", title); … }`
  - **Caption delivery**: LG is served via the `res` element with `protocolInfo="http-get:*:text/srt:*"` (the `FLAG_CAPTION_RES` path), not via `sec:CaptionInfoEx`.
- **Gerbera**: no LG-specific quirk is implemented; the docs list LG only as a supported renderer ("all TVs based on WebOS 6") — https://docs.gerbera.io/en/stable/supported-devices.html
- **Jellyfin**: "LG Smart TV" profile identified by User-Agent substring `LG`, `EnableAlbumArtInDidl=false`, `AlbumArtPn=JPEG_SM`, `MaxAlbumArtWidth/Height=480`, `MaxIconWidth=48` — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/LG%20Smart%20TV.xml
- **Kodi** (renderer side): `EMEDIACONTROLLERQUIRKS_X_MKV` is set when the remote device manufacturer contains "Samsung Electronics" — the same "Samsung wants video/x-mkv" class of hack — https://raw.githubusercontent.com/xbmc/xbmc/master/xbmc/network/upnp/UPnPInternal.cpp

---

### Philips TVs

*(Coverage here is thinner than Samsung/LG: Philips DLNA clients are less frequently discussed in the server issue trackers I searched. What follows is what I could actually source.)*

#### **What the client sends**

- **OBSERVED** — Philips UA: `Linux2.6/0.0 UPnP/1.0 PhilipsIntelSDK/1.4 DLNADOC/1.50` — https://sourceforge.net/p/minidlna/bugs/180/
- **OBSERVED** — **The UA changes to `Streamium` for the media GET when the item is MKV**, but not for AVI:
  > *"But when trying to play an mkv file the http GET request changes to: User-Agent: Streamium. The strange thing is that when the playing an AVI file, the user-agent does not change. I believe that there is something in the response of minidlna in the mkv case that confuses the TV and it changes its user agent to streamium."*
  — https://sourceforge.net/p/minidlna/bugs/180/
- **OBSERVED** — Philips TV device description (as seen by a DLNA client): `FriendlyName: woonkamertv`, `Manufacturer: Philips`, `ManufacturerUrl: http://www.philips.com`, `ModelDescription: UPnP Media Renderer 1.0`, `ModelName: Philips TV DMR`, `ModelNumber: 2k15MTK` — https://github.com/jellyfin/jellyfin-androidtv/issues/256
- **OBSERVED** — A newer Philips TV (50PUD6654/43) publishes **`urn:schemas-upnp-org:device:MediaRenderer:3`** and `RenderingControl:3` / `ConnectionManager:3` / `AVTransport:3` only — https://github.com/stupside/castor/pull/23
- **OBSERVED** — The related Philips middleware used by some Philips/Loewe sets is NetFront: `IPI/1.0 UPnP/1.0 DLNADOC/1.50`; ReadyMedia maps `IPI/1` to `ENetFrontLivingConnect` — https://sourceforge.net/p/minidlna/discussion/879956/thread/bb19aaf1/ and https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c
- **OPEN QUESTION** — Whether Philips TVs send ContentDirectory `Search`, and with what `SearchCriteria`/`SortCriteria`. No captured Philips `Search` request found.

#### **What it chokes on**

- **OBSERVED** — **The byte range must be inclusive of the last byte — off-by-one is fatal.** A Philips user bisected a "MKV files impossible to view" bug down to a single character in `upnphttp.c`:
  > *"you need fix 1 letter in file = upnphttp.c line 1317 — `while( offset < end_offset )` need change for `while( offset <= end_offset )` — '=' important for Philips TV"*
  — https://sourceforge.net/p/minidlna/bugs/245/
  - Upstream commit: *"upnphttp: fix requests for the last byte of a file — Philips TV's apparently do this with MKV files (SF bug #180)."* — https://github.com/Darkvater/minidlna/commit/de7070472ad5f86782defd098905ce80f33ce41d
  - Changelog: *"Fix requests for the last byte of a file, which affected MKV playback on Philips TV's."* (1.1.4) — https://raw.githubusercontent.com/Darkvater/minidlna/master/NEWS
  - Current code confirms the fix: `while( offset <= end_offset )` — https://raw.githubusercontent.com/necropotame/readymedia/master/upnphttp.c
- **OBSERVED** — **MKV specifically fails with "file not available"** even though the same TV plays the same MKV from MediaTomb on the same network:
  > *"The TV does not want to play mkv files, the error message displayed being 'file not available'. These are non-DTS mkv files. Mediatomb, for example, works fine with these files on the same TV without transcoding."*
  — https://sourceforge.net/p/minidlna/bugs/180/
- **OBSERVED** — **Philips external-subtitle support uses a vendor mechanism ReadyMedia never implemented.** A ReadyMedia maintainer: *"Since subtitles are not in the default DLNA specs, every manufacturer has implemented it in their own way. Minidlna sends the subs for LG, O!Play, BubbleUPNP, Samsung as far as I've seen. If you figure out which type Philips is using (CaptionInfoEx, or through protocolInfo), I'm sure Justin can add support for it."* — https://sourceforge.net/p/minidlna/discussion/879956/thread/bb19aaf1/
- **OPEN QUESTION** — Exact `protocolInfo` string required by Philips for MKV. The ReadyMedia/Mediatomb difference above (which server works, which does not) points at `res/@protocolInfo` and/or the container MIME type, but I found no Philips bug stating the required string.
- **OPEN QUESTION** — Whether Philips requires the `sec:`/`pv:` namespaces, or `dc:date`. Not sourced.

#### **Documented failure symptoms**

- *"this problem make impossible view MKV files with Philips TV … you need just change one symbol and all will be good!"* — https://sourceforge.net/p/minidlna/bugs/245/
- *"The TV does not want to play mkv files, the error message displayed being 'file not available'."* — https://sourceforge.net/p/minidlna/bugs/180/
- *"the TV changed its user agent to streamium"* for MKV playback — https://sourceforge.net/p/minidlna/bugs/180/
- *"My old Philips smart TV can't see Gerbera server anymore"* (turned out to be an Ethernet/network problem on the user's side, not a Philips quirk) — https://github.com/gerbera/gerbera/issues/2816

#### **Server-side workarounds observed in existing implementations**

- **ReadyMedia**: the `<=` inclusive-range fix (see above); `ENetFrontLivingConnect` (Philips-family middleware) gets `FLAG_DLNA | FLAG_FORCE_SORT | FLAG_CAPTION_RES` and `ENetFrontLivingConnect` is matched on `IPI/1` — https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c
- **Gerbera**: Philips/Streamium devices are listed as *supported renderers* (Streamium SL-300i/SL-400i/MX-6000i/NP1100/MCi900, WAS7500, WAK3300, WAC3500D, SLA-5500/5520, 37PFL9603D) but there is **no Philips-specific quirk flag** in the current Gerbera — https://docs.gerbera.io/en/stable/supported-devices.html
- **Plex DLNA profile for Philips** identifies the client from the **device description** rather than the User-Agent (`ModelName` substring `NMR`, `Manufacturer` substring `Philips`, `FriendlyName` substring `PhilipsTV`), and remaps containers: `<Container name="mkv" mimeType="video/vnd.dlna.mpeg-tts" />` plus `DlnaVideoProfile container="mpegts" codec="mpeg4" audioCodec="ac3" pn="MPEG_TS_HD_NA" mimeType="video/vnd.dlna.mpeg-tts"` — https://raw.githubusercontent.com/rivarolle/plex-dlna-profiles/master/Philips.xml
- **Plex/Jellyfin**: the Philips Android-TV app notes that *"Philips televisions search the network for DLNA supported servers"* as soon as they are switched on — https://github.com/jellyfin/jellyfin-androidtv/issues/256

---

### Panasonic TVs (VIERA)

#### **What the client sends**

- **OBSERVED** — UA: `Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50` — https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc (`// User-Agent(actionReq): Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50`) and https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Panasonic%20Viera.xml
- **OBSERVED** — A Panasonic client is also identified by `User-Agent` prefix `Panasonic MIL DLNA` in the Jellyfin profile (`<HttpHeaderInfo name="User-Agent" value="Panasonic MIL DLNA" match="Substring" />`), and Plex logs *"Mapped client to profile Panasonic Viera 3D TV using header User-Agent: Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50"* — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Panasonic%20Viera.xml and https://github.com/uglymagoo/plexmediaserver-installer/issues/3
- **OBSERVED** — Panasonic sends **hardcoded SortCriteria**, which is why servers override it: *"This quirk might help if your client sends hardcoded SortCriteria, e.g. old Panasonic TV sets"* — https://docs.gerbera.io/en/stable/supported-devices.html
- **OBSERVED** — The Panasonic DLNA client sends `SortCriteria` of `+upnp:class,+dc:date` (from the Gerbera issue analysis) — https://github.com/gerbera/gerbera/issues/2531
- **OPEN QUESTION** — Whether Panasonic sends `Search`/`SearchCriteria`, and its typical `RequestedCount`. No captured Panasonic `Browse` request was found in this research (the one source that showed a Panasonic Browse log was not fetchable).

#### **What it chokes on**

- **OBSERVED** — **It requires `dc:date` on containers and items, otherwise it reports the files as unsupported.** Field analysis:
  > *"Problem1. Panasonic VIERA do not accept none ascii `<res>` (like utf-8 filename) `<res>`"*
  > *"Problem2. require SortCriteria [+upnp:class,+dc:date]. usualy Gerbera not set dc:date."*
  and the resulting Gerbera SQL showed the sort being executed as an `INNER JOIN mt_metadata ON property_name = 'dc:date'` — i.e. **items without a `dc:date` simply drop out of the sorted result set**. The fix applied was to force a `dc:date` into the metadata of every container and item.
  — https://github.com/gerbera/gerbera/issues/2531
- **OBSERVED** — **Non-ASCII (UTF-8) filenames inside the `res` URL are rejected.** The patch replaces the real filename with a generic `file.<ext>`: `return fmt::format("{}{}", urlExt, extension);` — https://github.com/gerbera/gerbera/issues/2531
- **OBSERVED** — **Virtual containers confuse it**: *"it looks like the Panasonic TV has a problem with the virtual containers and how they are created"*, and *"you can try to remove `containers` which cause trouble with clients that cache all metadata before showing any files."* — https://github.com/gerbera/gerbera/issues/2531
- **OBSERVED** — **It opens many simultaneous connections**, so the server's child-process limit matters. ReadyMedia commit: *"At least some Panasonic clients try to open more than 5 simultaneous connections to the server. If we keep the default of 5 max children, it results in choppy playback on those clients. Make this setting configurable, and default to 50 max connections."* — https://github.com/Darkvater/minidlna/commit/1a78a94f7036aee13e62eef8dbfb430eb65df5c3
- **OBSERVED** — **External SRT subtitles do not work** on a VIERA (PR55VT50): *"Internal subtitles in MKV works well … I have tried to run those M2TS with external SRT (named same as movie), but no success."* and the reporter's conclusion: *"It looks like TVs client fail."* — https://sourceforge.net/p/minidlna/bugs/288/
- **OBSERVED** — **Some Panasonic TVs only become visible to the server if the server is restarted after the TV.** — https://github.com/UniversalMediaServer/UniversalMediaServer/issues/918 (search-result snippet only; the issue page itself did not load → ASSUMPTION on the wording, but the issue title *"Panasonic TV only recognized via UPnP if server started after TV"* was returned by the GitHub issue search and is quoted here as a title, not as body text).
- **OPEN QUESTION** — Whether VIERA is sensitive to `res/@duration` format, to `upnp:class` sub-classes, or to a missing `sec:httpDLNA`-style element. Not sourced.

#### **Documented failure symptoms**

- *"my Panasonic TV did show the folders, but now it claims that the files(folders) are not supported."* — https://github.com/gerbera/gerbera/issues/2531
- *"Panasonic VIERA do not accept none ascii `<res>` (like utf-8 filename)"* — https://github.com/gerbera/gerbera/issues/2531
- *"I have Panasonic PR55VT50. Internal subtitles in MKV works well"* but external SRT *"no success"* — https://sourceforge.net/p/minidlna/bugs/288/
- *"resulting in choppy playback on those clients"* (too many simultaneous connections) — https://github.com/Darkvater/minidlna/commit/1a78a94f7036aee13e62eef8dbfb430eb65df5c3

#### **Server-side workarounds observed in existing implementations**

- **ReadyMedia**: `clients.c` entry `{ EPanasonic, FLAG_DLNA | FLAG_FORCE_SORT, "Panasonic", "Panasonic", EUserAgent }` — the `FLAG_FORCE_SORT` makes the server ignore the client's SortCriteria and use `order by o.CLASS, d.DISC, d.TRACK, d.TITLE` — https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c and https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c
  - Changelog: *"Add forced sorting support for Panasonic devices."* (1.1.0) and *"Add a new force_sort_criteria option, to globally override the SortCriteria value sent by the client."* (1.1.2) and *"Add caption support for Panasonic TV's."* and *"Fix choppy playback with some file types on Panasonic clients by increasing the max connection limit."* (1.1.2) — https://raw.githubusercontent.com/Darkvater/minidlna/master/NEWS
  - Commit `e1c03bf` *"Add caption support for Panasonic TVs."*, commit `1c748c1` *"Consolidate client detection code. Add forced sorting support for Panasonic devices."* — https://github.com/Darkvater/minidlna/commit/e1c03bf689b268d98a2f3dda34adbe74b9c28c33 and https://github.com/Darkvater/minidlna/commit/1c748c1d5a85ae4f44ec36962d4f74933c2c4764
- **Gerbera**: `PanasonicTV` client type with the `FORCE_SORT_CRITERIA_TITLE` quirk (*"Always sort by `dc:title` ascending when browsing or searching."*) — https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc and https://docs.gerbera.io/en/stable/supported-devices.html
  - Note: the docs record *"Changed in version 3.1.0: Removed `PANASONIC`"* — the standalone `PANASONIC` flag was dropped, leaving only `FORCE_SORT_CRITERIA_TITLE`.
- **Jellyfin**: "Panasonic Viera" profile — matched on User-Agent substring `Panasonic MIL DLNA`; `AlbumArtPn=JPEG_SM`; `MaxAlbumArtWidth/Height=480`; `MaxIconWidth=48`; `EnableAlbumArtInDidl=false`; `TimelineOffsetSeconds=10`; `IgnoreTranscodeByteRangeRequests=false`; and it adds the **`xmlns:pv="http://www.pv.com/pvns/"`** attribute to the DIDL root — https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Panasonic%20Viera.xml
- **Plex**: *"Mapped client to profile Panasonic Viera 3D TV using header User-Agent: Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50"* — https://github.com/uglymagoo/plexmediaserver-installer/issues/3

---

## Cross-family checklist distilled for PhairDLNA

*(OBSERVED unless marked; each line points back to the detailed sections above.)*

1. Return **at most `RequestedCount`** items in a Browse response — Samsung dies otherwise (https://github.com/jellyfin/jellyfin/issues/2335).
2. Treat an **empty `<ObjectID>`** as the root container (Samsung sends it) — https://sourceforge.net/p/minidlna/bugs/311/, https://raw.githubusercontent.com/rclone/rclone/master/cmd/serve/dlna/cds.go
3. Accept **`RequestedCount` = -1** (Samsung sort menu) without erroring — https://sourceforge.net/p/minidlna/bugs/134/
4. Emit **plain `upnp:class`** values (`object.item.videoItem`, `object.container.storageFolder`) — vendor sub-classes crash Samsung — https://github.com/xbmc/xbmc/issues/22828
5. Emit **`dc:date`**, and prefer `yyyy-MM-dd` (Samsung) — always set it (Panasonic) — https://github.com/jellyfin/jellyfin/issues/2335, https://github.com/gerbera/gerbera/issues/2531
6. **Never chunk-encode** for HTTP/1.0 clients (Samsung) — https://github.com/gerbera/gerbera/pull/3954
7. **Byte ranges must be inclusive of the last byte** (Philips) — https://sourceforge.net/p/minidlna/bugs/245/
8. Keep **`res/@protocolInfo` + MIME type** exact; Samsung expects the "wrong" `video/x-mkv`, Panasonic rejects non-ASCII `res` URLs, LG wants `DLNA.ORG_CI=1` on thumbnails — https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c , https://github.com/gerbera/gerbera/issues/2531 , https://github.com/gerbera/gerbera/issues/2557
9. Always send **`Content-Length`**; avoid chunked for HTTP/1.0 (Samsung) — https://github.com/gerbera/gerbera/pull/3954
10. Implement Samsung's vendor extensions if you want its "BasicView" menus: `X_GetFeatureList`, `X_SetBookmark`, `X_GetObjectIDfromIndex`, `X_GetIndexfromRID`, `CaptionInfo.sec`, `sec:dcmInfo` — https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c , https://github.com/gerbera/gerbera/blob/master/src/upnp/quirks.cc

---

## Sources

| URL | What it contributed |
|---|---|
| https://sourceforge.net/p/minidlna/bugs/311/ | Samsung ES8000 `X_GetFeatureList` + `Browse` with empty ObjectID → UPnPError 402; `GET /icons/lrg.png HTTP/1.0` with `getcontentFeatures.dlna.org:1` / `transferMode.dlna.org:Interactive`; the DCM10 `ObjectID="0"` patch; "Samsung client to crash after a period" paging symptom. |
| https://sourceforge.net/p/minidlna/bugs/210/ | Samsung `X_GetFeatureList` response must be well-formed/escaped, else "endless loop"; note that `GetFeatureList` is standard only from ContentDirectory:2. |
| https://sourceforge.net/p/minidlna/bugs/134/ | Samsung sort menu sends `RequestedCount` -1 → UPnPError 701; Samsung's special sort ObjectIDs (`V_T`, `V_L`, `A_L`) found by sniffing Samsung's own AllShare server. |
| https://sourceforge.net/p/minidlna/bugs/125/ | Samsung music browse uses `PARENT_ID='A_L'`, which standard servers do not have → error on the TV. |
| https://sourceforge.net/p/minidlna/bugs/165/ | Samsung sends `Filter: *`, so vendor filter flags (FILTER_SEC_DCM_INFO) are not set → bookmarks break. |
| https://sourceforge.net/p/minidlna/bugs/237/ | Samsung F-series regression caused by advertising DCM10; the "remove FLAG_SAMSUNG_DCM10" workaround; DCM10 breaks E-series bookmarks. |
| https://sourceforge.net/p/minidlna/bugs/231/ | Samsung D5100 regression from the `SEC_HHP_` UA merge; "only Music" symptom; `SEC_HHP_HTS-*` also must not get DCM10. |
| https://sourceforge.net/p/minidlna/bugs/243/ | v1.1.3 lost bookmarks on Samsung E series. |
| https://sourceforge.net/p/minidlna/bugs/44/ | Files >2 GB reported as 2 GB on some Samsung firmware. |
| https://sourceforge.net/p/minidlna/bugs/79/ | Samsung `<<`/`>>` FF/REW keys: "function is unavailable"; client-side limitation. |
| https://sourceforge.net/p/minidlna/bugs/111/ | Samsung UE40C6500 rewind: left/right keys work, `>>`/`<<` do not. |
| https://sourceforge.net/p/minidlna/bugs/74/ | Samsung Series A AVI MIME trick broke seeking on B-series (`SamsungWiselinkPro/1.0`). |
| https://sourceforge.net/p/minidlna/bugs/78/ | "cant see and open files from source" + UPnPError 701 on Samsung D6530. |
| https://sourceforge.net/p/minidlna/bugs/157/ | "Not possible to access to the device" + UPnPError 701 on Samsung D6xxx. |
| https://sourceforge.net/p/minidlna/bugs/190/ | Samsung UE40D5000 "unknown file format" regression. |
| https://sourceforge.net/p/minidlna/bugs/211/ | LG TV scans the whole server on every power-on; network flood. |
| https://sourceforge.net/p/minidlna/bugs/181/ | LG HX806PH external .srt subtitles never offered. |
| https://sourceforge.net/p/minidlna/bugs/245/ | Philips: the one-character inclusive-range fix (`<=`) in `upnphttp.c`. |
| https://sourceforge.net/p/minidlna/bugs/180/ | Philips UA `Linux2.6/0.0 UPnP/1.0 PhilipsIntelSDK/1.4 DLNADOC/1.50`; UA switches to `Streamium` for MKV; "file not available". |
| https://sourceforge.net/p/minidlna/bugs/288/ | Panasonic VIERA external SRT subtitles fail; internal MKV subtitles work. |
| https://sourceforge.net/p/minidlna/bugs/324/ | LG TV contact crashes minidlna (NULL mime). |
| https://sourceforge.net/p/minidlna/discussion/879956/thread/258a6331f3/ | "Samsung TV workaround" thread referencing a Samsung community thread about newer Samsung TVs not showing DLNA servers. |
| https://sourceforge.net/p/minidlna/discussion/879956/thread/bb19aaf1/ | Subtitles are vendor-specific; ReadyMedia supports them for LG/O!Play/BubbleUPnP/Samsung; Philips/NetFront UA `IPI/1.0 UPnP/1.0 DLNADOC/1.50`. |
| https://raw.githubusercontent.com/Darkvater/minidlna/master/NEWS | ReadyMedia changelog: DCM10 enable/disable, "Work around LG TV ordering bug", "Update support for LG2012 TV", "Fix subtitles on LG TV's for items whose titles don't have a dot in them", "Fix requests for the last byte of a file, which affected MKV playback on Philips TV's", "Add forced sorting support for Panasonic devices", "Fix choppy playback with some file types on Panasonic clients by increasing the max connection limit". |
| https://github.com/Darkvater/minidlna/commit/b94f04d6ef5b9532088fadb9a874d5317d519495 | LG TV ordering bug: TV requests 54 but pages at 20 → duplicate folders / missing files; fix = server-side ordering. |
| https://github.com/Darkvater/minidlna/commit/de7070472ad5f86782defd098905ce80f33ce41d | Philips inclusive last-byte range fix. |
| https://github.com/Darkvater/minidlna/commit/e3a53fc8a3b78bdb30dfe6b84bd8c41239ef470c | DCM10 advertising makes Samsung C/D players always browse ContainerID 1. |
| https://github.com/Darkvater/minidlna/commit/ce16739213cfe4e652b5655606937bf8041a339c | `X_GetFeatureList` must be hidden when a custom root container is set. |
| https://github.com/Darkvater/minidlna/commit/c1bb37a38754e0fed6a55a2ebf204c68c3d2135b | DCM10 = video bookmarks + Samsung "BasicView" support. |
| https://github.com/Darkvater/minidlna/commit/4ec6cd005358f3474ce9caec4f0954b4756570ed | Samsung Series J dropped the old custom subtitle method → expose both. |
| https://github.com/Darkvater/minidlna/commit/1a78a94f7036aee13e62eef8dbfb430eb65df5c3 | Panasonic opens >5 simultaneous connections → choppy playback; default raised to 50 children. |
| https://github.com/Darkvater/minidlna/commit/533907539357df45d1923638d11192b98f7bf280 | LG 2012 no longer needs the '.' in the title for captions. |
| https://github.com/Darkvater/minidlna/commit/0cf2e5731a4f212558e56d59c91eb12c9e85b856 | "We need to trick A series only into playing AVI files" (Samsung A-series MIME hack). |
| https://github.com/Darkvater/minidlna/commit/fc6b47fe6aca39699668cbba74d110b988270abc | Malformed Samsung GetFeatureList reply → renderer endless loop. |
| https://raw.githubusercontent.com/necropotame/readymedia/master/clients.c | Samsung/LG/Panasonic/NetFront client detection table and all quirk flags. |
| https://raw.githubusercontent.com/necropotame/readymedia/master/clients.h | Flag definitions incl. `FLAG_SAMSUNG`, `FLAG_SAMSUNG_DCM10`, `FLAG_CONVERT_MS`, `FLAG_CAPTION_RES`. |
| https://raw.githubusercontent.com/necropotame/readymedia/master/upnpsoap.c | Samsung x-mkv MIME hack; LG title-dot hack; LG ordering override; `X_GetFeatureList`/`X_SetBookmark`; ms↔s bookmark conversion; `sec:dcmInfo`. |
| https://raw.githubusercontent.com/necropotame/readymedia/master/upnphttp.c | HTTP/1.0 vs chunked handling; inclusive range loop `while (offset <= end_offset)`; `CaptionInfo.sec` header; Samsung `transferMode.dlna.org:Interactive` tolerance. |
| https://raw.githubusercontent.com/necropotame/readymedia/master/upnpdescgen.c | `genRootDescSamsung` adding `sec:ProductCap`/`sec:X_ProductCap` = `smi,DCM10,getMediaInfo.sec,getCaptionInfo.sec`; advertised services (ContentDirectory:1, ConnectionManager:1, X_MS_MediaReceiverRegistrar). |
| https://raw.githubusercontent.com/necropotame/readymedia/master/containers.c | Samsung DCM10 magic containers `I`/`A`/`V`. |
| https://downloads.sourceforge.net/project/minidlna/minidlna/1.3.3/minidlna-1.3.3.tar.gz | ReadyMedia 1.3.3 sources: `clients.c` Series Q/QN entries and `FLAG_CONVERT_MS`, `upnpsoap.c` ms↔s bookmark conversion, `clients.h` flag definitions. |
| https://github.com/Darkvater/minidlna/blob/bc7ef770e0a3d9f20570fa04aaefadf483361466/upnpsoap.c | Upstream ReadyMedia source containing the "LG TV ordering bug" override, the Samsung "x-mkv" MIME hack and the LG title-dot caption hack. |
| https://github.com/jellyfin/jellyfin/issues/2335 | Samsung: "TV goes into error state and stop browsing when receives dlna response with NumberReturned > RequestedCount"; dc:date yyyy-MM-dd; "empty folders" symptoms; feature-list container ids. |
| https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Didl/DidlBuilder.cs | `dc:date` as `yyyy-MM-dd`; "Samsung sometimes uses 1 as root". |
| https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/ContentDirectory/ControlHandler.cs | `samsung.com_BASICVIEW` feature list with container id `"0"`. |
| https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Samsung%20Smart%20TV.xml | Samsung device profile (UA `SEC`, `xmlns:sec`, album art flags, timeline offset). |
| https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/LG%20Smart%20TV.xml | LG device profile (UA `LG`, album-art limits, `JPEG_SM`). |
| https://raw.githubusercontent.com/jellyfin/jellyfin-plugin-dlna/master/src/Jellyfin.Plugin.Dlna/Profiles/Xml/Panasonic%20Viera.xml | Panasonic Viera profile (`Panasonic MIL DLNA` UA, `xmlns:pv`, JPEG_SM, 480px album art). |
| https://github.com/gerbera/gerbera/issues/839 | LG webOS seeking needs `contentFeatures.dlna.org` + `realTimeInfo.dlna.org` + `transferMode.dlna.org`; thumbnails >160px ignored. |
| https://github.com/gerbera/gerbera/issues/2557 | LG webOS 5 thumbnail protocolInfo needs `DLNA.ORG_CI=1` + `DLNA.ORG_FLAGS` (QNAP comparison). |
| https://github.com/gerbera/gerbera/issues/2531 | Panasonic: non-ASCII `res` URLs rejected; `dc:date` + SortCriteria required; virtual containers problematic; the `file.ext` filename patch. |
| https://github.com/gerbera/gerbera/issues/3459 | Tizen QN85C not finding the server; UA `SHP, UPnP/1.0, Samsung UPnP SDK/1.0`; renaming the TV + power cycle fixes discovery. |
| https://github.com/gerbera/gerbera/issues/3926 | Samsung sends `HEAD ... HTTP/1.0` with `getCaptionInfo.sec: 1`; expects a `CaptionInfo.sec` response header. |
| https://github.com/gerbera/gerbera/issues/3152 | Tizen Samsung does not show external SRT subtitles; old Linux-based J5000 does. |
| https://github.com/gerbera/gerbera/issues/2816 | Philips TV not seeing the server (turned out to be a network issue). |
| https://github.com/gerbera/gerbera/pull/3954 | Chunked encoding + HTTP/1.0 → libupnp 406; `UPNP_USING_CHUNKED` → `UPNP_UNTIL_CLOSE`; Samsung TV could not play transcoded resources. |
| https://github.com/gerbera/gerbera/pull/3701 | Mapping of MiniDLNA Samsung flags onto Gerbera quirks; Series A/B/QN support. |
| https://github.com/gerbera/gerbera/blob/master/src/upnp/quirks.cc | Samsung quirk implementations (feature list, feature root incl. `V_T`/`A_T` IDs, object-ID/index mapping, bookmarks, `CaptionInfo.sec`). |
| https://github.com/gerbera/gerbera/blob/master/src/upnp/client_manager.cc | Panasonic/NetFront/Samsung client detection by User-Agent. |
| https://github.com/gerbera/gerbera/blob/master/src/request_handler/device_description_handler.cc | `sec:ProductCap`/`sec:X_ProductCap` for DCM10 clients; `M-DMS-1.50` for Samsung. |
| https://docs.gerbera.io/en/stable/supported-devices.html | Quirk semantics (SAMSUNG_HIDE_DYNAMIC, DCM10, CAPTION_PROTOCOL, bookmark sec/ms), supported LG/Philips renderers, `FORCE_SORT_CRITERIA_TITLE` for old Panasonic sets. |
| https://forum.serviio.org/viewtopic.php?f=10&t=30057 | Samsung 6 Series UA, M-SEARCH targets (MediaServer:1 and RemoteUIServerDevice:1), deviceDescription/serviceDescription fetches, `ContentDirectory:1#GetSearchCapabilities`. |
| https://forum.rclone.org/t/dlna-not-working-on-recent-samsung-tv/50480 | 2023/24 Samsung Tizen UA `SEC_HHP_55" OLED`, `BrowseMetadata` + `RequestedCount 0`, TV shows no files; rclone root description with `sec:ProductCap`. |
| https://raw.githubusercontent.com/rclone/rclone/master/cmd/serve/dlna/cds.go | rclone workarounds: empty ObjectID → root; strip file extension from `dc:title` "to prevent Samsung TVs from duplicating it"; `X_GetFeatureList`/`X_SetBookmark`; `sec:ProductCap`. |
| https://eu.community.samsung.com/t5/tv/nas-won-t-show-on-tv/td-p/11370913 | Samsung community: 2024 Q80D / QN95 not showing the NAS; renaming the TV to `SamsungTV` fixes it. |
| https://developer.samsung.com/smarttv/develop/guides/fundamentals/retrieving-platform-information.html | Samsung per-year browser/app User-Agent table (Tizen 2.3 … 10.0, TV SDK 4.0/4.5/5.1). |
| https://github.com/xbmc/xbmc/issues/22828 | Samsung client vs vendor `upnp:class`, empty leaves, "crash the TV dlna client", base64 object IDs. |
| https://github.com/xbmc/xbmc/commit/7f5c291395d1245a2c10b776cc784083b21e11e4 | Kodi: UA `Samsung/1.0 UPnP/1.0` → ONLYSTORAGEFOLDER + BASICVIDEOCLASS quirks. |
| https://github.com/xbmc/xbmc/commit/0a7eaf90eb1b7acc3d7636894503cc7e0070240e | Kodi: `virtualpath://upnproot` must be flagged as a folder for Samsung. |
| https://raw.githubusercontent.com/xbmc/xbmc/master/xbmc/network/upnp/UPnPInternal.cpp | Kodi/Platinum client-quirk mechanism; `EMEDIACONTROLLERQUIRKS_X_MKV` for Samsung manufacturer. |
| https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1960 | LG webOS Browse with full attribute Filter and `RequestedCount 30`; "known problem with LG TVs, they like to browse all of the server folders by themselves". |
| https://emby.media/community/topic/23009-lg-tv-lw5590-dlna-connection-failed-to-htpc/ | LG UA `LGE_DLNA_SDK/1.5.0`, `BrowseMetadata` RequestedCount 0 then `BrowseDirectChildren` RequestedCount 16, Filter list. |
| https://emby.media/community/topic/20785-how-work-dlna-profiles/ | LG `BrowseMetadata` request with `RequestedCount 0`, Filter `*`. |
| https://emby.media/community/topic/17224-dlna-profile-for-lg-webos-tv/ | LG webOS profile thread; `RequestedCount 99` browse and the long `res@…` Filter list. |
| https://emby.media/community/topic/123549-playback-error-lg-tv-livetv/ | LG webOS Browse with `RequestedCount 30` and the `res@…` Filter. |
| https://raw.githubusercontent.com/rivarolle/plex-dlna-profiles/master/Philips.xml | Plex Philips client profile: identification via device description (`NMR`, `Philips`), mkv → `video/vnd.dlna.mpeg-tts`, `MPEG_TS_HD_NA`. |
| https://github.com/stupside/castor/pull/23 | Philips 50PUD6654/43 publishes MediaRenderer:3 and AVTransport:3 only; version tolerance of UPnP service types. |
| https://github.com/jellyfin/jellyfin-androidtv/issues/256 | Philips TV device description fields (ModelName "Philips TV DMR", ModelNumber 2k15MTK, Manufacturer Philips). |
| https://github.com/uglymagoo/plexmediaserver-installer/issues/3 | Plex mapping of `Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50` to the Panasonic Viera profile. |
| https://github.com/azizulhakim/u-pot/blob/1781156d/device-descriptors/Samsung%20SmartTV/dmr.xml | (ASSUMPTION — search snippet only) Samsung TV DMR description containing `sec:ProductCap` `Tizen,Y2017,WebURIPlayable,…`. |
