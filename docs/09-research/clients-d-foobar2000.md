# foobar2000 as a DLNA/UPnP CLIENT — primary-source findings

Research date: 2026-11-14. Every URL below was actually fetched (HTTP 200 unless noted).
Label key: **STANDARD** = protocol/spec-level fact · **OBSERVED** = directly seen in a fetched
page/file · **ASSUMPTION** = inference from observed data, not stated by a source ·
**OPEN QUESTION** = could not be resolved with the sources reachable.

---

## TL;DR

- Upstream foobar2000's **overview page and FAQ contain zero mentions of "UPnP" or "DLNA"**
  (verified by fetching and grepping the raw HTML). [OBSERVED]
- **BUT** upstream foobar2000 **v2.25 (2025-09-01) and later ship a built-in UPnP feature that is
  explicitly "both client (play to another device) and server (be controlled by another device)".**
  [OBSERVED] The upstream client half is a **control point aimed at MediaRenderers ("play to")**,
  *not* a ContentDirectory browser of remote media servers. [OBSERVED]
- The **third-party** components that give classic foobar2000 a real DLNA client are:
  **`foo_upnp`** (bubbleguuum — Renderer + Media Server + **Control Point** + **UPnP Browser**,
  x86-only, last updated 2015-01-13) and **`foo_out_upnp`** ("UPnP MediaRenderer Output",
  Peter — now obsolete, folded into foobar2000 v2.25). [OBSERVED]
- The names **`fooUPnP`**, **`foo_dlna`**, **`foo_upnp_server`**, and **`UPME`** could **not** be
  verified as existing. [OBSERVED / OPEN QUESTION]
- The premise "foobar2000 for Android source is on GitHub (`mushed82/foobar2000-android`)" is
  **false**: that repo and even that GitHub *user* return HTTP 404. foobar2000 mobile is closed
  source. [OBSERVED]

---

### foobar2000

#### What exists (and what does not)

**Upstream website — negative findings (important, and source-backed)**

- `https://www.foobar2000.org/` — fetched; raw HTML contains **0** occurrences of `upnp` and **0**
  of `dlna` (case-insensitive). The only network wording is a feature bullet:
  > "Stream music on your local network; foobar2000 can be used to control network audio players or foobar2000 running on another device."

  [OBSERVED] Note this is *generic* network-streaming language; the words "UPnP"/"DLNA" do not appear.
- `https://www.foobar2000.org/FAQ` — fetched; raw HTML contains **0** occurrences of `upnp` and
  **0** of `dlna`. There is no FAQ entry about UPnP, DLNA, or any media-server browsing. [OBSERVED]
- Consequence: **upstream foobar2000's public FAQ/overview do not document a DLNA client at all.**
  Any statement of the form "foobar2000 has no DLNA client" is **true for the documentation and
  true for all releases before v2.25**, but is **no longer true of the shipped product**. [OBSERVED]

**Upstream shipped code — the built-in UPnP feature (v2.25+)**

- `https://www.foobar2000.org/releasenotes-2.25` — fetched. Verbatim:
  > "This release introduces built-in network audio streaming feature, replacing old UPnP MediaRenderer Output component, as well as adds SMB & FTP library indexing from foobar2000 mobile."
  > "**UPnP streaming** — Enable remote control in Preferences / Networking / UPnP remote control, to make this foobar2000 instance visible on the network."
  > "foobar2000 instances with remote control enabled will be shown in other foobar2000 instances as output devices."
  > "Built-in UPnP media renderer, **both client (play to another device) and server (be controlled by another device)**."
  > "Other foobar2000 instances on your local network, with UPnP remote control feature active, will be listed as output devices."
  > "foobar2000-to-foobar2000 playback shows tags with album covers on the other end, using **Ogg FLAC as stream container**."

  [OBSERVED]
- `https://www.foobar2000.org/changelog` — fetched; same two bullets appear under `2.25`
  (and `2.25.1`: "Suppressed UPnP initialization if foobar2000 is invoked by installer."). [OBSERVED]
- `https://www.foobar2000.org/changelog-sdk` — fetched; SDK change log states:
  > "SDK/playback_control: Published 2.25 extensions: **playback_control_v4 (UPnP volume control)**."
  > "SDK/playOn: Published new 2.26 API for **sending network streams to renderer devices**."

  [OBSERVED] → the public SDK exposes a control-point surface; the wire protocol itself is not public.
- **Scope limit of the built-in desktop client** — [OBSERVED] The desktop `/changelog` contains **zero**
  entries mentioning browsing remote media servers, `ContentDirectory`, `DIDL`, or `SSDP`, whereas
  the **mobile** changelog contains many. Conclusion: the desktop built-in "client" half is a
  **renderer-targeted control point ("push / play to") only**, with **no remote-media-server
  browsing**. [ASSUMPTION, well supported by the changelog diff above]

**Third-party components that exist (all verified by fetching their pages)**

1. **`foo_upnp` — "UPnP/DLNA Renderer, Server, Control Point"** — https://www.foobar2000.org/components/view/foo_upnp
   - By **bubbleguuum**; "Powered by Platinum UPnP SDK: http://plutinosoft.com/platinum".
   - "Current version: **0.99.49, released on 2015-01-13**"; "**Supported system: Windows 32-bit**";
     "Works with foobar2000 v1.0 and newer."
   - Declares three roles: *Media Server*, *Media Server Browser (UPnP Browser)*, *Control Point
     (UPnP Controller)*, plus a *Renderer* role ("let foobar2000 playback be controllable by Control Points").
   - Official documentation: http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt (fetched, 281 lines).
   - **Listing caveat:** it does **not** appear on the default https://www.foobar2000.org/components
     list; it appears on https://www.foobar2000.org/components/system/x86 (32-bit filter), with tags
     "Columns UI panel, Default UI element, media library viewer, remote control, remote library,
     streaming". [OBSERVED] → *This is the only classic-foobar2000 component that is a genuine
     DLNA **client** (browses remote Media Servers via ContentDirectory).*
2. **`foo_out_upnp` — "UPnP MediaRenderer Output"** — https://www.foobar2000.org/components/view/foo_out_upnp
   - By **Peter**; "This component presents any UPnP MediaRenderer devices present on your home
     network as foobar2000 output devices."
   - "**Component obsolete, improved version of it is now included with foobar2000 v2.25.**"
   - Current version **1.4**, released **2022-08-29**; Windows 32-bit + 64-bit; foobar2000 v1.6+.
   - Release history: https://www.foobar2000.org/components/view/foo_out_upnp/releases (fetched).
   - Tags: "output, remote control, streaming". [OBSERVED]
3. **`foo_upnp_workaround` — "UPnP WAV Playback Workaround DSP"** — https://foobar.hyv.fi/pre1.4/?view=foo_upnp_workaround
   - v1.0, released 2017-12-23, 52.4 KB, Modified ISC license. [OBSERVED]
4. **`foo_noo` / `foo_noo_upnp`** — https://github.com/oxyflour/foo_noo_upnp (fetched)
   - "upnp media server, browser and controller based on foo_noo"; a Node.js/React app whose
     `package.json` (https://raw.githubusercontent.com/oxyflour/foo_noo_upnp/master/package.json,
     fetched) depends on **`node-ssdp`** and **`peer-upnp`** — i.e. SSDP discovery + a UPnP stack.
   - **Not** in the official components repo: https://www.foobar2000.org/components/view/foo_noo_upnp
     returns the byte-identical "component not found" page. [OBSERVED]
5. **Official-repo remote-control components that are NOT DLNA** — from
   https://www.foobar2000.org/components/tag/remote+control/system/x86 (fetched):
   "**Beefweb Remote Control** 0.10 — Provides web interface and REST-like API for controlling
   player remotely", "**Telnet Daemon** 1.0 — This component provides an unsecure Telnet based
   remote control interface", "UPnP MediaRenderer Output 1.4", "UPnP/DLNA Renderer, Server,
   Control Point 0.99.49". [OBSERVED] → foobar2000's popular remote-control story is HTTP/Telnet,
   not DLNA.
6. **Official-repo HTTP remote-library clients (not DLNA)** —
   https://www.foobar2000.org/components/tag/network (fetched): "**Navidrome / Subsonic Browser**
   1.41.0" and "**OpenSubsonic/Navidrome client implementation for foobar2000** 0.3.2", both tagged
   "network, remote library, streaming". [OBSERVED]

**Names in the task brief that DO NOT exist (verified negatives)**

- **"fooUPnP"** — no such project. GitHub repo search API `q=fooupnp` → `total_count: 0`
  (https://api.github.com/search/repositories?q=fooupnp). Not on foobar2000.org/components. The real
  component is **`foo_upnp`** (bubbleguuum). [OBSERVED]
- **`foo_dlna`** — https://www.foobar2000.org/components/view/foo_dlna returns the byte-identical
  "component not found" page (md5 `55dd4b3a9aa178cf38211fd69b2b2c7a`, identical to the
  `foo_httpcontrol` probe). Not listed anywhere in the components repo. [OBSERVED]
- **`foo_upnp_server`** — same not-found page, same md5. [OBSERVED]
- **`foo_httpcontrol`** — also returns the not-found page in the official repo (it is hosted
  elsewhere; a maintained fork exists at https://github.com/regorxxx/foo_httpcontrol_fb2kc).
  [OBSERVED for the not-found page; the fork URL was seen in GitHub search results only]
- **"UPME"** — **no evidence found.** Not in the components repo, not in any changelog, not in
  GitHub repo search (`upme+upnp+media+explorer` → API rate-limited, then a web search for
  `"UPME" UPnP DLNA renderer client open source` returned only unrelated projects).
  → **OPEN QUESTION**: "UPME" as a foobar2000 DLNA component appears to be a conflation; I could
  not resolve it to any real artifact. [OPEN QUESTION]
- **"UPnP Tools" component** — *named* in the desktop changelog ("Removed Wine workarounds from UPnP
  Tools component, no longer needed with above fix", 2.26.2 preview, 2026-10-14) but there is **no
  component page** for it in the public components repository. [OBSERVED that the name is used];
  [ASSUMPTION] that it is a bundled/pre-installed component rather than a public download.
- **GitHub `foobar2000` org** — https://github.com/foobar2000 exists but "doesn't have any public
  repositories yet" (page text matched). [OBSERVED]

---

#### What the client sends

*(Only for genuinely identified client code; each bullet is source-backed.)*

- **Renderer-targeted push (upstream v2.25+, classic foobar2000):** foobar2000 discovers UPnP
  MediaRenderer devices and drives them via AVTransport; for foobar2000→foobar2000 it streams
  **Ogg FLAC** so that tags and album covers arrive at the far end —
  "foobar2000-to-foobar2000 playback shows tags with album covers on the other end, using Ogg FLAC
  as stream container." https://www.foobar2000.org/releasenotes-2.25 [OBSERVED]
- **Icecast-style metadata on the pushed stream:** "UPnP: Made send **icy-meta style song titles**
  to devices that ask for them." https://www.foobar2000.org/changelog-2.26.2-preview [OBSERVED]
- **OpenHome / Linn OpenHome renderers are a first-class target:**
  "Enabled discovery of **OpenHome UPnP devices as output devices**" (desktop 2.25.10 /
  mobile 2.25.7); "UPnP output: Enabled streaming to OpenHome devices." and "Made discover OpenHome
  UPnP renderers." (mobile 2.26 preview changelog) —
  https://www.foobar2000.org/changelog , https://www.foobar2000.org/changelog-mobile-2.26-preview [OBSERVED]
- **SDK-level API (upstream):** `playback_control_v4` = "UPnP volume control";
  `playOn` = "API for sending network streams to renderer devices".
  https://www.foobar2000.org/changelog-sdk [OBSERVED]
- **`foo_upnp` Controller — AVTransport-style renderer control:** "This component acts as a Control
  Point and allow to control **playback and volume** of selected UPnP Av Renderer or Linn Ds
  Renderer." / "the Controller will **instruct the Renderer to play tracks in order**. Current track
  is the selected one." / the "UPnP Controller Playback queue" playlist "is the equivalent of the
  KinskyDesktop playlist and **represents what's to be played by the UPnPAV Renderer**" / for Linn Ds
  Playlist sources "the playlist **mirrors the remote on-device playlist**. Modifying the playlist
  modify the remote playlist." http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **`foo_upnp` Browser — ContentDirectory client:** "browse **remote Media Servers** for local
  playback or remote playback to an UPnP Renderer on the network"; "browse and play music via the
  Internet from another foo_upnp Media Server, **with optional transcoding to mp3 to save
  bandwidth**"; "UPnP Audio items can be sent to a playlist"; "**Search is supported only on remote
  foobar2000 media servers running this plugin**"; "Fetch remote album art" (configurable);
  "At the moment, the UPnP Browser does not display the individual tracks in the tree, so you can
  only add / insert a whole node content (usually an album)". Same URL. [OBSERVED]
- **Device-description URL handling (foobar2000 mobile's ContentDirectory client):** "Due to your
  server was added manually, **foobar2000 tries to fetch device description by default path
  `/DeviceDescription.xml`**." and "DLNA server may have any other entry point, e.g. native QNAP
  DLNA server has `/rootDesc.xml`. The server informs a client about this entry point when it is
  **discovered by the client in the local network**. But in our case the server cannot inform
  foobar2000 about its entry point and so the player just uses the default one."
  https://github.com/phts/qnap-dlna-adapter-for-foobar2k-mobile (README, fetched) [OBSERVED]
- **Content URLs are taken from the server's XML and fetched directly:** "DLNA server returns XML
  responses containing **hard-coded local URLs of the content** and player is not able to reach them
  outside of local network." Same URL. [OBSERVED]
- **Content-type leniency (mobile client):** "Enabled playback of FLAC over HTTP **without
  content-type**, served by certain buggy UPnP server."
  https://www.foobar2000.org/changelog-mobile-2.26-preview [OBSERVED]
- *(Comparison only, NOT foobar2000 — a modern open-source foobar2000-like player.)* `fooyin-dlna`
  (C++/Qt) documents its client contract: ContentDirectory service URL discovery "tries
  `description.xml`, `rootDesc.xml`, `root.xml`, `device.xml`, `rootDesc`, and the server root";
  "Content catalog browsing: folders as a tree, tracks as a table"; "**Pagination for large lists
  (500 items per request)**"; "**SOAP Browse, DIDL-Lite parsing**"; "Send tracks: Play (temporary
  playlist + start playback), Add (to the current playlist), Queue (to the queue)".
  https://github.com/giiirey/fooyin-dlna (README, fetched) [OBSERVED]

#### What it chokes on

- **Receivers that do not support LPCM; continuous WAV breaks them:** "Workaround DSP for UPnP
  receivers that do not support LPCM. **If the receiver is sent continuous WAV stream the playback
  stops once maximum size is reached.** To be used with foo_out_upnp."
  https://foobar.hyv.fi/pre1.4/?view=foo_upnp_workaround [OBSERVED]
- **Devices that refuse Pause (think live stream):** "**Fixes for devices that refuse to pause what
  whey perceive as a live stream, a stop command is issued instead.**"
  https://www.foobar2000.org/components/view/foo_out_upnp/releases (1.3) [OBSERVED]
- **Devices advertising multiple virtual devices in one device XML:** "Fixed detection of UPnP
  devices which present multiple virtual devices in one XML. **This includes Sonos and other
  brands.**" Same URL (1.3) [OBSERVED]
- **Manufacturer-specific discovery breakage:** "Fixed discovery of **Denon & Marantz** devices."
  (1.3.1); "Fixed discovery of **Pioneer and Devialet** devices (1.3 regression)." (1.3.2); mobile
  2.26 preview: "Fixed UPnP incompatibility with **Denon receivers**."; desktop 2.26.2 preview:
  "**Attempted fix for playback ending prematurely on Denon devices.**" [OBSERVED]
- **SSDP discovery disagreements / rare-device discovery bugs:** mobile 2.26 preview: "Resolved
  **UPnP discovery disagreement with BubbleUPnP**."; "Fixed a longtime UPnP search bug affecting
  discovery of **specifc rare devices**." [OBSERVED]
- **SSDP blocked by firewall → no servers visible:** "I have problems seeing servers on the LAN →
  **Check that your firewall do not block UDP multicast**."
  http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **HTTP seeking unsupported for some formats:** "seeking in **FLAC files streamed as FLAC** can give
  **'Unsupported file format'** errors"; "some audio formats **do not support HTTP seeking (ogg)**."
  Same URL. [OBSERVED]
- **Non-seekable/high-latency transport:** desktop changelog 2.1: "Changed how DSP settings are
  applied when playing through a **high-latency output such as UPnP**"; mobile 2.25: "Fixed
  **seeking in tracks played over HTTP** not working correctly in certain scenarios." [OBSERVED]
- **Sample-rate mismatches with renderers (PS3):** "I have problems with streaming on my PS3 →
  Use a wired connection wheneven possible. If using wireless, change your wifi channel id. **Set the
  output frequency to 48Khz**." http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Server-side quirks the client must tolerate:** mobile changelog — "Fixed UPnP incompatibility with
  **Logitech Music Server**." (1.6.4); "latest version of **Gerbera** server is now supported"
  (1.6.3); "Worked around **wrong M4A mime type** served by Gerbera UPnP server" (1.2.17);
  "**Workaround for relative URLs in UPnP server replies**" (1.6.4). [OBSERVED]
- **Unresponsive servers / ping failures:** mobile changelog 2.25.2: "**Fixed lockups with
  unresponsive UPnP server bookmarks on main page.**"; mobile 2.26 preview 2026-10-01: "Fixed UPnP
  server bookmarks **stuck in disabled state if servers weren't pingable** when the view was loaded
  - or if shown on main screen, when app was started."; mobile 2.25: "Changed behavior of **UPnP
  server ping** when viewing media server list, mitigated potential unwanted app shutdowns."
  [OBSERVED]
- **Android local-network sandboxing (SSDP/multicast):** "Android: Made compatible with **Android 17
  local network access sandboxing**." mobile 2.26 preview. [OBSERVED]
- **Linn DS / OpenHome renderer limitations (foo_upnp):** "Linn Playlist modifications done
  externally (for example using KinskyDesktop) are not reflected yet"; "Linn Pre-amp only products
  are not listed"; "no standby, random, repeat"; "reordering the Linn Playlist clicking on a column
  header will **totally break the playlist**"; "reordering items in the Linn Playlist by draging items
  only move the first n consecutive selected items"; "reordering items in the Linn Playlist from top
  to bottom generates **too much insertions/deletions**". http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Mobile media-server URL entry was broken and restored:** "**Restored ability to manually input
  UPnP media server URLs.**" (mobile 1.6.2). [OBSERVED]

#### Documented failure symptoms

- "**Unsupported file format**" error while seeking in FLAC streamed as FLAC.
  http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Playback stops mid-track at the maximum WAV size** on LPCM-incapable receivers.
  https://foobar.hyv.fi/pre1.4/?view=foo_upnp_workaround [OBSERVED]
- **Playback ends prematurely on Denon devices** (fixed "attempted"ly in 2.26.2).
  https://www.foobar2000.org/changelog-2.26.2-preview [OBSERVED]
- **Renderer/device not appearing in the output-device list at all** (Denon/Marantz/Pioneer/Devialet/
  Sonos, and multi-virtual-device XML) — https://www.foobar2000.org/components/view/foo_out_upnp/releases [OBSERVED]
- **No servers visible on the LAN** → firewall blocking UDP multicast (SSDP).
  http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **App lockups** on unresponsive UPnP server bookmarks (mobile main page), and UPnP
  "**resource leak** regression" plus "**UPnP renderer lockup regression**" entries.
  https://www.foobar2000.org/changelog-android , https://www.foobar2000.org/changelog-mobile-2.26-preview [OBSERVED]
- **"Add to playlist" silently disabled for remote UPnP folders** (mobile 1.3.1).
  https://www.foobar2000.org/changelog-android [OBSERVED]
- **UPnP stack initialising during installer runs** (desktop 2.25.1 workaround).
  https://www.foobar2000.org/changelog [OBSERVED]
- **Wrong/absent content-type** breaking FLAC-over-HTTP playback from buggy servers.
  https://www.foobar2000.org/changelog-mobile-2.26-preview [OBSERVED]
- **Content unreachable outside the LAN** because the server's XML hard-codes local URLs.
  https://github.com/phts/qnap-dlna-adapter-for-foobar2k-mobile [OBSERVED]

#### Server-side workarounds observed in existing implementations

- **Per-device transcoding / format adaptation (the big one).** `foo_upnp`'s Media Server exposes
  "powerful per device streaming options, to adapt audio format to the capabilities of the remote
  playing device. **Conversion to WAV or LPCM, transcoding to mp3, resampling, replaygain,
  bitdepth conversion, conversion to stereo, DSP chain,...**"
  https://www.foobar2000.org/components/view/foo_upnp [OBSERVED]
- **Bandwidth-aware transcoding for remote playback:** "browse and play music via the Internet from
  another foo_upnp Media Server, **with optional transcoding to mp3 to save bandwidth**".
  http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Container choice to carry metadata + artwork:** upstream v2.25 uses "**Ogg FLAC as stream
  container**" so "tags with album covers" reach the renderer.
  https://www.foobar2000.org/releasenotes-2.25 [OBSERVED]
- **Renderer-side standard completeness (mobile):** "Revised UPnP renderer code, **implemented various
  missing methods required by the standard**; made compatible with various popular UPnP remote
  control applications."; "UPnP renderer: **Implemented SetNextAVTransportURI** method to allow
  gapless playback with compatible clients."; "UPnP renderer: **Made accept MP3 (audio/mpeg)
  streams**."; "**Enabled audio/x-wav type** in UPnP renderer."; "UPnP renderer compatibility
  improved further (**M-SEARCH handling, correct content type in SOAP responses**)."
  https://www.foobar2000.org/changelog-mobile-2.26-preview [OBSERVED]
- **Browse-tree modelling of the DIDL container hierarchy (server-side):** "customizable browse tree";
  server-side XML `foo_upnp_ml_tree.xml` with clauses like
  `<spec type="object.container.genre.musicGenre">%genre%</spec>` and
  `<spec type="object.container.genre.musicAlbum">%album%</spec>`, plus
  `<SubTree label="..." query="compilation PRESENT">`. http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Server must implement Search for the client's Search to work:** "At the moment, **Search is
  supported only on remote foobar2000 media servers running this plugin**."
  Same URL. [OBSERVED] → a strong hint that Search is a non-standard server extension.
- **Interop note for server implementers:** "works well when accessed from **WMP12** (possibly also
  with WMP11, untested)"; "accessible from the Internet (only from foobar2000 or an iPhone/iTouch
  with **PlugPlayer**)". https://www.foobar2000.org/components/view/foo_upnp [OBSERVED]
- **Internet-facing server config:** public server IP + router port-forward + "**login/password to
  protect remote access**"; "it is possible to use **port 80** as the server's port".
  http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt [OBSERVED]
- **Client-side DSP workaround (pairs with the LPCM issue):** `foo_upnp_workaround` — a DSP inserted
  in front of `foo_out_upnp` to reshape the stream for LPCM-incapable receivers.
  https://foobar.hyv.fi/pre1.4/?view=foo_upnp_workaround [OBSERVED]
- **Alternative server-side device-description entry point:** the QNAP adapter exists purely so the
  server can advertise `/DeviceDescription.xml` and rewrite content URLs to a public hostname —
  i.e. the *server* must cooperate with the client's hard-coded assumptions.
  https://github.com/phts/qnap-dlna-adapter-for-foobar2k-mobile [OBSERVED]

---

### foobar2000 for Android/iOS (a SERVER, not a client)

**Correction to the brief: mobile foobar2000 is BOTH a server and a client, since v2.25.**
The label "a SERVER, not a client" is accurate for the *renderer/remote-control* feature, but
mobile also ships a genuine **ContentDirectory client** that browses remote UPnP media servers.

**Server (renderer) role**

- `https://www.foobar2000.org/releasenotes-mobile-2.25` — fetched. Verbatim:
  > "This release introduces built-in network audio streaming feature, **based on UPnP MediaRenderer
  > Output component for classic foobar2000**."
  > "**UPnP streaming** — Enable remote control in More Settings / Remote Control, to make this
  > foobar2000 instance visible on the network."
  > "foobar2000 instances with remote control enabled will be shown in other foobar2000 instances as
  > output devices (in foobar2000 mobile: More Settings / Audio / Output Device)."
  > "Built-in UPnP media renderer, **both client (play to another device) and server (be controlled
  > by another device)**."
  > "foobar2000-to-foobar2000 playback shows tags with album covers on the other end, using **Ogg
  > FLAC as stream container**."

  [OBSERVED]
- `https://www.foobar2000.org/changelog-android` and `https://www.foobar2000.org/changelog-ios`
  (both fetched) — mobile renderer evolution:
  - 2.26.2: "UPnP: Made send icy-meta style song titles to devices that ask for them."
  - 2.25.7: "Improved implementation of built-in UPnP Media Renderer, implemented **gapless playback
    compatible with popular UPnP control apps**."; "Enabled discovery of OpenHome UPnP devices as
    output devices."
  - 2.25: "Built-in UPnP media renderer, both client (play to another device) and server (be
    controlled by another device)."
  [OBSERVED]
- `https://www.foobar2000.org/changelog-mobile-2.26-preview` — the server-side implementation
  details most useful as a reference for a working renderer:
  - "Revised UPnP renderer code, implemented various missing methods required by the standard"
  - "UPnP renderer: Implemented **SetNextAVTransportURI** method to allow gapless playback with
    compatible clients."
  - "UPnP renderer: Made accept **MP3 (audio/mpeg)** streams."
  - "Enabled **audio/x-wav** type in UPnP renderer."
  - "UPnP renderer compatibility improved further (**M-SEARCH handling, correct content type in SOAP
    responses**)."
  - "UPnP output: Enabled streaming to OpenHome devices." / "Made discover OpenHome UPnP renderers."
  - "Android: Made compatible with **Android 17 local network access sandboxing**."
  [OBSERVED]
- **Media-server-publishing role (server):** "expose the foobar2000 Media Library, playlists and
  'what is currently playing'"; "stream album art"; "customizable browse tree" — that is `foo_upnp`'s
  Media Server mode on desktop, and mobile inherits the same architecture.
  https://www.foobar2000.org/components/view/foo_upnp [OBSERVED]

**Client role on mobile (contradicts "not a client")**

- `https://www.foobar2000.org/changelog-android` (covers both Android and iOS histories; identical
  entries appear on https://www.foobar2000.org/changelog-ios):
  - 1.2.22: "**Improved UPnP browsing performance**, last open UPnP folder is now remembered."
  - 1.3.1: "Fixed a bug that disabled '**add to playlist**' for remote folders such as UPnP."
  - 1.6.2: "**Restored ability to manually input UPnP media server URLs**."
  - 1.6.3: "Fixed various UPnP related bugs; **latest version of Gerbera server is now supported**."
  - 1.6.4: "Fixed various issues with **manually-entered UPnP media servers**"; "Fixed **foo_upnp**
    stream looping regression."; "Fixed UPnP incompatibility with Logitech Music Server."
  - 2.25.2: "Fixed lockups with unresponsive UPnP **server bookmarks** on main page."
  [OBSERVED]
- `https://www.foobar2000.org/releasenotes-mobile-2.25`: "Changed behavior of **UPnP server ping
  when viewing media server list**, mitigated potential unwanted app shutdowns." [OBSERVED]
  → mobile keeps a list of remote media servers and pings them: a ContentDirectory client.

**Source code: the brief's premise is FALSE — there is no public foobar2000 mobile source**

- `https://github.com/mushed82/foobar2000-android` → **HTTP 404** (fetched). [OBSERVED]
- `https://github.com/mushed82` (the user account) → **HTTP 404** (fetched) — the account does not
  exist. [OBSERVED]
- GitHub API `https://api.github.com/repos/mushed82/foobar2000-android` → `{"message":"Not Found"}` (fetched). [OBSERVED]
- `https://github.com/foobar2000` (org) → exists but "**doesn't have any public repositories yet**" (fetched). [OBSERVED]
- `https://www.foobar2000.org/android` and `https://www.foobar2000.org/ios` (both fetched): the
  license is the closed "**foobar2000 mobile license**"; neither page mentions UPnP/DLNA (0 hits).
  [OBSERVED]
- **Consequence:** the requested "fetch the mobile UPnP **server** source file and describe what it
  sends" could **not** be satisfied from a source file — there is no public server source to fetch.
  The server's observable behaviour is documented instead from
  https://www.foobar2000.org/releasenotes-mobile-2.25 ,
  https://www.foobar2000.org/changelog-android ,
  https://www.foobar2000.org/changelog-ios and
  https://www.foobar2000.org/changelog-mobile-2.26-preview . [OBSERVED]
  **OPEN QUESTION:** the exact bytes the mobile renderer emits (SOAP envelopes, DIDL-Lite templates,
  AVTransport argument encoding) are not published anywhere I could reach; the renderer
  implementation is closed. Treat "what the server sends" as reconstructed from changelog wording,
  not from code.
- GitHub repo search for a public foobar2000 mobile source tree: `q=foobar2000+mobile+upnp` →
  `total_count: 0`; `q=foobar2000+upnp` → 4 hits, none a foobar2000 DLNA client (they are
  `pcjustin/hifi-discord-presence`, `yukitaka0915-se/M3uEditorWinForms`,
  `OrionNebula/hyper-media-control`, `silasmariusz/TuneThatHue`);
  `q=foobar2000+dlna` → 2 hits, both already covered. [OBSERVED]

---

## Sources

| URL | What it contributed |
|---|---|
| https://www.foobar2000.org/ | Upstream overview; 0 "upnp"/"dlna" hits; verbatim "Stream music on your local network…" feature bullet |
| https://www.foobar2000.org/FAQ | Upstream FAQ; 0 "upnp"/"dlna" hits; no DLNA entry at all |
| https://www.foobar2000.org/components | Full component list (163 components); "UPnP MediaRenderer Output" present; `foo_upnp` absent |
| https://www.foobar2000.org/components/system/x86 | 32-bit-only listing; where `foo_upnp` (0.99.49, 2015-01-13) actually lives, with its tags |
| https://www.foobar2000.org/components/view/foo_out_upnp | "UPnP MediaRenderer Output" page; "Component obsolete, improved version of it is now included with foobar2000 v2.25"; v1.4 / 2022-08-29 |
| https://www.foobar2000.org/components/view/foo_out_upnp/releases | Changelog 1.3/1.3.1/1.3.2 — device-specific discovery fixes, Pause→Stop fallback, multi-virtual-device XML |
| https://www.foobar2000.org/components/view/foo_upnp | "UPnP/DLNA Renderer, Server, Control Point" by bubbleguuum; Platinum UPnP SDK; Media Server / Browser / Control Point roles; 32-bit only |
| https://www.foobar2000.org/components/view/foo_upnp/releases | Only release 0.99.49 (2015-01-13) |
| https://www.foobar2000.org/components/tag/remote+control/system/x86 | Beefweb Remote Control, Telnet Daemon — foobar2000's non-DLNA remote-control components |
| https://www.foobar2000.org/components/tag/network | Navidrome / Subsonic + OpenSubsonic clients (HTTP-based, not DLNA) |
| https://www.foobar2000.org/components/view/foo_dlna · /foo_upnp_server · /foo_httpcontrol · /foo_noo_upnp | All return the byte-identical "component not found" page (md5 55dd4b3a9aa178cf38211fd69b2b2c7a) → negative evidence |
| http://bubbleguuum.free.fr/foo_upnp/foo_upnp.txt | **Primary foo_upnp documentation** — Controller/Browser behaviour, Linn DS limitations, KNOWN ISSUES (FLAC seek, ogg HTTP seeking), PS3 48 kHz advice, UDP-multicast/firewall, search-only-on-foo_upnp-servers, browse-tree XML, Internet access/login, logging |
| https://foobar.hyv.fi/pre1.4/?view=foo_upnp_workaround | "UPnP WAV Playback Workaround DSP" — LPCM-less receivers, continuous WAV stops at max size |
| https://www.foobar2000.org/releasenotes-2.25 | Desktop v2.25 built-in UPnP: "both client … and server", Ogg FLAC stream container, Preferences/Networking/UPnP remote control |
| https://www.foobar2000.org/releasenotes-mobile-2.25 | Mobile v2.25 built-in UPnP (client+server), UPnP server ping on media-server list |
| https://www.foobar2000.org/changelog | Desktop change log — 2.25 / 2.25.1 / 2.25.9 / 2.25.10 / 2.25.11 / 2.26.2 UPnP entries; **no** remote-media-server browsing entries |
| https://www.foobar2000.org/changelog-2.26.2-preview | "icy-meta style song titles", "playback ending prematurely on Denon devices", "Removed Wine workarounds from UPnP Tools component" |
| https://www.foobar2000.org/changelog-android | Mobile/Android history — UPnP browsing, server bookmarks, manually entered media servers, Gerbera/Logitech compat, relative-URL workaround, M4A mime workaround |
| https://www.foobar2000.org/changelog-ios | Same mobile history for iOS (corroborates Android) |
| https://www.foobar2000.org/changelog-mobile-2.26-preview | Renderer implementation details: SetNextAVTransportURI, audio/mpeg + audio/x-wav, M-SEARCH handling, SOAP content type, OpenHome discovery, Denon/BubbleUPnP/rare-device discovery fixes, resource-leak & lockup fixes, FLAC without content-type, Android 17 local-network sandboxing |
| https://www.foobar2000.org/changelog-sdk | `playback_control_v4` (UPnP volume control) and `playOn` (send network streams to renderer devices) SDK APIs |
| https://www.foobar2000.org/android · https://www.foobar2000.org/ios | Download pages; closed "foobar2000 mobile license"; 0 UPnP/DLNA mentions |
| https://www.foobar2000.org/news | News timeline — v2.25 preview "built-in UPnP media renderer" (2025-05-09), v2.25 release (2025-09-01), v1.4.5 UPnP MediaRenderer Output update (2019-05-08), v1.2 first UPnP component (2012-12-17) |
| https://www.foobar2000.org/SDK · https://www.foobar2000.org/RTFM | SDK landing page and online-docs entry point (RTFM returned 403 to both fetchers) |
| https://github.com/mushed82/foobar2000-android · https://github.com/mushed82 · https://api.github.com/repos/mushed82/foobar2000-android | All 404 → **no public mobile source**; premise refuted |
| https://github.com/foobar2000 | Official GitHub org: "doesn't have any public repositories yet" |
| https://api.github.com/search/repositories?q=fooupnp | `total_count: 0` → no "fooUPnP" project |
| https://api.github.com/search/repositories?q=foobar2000+upnp / +dlna / foobar2000+mobile+upnp | Full GitHub inventory: only 4 / 2 / 0 relevant repos; none is a foobar2000 DLNA client |
| https://github.com/oxyflour/foo_noo_upnp (+ raw package.json) | foo_noo UPnP server/browser/controller web app; node-ssdp + peer-upnp dependencies |
| https://github.com/phts/qnap-dlna-adapter-for-foobar2k-mobile | **Client contract evidence**: default `/DeviceDescription.xml` fetch, SSDP-provided entry points (e.g. `/rootDesc.xml`), hard-coded local content URLs in server XML |
| https://github.com/giiirey/fooyin-dlna | Open-source DLNA **client** reference (not foobar2000): ContentDirectory URL discovery list, SOAP Browse, DIDL-Lite, 500-item pagination, Play via temporary playlist |
| https://foobar2000.xrea.jp/?General+64bit | Real-world usage of the client: BubbleUPnP appears in foobar2000's output list as "UPnP: BubbleUPnP (P00A)" |
| https://www.foobar2000.org/other · https://www.foobar2000.org/windows | Other-projects and platform pages; 0 UPnP/DLNA mentions |

**Sources attempted but unreachable (Cloudflare-blocked from this network; claims from them are
NOT asserted anywhere above):** https://wiki.hydrogenaud.io/index.php?title=Foobar2000:Components/UPnP_MediaRenderer_Output_(foo_out_upnp)
and https://hydrogenaud.io/index.php/topic,98370 (the official "Discussion" link on the foo_out_upnp
page), https://www.foobar2000.org/RTFM. → **OPEN QUESTION:** the hydrogenaudio wiki page and the
foo_out_upnp forum thread may contain further device-compatibility detail that is not reflected here.
