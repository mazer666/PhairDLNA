# DLNA / UPnP AV MediaServer Ecosystem & Client Compatibility Research

**Project:** PhairDLNA — a new, from-scratch, pure-Rust DLNA/UPnP AV Media Server (macOS first, AGPL, privacy-first)
**Phase:** Requirements — no implementation
**Document date:** see git/file timestamp
**Author role:** DLNA Ecosystem Analyst

---

## 0. How to read this document

Every non-trivial statement carries one of four labels:

| Label | Meaning |
|---|---|
| **STANDARD** | Normative behaviour defined by UPnP Forum / DLNA specifications, or a protocol fact that is not in dispute. |
| **OBSERVED** | Empirically documented behaviour of a real implementation or client, backed by a source URL that was actually fetched for this document. |
| **ASSUMPTION** | A plausible inference that is **not** backed by a fetched source. Treated as a hypothesis to be validated on hardware. |
| **OPEN QUESTION** | A question this document could not answer from primary sources. |

**Method rules honoured in this document**

* No claim of having run, tested, captured or sniffed anything. Nothing here was verified on hardware.
* No issue numbers are invented. Every issue reference is a URL that was fetched during this research.
* Where a statement could not be sourced, it is explicitly marked **ASSUMPTION** or **OPEN QUESTION** rather than being quietly asserted.
* No personal media, no real user paths, no device MACs/IPs of individuals.
* Client-quirk statements carry their source URL inline. Server-side statements cite repository paths and doc URLs.

---

## 1. Existing implementations

### 1.1 Landscape at a glance

| Project | Language | License | Protocol coverage | Multi-root | Index | Subs | Transcode | Maintainership signal |
|---|---|---|---|---|---|---|---|---|
| **ReadyMedia / MiniDLNA** | C | GPL-2.0 (discovery code BSD, from MiniUPnP) | SSDP + HTTP + SOAP, `MediaServer:1`, `ContentDirectory:1`, `ConnectionManager:1`, `X_MS_MediaReceiverRegistrar:1`, DLNA `DMS-1.50`, GENA subscribe | Yes (`media_dir=` x N; `merge_media_dirs=` option) | SQLite `files.db` + inotify/kqueue | Yes (`.srt`/`.smi` external + `sec:CaptionInfoEx`, `pv:subtitleFileType`) | Yes (ffmpeg pipe) | Release 1.3.3 on 2023‑06‑02; patch‑level activity since; single primary maintainer |
| **Gerbera** (ex‑MediaTomb) | C++ | GPL-2.0 | SSDP + HTTP + SOAP, `MediaServer:1`, `ContentDirectory:1`, `ConnectionManager:1`, `X_MS_MediaReceiverRegistrar:1`, DLNA partial (`DMS-1.50`), GENA | Yes (multiple `<location>`) | SQLite (SQLite3 / MySQL / MariaDB) | Yes (external + embedded, `pv:`/`sec:`) | Yes (ffmpeg profile-driven) | Active: docs say "Copyright … 2016‑2026"; large, actively triaged issue tracker |
| **Platinum UPnP SDK** (Plutinosoft) | C++ | GPL-2.0 **or** commercial (dual-license) | Full UPnP stack (device/ctrl point) + sample MediaServer / MediaRenderer / Control Point | Sample app only | None (sample app walks the filesystem live) | No | No | Repo last push 2023‑06‑15; API frozen since ~2010 |
| **uPnP Media Server** (CyberGarage) | Java | Apache-2.0 (`cybergarage-upnp` Maven: `org.cybergarage.upnp:core`, `:std`) | SSDP + SOAP + GENA, `MediaServer:1` + `ContentDirectory:1` example | Example app only | None | No | No | Active library, but the media-server piece is a reference example, not a product |
| **Asset UPnP** (Illustrate) | — | **Proprietary / closed source** (free 30‑day trial + paid Premium) | DLNA audio‑only DMS | Yes ("Asset Collections") | Own DB | No (audio only) | Audio only (WAVE/LPCM/mp3) | Actively sold, 2026 build in the wild |
| **Serviio** | Java | Proprietary (free / Pro) | DLNA MediaServer + PlayTo | Yes | Yes | Yes | Yes | Actively sold |
| **Universal Media Server** | Java | GPL-2.0 | DLNA DMS, `PlayTo`, per‑renderer profiles | Yes | Yes | Yes | Yes (ffmpeg/mplayer) | Active: repo pushed 2026‑11‑14, 2.7k stars, 406 open issues |
| **Plex Media Server** | C++ (closed) | Proprietary (Plex Pass features) | DLNA DMS built on **Platinum + Neptune** | Yes | Yes | Yes | Yes | Active; **DLNA now off by default** for Plex Home |
| **Emby Server** | C# (closed) | Proprietary (free / Premiere) | DLNA DMS + "DLNA Play To", per‑device XML profiles | Yes | Yes | Yes | Yes | Active; DLNA off by default in newer releases |
| **Jellyfin DLNA plugin** | C# | GPL-3.0 | DLNA DMS + "PlayTo", **32 shipped device profiles** | Yes | Yes | Yes | Yes | Very active: pushed 2026‑11‑11, 91 stars, 92 open issues |
| **rclone `serve dlna`** (via `anacrolix/dms`) | Go | MIT | SSDP + SOAP, `MediaServer:1`, `ContentDirectory:1` | Single remote | No index | No | No | Active upstream in rclone |
| **crab-dlna** | Rust | MIT OR Apache-2.0 | DLNA **client / renderer** (not a server) | — | — | — | — | Small, active (99 stars) |
| **rusty-DLNA** | Rust | MIT | DLNA server, "dependency-free, purely safe Rust" | — | — | — | — | 40 stars, pushed 2026‑10‑03 — **direct competitor to PhairDLNA** |

**Note on the requested "Servdlna" entry.** No project named `servdlna` could be located in any primary source: the GitHub repository search API returns `total_count: 0` for `servdlna` and `servdlna in:name`, `https://github.com/servdlna/servdlna`, `xsco/servdlna` and `sourceforge.net/projects/servdlna/` are all 404, and the name does not appear in the Wikipedia "List of UPnP AV media servers and clients". The closest living matches are **Serviio** (proprietary Java), **rclone `serve dlna`**, **simpleDLNA** (C#, BSD‑2) and **rusty-DLNA** (Rust, MIT). → **OPEN QUESTION** — confirm the intended project before comparing against it.

---

### 1.2 ReadyMedia / MiniDLNA

**OBSERVED** — The canonical tree is the SourceForge git (`https://sourceforge.net/p/minidlna/git/ci/master/tree/`); this document quotes `clients.c`, `upnpsoap.c`, `upnphttp.c`, `upnpdescgen.c`, `upnpevents.c`, `options.c`, `minidlna.conf` from that tree.

* **Language / license.** C. GPL‑2.0 for the server; the SSDP/discovery layer is inherited from **MiniUPnP** under the BSD revised licence (`LICENCE.miniupnpd`). *(`minidlna.c` header: "MiniDLNA is free software; … version 2 of the GNU General Public License"; `README`: "Parts of the software including the discovery code are licensed under the BSD revised license".)*
* **Architecture.** Single daemon, fork‑per‑connection HTTP server (`USE_FORK`), SQLite index (`files.db`), `monitor.c` with inotify (`monitor_inotify.c`) and kqueue (`monitor_kqueue.c`) back‑ends, `tagutils` + `libavformat`/ffprobe for metadata, `libexif`/`libjpeg`/`libid3tag`/`libFLAC`/`libvorbis` for tags and artwork.
* **Protocol coverage — exact, from the device description template** (`upnpdescgen.c`):
  * `specVersion` major 1 / minor 0
  * `deviceType` = `urn:schemas-upnp-org:device:MediaServer:1`
  * `dlna:X_DLNADOC` = `DMS-1.50`
  * services: `urn:schemas-upnp-org:service:ContentDirectory:1` / SCID `urn:upnp-org:serviceId:ContentDirectory`; `urn:schemas-upnp-org:service:ConnectionManager:1` / SCID `urn:upnp-org:serviceId:ConnectionManager`; `urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1` / SCID `urn:microsoft.com:serviceId:X_MS_MediaReceiverRegistrar`
  * **No `ContentDirectory:2` and no `MediaServer:2`.**
  * GENA: `upnpevents.c` implements SUBSCRIBE for the ContentDirectory, ConnectionManager and X_MS_MediaReceiverRegistrar event URLs (initial event vars are produced by `getVarsContentDirectory` etc.).
* **Multi‑root.** `minidlna.conf` supports multiple `media_dir=` lines, each optionally restricted by type prefix (`A`, `V`, `P`, `PV`), plus `merge_media_dirs=` (default off) and `root_container=`. *(`minidlna.conf`; `options.c` maps `merge_media_dirs`, `root_container`, `max_connections`, `notify_interval`.)*
* **Indexing.** SQLite, `DB_VERSION` migration path, non‑destructive rescan, `inotify`/kqueue change monitoring, `.nfo` sidecar support.
* **Subtitles.** External SRT (and SMI) discovered next to the video; exposed to clients in three different ways depending on client: a `res` with `protocolInfo="http-get:*:text/srt:*"`, a `sec:CaptionInfoEx` element, and `pv:subtitleFileType` / `pv:subtitleFileUri` attributes on the `res`. See `upnpsoap.c` (`add_res`, `FLAG_CAPTION_RES`, `FILTER_SEC_CAPTION_INFO_EX`, `FILTER_PV_SUBTITLE`) and `clients.h` (`FLAG_CAPTION_RES`).
* **Transcoding.** ffmpeg pipe with a separate "seek" directory scheme; `Accept-Ranges: none` and `DLNA.ORG_OP=10` (transcoded) are advertised for transcoded resources.
* **Maintainership.** Last release **1.3.3, 2023‑06‑02** (changelog: HTTP chunk‑length parsing fix, Dutch/Swedish translations, symlink deletion fix). 1.3.2 was 2022‑08‑30; 1.3.0 2020‑11‑24. SourceForge tickets were still being opened/updated in 2025 (e.g. ticket #86 opened 2025‑08‑07). Effectively one primary maintainer (Justin Maggard) plus occasional contributors.
* **Strengths.** Extremely broad container/codec knowledge (its `upnpsoap.c` emits DLNA `DLNA.ORG_PN` profile names such as `MPEG_TS_SD_NA`, `MPEG_PS_NTSC`, `AVC_TS_HD_50_AC3`), a long list of per‑client quirks, correct HTTP range semantics, and a tiny footprint.
* **Weaknesses.** C, single‑threaded control plane, SQLite schema churn (full rescan on `DB_VERSION` mismatch), no transcoding‑aware seeking, `ContentDirectory:1` only, a client‑quirk model that is a flat static table, and effectively unmaintained for the last two years.

**The ReadyMedia changelog is itself a catalogue of client bugs.** Verbatim entries (from `https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/`):

* 1.1.4 — *"Fix requests for the last byte of a file, which affected MKV playback on Philips TV's."*
* 1.1.5 — *"Work around LG TV ordering bug."*
* 1.1.2 — *"Fixed some clients playing artwork instead of movie."*; *"Fix choppy playback with some file types on Panasonic clients by increasing the max connection limit."*; *"Support DLNA/UPnP-AV searches issued by clients using the Grilo framework."*; *"Add an extra folder level if there are multiple media locations."*
* 1.1.3 — *"Disable Samsung DCM10 capability, as it breaks compatibility with new models."*; *"Fix Samsung browsing when root_container is set."*
* 1.2.0 — *"Add quirk to fix video playback on Hyundai TVs."*; *"Improve Samsung BD-J5500 support."*
* 1.2.1 — *"Added a workaround for video thumbnails on some Samsung clients."*; *"Fixed content browsing issues with some Samsung TVs."*; *"Added Movian client detection and subtitle support."*
* 1.3.0 — *"Fixed an issue with VLC where browse results would be truncated."*; *"Worked around bad SearchCriteria from the Control4 Android app."*; *"Fixed bookmarks on Samsung Q series."*

---

### 1.3 Gerbera

**OBSERVED** — Sources: `https://github.com/gerbera/gerbera`, `https://raw.githubusercontent.com/gerbera/gerbera/master/README.md`, `https://docs.gerbera.io/en/latest/`, `https://docs.gerbera.io/en/latest/supported-devices.html`, `https://raw.githubusercontent.com/gerbera/gerbera/master/src/upnp/upnp_common.h`.

* **Language / license.** C++, GPL‑2.0 (README: "Gerbera is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License version 2"; copyright line "2016‑2026 Gerbera Contributors").
* **Architecture.** Descended from MediaTomb 0.12.1‑GIT. CMake build, pugixml for XML, an embedded web UI (tree view of DB + filesystem), SQLite (or MySQL/MariaDB) storage, timed or inotify rescans, user‑defined "layouts" that reshape the library tree from extracted metadata, ffmpeg‑based transcoding with per‑profile configuration, scriptable transcoding, on‑the‑fly thumbnails via `libffmpegthumbnailer`, last.fm scrobbling, external‑URL items.
* **Protocol coverage — exact, from `src/upnp/upnp_common.h`:**
  * `UPNP_DESC_CDS_SERVICE_TYPE "urn:schemas-upnp-org:service:ContentDirectory:1"`, `UPNP_DESC_CDS_SERVICE_ID "urn:upnp-org:serviceId:ContentDirectory"`
  * `UPNP_DESC_CM_SERVICE_TYPE "urn:schemas-upnp-org:service:ConnectionManager:1"`, `UPNP_DESC_CM_SERVICE_ID "urn:upnp-org:serviceId:ConnectionManager"`
  * `urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1`
  * `dlna:X_DLNADOC` = `DMS-1.50`, device type `urn:schemas-upnp-org:device:MediaServer:1`
  * **Again no `ContentDirectory:2`.**
* **DLNA compliance is explicitly partial.** The Gentoo wiki (`https://wiki.gentoo.org/wiki/Gerbera`) states verbatim: *"Gerbera is not a DLNA media server. Gerbera does have partial DLNA support for some DLNA devices e.g. PlayStation 3."* and prescribes `<protocolInfo extend="yes">` plus, for DLNA‑enabled TVs such as Samsung, a `custom-http-headers` block:
  ```xml
  <add header="transferMode.dlna.org: Streaming"/>
  <add header="contentFeatures.dlna.org: DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01500000000000000000000000000000"/>
  ```
* **The single most valuable Gerbera artefact for us: the quirk system.** `https://docs.gerbera.io/en/latest/supported-devices.html` documents automatic client detection from the request signature and a set of device flags. Verbatim flag list (each with its documented meaning):
  * `SAMSUNG` — Add "CaptionInfo.sec" to video header
  * `SAMSUNG_BOOKMARK_SEC` / `SAMSUNG_BOOKMARK_MSEC` — Restore last played position when the media is played again (only for Samsung) as second / as millisecond
  * `SAMSUNG_FEATURES` — Activate Samsung features protocol (support for additional request like `X_GetFeatureList`, `X_GetObjectIDfromIndex`, `X_GetIndexfromRID`)
  * `SAMSUNG_HIDE_DYNAMIC` — Some Samsung devices do not work with dynamic folders on top level, so this flag hides them for those devices
  * `NoXmlDeclaration` — Don't send `<?xml ...?>` declaration in response
  * `NO_SEC_NAMESPACE` / `NoSecNamespace` — Don't generate `sec:` attributes and namespace entry
  * `PV_SUBTITLES` — Add attributes `pv:subtitleFileType` and `pv:subtitleFileUri` to video files if subtitles exist
  * `STRICTXML` — Encode most of special chars also, like `'` as `&apos;`
  * `HIDE_THUMBNAIL_RESOURCE` / `HIDE_SUBTITLE_RESOURCE` / `HIDE_TRANSCODE_RESOURCE`
  * `SIMPLE_DATE` — "UPnP Client needs simple dates. Return `dc:date` without timezone as Zulu time."
  * `DCM10` — Add DCM10 as device capability
  * `ASCIIXML` — "UPnP client supports only ascii characters"
  * `NO_CONVERSION` — "UPnP client does not support conversion"
  * `SHOW_INTERNAL_SUBTITLES` — "UPnP client does not automatically read internal subtitles from stream"
  * `FORCE_SORT_CRITERIA_TITLE` — "Always sort by `dc:title` ascending when browsing or searching. This quirk might help if your client sends hardcoded SortCriteria, e.g. old Panasonic TV sets"
  * `CAPTION_PROTOCOL` — Add protocolInfo to `sec:CaptionInfoEx` for Samsung
  * Manual override: `<client userAgent="Microsoft-DLNA" flags="0"/>` / `<client ip="192.168.1.69" flags="SAMSUNG"/>`
* **Gerbera's documented client list** (same page) includes: `StandardUPnP` (`UPnP/1.0`), `BubbleUPnP`, `SamsungAllShare`, `SamsungSeriesQ/QN/CDE/A/B`, `SamsungBDP` (BD‑D5100), `SamsungBDJ5500`, `EC-IRadio` (Dual CR 510), `FSL/Frontier Silicon`, `PanasonicTV` ("all Panasonic TV"), `BoseSoundtouch` (`STRICTXML`), `Yamaha RX` (`ASCIIXML`), `Freebox Player` (`NO_CONVERSION`), `Universum DAB+ Internet Radio` (`NoSecNamespace`). Tested renderers include *LG "All TV based on WebOS 6"*, *Philips 37PFL9603D / SLA‑5500 / SLA‑5520*, *Microsoft Windows Explorer and Windows MediaPlayer 12*, *Panasonic*, *Sony*, *VLC*, *Roku*.
* **Known Gerbera documentation gaps / user‑visible weaknesses (OBSERVED):**
  * *"It does not seem to sue the UPnP Browse action anymore, but now uses the optional Search action which is not implemented in Gerbera."* — HP MediaSmart TV (supported‑devices page). → **Gerbera does not implement the ContentDirectory `Search` action.**
  * The same page notes D‑Link DSM‑320 only plays AVI if the mimetype is mapped to `video/avi`, and DSM renderers need `<manufacturerURL>redsonic.com</manufacturerURL>` + `<modelNumber>105</modelNumber>` to get SRT subtitles working.
  * The Gentoo wiki page documents the DLNA gap above.
* **Maintainership.** Very active. The docs site is versioned "3.2.2_git" / "3.3.0"; the issue tracker has a large, actively‑triaged backlog (e.g. `https://github.com/gerbera/gerbera/issues/4065` opened 2026‑11‑13).

---

### 1.4 Platinum UPnP SDK (Plutinosoft)

**OBSERVED** — Sources: `https://github.com/plutinosoft/Platinum`, `https://sourceforge.net/projects/platinum/`, `https://raw.githubusercontent.com/plutinosoft/Platinum/master/Source/Core/PltUPnP.cpp`, `https://github.com/xbmc/xbmc/blob/.../lib/libUPnP/Platinum/Source/Devices/MediaServer/PltFileMediaServer.cpp`.

* **Language / license.** C++ on the Neptune runtime. GPL‑2.0 **or** a paid commercial licence — a genuine dual licence ("OEMs … who do not wish to distribute the source code for the commercially licensed software under version 2 … must enter into a commercial license agreement with Plutinosoft, LLC"). GPL‑2.0 is the licence to copy if you are open source.
* **Architecture.** A *framework*, not a product: `Platinum` provides SSDP device/control‑point, HTTP server, SOAP action dispatch, GENA, DIDL generation (`PltDidl`), HTTP range handling, and a file‑based MediaServer delegate (`PltFileMediaServer`) plus a MediaRenderer and a Control Point sample.
* **Maintainership.** Repo created 2015‑04‑22, **last push 2023‑06‑15**, 525 stars / 146 forks / 21 open issues. The API is effectively frozen; the last meaningful release was 0.6.1 (2010).
* **Historical significance.** Platinum is the ancestor of a large part of the ecosystem and is still shipping inside commercial products:
  * **Kodi still ships Platinum** as its UPnP client: `xbmc/xbmc` contains `lib/libUPnP/Platinum/Source/Devices/MediaServer/PltFileMediaServer.cpp` and `xbmc/filesystem/UPnPDirectory.cpp` (`SPDX-License-Identifier: GPL-2.0-or-later`), and uses `PLT_MediaObjectListReference`, `BrowseSync`, `PLT_MediaBrowser`.
  * **Plex Media Server's DLNA server is built on Platinum**: Plex's hidden server settings expose `DlnaPlatinumLoggingLevel` ("Sets level for the separate, 'Neptune DLNA' logging") — `https://support.plex.tv/articles/201105343-advanced-hidden-server-settings/`. The Platinum + Neptune lineage is therefore *the* commercial DLNA implementation in the field.
* **Weaknesses.** Unmaintained; carries a known historic remote buffer overflow in `soap_action_name` parsing (CVE‑era advisory, `https://exploit-database.com/exploits/multiple/dos/15346.c`, fixed in Platinum 0.6.1). Dual licensing makes it unattractive for an AGPL project. Also a **known upstream defect to avoid inheriting**: the file media server delegate performs URL‑decoding in a way that the upstream code itself flags as needing double‑decode protection (comment in `PltFileMediaServer.cpp`: *"We're intentionally prepending a known urlencoded string to detect it when we receive the request urlencoded or already decoded to avoid double decoding"*).

---

### 1.5 uPnP Media Server (CyberGarage)

**OBSERVED** — Sources: `https://github.com/cybergarage/cybergarage-upnp`, `https://raw.githubusercontent.com/cybergarage/cybergarage-upnp/master/README.md`, and the repository tree (487 files) which contains `examples/media-server/src/main/resources/description/service/ContentDirectory1.xml` and `ConnectionManager1.xml`, `std/src/main/java/org/cybergarage/upnp/std/av/server/MediaServer.java`.

* **Language / license.** Java. Published to Maven Central as `org.cybergarage.upnp:core` and `org.cybergarage.upnp:std`.
* **Architecture.** A UPnP *development package*: `core` implements SSDP, GENA, SOAP and HTTP; `std` provides standard device implementations (`org.cybergarage.upnp.std.av.server.MediaServer`, `...av.controller.MediaController`, `...av.renderer.MediaRenderer`) plus a runnable `examples/media-server` with a Swing UI.
* **Protocol coverage.** `MediaServer:1` with **ContentDirectory:1** (the example ships `ContentDirectory1.xml`), plus ConnectionManager and GENA. It is a *reference implementation*, not a media product: no index, no transcoding, no subtitle handling, no multi‑root library management, no metadata extraction.
* **Maintainership.** The library is alive and modernised (Maven Central artefacts, `MediaBrowseSecurityTest` present in `std/src/test/...`), but the media server is an example. The original CyberGarage "uPnP Media Server" product / `cybergarage/upnp-media-server` GitHub path does not exist (404) — the surviving artefact is `cybergarage/cybergarage-upnp`.
* **Relevance to PhairDLNA.** Not a competitor; it is a **reference for protocol correctness**, including a `MediaBrowseSecurityTest` worth reading before we write our own Browse/Search parser hardening.

---

### 1.6 Asset UPnP (Illustrate)

**OBSERVED** — Source: `https://www.dbpoweramp.com/asset-upnp-dlna.htm`.

* **Licence.** **Proprietary, closed source.** A 30‑day fully‑functional trial plus a paid "Asset UPnP Premium"; installers are shipped per‑OS (Windows, QNAP, Synology DSM, macOS, x64 Linux, x64 ARM/Raspberry Pi). The GitHub repository `git-developer/asset-upnp` is only GPL‑3.0 *Docker packaging* of the binary, not the product.
* **Scope.** Deliberately **audio‑only** UPnP/DLNA server: internet radio, dynamic playlists, customisable browse tree with advanced search (album by release/style/composer/conductor), WAVE/LPCM/mp3 decoding for devices that cannot play the source format, DSD/DFF passthrough and DSD‑DoP, PC audio capture, ReplayGain with volume matching, "Asset Collections" (multiple instances with different libraries).
* **Strengths.** The vendor's own pitch is the interesting part: *"When comparing to other UPnP servers, it is surprising how many simple elements are wrong, such collating albums with the same name as one huge album, after all you could have 10 Greatest Hits albums."* → **OBSERVED** that album/folder identity handling is a real differentiator.
* **Weaknesses for our purposes.** Closed source (nothing to learn from, nothing to reuse), audio‑only (no video at all), and paid.
* **Why it still matters.** It is one of the very few DLNA servers that ships a **macOS** build — proof that macOS is a viable DLNA server target, and a direct competitor for the same "share my library to the TV" job on macOS.

---

### 1.7 Proprietary / commercial servers (published behaviour only)

#### Plex Media Server

**OBSERVED** — Sources: `https://support.plex.tv/articles/200350536-dlna/`, `https://support.plex.tv/articles/203948786-plex-home-security-changes/`, `https://support.plex.tv/articles/201105343-advanced-hidden-server-settings/`, `https://forums.plex.tv/t/dlna-server/738374`, `https://forums.plex.tv/t/bug-plex-dlna-limitation-applies-3840-1920-despite-upperbound-set-in-the-xml-profile/820104`.

* Plex Media Server ships a **DLNA server** alongside its native protocol, exposed at *Settings → Server → DLNA*. Documented advanced settings: *"DLNA client preferences — Client-specific configuration settings for the DLNA server"*, *"DLNA default protocol info — The protocol information string used in the `GetProtocolInfo` responses"*, *"DLNA media renderer discovery interval"*, *"DLNA server announcement lease time"* (default 1800 s), *"DLNA server description icons"*.
* The DLNA server is implemented on **Platinum + Neptune** (`DlnaPlatinumLoggingLevel`).
* **Product decision worth copying — and worth *not* copying.** Plex now *disables* the DLNA server by default when Plex Home is enabled: *"The DLNA server built in to Plex Media Server will be disabled completely. DLNA access does not use a specific user account and so it would have full access to all of your media."* This is a **privacy/security** lesson for PhairDLNA: DLNA has no per‑user authentication, so a DLNA server is an open door to the whole library. PhairDLNA is privacy‑first, so this deserves an explicit design decision (e.g. opt‑in, on‑LAN‑only, with a clear warning), not silence.
* **Known limitation.** Plex's per‑device DLNA profiles ("DLNA client preferences") are XML that identify the client by `User-Agent` / `FriendlyName` / `DeviceDescription` and define `TranscodeTargets`, `DirectPlayProfiles`, `CodecProfiles` with `UpperBound` limitations — and users report that the `UpperBound` values are ignored, e.g. *"[BUG] Plex DLNA 'limitation applies: 3840 > 1920' despite 'UpperBound' set in the XML profile"* (`forums.plex.tv/t/…/820104`), with a log line *"FILM BACKUP - 2160p - video.width limitation applies: 3840 > 1920"*. A Panasonic `VIERA` client was identified as *"Panasonic MIL DLNA CP UPnP/1.0 DLNADOC/1.50"*.
* Plex is **not** a DLNA control point: *"Plex, like Sonos, Roon, Bluesound and others, does not do DLNA/uPnP, as a matter of course."* (`forums.plex.tv/t/why-dlna-renderer-discovery-interval-but-no-dlna-renderer-streaming/612066`).

#### Emby Server

**OBSERVED** — Sources: `https://emby.media/community/topic/17224-dlna-profile-for-lg-webos-tv/`, `https://emby.media/community/topic/139600-how-to-customize-dlna-server/`, `https://emby.media/community/topic/129786-upnp-only-works-for-a-short-period-of-time/`, `https://emby.media/community/topic/121696-dlna-wont-work-on-synology-via-docker-or-natively/`, `https://github.com/MediaBrowser/Emby/issues/3042`.

* Emby exposes a **DLNA server** plus a separate *"Enable DLNA Play To"* push feature, per‑device XML "DLNA profiles" identified by `Identification` header/device‑description matching, with `TranscodeTargets`, `DirectPlayProfiles` and `CodecProfiles`.
* The **LG WebOS profile** thread is a rich source of concrete behaviour: *"the official release of Emby server has a bug for the external subtitles over DLNA"*; *"My guess is it's forcing transcoding because of the 'PGSSUB' subtitles"*; log line *"Debug - UpnpContentDirectory: Bitrate exceeds DirectPlay limit"*; and the decisive detail — *"the profile 'LG Smart TV' is not the one from this thread, this one is named 'LG Smart+ TV' … It is currently using the default one which has a limit of 10Mbps"* — i.e. **profile selection is order‑sensitive and the default profile silently wins**, causing forced transcoding.
* Emby's **DLNA server is off by default** in newer releases ("In newer server releases it is off by default now"), and users report it is *"like that checkbox for DLNA Server isn't doing anything"* on Synology — the Emby DLNA server was discovered only by BubbleUPnP while Plex, Jellyfin, Synology Media Server, BubbleUPNP and Subsonic were all visible to the same clients (`emby.media/community/topic/121696-…`).
* Emby also auto‑creates **UPnP port‑mapping rules on the user's router** at install time, which the project later had to add explicit startup‑wizard opt‑ins for (`https://github.com/MediaBrowser/Emby/issues/3042`). → **OBSERVED** that shipping a UPnP/DLNA stack implies IGD/NAT‑traversal side effects that a privacy‑first product must consciously avoid.
* Emby offers a dedicated **"Enable DLNA debug logging"** switch, and its DLNA logs go into the normal server log (`emby.media/community/topic/129786-…`).

#### Jellyfin DLNA plugin

**OBSERVED** — Sources: `https://github.com/jellyfin/jellyfin-plugin-dlna`, the repository tree, and the individual profile XML files.

* GPL‑3.0, C#, created 2023‑10‑31, **last push 2026‑11‑11**, 91 stars, 92 open issues. The most actively developed DLNA MediaServer implementation in the world today.
* It ships **32 device profiles** under `src/Jellyfin.Plugin.Dlna/Profiles/Xml/`: `Default`, `Denon AVR`, `DirecTV HD-DVR`, `Dish Hopper-Joey`, `LG Smart TV`, `Linksys DMA2100`, `Marantz`, `MediaMonkey`, `Panasonic Viera`, `Popcorn Hour`, `Samsung Smart TV`, `Sharp Smart TV`, `Sonos`, `Sony Blu-ray Player` (2013/2014/2015/2016/generic), `Sony Bravia (2010)…(2014)`, `Sony PlayStation 3`, `Sony PlayStation 3 (Movian)`, `Sony PlayStation 4`, `WDTV Live`, `WiiM`, `Windows Media Player`, `Xbox One`, `Yamaha MusicCast`, `foobar2000`.
* **This profile corpus is the single best public catalogue of client quirks.** Verbatim excerpts:
  * `Xbox One.xml` — identification by header `FriendlyName.DLNA.ORG` containing `XboxOne` **and** `User-Agent` containing `NSPlayer/12`; `TimelineOffsetSeconds` = 40.
  * `Windows Media Player.xml` — identification by `User-Agent` containing `Windows-Media-Player`; `Identification/ModelName` = "Windows Media Player".
  * `foobar2000.xml` — identification by `FriendlyName` containing `foobar` and `User-Agent` containing `foobar`; `SupportedMediaTypes` = **Audio only**.
  * `Samsung Smart TV.xml` — identification by `User-Agent` containing `SEC_`; `XmlRootAttributes` adds `xmlns:sec="http://www.sec.co.kr/"`; `AlbumArtPn` = `JPEG_SM`, album art capped at 480×480; `EnableSingleAlbumArtLimit` = true; external subtitles via `CaptionInfoEx`; `EnableMSMediaReceiverRegistrar` = false.
  * `LG Smart TV.xml` — identification by `User-Agent` containing `LG`; `EnableAlbumArtInDidl` = **false** (album art is *not* advertised in DIDL at all); `TimelineOffsetSeconds` = 10; external subtitles with the plain (non‑`CaptionInfoEx`) method.
  * `Panasonic Viera.xml` — identification by `User-Agent` containing `Panasonic MIL DLNA` and `FriendlyName` `VIERA`.
  * `Sony Bravia (2014).xml` — identification by `FriendlyName` **regex** on the model string and `X-AV-Client-Info` **regex** matching; `AlbumArtPn` = `JPEG_TN` (not `JPEG_SM`); `SonyAggregationFlags` = 10; the profile even identifies as *"Windows Media Player Sharing"* / Microsoft Corporation, because the Bravia advertises itself that way.
  * `Default.xml` — `EnableMSMediaReceiverRegistrar` = true.
* **Interpretation (OBSERVED):** every serious DLNA server ships a *device‑profile / quirk* subsystem keyed on `User-Agent`, `X-AV-Client-Info`, `FriendlyName` (SSDP header), `FriendlyName.DLNA.ORG` and `modelName`. PhairDLNA should treat this as a **first‑class, data‑driven subsystem**, not `if user_agent.contains("samsung")` scattered through the code.

---

## 2. What the established solutions DO WELL — the newcomer must copy this

1. **A data‑driven client‑quirk layer.** Gerbera's flag table (`supported-devices.html`), ReadyMedia's `clients.c`/`clients.h` table, Plex/Emby/Jellyfin's XML device profiles. Three independent implementations converged on the same shape: *detect client → apply named quirk set*. **Copy this first.**
2. **A generous, "one root, many media types" library model.** ReadyMedia supports N `media_dir=` entries with type prefixes and a `merge_media_dirs` switch; Gerbera supports multiple `<location>` elements and user‑defined layouts; Emby/Plex map libraries to DLNA devices. **Copy: multiple roots, with a *documented, configurable* merge/extra‑folder behaviour.**
3. **An incremental index, not a scan‑on‑start.** ReadyMedia: SQLite + inotify/kqueue + non‑destructive rescan + `DB_VERSION` migration. Gerbera: SQLite/MySQL + timed/inotify rescan. **Copy: an index that survives restarts and updates incrementally.**
4. **A real HTTP range/seek implementation.** ReadyMedia implements `Range`, `Content-Range`, `Accept-Ranges`, `416`, `TimeSeekRange.dlna.org`, `getAvailableSeekRange.dlna.org` and `X-AvailableSeekRange` (`upnphttp.c`). **Copy: byte ranges + the DLNA time‑seek headers.**
5. **Per‑container DLNA profile names (`DLNA.ORG_PN`).** ReadyMedia emits `MPEG_TS_SD_NA`, `MPEG_PS_NTSC`, `AVC_TS_HD_50_AC3`, `JPEG_TN` and friends, and renames them per client (e.g. it rewrites `AVC_TS_MP_SD_AC3` → `AVC_TS_HD_50_AC3` for Sony Bravia, `upnpsoap.c` `case ESonyBravia:`). **Copy: a container→`DLNA.ORG_PN` table, plus per‑client rewrites.**
6. **A broad codec/container matrix rather than a minimal one.** ReadyMedia covers ASF/WMV, MP4, MKV, AVI, FLV, MOV/MP4/TS, WebM/RM/RMVB (added 1.3.2), MP3/FLAC/OGG/WAV/M4A, JPEG/PNG/GIF, DSD. **Copy breadth.**
7. **Subtitles as a first‑class, multi‑dialect feature.** SRT/SMI/ASS/SSA external + embedded, exposed through `sec:CaptionInfoEx`, `pv:subtitleFileType`/`pv:subtitleFileUri`, `CaptionInfo.sec` HTTP header and a dedicated `text/srt` resource — because **different clients understand different dialects**. **Copy all three dialects.**
8. **A diagnostic/logging story.** ReadyMedia has per‑subsystem log levels (`general,artwork,database,inotify,scanner,metadata,http,ssdp,tivo`); Emby has "Enable DLNA debug logging"; Gerbera logs SOAP traffic and rendered DIDL. **Copy: per‑subsystem log levels, a DIDL dump, and a SOAP trace.**
9. **SSDP robustness.** ReadyMedia 1.3.1/1.3.3 hardened SSDP request parsing (a crash fix and a chunk‑length fix); 1.3.2 added DNS‑rebinding protection. **Copy: treat SSDP/HTTP input as hostile.**
10. **A `ContentDirectory` implementation that tolerates sloppy clients.** ReadyMedia's `parse_sort_criteria()` accepts `upnp:class`, `dc:title`, `dc:date`, `upnp:originalTrackNumber`, `upnp:album` and **errors out** for anything else when the client is DLNA‑flagged (`upnpsoap.c`: *"If it's a DLNA client, return an error for bad sort criteria"*). Gerbera instead logs *"Unknown sort key 'dc:title' in '+dc:title'"* and continues (see `https://github.com/gerbera/gerbera/issues/2215`, which shows Windows Media Player sending `+dc:title` and `+upnp:originalTrackNumber,+dc:title`). **Copy: never fail a Browse because of an unknown sort key — fall back to a safe order.**
11. **A `Search` action, not just `Browse`.** ReadyMedia implements `Search` ("Support DLNA/UPnP-AV searches issued by clients using the Grilo framework", 1.1.2; "Worked around bad SearchCriteria from the Control4 Android app", 1.3.0) and even exposes `GetSearchCapabilities`/`GetSortCriteria`. Gerbera does **not** (HP MediaSmart TV breaks). **Copy: implement `Search` too — it is cheap once `SearchCriteria` parsing exists.**
12. **The Microsoft `X_MS_MediaReceiverRegistrar` service**, present in ReadyMedia, Gerbera, rclone/DMS and Jellyfin's `Default` profile. **Copy: it unlocks Xbox "Play To".**
13. **Read‑only, filesystem‑driven operation with no cloud dependency.** Every open‑source server here is local‑only. **Copy: no telemetry, no account, no outbound calls — this is a real differentiator for a privacy‑first product.**

---

## 3. What they DO POORLY — the gaps a new implementation can beat

1. **Stale architecture and near‑abandoned maintenance.** ReadyMedia's last release is 2023‑06‑02 and it is C with a global‑mutable client table and a SQLite schema that forces a full rescan on version bump; Platinum's last push is 2023‑06‑15 and it carries a 2010 CVE; the original CyberGarage media server is gone.
2. **No modern native UX.** None of the open‑source servers offers a first‑class macOS app: no Finder integration, no Quick Look, no Control‑Centre toggle, no Spotlight, no Energy‑aware behaviour, no automatic permission flow. Asset UPnP and Serviio are Windows‑first or cross‑platform without a Mac app. → **PhairDLNA's biggest opportunity.**
3. **Poor diagnostics.** ReadyMedia logs to a file with coarse levels; there is no "why does device X fail?" view, no request/response inspector, no per‑client compatibility report. Plex ships `DlnaPlatinumLoggingLevel` as a *hidden* setting; Emby hides "Enable DLNA debug logging" behind a menu. → **Opportunity: a first-class "Client Compatibility" inspector showing exactly what each device requested and where the server diverged.**
4. **The quirk layer is a flat static table, not a data model.** ReadyMedia's `clients.c` is a C array with bit flags; adding a device means a recompile. Gerbera's is better (XML, per‑IP overrides) but still not a "device profile" concept with capability declaration. → **Opportunity: declarative device profiles, JSON/YAML, hot‑reloadable, shipped with the app.**
5. **ContentDirectory:2 is nobody's problem.** Both ReadyMedia and Gerbera ship **only `ContentDirectory:1`** (verified in `upnpdescgen.c` / `upnp_common.h`). The `MediaServer:2` device template (UPnP Forum, 2006‑05‑31) defines `ContentDirectory:2` and `ConnectionManager:2`, and explicitly says *"a MediaServer:2 implementation MUST respond to all SSDP queries that specify MediaServer:1 and MUST respond to all actions defined by the MediaServer:1"* — but no mainstream server implements :2. → **Opportunity: accept `:2` action requests, and consider advertising `:2` — see §5 for the SCID collision caveat.**
6. **No graceful degradation of the library tree.** Samsung's "dynamic folders on top level" problem, Panasonic's hardcoded `SortCriteria`, the "extra folder level if there are multiple media locations" hack — all servers bolt on one‑off hacks. → **Opportunity: a library model that is stable by construction.**
7. **The album/folder identity problem is unsolved and visible.** Asset UPnP's own marketing complains that other servers *"collating albums with the same name as one huge album"*. → **Opportunity: correct per‑album identity (disc‑aware) out of the box.**
8. **Transcoding is a first‑class citizen in the old servers and a liability in ours.** ReadyMedia shells out to ffmpeg with a "seek directory" hack; Gerbera's transcoding is profile‑driven but complex; Emby/Jellyfin bake transcoding into the device profile. For a privacy‑first, macOS‑first product, native‑toolchain transcoding (VideoToolbox/AudioToolbox) is both a differentiator and a way to avoid shipping ffmpeg.
9. **Security posture is an afterthought.** Emby auto‑creates router port mappings without consent; Plex's DLNA exposes the *entire* library with *no* authentication and has now disabled it by default for that reason; ReadyMedia needed CVE fixes for DNS rebinding, SSDP parsing, SUBSCRIBE callback validation and chunked‑length parsing. → **Opportunity (and obligation): LAN‑only bind, no IGD, no UPnP port mapping, strict callback validation, and an explicit "DLNA has no auth — here is what that means" warning.**
10. **No macOS‑native filesystem semantics.** macOS has resource forks, `.DS_Store`, APFS firmlinks, `NFD` vs `NFC` Unicode normalisation, and sandbox/Full‑Disk‑Access permission models. None of the existing servers address these; ReadyMedia even had to fix *"potential stack smash vulnerability in getsyshwaddr on macOS"* (1.3.0). → **Opportunity: get Unicode normalisation and macOS path semantics right from day one.**
11. **No privacy posture at all.** Plex/Emby/Jellyfin all tie the media server to an account/telemetry‑bearing product. → **Opportunity: a DLNA server that provably makes no outbound connections.**
12. **PlayTo / renderer‑push is half‑implemented everywhere.** Plex cannot push to DLNA renderers at all; Emby's "DLNA Play To" sends one track at a time and cannot be stopped mid‑queue (community thread `5770`); ReadyMedia has the `X_MS_MediaReceiverRegistrar` stub. → **Opportunity: implement `PlayTo` properly (playlist‑level control + stop semantics), or do not implement it and say so.**
13. **Device‑profile data is trapped in each project's format.** Emby's XML, Plex's XML, Jellyfin's XML, Gerbera's XML, UMS's `.conf` files — five incompatible dialects of the same knowledge. → **Opportunity: a community‑maintained, open, shared device‑profile schema.**

---
