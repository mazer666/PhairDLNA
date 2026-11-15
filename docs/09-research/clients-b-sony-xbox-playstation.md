# DLNA Client Family Research — Sony Bravia, Xbox One/Series X|S, PlayStation 4/5
Status: **Research input — evidence, not norm** · Owner role: Interoperability Specialist · Version/date: 1.0 / 2026-11-14
> Committed so that every citation in the feature specifications resolves. Where this document and a requirement disagree, the requirement wins.

Scope: real-world behaviour and quirks of DLNA **clients** (control points / renderers) in these three
families, as observed by real media servers, from primary sources (server source code, server
renderer-profile repositories, upstream issue trackers, vendor docs).

Label legend (every statement carries one):

- **STANDARD** — behaviour mandated by the UPnP AV / DLNA specs.
- **OBSERVED** — recorded in a source I actually fetched (server source, config, log excerpt, or issue text).
- **ASSUMPTION** — plausible but not verified against a fetched source (explicitly flagged).
- **OPEN QUESTION** — I could not find evidence either way.

### Method notes & fetch limitations (read this before trusting a line below)

- Everything tagged **OBSERVED** comes from a URL listed in `## Sources` that I fetched in this session
  (raw source files, renderer `.conf` files, device-profile XML, GitHub issue bodies/comments, or
  GitHub-repo trees). Where a statement quotes user wording it is taken verbatim from the fetched text.
- Three sites are **Cloudflare-blocked** to my fetcher (`sourceforge.net`, `universalmediaserver.com`,
  `sony.co.uk` / `sony-asia.com`, `support.microsoft.com` / `support.xbox.com` — the last renders
  client-side only). For content that exists only on those sites I marked the label
  **OBSERVED (search-index copy)** and explicitly said so, because I could not independently open the
  page. Treat those as "strong but second-hand".
- I did **not** test anything. No hardware was available. Every "client sends / chokes on" line is
  either a captured artefact (header, log line, DIDL fragment) shipped in a fetched file, or a
  third-party observation.

---

## Sony Bravia TVs (and Sony home theatre / BDP players)

### What the client sends

- **OBSERVED** — Bravia identifies itself with the vendor header `X-AV-Client-Info`, e.g.
  `av=5.0; cn="Sony Corporation"; mn="BRAVIA KDL-40EX503"; mv="1.7";`
  ReadyMedia matches on the literal string `BRAVIA` in that header (match type `EXAVClientInfo`):
  <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/clients.c>
- **OBSERVED** — newer Bravia (2021 "X-series") send a second vendor header plus a plain UPnP UA:
  `User-Agent: UPnP/1.0`, `X-AV-Physical-Unit-Info: pa="BRAVIA KD-50X80J";`,
  `X-AV-Client-Info: av=5.0; cn="Sony Corporation"; mn="BRAVIA KD-50X80J"; mv="3.0";`
  (captured verbatim in UMS's renderer profile for these TVs):
  <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-X-Series-TV-2019-and-later.conf>
- **OBSERVED** — Emby/Jellyfin detect Bravia by regex on that header and by friendly name:
  `HttpHeaderInfo name="X-AV-Client-Info" value=".*KDL-\d{2}[WR][5689]\d{2}A.*" match="Regex"` and
  `<FriendlyName>KDL-\d{2}[WR][5689]\d{2}A.*</FriendlyName>`:
  <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Sony%20Bravia%20%282013%29.xml>
- **OBSERVED** — Bravia's DLNA profile (from the same Emby profile): `SonyAggregationFlags = 10`
  (i.e. Sony-specific aggregation metadata is expected), and the server must add the Sony AV namespace
  to the DIDL root: `<XmlAttribute name="xmlns:av" value="urn:schemas-sony-com:av" />`.
- **OBSERVED** — Sony devices use the `av:mediaClass` element so that the TV can filter items by media
  type; ReadyMedia added support for it in 1.0.23 ("Add support for the `av:mediaClass` tag, so some
  Sony devices can filter items by media type"):
  <https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/> (changelog text as surfaced by
  search; direct page fetch blocked — **OBSERVED (search-index copy)**)
- **OBSERVED** — `TransferProtocolInfo` / `DLNA.ORG_PN` is the negotiation axis for Bravia: ReadyMedia
  has a dedicated `case ESonyBravia:` that *rewrites* the advertised profile for
  "BRAVIA KDL-##*X### series TVs" from `AVC_TS_MP_SD_AC3` / `AVC_TS_MP_HD_AC3` / `AVC_TS_HP_HD_AC3`
  to `AVC_TS_HD_50_AC3…`:
  <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnpsoap.c>
- **OPEN QUESTION** — I found **no** fetched capture of a Bravia `Browse` request showing its
  `SortCriteria`, `Filter`, `RequestedCount` or `StartingIndex`. (For contrast, a captured Samsung
  request in a ReadyMedia bug shows `Filter=*`, `StartingIndex=0`, `RequestedCount=200`,
  `SortCriteria=` empty, `SOAPACTION: "urn:schemas-upnp-org:service:ContentDirectory:1#Browse"` —
  <https://sourceforge.net/p/minidlna/bugs/179/> — but that is a Samsung TV, **not** Bravia.)
- **OPEN QUESTION** — no fetched evidence of Bravia probing `ContentDirectory:2`; every server I read
  implements only `ContentDirectory:1` (see "Server-side workarounds" for the exact macro), so a `:2`
  probe would simply be unanswered.
- **OPEN QUESTION** — no fetched evidence that Bravia sends a distinct `TransferProtocolInfo` list in
  `ConnectionManager:GetProtocolInfo`; the Bravia-specific handling is in the DIDL `res/@protocolInfo`
  string instead.

### What it chokes on

- **OBSERVED (search-index copy)** — *Wrong DLNA profile string ⇒ folders visible but empty.*
  A ReadyMedia forum thread reports, for a European Sony Bravia: "There's a small glitch for European
  MPEG2 TS streams with European Sony Bravias. For the MPEG_TS stream, most Bravias only support the
  type MPEG_TS_SD_EU_ISO, whereas minidlna is hard-coded (in metadata.c) to use type
  MPEG_TS_SD_NA_ISO." and "The symptoms are right, folders show up but no content. If you trace the
  communication between the server and the TV, you'll see that minidlna is advertising the video.
  However, as the DLNA profile advertised by minidlna is not compatible with the Bravia, the Bravia
  does not list the videos."
  <https://sourceforge.net/p/minidlna/discussion/879957/thread/b49d2aae/> (page is Cloudflare-blocked to
  my fetcher; text surfaced via the search index — **second-hand**).
- **OBSERVED** — ReadyMedia 1.3.3 now derives the region from the stream itself:
  `MPEG_TS_` + `HD_NA` / `SD_` + (`EU` when height is 576 or 288, else `NA`) + `_ISO` / `_T`:
  <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/metadata.c>
- **OBSERVED** — `DLNA.ORG_FLAGS` must not be sent to Bravia. UMS sets `SendDLNAOrgFlags = false`
  in its generic Bravia profile (and only this device family does so in the shipped renderer set):
  <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-Bravia.conf>
  UMS's own option docs state the general rule: "If this value is wrong, the renderer may refuse
  listing or playing this file." — <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/DefaultRenderer.conf>
- **OBSERVED** — Bravia needs *regional localisation* of the DLNA parameters: UMS option
  `DLNALocalizationRequired = true`, documented as "Serve different flavors of localization in the DLNA
  parameters (PAL/NTSC, NA/EU/JP) to allow every world wide renderer to see the files. **Important for
  Sony Bravia TVs.**" (same two UMS URLs as above).
- **OBSERVED** — Bravia wants an *accurate* DLNA.ORG_PN: UMS option `AccurateDLNAOrgPN` — "Spends a
  longer time analyzing the contents of files to determine the most accurate information to send
  renderers. Some renderers like Sony TVs need the extra accuracy." (DefaultRenderer.conf URL above).
- **OBSERVED (search-index copy)** — Bravia's DLNA requests carry both `DLNA.ORG_PN=` **and**
  `SONY.COM_PN=` variants; a ReadyMedia bug reporter sniffed from a KDL xxW5800:
  `video/mpeg:DLNA.ORG_PN=AVC_TS_HD_24_AC3_ISO;SONY.COM_PN=AVC_TS_HD_24_AC3_ISO` and
  `video/vnd.dlna.mpeg-tts:DLNA.ORG_PN=AVC_TS_HD_24_AC3;SONY.COM_PN=AVC_TS_HD_24_AC3`.
  <https://sourceforge.net/p/minidlna/bugs/101/> (**second-hand**, page blocked to my fetcher).
- **OPEN QUESTION** — no fetched source shows Bravia rejecting a missing `dc:date`, a missing
  `res/@duration`, a missing `sec:httpDLNA`, a 404 (instead of 206) on a range request, chunked
  transfer encoding, HTTP/1.0 responses, a missing `Content-Length`, a suffix range (`bytes=-N`), or
  non-ASCII filenames. These are plausible Bravia complaints in the wild but I could not source them.

### Documented failure symptoms

- **OBSERVED (search-index copy)** — "Directory is empty" / "folders show up but no content" when the
  advertised DLNA profile is not in the TV's certificate (ReadyMedia thread, URL above).
- **OBSERVED (search-index copy)** — "Changing the two instances of _NA to _EU in metadata.c fixed the
  problem, otherwise my MPEG TS streams would not be recognized (only the MPEG PS ones)" — European
  BRAVIA KDL-40W5500 user (same thread).
- **OBSERVED (search-index copy)** — Sony's own support text: "My home network certified server device
  is listed on my TV but the files are inaccessible" and the remedy is a manual "Update list" under
  *Settings > Network Settings > Server Display Settings*, plus a "Server Diagnostics" run:
  <https://www.sony.co.uk/electronics/support/televisions-projectors-lcd-tvs/klv-s40a10e/articles/00126002>
  (**second-hand** — the page is bot-blocked to my fetcher).
- **OBSERVED** — a Gerbera user with a "Sony Bravia client" reported that MPEG-II (`.mpg`) files were
  not recognised/scanned at all (server-side scan gap surfaced *because* of a Bravia client):
  <https://github.com/gerbera/gerbera/issues/516>
- **OPEN QUESTION** — "requires re-index" and "only first page" as *Bravia-specific* symptoms: the
  re-index advice I found (`minidlna -R`) is generic server-side, not Bravia-specific.

### Server-side workarounds observed in existing implementations

- **ReadyMedia / minidlna** — client detection entry:
  `{ ESonyBravia, FLAG_DLNA, "Sony Bravia", "BRAVIA", EXAVClientInfo }` in
  `clients.c`, plus the profile-rewrite block `case ESonyBravia:` in `upnpsoap.c`
  (<https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/clients.c>,
  <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnpsoap.c>).
- **ReadyMedia / minidlna** — `av:mediaClass` support for Sony filtering (changelog 1.0.23) and
  region-aware MPEG_TS profile naming in `metadata.c`.
- **ReadyMedia / minidlna** — Sony **BDP** players get a MIME lie: "Sony BDP-S370 won't play MKV unless
  we pretend it's a DiVX file" — `upnphttp.c` rewrites `x-matroska`/`mpeg` → `divx` for `ESonyBDP`
  (<https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnphttp.c>). Not a Bravia-TV quirk,
  but the same vendor's player family.
- **UMS** — a full per-device renderer profile:
  `UserAgentAdditionalHeader = X-AV-Client-Info`, `UserAgentAdditionalHeaderSearch = BRAVIA`,
  `UpnpDetailsSearch = BRAVIA`, `SeekByTime = true`, `DLNALocalizationRequired = true`,
  `TranscodeVideo = MPEGTS-H264-AC3`, `TranscodeAudio = WAV`, `KeepAspectRatioTranscoding = true`,
  `H264LevelLimit = 5.1`, `ForceJPGThumbnails = true`, `ThumbnailAsResource = true`,
  `CreateDLNATreeFaster = true`, `SendDLNAOrgFlags = false`
  (<https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-Bravia.conf>).
- **UMS** — `AccurateDLNAOrgPN = true` for "Some renderers like Sony TVs" (DefaultRenderer.conf).
- **Emby / Jellyfin** — Bravia is served with a **masqueraded identity**: the profile declares
  `<Manufacturer>Microsoft Corporation</Manufacturer>`,
  `<ModelName>Windows Media Player Sharing</ModelName>`, `<ModelNumber>3.0</ModelNumber>` while
  matching on Sony's header — i.e. the server deliberately pretends to be a Microsoft DMS to the TV.
  It also sets `SonyAggregationFlags = 10`, `AlbumArtPn = JPEG_TN`, `EnableSingleAlbumArtLimit = true`,
  and per-codec `ResponseProfile`s such as
  `orgPn="AVC_TS_HD_24_AC3_T,AVC_TS_HD_50_AC3_T,AVC_TS_HD_60_AC3_T,AVC_TS_HD_EU_T"` with
  `mimeType="video/vnd.dlna.mpeg-tts"` and a `PacketLength == 192` / `192`-vs-`188` condition.
  (<https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Sony%20Bravia%20%282013%29.xml>)
- **Gerbera** — has **no** Sony/Bravia-specific client profile or quirk: a full grep of the current
  master tree finds only doc mentions (`doc/supported-devices.rst` lists PS3/PS4) and no
  `BRAVIA` / `X-AV-Client-Info` handling in code
  (<https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst>).

---

## Microsoft Xbox One / Xbox Series X|S

### What the client sends

- **OBSERVED** — Xbox One's DLNA user-agent family, captured verbatim in UMS's renderer profile:
  ```
  User-Agent: NSPlayer/12.00.9600.16411 WMFSDK/12.00.9600.16411
  User-Agent: NSPlayer/12.00.11636.0000 WMFSDK/12.00.11636.0000
  FriendlyName: Xbox-SystemOS
  ModelName: Xbox One
  FriendlyName.DLNA.ORG: XboxOne
  User-Agent: Microsoft-Windows/6.2 UPnP/1.0 Microsoft-DLNA DLNADOC/1.50
  ```
  with the detection rule `UserAgentSearch = XboxOne|Xbox-SystemOS|NSPlayer/12`
  (<https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Microsoft-XboxOne.conf>).
- **OBSERVED** — Emby/Jellyfin identify the Xbox One by the SSDP header `FriendlyName.DLNA.ORG: XboxOne`
  (substring match) **or** by `User-Agent` containing `NSPlayer/12`:
  <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Xbox%20One.xml>
- **OBSERVED** — Xbox 360 (previous generation, still shipped as a DLNA client) sends
  `User-Agent: Xbox/2.0.17150.0 UPnP/1.0 Xbox/2.0.17150.0`, `friendlyName=Xbox 360`,
  `modelName=Xbox 360`; UMS detects with `UserAgentSearch = XBOX|XENON`:
  <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Microsoft-Xbox360.conf>
- **OBSERVED** — the Xbox One announces itself with a **new UUID on every boot**, so naive servers
  accumulate one device entry per restart: "Every time my xbox one is restarted emby detects it as a
  new DLNA device and adds it to the device list… after a while i have a list of 30 xbox ones in my
  device list that i have to manually remove one by one"; the Emby fix was to drop Xbox One from the
  device list because "It is creating a new UUID on every startup which means we can't unique identify it."
  <https://github.com/MediaBrowser/Emby/issues/2769>
- **OBSERVED** — the Xbox One needs **SSDP alive NOTIFY traffic** to show the server at all: "What I've
  read you will have to enable the 'Blast Alive Messages' feature in the server in order for XB1 to
  detect it." and "It doesn't show up for me immediately like all my PC's and Sonos devices do but it
  does show up within 30 seconds I'm assuming this is the 'blast' doing its thing."
  <https://emby.media/community/topic/10359-xbox-one-dlna-media-player-preview/>
- **OBSERVED** — Emby does **not** use the Microsoft MediaReceiverRegistrar for Xbox One
  (`EnableMSMediaReceiverRegistrar = false`) and applies `TimelineOffsetSeconds = 40`
  (Xbox One profile XML, URL above). Gerbera keeps an MRR implementation but comments it as
  "These functions were only implemented to enable Xbox360 support."
  (<https://github.com/gerbera/gerbera/blob/master/src/upnp/mr_reg_service.h>).
- **OBSERVED** — the Xbox One's advertised `ProtocolInfo` set as understood by Emby (the server-side
  answer to the client's `GetProtocolInfo` negotiation):
  `http-get:*:video/mpeg:*, http-get:*:video/mp4:*, http-get:*:video/vnd.dlna.mpeg-tts:*,
  http-get:*:video/avi:*, http-get:*:video/x-matroska:*, http-get:*:video/x-ms-wmv:*,
  http-get:*:video/wtv:*, http-get:*:audio/mpeg:*, http-get:*:audio/mp3:*, http-get:*:audio/mp4:*,
  http-get:*:audio/x-ms-wma*, http-get:*:audio/wav:*, http-get:*:audio/L16:*,
  http-get:*:image/jpeg:*, http-get:*:image/png:*, http-get:*:image/gif:*, http-get:*:image/tiff:*`
  (Xbox One profile XML, URL above).
- **OPEN QUESTION** — I found **no** fetched capture of an Xbox One/Series `Browse` action showing
  `SortCriteria`, `Filter`, `RequestedCount` or `StartingIndex`. Specifically, the widely repeated claim
  that "Xbox wants dc:date" / "Xbox uses a different SortCriteria" is **not** confirmed by anything I
  fetched. The closest structural hints are server-side: Emby's Xbox profile sets `TimelineOffsetSeconds = 40`
  and disables album art in DIDL, and UMS sets `force_sort_criteria`-adjacent behaviour only for
  Panasonic/Roku (not Xbox).
- **OPEN QUESTION** — no fetched evidence of `ContentDirectory:2` probing by Xbox. (Emby, Gerbera and
  ReadyMedia all implement `ContentDirectory:1` only; Emby's own handler comments that
  "Upnp inspector sends 0 as requestedCount when it wants everything", i.e. `RequestedCount = 0` means
  "give me everything" — a server must not treat 0 as "give me nothing":
  <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/ContentDirectory/ControlHandler.cs>)

### What it chokes on

- **OBSERVED** — **album art inside DIDL breaks it**: Emby's Xbox One profile sets
  `EnableAlbumArtInDidl = false` and `EnableSingleAlbumArtLimit = false`
  (Xbox One profile XML) — a server-side suppression hack, not a documented Microsoft statement.
- **OBSERVED** — **MKV seek/FF-RW is broken**: "My issue now is that i cannot fast forward or rewind
  MKV's. Other users are able to using different DLNA servers, but doesnt work with MB3."
  <https://emby.media/community/topic/10359-xbox-one-dlna-media-player-preview/>
- **OBSERVED** — **the wrong item plays**: "instead of playing the episode I selected (episode 5), it
  played episode 1. This happened no matter which episode I selected to play."
  (same Emby thread).
- **OBSERVED** — **playback stalls in short bursts**: "most were pausing every 13 or so seconds and if I
  selected them directly, vs selecting the first movie and then going 'next' to them, they would fail at
  a buffer spot and go back to the first movie." (same Emby thread).
- **OBSERVED** — **only a subset of the library is visible**: "I can see my MB server within the media
  player app, but it only shows two folders: TV and Movies. I have a few others such as cartoons,
  anime, etc. What would prevent the other folders from showing up?" (same Emby thread).
- **OBSERVED (symptom)** — users across the web report the classic "server found, folders empty":
  "the Xbox One is picking up that there is a server however it's saying all the folders are empty"
  (<https://www.reddit.com/r/xbox/comments/3b28pv/issues_with_dlna_server_nas/>); "the xbox one sees the
  servers and folders but says they are empty" (Plex/XBMC/TVersity/TWonky/Windows Media Player tested)
  (<https://gamerz-forum.com/threads/xbox-one-dlna-media-server.350376/>); and "the Movies & TV apps on my
  Xbox One consoles are showing empty directories when I navigate By Folder. I've found I can usually
  navigate another way, such as By Date or By Title, and the contents are listed and quite playable."
  (<https://www.reddit.com/r/xboxone/comments/a9vl6g/tech_movies_tv_shows_empty_directories_on_my_nas/>).
  **ASSUMPTION** — that the cause is the Xbox's own folder-vs-sort navigation rather than the server's
  DIDL; nothing I fetched proves a server-side cause.
- **OBSERVED** — **Xbox Series X|S**: a UWP DLNA player reports the platform does not work at all:
  "DLNA/UPNP doesn't seem to work on Xbox… OS: Xbox Series X. Latest version… TV app from MS works fine
  on same platform." (<https://github.com/huynhsontung/Screenbox/issues/277>)
- **ASSUMPTION (second-hand)** — Microsoft states "The XBOX Media Player app is no longer supported as
  of June 2026" (<https://support.xbox.com/en-US/help/hardware-network/display-sound/media-player-faq>,
  mirrored at <https://support.microsoft.com/en-us/topic/stream-music-and-video-content-to-your-xbox-console-using-playto-687e95c3-2c82-85ea-84d3-7b4330efa262>).
  Both pages render client-side only (no server HTML), so I could not read them directly — treat the
  deprecation date as unverified. If true, the Xbox One/Series *stock* DLNA client is being retired.
- **OPEN QUESTION** — no fetched evidence that Xbox rejects missing `dc:date`, missing `res/@duration`,
  a missing `sec:httpDLNA`, chunked transfer encoding, HTTP/1.0 responses, a missing `Content-Length`,
  suffix ranges (`bytes=-N`), or non-ASCII filenames. I also found no evidence of a hard "exact
  protocolInfo string" match requirement.

### Documented failure symptoms

- **OBSERVED** — "shows empty folders / all folders are empty" (Reddit r/xbox, Reddit r/xboxone,
  gamerz-forum — URLs above).
- **OBSERVED** — "shows files but no playback" / "cannot fast forward or rewind MKV's" (Emby forum).
- **OBSERVED** — "stops after N seconds": "pausing every 13 or so seconds" (Emby forum).
- **OBSERVED** — "plays the wrong item": episode 1 instead of the selected episode (Emby forum).
- **OBSERVED** — "requires re-index"-like behaviour: the console has to be **fully rebooted** for the
  server to reappear: "A couple times I've also had to fully shutdown my console and do a full reboot to
  get the MediaBrowser DLNA server to show up… Occasionally I have had to fully reboot the XBOX ONE to
  get the media player preview to work correctly" (Emby forum).
- **OBSERVED** — server-side device-list pollution from a new UUID on every boot (Emby #2769).
- **OBSERVED** — Xbox 360 as a DLNA-only playback device (music while gaming):
  <https://github.com/jellyfin/jellyfin-plugin-dlna/issues/260>

### Server-side workarounds observed in existing implementations

- **ReadyMedia / minidlna** — a hard identity hack for Xbox 360, in `upnphttp.c`:
  ```c
  /* If it's a Xbox360, we might need a special friendly_name to be recognized */
  if( h->req_client && h->req_client->type->type == EXbox ) {
      ... strcpy(modelnumber, "1"); ... snprintf(friendly_name+i, ..., ": 1"); sendXMLdesc(h, genRootDesc); ...
  ```
  i.e. the root description is re-sent with model number `1` and a `": 1"` suffix appended to the
  friendly name, because otherwise the Xbox ignores the server. This matches ReadyMedia 1.0.25's
  changelog: "Forcibly tweak the model number for Xbox360 clients, or they might ignore us."
  (<https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnphttp.c> and
  <https://sourceforge.net/projects/minidlna/files/minidlna/1.1.2/> — **second-hand** for the changelog).
- **ReadyMedia / minidlna** — client table entry (Xbox 360 only; there is **no** Xbox One entry in
  ReadyMedia, so Xbox One falls through to the generic `Generic DLNA 1.5` / `Generic UPnP 1.0` buckets):
  `{ EXbox, FLAG_MIME_AVI_AVI | FLAG_MS_PFS, "Xbox 360", "Xbox/", EUserAgent }` in `clients.c`.
- **UMS** — `Microsoft-XboxOne.conf`: `SeekByTime = exclusive`, `TranscodeVideo = MPEGTS-H264-AC3`,
  `TranscodeAudioTo441kHz = true`, `HalveBitrate = true`, `LoadingPriority = 1`, plus a tight
  `Supported =` format table (e.g. `f:mkv v:h264|mpeg1|mpeg2 a:aac-lc|ac3|mp3|mpa m:video/x-matroska`).
  `Microsoft-Xbox360.conf` additionally uses `TranscodeAudio = MP3`, `DefaultVBVBufSize = true`,
  `TranscodedVideoAudioSampleRate = 44100`, `CreateDLNATreeFaster = true`.
- **Emby / Jellyfin** — a dedicated `Xbox One` device profile: `EnableAlbumArtInDidl=false`,
  `EnableSingleAlbumArtLimit=false`, `EnableSingleSubtitleLimit=false`, `EnableMSMediaReceiverRegistrar=false`,
  `TimelineOffsetSeconds=40`, `MaxStreamingBitrate=140000000`, `AlbumArtPn=JPEG_SM`,
  `MaxAlbumArtWidth/Height=480`, `RequiresPlainVideoItems=false`, `RequiresPlainFolders=false`,
  plus `DirectPlayProfile`/`TranscodingProfile`/`CodecProfile` tables that cap the Xbox to 1920×1080,
  30 fps, H.264 level ≤ 4.1, ≤ 6 audio channels.
- **Emby** — server-side fix for the UUID churn: stop listing Xbox One as a trackable device
  (<https://github.com/MediaBrowser/Emby/issues/2769>).
- **Emby** — "enable the 'Blast Alive Messages' feature in the server in order for XB1 to detect it"
  (<https://emby.media/community/topic/10359-xbox-one-dlna-media-player-preview/>) — i.e. keep SSDP alive
  NOTIFY running continuously.
- **Gerbera** — keeps a MediaReceiverRegistrar "only … to enable Xbox360 support"
  (<https://github.com/gerbera/gerbera/blob/master/src/upnp/mr_reg_service.h>); **no** Xbox One
  profile/quirk exists in the current tree.
- **dlna-here** (modern reference server) records the negative result explicitly: "Xbox audio, video,
  and image validation confirms that the tested workflows **do not require a DLNA DMS marker or
  Microsoft registrar service**. Time-based seeking is not a fit unless it can be implemented correctly
  without content probing or a media index." (<https://raw.githubusercontent.com/pseudosavant/dlna-here/main/ROADMAP.md>)

---

## PlayStation 4 / PlayStation 5

> **Headline finding (contradicts the common assumption):** the **PS4 *is* a DLNA client / renderer**
> (it has a "Media Player" app that browses and plays DLNA MediaServers), while the **PS5 is not a
> DLNA renderer at all** and has no DLNA media-player app.

### What the client sends

- **OBSERVED** — PS4 user agents, captured in UMS's renderer profile:
  ```
  User-Agent: PS4Application libhttp/1.000 (PS4) libhttp/2.51 (PlayStation 4)
  User-Agent: libhttp/2.51 (PlayStation 4)
  ```
  matched by `UserAgentSearch = PlayStation 4`
  (<https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-PlayStation4.conf>).
- **OBSERVED** — a *live* PS4 HTTP request captured in a UMS issue log:
  ```
  GET /get/51/XXXXXXXXX.720p.WEBRip.X264-XXXXX.mkv_transcoded_to.ts HTTP/1.1
  Host: 192.168.1.101:5002
  User-Agent: MediaPlayer/4.00 libhttp/6.00 (PlayStation 4)
  Connection: Keep-Alive
  ```
  (<https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1593>)
- **OBSERVED** — Emby/Jellyfin match the PS4 by `User-Agent` containing `PLAYSTATION 4` **or**
  `X-AV-Client-Info` containing `PLAYSTATION 4`:
  <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Sony%20PlayStation%204.xml>
- **OBSERVED** — PS4 capability set as encoded by servers: `TranscodeVideo = MPEGTS-H264-AC3`,
  `SeekByTime = exclusive`, `TranscodedVideoFileSize = -1`, `H264LevelLimit = 4.2`,
  `HalveBitrate = true`, `RemoveTagsFromSRTSubtitles = false`, and a strict `Supported =` table
  (e.g. `f:mp4 v:h264 a:aac-lc|ac3 w:3840 h:2160 m:video/mp4`; `f:mkv v:h264 a:aac-lc|ac3|mp3`).
  UMS PS4 profile URL as above.
- **OBSERVED** — PS3 (older sibling, still a DLNA client) sends `User-Agent: PLAYSTATION 3` and
  `X-AV-Client-Info: av=5.0; cn="Sony Computer Entertainment Inc."; mn="PLAYSTATION 3"; mv="1.0";`
  and is matched by `UserAgentSearch = PLAYSTATION 3`; its PS3-specific hacks include
  `MimeTypesChanges = video/avi=video/x-divx`, `WrapEncodedAudioIntoPCM = true`,
  `TranscodeFastStart = true`, `SeekByTime = exclusive`:
  <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-PlayStation3.conf>
  (ReadyMedia matches the same device via `X-AV-Client-Info` `PLAYSTATION 3` / UA `PLAYSTATION`:
  <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/clients.c>)
- **OBSERVED** — Emby's PS4 profile also sets `SonyAggregationFlags = 10`,
  `EnableAlbumArtInDidl = false`, `EnableSingleAlbumArtLimit = true`,
  `EnableMSMediaReceiverRegistrar = false`, `TimelineOffsetSeconds = 0`, `AlbumArtPn = JPEG_TN`.
  Also note the PS4 `ResponseProfile` for AVI: `orgPn="AVI" mimeType="video/divx"` — i.e. the PS4 wants
  AVI advertised as `video/divx`.
- **OPEN QUESTION** — no fetched capture of a PS4/PS5 `Browse` action's `SortCriteria`, `Filter`,
  `RequestedCount` or `StartingIndex`; no evidence of `ContentDirectory:2` probing.
- **OPEN QUESTION** — no fetched evidence of PS4-specific `dc:date` / `res/@duration` /
  `sec:httpDLNA` requirements.

### What it chokes on

- **OBSERVED** — **the PS4 hides items it cannot play natively**, so anything that would need
  transcode/remux simply does not appear in the browser: "Playstation 4 Media Player over DLNA can't see
  any files which have unsupported video or audio codec. The only media that shows up are natively
  supported formats. This way transcoding over DLNA is impossible to select."
  (<https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36>)
- **OBSERVED** — same issue, user wording: "my PS4 can play some things from Jellyfin, but others (looks
  like definitely all H265 and avi) don't even show up in their directories"
  (<https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36>).
- **OBSERVED** — **"Cannot play the media"** is the canonical PS4 failure: "When I have transcoding
  disabled, some files that are incompatible still show up, but upon selection the ps4 immediately aborts
  with Cannot play the media."
  (<https://github.com/gerbera/gerbera/issues/1043>)
- **OBSERVED** — **the PS4 gives the server a short start-up window**: "I found that if ffmpeg could
  finish before the transcoding kill timer killed it (like 15 or 20 seconds), then the video could start
  playing, otherwise it would just say 'Cannot play the media'." and "the PS4 client would then freeze
  (like, crash) several seconds into the playback, always at the same point."
  (<https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36>)
- **OBSERVED** — **the container/profile string matters more than the codec**: after switching the
  transcode target to mkv/mp4 with h264+aac, "this change allowed the files to show up in the list at
  all. So it does seem like the ts container, or maybe ts with the particular codecs used, is a problem
  for PS4." (<https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36>)
- **OBSERVED** — **MIME / `DLNA.ORG_PN` correctness decides whether audio shows up**: a `.ogg` file was
  "still marked as video files for PS4"; the reporter concluded "this is due to this problem that ogg
  files does not show up in PlayStation-4 media player's play list", and the UMS maintainer replied that
  the `DLNA.ORG_PN`/MIME logic "is probably the primary problem" and "Some renderers will accept invalid
  values but reject missing values and vice versa… Some renderers will only accept valid values, which
  makes transcoding effectively impossible".
  (<https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1430>)
- **OBSERVED** — **`upnp:class` must be right**: the DIDL fragment in the same issue shows
  `<upnp:class>object.item.audioItem.musicTrack</upnp:class>` for an audio-only item, and the
  maintainer adds "I don't remember/know how the PS4 deals with this, but knowing Sony I guess it's
  pretty picky about it." (same URL).
- **OBSERVED** — **SSDP alive expiry kills PS4 playback mid-stream**: "If you experience disconnection
  problems from your device, e.g. Playstation 4, when streaming videos after about 5 minutes, you can try
  changing the alive value to 86400 (which is 24 hours)."
  (<https://github.com/gerbera/gerbera/blob/master/doc/config-server.rst>)
- **OBSERVED** — Gerbera's device list says the same in prose: "Playstation 4 — MediaPlayer seems to be
  flawed, so SSDP advertisements stop playback of videos. Set alive value in config.xml to e.g. 86400."
  and, for PS3, "Firmware 1.80 introduces UPnP/DLNA support."
  (<https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst>)
- **OBSERVED (PS3)** — **seek-by-time must be exclusive**: UMS documents "some (e.g. the PS3 [1] and
  Panasonic TVs) behave unreliably if both are permitted when transcoding files… Seeking via 'Go To'
  with the PS3 (firmware 4.31) and MEncoder works with 'exclusive', but hangs or crashes with 'true'."
  (<https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/DefaultRenderer.conf>)
- **OPEN QUESTION** — no fetched source shows PS4 rejecting a missing `dc:date`, chunked transfer
  encoding, HTTP/1.0 responses, a missing `Content-Length`, suffix ranges (`bytes=-N`), 404-instead-of-206
  on range requests, or unicode filenames.

### Documented failure symptoms

- **OBSERVED** — "shows files but no playback": "ps4 immediately aborts with Cannot play the media"
  (<https://github.com/gerbera/gerbera/issues/1043>).
- **OBSERVED** — "missing items": "only H264-AC3 etc... files appear which the PS4 natively supports"
  and "definitely all H265 and avi … don't even show up in their directories"
  (<https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36>).
- **OBSERVED** — "stops after N seconds": PS4-side mid-stream disconnect attributed to SSDP alive
  expiry (Gerbera doc, ~5 minutes, URL above); separately "the PS4 client would then freeze (like, crash)
  several seconds into the playback, always at the same point" (plugin-dlna #36).
- **OBSERVED** — transcoding is "aborted and restarted multiple times" from the server's point of view
  (Gerbera #1043 log excerpt).
- **OBSERVED** — "ogg files does not show up in PlayStation-4 media player's play list" (UMS #1430).
- **OBSERVED** — general severity assessment from a user: "I get the impression the PS4 media client is
  very particular and fragile" (plugin-dlna #36).

### PS5 specifically

- **OBSERVED** — **the PS5 is not a DLNA renderer**: "Stream to the PS5 Pro over DLNA — It is **not** a
  DLNA renderer. Its SSDP replies on port 40001 belong to Remote Play / Second Screen." and "Look for
  the 'Media Player' app in PS Store — It does not exist for PS5 — that was a PS4 app. Media Gallery
  plays from USB only."
  (<https://raw.githubusercontent.com/Octanium91/plex-ps5-local-playback/main/README.md>)
- **OBSERVED (corroborating absence)** — UMS ships renderer profiles for `Sony-PlayStation3.conf`,
  `Sony-PlayStation4.conf`, `Sony-PlayStationVita.conf` — and **none** for PS5
  (repo tree listing: <https://api.github.com/repos/UniversalMediaServer/UniversalMediaServer/git/trees/main?recursive=1>).
  Likewise Emby/Jellyfin ship `Sony PlayStation 3.xml` and `Sony PlayStation 4.xml` and no PS5 profile
  (<https://github.com/MediaBrowser/Emby/tree/master/Emby.Dlna/Profiles/Xml>).
  **ASSUMPTION** — the natural inference "the PS5 cannot be a DLNA renderer" follows from those two
  absences plus the README statement, but no server maintainer states it in so many words.
- **OBSERVED (second-hand)** — the PS5 *can* be used as a DLNA **client** only through third-party
  software: the UMS forum thread "Question about PS5" reports "the PS5 does not have a media player
  like the PS4. At most there is the gallery app that can play music and some videos from the USB… As
  far as I can tell there is no way of playing over the network", and users add that the Rad Player /
  LittlStar VR app ($ Premium) provides an "UPnP Media Renderer" on PS4/PS5.
  <https://www.universalmediaserver.com/forum/viewtopic.php?t=15656> (**second-hand**, forum is
  Cloudflare-blocked to my fetcher; the text came from the search index).
- **OBSERVED** — jailbroken PS5 homebrew explicitly ships a DLNA renderer ("PS Play 2.1 — All-in-one
  media hub for jailbroken PlayStation 5 — DLNA receiver, DLNA/UPnP browser… DLNA Hub — UPnP
  MediaRenderer:1 cast target"), which is itself evidence that the stock PS5 has none:
  <https://github.com/mounirhero/ps-play> (**second-hand**, README surfaced via search index).
- **OPEN QUESTION** — whether the PS5 (or "Xbox Series X") is a usable DLNA *client* today. dlna-here's
  roadmap lists "Xbox Series X" among the clients it intends to exercise for compatibility, while its
  validated matrix only lists "Xbox Media Player" — i.e. the PS5 is not in its matrix at all
  (<https://raw.githubusercontent.com/pseudosavant/dlna-here/main/ROADMAP.md>,
  <https://raw.githubusercontent.com/pseudosavant/dlna-here/main/COMPATIBILITY.md>).

### Server-side workarounds observed in existing implementations

- **UMS** — `Sony-PlayStation4.conf` (URL above): `SeekByTime = exclusive`,
  `TranscodeVideo = MPEGTS-H264-AC3`, `TranscodedVideoFileSize = -1` (transcoded size is unknown, so
  don't advertise one), `H264LevelLimit = 4.2`, `HalveBitrate = true`,
  `RemoveTagsFromSRTSubtitles = false`, `SupportedExternalSubtitlesFormats = SUBRIP`.
- **UMS** — `Sony-PlayStation3.conf`: `MimeTypesChanges = video/avi=video/x-divx` (tell the PS3 that AVI
  is DivX), `WrapEncodedAudioIntoPCM = true`, `WrapDTSIntoPCM = true`, `TranscodeFastStart = true`,
  `SeekByTime = exclusive`, `CreateDLNATreeFaster = true`.
- **UMS** — `DefaultRenderer.conf` documents the two PS-relevant semantics used above:
  `SeekByTime = exclusive` (seek-by-time vs seek-by-byte exclusivity) and
  `AccurateDLNAOrgPN` ("Some renderers like Sony TVs need the extra accuracy").
- **Emby / Jellyfin** — `Sony PlayStation 4.xml` device profile: `TranscodingProfile … transcodeSeekInfo="Bytes"`
  for audio but `"Auto"` for video, `ResponseProfile container="avi" orgPn="AVI" mimeType="video/divx"`,
  `EnableAlbumArtInDidl=false`, `SonyAggregationFlags=10`, `ProtocolInfo` list that includes
  `http-get:*:video/x-matroska:*` and `http-get:*:video/avi:*` (URL above).
- **ReadyMedia / minidlna** — PS3 client entry with `FLAG_DLNA | FLAG_MIME_AVI_DIVX`
  (`{ EPS3, FLAG_DLNA | FLAG_MIME_AVI_DIVX, "PLAYSTATION 3", "PLAYSTATION 3", EXAVClientInfo }`),
  i.e. the same MIME-lie trick used for the Sony BDP: AVI is advertised as DivX.
  (<https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/clients.c>)
- **Gerbera** — no PS3/PS4 client profile in code; only documentation. Its recommended remedy for the
  PS4 mid-stream drop is to raise the SSDP `alive` interval to 86400
  (<https://github.com/gerbera/gerbera/blob/master/doc/config-server.rst>,
  <https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst>).
- **Gerbera** — implements only `ContentDirectory:1`:
  `#define UPNP_DESC_CDS_SERVICE_TYPE "urn:schemas-upnp-org:service:ContentDirectory:1"`
  (<https://github.com/gerbera/gerbera/blob/master/src/upnp/upnp_common.h>); same for ReadyMedia, whose
  SOAP handler only ever dispatches `urn:schemas-upnp-org:service:ContentDirectory:1`
  (<https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnpsoap.c>).
  So a `:2` probe from any of these three families goes unanswered.

---

## Cross-family notes worth carrying into the implementation

- **STANDARD** — `ContentDirectory:1` is the only service version implemented by every server examined
  here (ReadyMedia, Gerbera, Emby/Jellyfin, UMS). A client that probes `ContentDirectory:2` gets nothing
  back; implement `:1` first and only add `:2` if you find a client that insists.
- **OBSERVED** — `RequestedCount = 0` means "everything", not "nothing" (Emby's handler explicitly
  special-cases it: "Upnp inspector sends 0 as requestedCount when it wants everything").
- **OBSERVED** — the single most load-bearing DIDL string across all three families is
  `res/@protocolInfo` + `DLNA.ORG_PN`: a wrong profile makes Bravia show empty folders, makes the PS4
  hide the item, and makes UMS refuse to list the file ("If this value is wrong, the renderer may refuse
  listing or playing this file").
- **OPEN QUESTION** — for all three families I found **no** primary evidence that a missing `dc:date` or
  a missing `res/@duration` is fatal, and **no** evidence of chunked-encoding or HTTP/1.0 hostility
  specific to these clients. Treat those as generic-DLNA concerns rather than family quirks unless
  further evidence appears.

---

## Sources

Each entry: URL → what it contributed.

**Server source code (fetched)**

- <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/clients.c> — ReadyMedia client table:
  `ESonyBravia` (X-AV-Client-Info "BRAVIA"), `EPS3` (PLAYSTATION 3 / UA "PLAYSTATION", `FLAG_MIME_AVI_DIVX`),
  `EXbox` ("Xbox 360", UA "Xbox/"), `ESonyBDP` (`mv="2.0"`), `EMediaRoom` ("Microsoft-IPTV-Client").
- <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnpsoap.c> — ReadyMedia DIDL builder:
  `case ESonyBravia:` profile rewrite (AVC_TS_*_AC3 → AVC_TS_HD_50_AC3) for BRAVIA KDL-##*X### series;
  ContentDirectory:1-only SOAP dispatch.
- <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/upnphttp.c> — ReadyMedia HTTP layer:
  Xbox 360 root-description hack (modelnumber "1", friendly_name ": 1"); Sony BDP MKV→divx MIME lie.
- <https://sources.debian.org/data/main/m/minidlna/1.3.3+dfsg-2/metadata.c> — ReadyMedia DLNA.ORG_PN
  derivation for MPEG_TS (`HD_NA` / `SD_EU` / `SD_NA`, `_ISO` / `_T`).
- <https://github.com/gerbera/gerbera/blob/master/src/upnp/upnp_common.h> — Gerbera implements only
  `ContentDirectory:1` (SCID).
- <https://github.com/gerbera/gerbera/blob/master/src/upnp/mr_reg_service.h> — "media receiver registrar
  service for the Xbox 360 … only implemented to enable Xbox360 support".
- <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/ContentDirectory/ControlHandler.cs> —
  Emby's Browse handler: `RequestedCount=0` means "everything"; Filter/SortCriteria defaults.
- <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Xbox%20One.xml> —
  Xbox One device profile (UA `NSPlayer/12`, `FriendlyName.DLNA.ORG: XboxOne`,
  `EnableAlbumArtInDidl=false`, `EnableMSMediaReceiverRegistrar=false`, `TimelineOffsetSeconds=40`,
  ProtocolInfo list).
- <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Sony%20Bravia%20%282013%29.xml> —
  Bravia profile (X-AV-Client-Info regex, Microsoft "Windows Media Player Sharing" masquerade,
  `SonyAggregationFlags=10`, `xmlns:av`, AVC_TS_HD_*_AC3_T response profiles).
- <https://raw.githubusercontent.com/MediaBrowser/Emby/master/Emby.Dlna/Profiles/Xml/Sony%20PlayStation%204.xml> —
  PS4 profile (UA / X-AV-Client-Info "PLAYSTATION 4", `orgPn="AVI" mimeType="video/divx"`,
  `EnableAlbumArtInDidl=false`, `SonyAggregationFlags=10`).
- <https://github.com/MediaBrowser/Emby/tree/master/Emby.Dlna/Profiles/Xml> — device-profile directory
  listing (PS3/PS4 present, no PS5 profile).

**Renderer profiles / capability databases (fetched)**

- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Microsoft-XboxOne.conf> —
  Xbox One UA strings, detection strings, `SeekByTime=exclusive`, `TranscodeAudioTo441kHz`, `HalveBitrate`.
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Microsoft-Xbox360.conf> —
  Xbox 360 UA `Xbox/2.0.17150.0 UPnP/1.0 …`, `XBOX|XENON` detection, MS compatibility notes.
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-PlayStation4.conf> —
  PS4 UA strings, `SeekByTime=exclusive`, `H264LevelLimit=4.2`, `TranscodedVideoFileSize=-1`.
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-PlayStation3.conf> —
  PS3 UA / X-AV-Client-Info, `MimeTypesChanges video/avi=video/x-divx`, PCM wrapping.
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-Bravia.conf> —
  generic Bravia profile: `DLNALocalizationRequired`, `SendDLNAOrgFlags=false`, `ThumbnailAsResource`,
  `ForceJPGThumbnails`, `CreateDLNATreeFaster`.
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/Sony-X-Series-TV-2019-and-later.conf> —
  modern Bravia headers (`X-AV-Client-Info` mv="3.0", `X-AV-Physical-Unit-Info`).
- <https://raw.githubusercontent.com/UniversalMediaServer/UniversalMediaServer/main/src/main/external-resources/renderers/DefaultRenderer.conf> —
  option semantics: `DLNALocalizationRequired` ("Important for Sony Bravia TVs"), `SeekByTime=exclusive`
  (PS3 "Go To" hangs/crashes with "true"), `AccurateDLNAOrgPN` ("Some renderers like Sony TVs need the
  extra accuracy"), `SendDLNAOrgFlags`/`DLNAOrgPN` ("may refuse listing or playing this file"),
  `CreateDLNATreeFaster`.
- <https://api.github.com/repos/UniversalMediaServer/UniversalMediaServer/git/trees/main?recursive=1> —
  renderer inventory: PS3/PS4/Vita profiles exist, no PS5, no Xbox Series.

**Documentation (fetched)**

- <https://github.com/gerbera/gerbera/blob/master/doc/config-server.rst> — SSDP `alive` interval; explicit
  PS4 disconnect-after-5-minutes note and the 86400 remedy.
- <https://github.com/gerbera/gerbera/blob/master/doc/supported-devices.rst> — Sony section: PS3 "Firmware
  1.80 introduces UPnP/DLNA support"; PS4 "MediaPlayer seems to be flawed, so SSDP advertisements stop
  playback of videos. Set alive value in config.xml to e.g. 86400."; Microsoft section lists only Windows
  clients.

**Issue trackers (fetched issue bodies + comments)**

- <https://github.com/gerbera/gerbera/issues/1043> — PS4: "Cannot play the media", repeated transcode
  abort/restart.
- <https://github.com/gerbera/gerbera/issues/516> — Sony Bravia client, `.mpg` files not recognised.
- <https://github.com/gerbera/gerbera/issues/2333> — Xbox One as playback destination; the hang turned out
  to be a Gerbera import deadlock, **not** an Xbox quirk (useful negative result).
- <https://github.com/gerbera/gerbera/issues/251> — log shows `Playstation 4 via MediaPlayer` as a client.
- <https://github.com/jellyfin/jellyfin-plugin-dlna/issues/36> — PS4 hides unsupported items, 15–20 s
  start-up window, "Cannot play the media", freeze/crash, ts-container suspicion.
- <https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1430> — PS4 + ogg: MIME /
  DLNA.ORG_PN decides visibility; `upnp:class` example; "Some renderers will accept invalid values but
  reject missing values".
- <https://github.com/UniversalMediaServer/UniversalMediaServer/issues/1593> — captured PS4 HTTP request
  (`User-Agent: MediaPlayer/4.00 libhttp/6.00 (PlayStation 4)`, `Connection: Keep-Alive`).
- <https://github.com/MediaBrowser/Emby/issues/2769> — Xbox One creates a new UUID per boot; device-list
  pollution; Emby's fix.
- <https://github.com/huynhsontung/Screenbox/issues/277> — Xbox Series X: "DLNA/UPNP doesn't seem to work
  on Xbox"; MS TV app works.
- <https://github.com/jellyfin/jellyfin/issues/2164> — Jellyfin startup log enumerating the shipped device
  profiles incl. "Xbox One.xml", "Sony Bravia (2013).xml", "Sony PlayStation 4.xml"; WMP Browse logs.
- <https://github.com/jellyfin/jellyfin-plugin-dlna/issues/260> — Xbox 360 as DLNA-only music device.

**Forums / community (fetched)**

- <https://emby.media/community/topic/10359-xbox-one-dlna-media-player-preview/> — Xbox One DLNA preview:
  "Blast Alive Messages" requirement, ~30 s discovery delay, console reboot needed, MKV FF/RW broken,
  wrong episode played, "pausing every 13 or so seconds", "only shows two folders: TV and Movies".
- <https://www.reddit.com/r/xboxone/comments/a9vl6g/tech_movies_tv_shows_empty_directories_on_my_nas/> —
  "showing empty directories when I navigate By Folder … By Date or By Title … contents are listed".
- <https://www.reddit.com/r/xbox/comments/3b28pv/issues_with_dlna_server_nas/> — "it's saying all the
  folders are empty".
- <https://gamerz-forum.com/threads/xbox-one-dlna-media-server.350376/> — "sees the servers and folders
  but says they are empty" (Plex/XBMC/TVersity/TWonky/WMP).

**Reference implementation / community docs (fetched)**

- <https://raw.githubusercontent.com/pseudosavant/dlna-here/main/ROADMAP.md> — Xbox Media Player validated
  (discover/browse/play audio+video+images); "do not require a DLNA DMS marker or Microsoft registrar
  service"; "Xbox Series X" listed as ongoing test target; no PS5 in the matrix.
- <https://raw.githubusercontent.com/pseudosavant/dlna-here/main/COMPATIBILITY.md> — validated-player table
  incl. "Xbox Media Player | Discovery, browsing, and playback of tested audio, video, and images".
- <https://raw.githubusercontent.com/Octanium91/plex-ps5-local-playback/main/README.md> — "Stream to the
  PS5 Pro over DLNA — It is not a DLNA renderer. Its SSDP replies on port 40001 belong to Remote Play /
  Second Screen."; "the 'Media Player' app … does not exist for PS5 — that was a PS4 app. Media Gallery
  plays from USB only."

**Search-index-surfaced only (direct fetch blocked by Cloudflare / JS-only rendering) — treat as second-hand**

- <https://sourceforge.net/p/minidlna/discussion/879957/thread/b49d2aae/> — Bravia empty folders caused by
  `MPEG_TS_SD_NA_ISO` vs `MPEG_TS_SD_EU_ISO`; "folders show up but no content"; "Changing the two
  instances of _NA to _EU in metadata.c fixed the problem".
- <https://sourceforge.net/p/minidlna/bugs/101/> — sniffed Bravia `DLNA.ORG_PN=` / `SONY.COM_PN=` pairs
  from a KDL xxW5800.
- <https://sourceforge.net/p/minidlna/bugs/179/> — example of a DLNA client's Browse SOAP body
  (ContentDirectory:1, Filter `*`, StartingIndex 0, RequestedCount 200, empty SortCriteria) — Samsung, not
  Bravia.
- <https://sourceforge.net/p/minidlna/discussion/879957/thread/aaf08288/> — "Now my xbox one is Generic
  DLNA 1.5 and working perfectly" (i.e. ReadyMedia has no Xbox One profile; it lands in the generic
  DLNA 1.5 bucket).
- <https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/> and
  <https://sourceforge.net/projects/minidlna/files/minidlna/1.1.2/> — ReadyMedia changelog:
  "Forcibly tweak the model number for Xbox360 clients, or they might ignore us" (1.0.25);
  "Add support for the `av:mediaClass` tag, so some Sony devices can filter items by media type" (1.0.23).
- <https://www.universalmediaserver.com/forum/viewtopic.php?t=15656> — "the PS5 does not have a media
  player like the PS4… At most there is the gallery app… no way of playing over the network"; third-party
  Rad Player adding a UPnP MediaRenderer on PS4/PS5.
- <https://github.com/mounirhero/ps-play> — jailbroken-PS5 homebrew shipping its own "UPnP
  MediaRenderer:1" (evidence that stock PS5 has none).
- <https://support.xbox.com/en-US/help/hardware-network/display-sound/media-player-faq> (and
  <https://support.microsoft.com/en-us/topic/stream-music-and-video-content-to-your-xbox-console-using-playto-687e95c3-2c82-85ea-84d3-7b4330efa262>)
  — "The XBOX Media Player app is no longer supported as of June 2026" (JS-only page; not independently
  read).
- <https://www.sony.co.uk/electronics/support/televisions-projectors-lcd-tvs/klv-s40a10e/articles/00126002>
  — Bravia "server listed but files are inaccessible", "Update list", "Server Diagnostics" (bot-blocked).
