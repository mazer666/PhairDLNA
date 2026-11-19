# PhairDLNA — Platform Integration Brief (macOS first; Linux & Windows equivalents)
Status: **Research input — evidence, not norm** · Owner role: Architect / Platform specialist · Version/date: 1.5 / 2026-11-18
> Committed so that every citation in the feature specifications resolves. Where this document and a requirement disagree, the requirement wins.

Purpose: Record platform evidence and assumptions; this brief does not approve product or security decisions.
Revision history: 1.5 / 2026-11-18 — links the folder-access evidence requirement to the repeatable OD-15 protocol; 1.4 aligns the platform matrix and D1 with OD-15, removes the assumed root daemon/TCC transfer architecture and updates the primary app target.

**Status:** Requirements-phase decision brief · **Date:** 2026-11-14 · **Author:** Platform Integration Specialist
**Scope:** macOS (arm64 + x86_64) as the first shipping platform, with Linux and Windows designed in from the start.
**Product constraints (already fixed by the owner):** macOS first · Linux & Windows designed in from day one · Rust core must stay platform-neutral behind clean adapters · headless server must run with no GUI · native macOS application with a main window and menu-bar status item · AGPL-3.0-or-later · free, ad-free, telemetry-free, offline/LAN-only.

## Label legend

| Label | Meaning |
|---|---|
| **VERIFIED** | Stated by a primary source that was actually fetched for this brief; URL given inline or in *Sources*. |
| **ASSUMPTION** | My inference from verified facts or from community practice. Not stated by the cited source. |
| **OPEN QUESTION** | Genuinely unknown, ambiguous in Apple's own docs, or unverifiable without a device. |
| **UNVERIFIED** | A claim I could not confirm against a fetched primary source (usually a forum report or folklore). |

Apple's developer documentation pages are JavaScript-rendered. For this brief they were fetched as their underlying JSON/markdown payloads
(`https://developer.apple.com/tutorials/data/documentation/<slug>.md` and `…/technotes/<slug>.json`). The canonical human-facing URL is
given in *Sources*; the payload URL is the one actually retrieved.

---

## 0. Executive summary (read this if nothing else)

1. **The single most consequential macOS fact for PhairDLNA is Local Network Privacy (macOS 15+).** SSDP is a *local network operation*:
   sending UDP multicast **and** receiving incoming UDP multicast both require the Local Network privilege. A per-user `launchd` **agent**
   is *not* exempt; a `launchd` **daemon** and any process running as **root** *are* exempt. **VERIFIED** (TN3179).
   → PLT-013 currently requires a system-domain `LaunchDaemon`, but OD-15 gates that choice on a process identity and access model that also satisfy OPS-015/PLT-018. A per-user service remains a candidate if the owner revises PLT-013 and its Local Network behavior is verified.
2. **The Unicode trap is subtler than "macOS stores NFD".** Modern macOS uses **APFS**, which is *normalization-insensitive* (NFC and NFD
   spellings of the same name are **the same file**) but *preserves the on-disk normalization*. HFS+ (still mountable) stores **NFD**.
   Linux filesystems are byte-preserving, so NFC and NFD are **two different files** there; SMB uses **NFC**. **VERIFIED** (Apple APFS
   Guide FAQ, TN1150, QA1173, Unicode FAQ).
   → The index must key on a **normalized identity** and separately keep the **raw path**.
3. **Distribution has two realistic options for a free, non-revenue project** (§3): (A) *unsigned/ad-hoc binary, `curl`/`brew` install* —
   zero cost, no Apple account, but Gatekeeper/`xattr` friction and (on Apple silicon) a mandatory code signature anyway;
   (B) *Developer ID + notarization* — best UX, but requires a paid Apple Developer Program membership (**99 USD/year**) and a real
   maintainer identity. **VERIFIED**.
   → Recommendation: ship **A now**, structure for **B later**; do not let the distribution choice block the platform adapter design.
4. **A pure-Rust macOS menu bar app with no Swift and no webview is realistic.** `muda` implements its macOS backend on
   `objc2` + `objc2-app-kit` (i.e. `NSStatusItem`), and `tray-icon` wraps `muda`. **VERIFIED** (crates.io metadata + docs.rs Cargo.toml).
   The blocker is not the language — it is that a *good* menu bar app should be an `.app` bundle, and a bundle is what `SMAppService`
   requires for helper registration.
5. **The recommended platform adapter interface is in §10.** It is deliberately small: 8 traits, no Apple types in the signatures.

---

## 1. macOS networking

### 1.1 SSDP multicast specifics

| Fact | Label | Source |
|---|---|---|
| SSDP discovery uses the reserved IPv4 multicast address **239.255.255.250** on port **1900**; the `HOST` header is `239.255.255.250:1900`. | VERIFIED | UPnP Device Architecture 1.1, §1 "Discovery" and §2 "Description" |
| If the port number is omitted, the receiver **MUST** assume **1900**. | VERIFIED | same |
| Control points **MUST** listen on port 1900 (or the `SEARCHPORT.UPNP.ORG` port) for unicast search messages. | VERIFIED | same |
| Control points multicast `M-SEARCH` to 239.255.255.250:1900; devices multicast `NOTIFY` to the same group. | VERIFIED | same |
| For IPv6: devices and control points **MUST** listen for multicast on **FF02::C** (link-local), **FF05::C** (site-local) and **FF0E::C** (global). The `HOST` header uses the bracketed form, e.g. `HOST: [FF02::C]:1900`. | VERIFIED | UPnP DA 1.1, Annex A (IPv6) |
| Multicast SSDP sent from a link-local source **MUST** use link-local scope FF02::C; from a global source, FF0E::C or FF05::C. | VERIFIED | UPnP DA 1.1, Annex A |
| Devices **MUST** multicast SSDP messages **for each UPnP-enabled interface** (i.e. per-interface sockets). | VERIFIED | UPnP DA 1.1, Annex A |
| UPnP is IPv4-oriented; IPv6-only devices are not allowed; "Devices and control points **MUST** support IPv4-only". | VERIFIED | UPnP DA 1.1, Annex A |

**Practical consequence (ASSUMPTION):** PhairDLNA must open **one SSDP UDP socket per IPv4 interface** and **one per IPv6 interface**,
each joined to the appropriate multicast group, and each bound to the interface's address. A single socket on `0.0.0.0` will *receive*
multicast but will **not** produce correct per-interface source addresses in `LOCATION` headers, and will fail on multi-homed hosts
(wired + Wi-Fi) where the UPnP spec explicitly wants one advertisement per interface.

### 1.2 Local Network privacy (macOS 15+)

Source for this entire subsection: **TN3179 — Understanding local network privacy** (Apple Developer Technical Note), fetched as JSON payload.

| Fact | Label |
|---|---|
| Local network privacy is supported on macOS **from macOS 15** (iOS 14, iPadOS 14, visionOS 1; **not** tvOS/watchOS). | VERIFIED |
| macOS keeps **separate Local Network privacy state per user account**. | VERIFIED |
| macOS **automatically allows** local network access for: (a) **any daemon started by launchd**, (b) **any program running as root**, (c) **command-line tools run from Terminal or over SSH** (including their child processes). | VERIFIED |
| **ASIDE in TN3179:** *"The exception for launchd daemons doesn't apply to launchd agents. If you're building a launchd agent, see the discussion below."* → **a `LaunchAgent` is NOT exempt**; it can trigger the prompt. | VERIFIED |
| A *local network operation* includes: outgoing TCP connect, sending UDP unicast/multicast/broadcast, connecting a UDP socket, **receiving an incoming UDP multicast**, DNS resolution of a `.local` name, and **all Bonjour operations** (register/browse/resolve). | VERIFIED |
| **NOT** a local network operation: *listening* for and accepting incoming TCP connections; receiving an incoming UDP **unicast** packet; resolving a non-`.local` DNS name; traffic from `NSURLSession`/`URLSession`-family helpers / WebKit / Safari (Apple names the exact frameworks in the doc's cross-references). | VERIFIED |
| The check is implemented deep in the networking stack and therefore applies to **all** networking APIs: BSD sockets, Network framework, etc. | VERIFIED |
| **Multicast entitlement** `com.apple.developer.networking.multicast` is required on iOS/iPadOS/visionOS for send/receive multicast and for "arbitrary Bonjour service types"; it is **not required on macOS**. | VERIFIED |
| If the system presents the alert, it **may deny the operation immediately, before the user responds**. → retry logic required. | VERIFIED |
| There is **no API to query** whether the process currently has local network access (FB8711182). Detect denial via `kDNSServiceErr_PolicyDenied` (-65570) for Bonjour, or `NWConnection.currentPath.unsatisfiedReason == .localNetworkDenied` for TCP. | VERIFIED |
| If the user revokes the privilege, an **established local TCP connection closes**. | VERIFIED |
| macOS **fails to display the alert** when a process with a very short lifespan performs a local network operation (FB16131937). TN3179's workaround: *"update your code to not exit immediately after a local network operation fails."* | VERIFIED |
| macOS 15.1 fixed "a number of local network privacy bugs"; retest on 15.1+. The `AllowedEthernetLocalNetworkAddresses` / `AllowedWiFiLocalNetworkAddresses` defaults **debuted in macOS 15.5**. | VERIFIED |
| On macOS 15.5+, two system-wide user defaults allow *all* programs to reach a CIDR: `defaults write com.apple.network.local-network AllowedEthernetLocalNetworkAddresses -array "169.254.0.0/16"` (and the Wi-Fi twin), in domain `com.apple.network.local-network`. Requires `sudo` and a restart. | VERIFIED |
| Local network privacy tracks program identity by **code signature**, and uses the **main executable's UUID**. Unsigned or ad-hoc-signed code ("Sign to Run Locally") makes identity tracking unreliable → *"local network privacy may behave weirdly"*. | VERIFIED (TN3179) + UNVERIFIED (Apple Developer Forums thread 792453, which additionally recommends a stable Apple-issued identity). |
| The macOS Local Network list in System Settings gained `+`/`-` buttons to reset a program's privilege only in **macOS 27.2**; on earlier versions the only reset is a new user account or a VM snapshot. | VERIFIED |

**Direct answers to the questions asked:**

* **What triggers the prompt?** The first time *this program's* process performs a local network operation — for PhairDLNA that means
  sending an SSDP `M-SEARCH` multicast, or **receiving** a multicast `NOTIFY`/unicast reply, or any Bonjour browse. TN3179's table is
  explicit that *"Receiving an incoming UDP multicast"* **requires** local network access.
* **Which API / permission?** There is no explicit permission request API. The system shows the alert implicitly on first local-network
  operation. The declarative half is the `Info.plist` string `NSLocalNetworkUsageDescription`.
* **Does a background launchd agent show the prompt?** **Yes, an agent can.** TN3179's aside removes agents from the launchd-daemon
  exemption. Two mitigations are documented: (a) set `AssociatedBundleIdentifiers` in the agent's plist so macOS knows which app is the
  "responsible code" (see §4); (b) **do not exit immediately on failure** (FB16131937) or the alert never appears and the agent just
  silently fails.
* **Requirement to bind to specific interfaces?** TN3179 is explicit: *"If you can't use Bonjour … run your service discovery code on all
  broadcast-capable interfaces."* and *"If you must work with a specific type of interface, call `getifaddrs` to get the full interface list
  and then filter that list by the interface type. Get the functional type of the interface using the `SIOCGIFFUNCTIONALTYPE` ioctl."*
  TN3179 also warns that **BSD interface names such as `en0` are not API** and that a Wi-Fi interface is not always `en0`.
  → **VERIFIED.** Bind one SSDP socket per broadcast-capable interface, discovered via `getifaddrs` + `SIOCGIFFUNCTIONALTYPE`.

### 1.3 Info.plist keys

| Key | Purpose | Label |
|---|---|---|
| `NSLocalNetworkUsageDescription` | "A message that tells people why the app is requesting access to the local network." Declared as `macOS 11.0+`, but the *feature* it gates (Local Network privacy) only exists on macOS 15+. Required for anything using Bonjour **or** direct unicast/multicast to local hosts. | VERIFIED — Apple docs page + `Protected resources` page |
| `NSBonjourServices` | "Bonjour service types browsed by the app." An array of strings; format `_ipp._tcp`, `_myservice._udp` (leading underscore, `_tcp`/`_udp` suffix). **Include all service types your app expects to use.** | VERIFIED — Apple docs page |
| TN3179 calls the key `NSLocalUsageDescription` in two places — that is a **typo in the Apple doc**; the real key is `NSLocalNetworkUsageDescription` (the key that actually exists in `BundleResources`). | VERIFIED (both keys documented; the inconsistency is visible in the TN3179 text) |
| App extensions: put `NSLocalNetworkUsageDescription` and `NSBonjourServices` on the **container app's** `Info.plist`, not the extension's. | VERIFIED (TN3179) |
| If you ship a launchd agent **not** installed via `SMAppService`, set **`AssociatedBundleIdentifiers`** in the launchd property list so macOS knows the responsible code. | VERIFIED (TN3179 + `launchd.plist(5)`) |

**Note on standalone executables:** TN3179 says most standalone executables (not a bundle's main executable) don't need the key, because
they are either automatically allowed (daemons, CLI tools) or have an app acting as their responsible code. If a standalone executable
*does* need it, embed an `Info.plist` **inside the executable** (Xcode build setting "Create Info.plist Section in Binary build setting").
→ **VERIFIED.** For a Rust binary this is `#[link_section = "__TEXT,__info_plist"]` (or `__DATA,__info_plist` on newer linkers) — **ASSUMPTION**
that this is how to embed it in Rust; the Apple mechanism is verified, the Rust incantation is not.

---

## 2. macOS firewall

**Read this section with the uncertainty flag on.** Apple's user-facing documentation describes the firewall's *policy model* precisely but
says almost nothing about *unsigned vs signed* binaries as a category, and says nothing at all about headless daemons.

### 2.1 What the firewall does with a listening socket

| Fact | Label | Source |
|---|---|---|
| macOS's application firewall is configured under System Settings > Network > Firewall. Configurations supported: *Block all incoming connections regardless of app*; *Automatically allow built-in software*; *Automatically allow downloaded and signed software*; *Add or deny access based on user-specified apps*; *prevent ICMP probing/portscan responses* (stealth mode). | VERIFIED | Apple Platform Security, "Firewall security in macOS" |
| "**Automatically allow downloaded signed software to receive incoming connections** — Allow downloaded apps and services that are **signed by a valid certificate authority** to be automatically added to the list of allowed apps, **without your authorization**." (On by default.) | VERIFIED | Apple Support, "Manage Firewall" |
| "When the Mac detects an attempt to connect to an app you haven't added to the list and given access to, an alert message appears asking if you want to allow or deny the connection over the network or internet. **Until you take action, the message remains, and any attempts to connect to the app are denied.**" | VERIFIED | Apple Support, "Manage Firewall" |
| MDM-equivalent: the `Firewall` payload exposes `AllowSigned`, `AllowSignedApp`, and an `Applications` list. `AllowSignedApp` "If `true`, the system allows **downloaded signed software** to receive incoming connections." Default `true`. | VERIFIED | Apple Developer, Device Management `Firewall` payload |

### 2.2 Does an unsigned binary get blocked? (be honest)

- **VERIFIED:** the *policy* is per-application and the "auto-allow" list is populated from software **signed by a valid certificate
  authority**. An unsigned/ad-hoc binary is therefore **not** auto-allowed and is subject to the prompt + deny-until-answered rule.
- **UNVERIFIED (Apple Developer Forums, thread 726057 — a report, not documentation):** with the firewall **enabled**, an **unsigned**
  binary triggered the "allow incoming connections" prompt; the same binary **signed with an Apple-issued certificate** received traffic
  **without any prompt** (consistent with "Automatically allow downloaded signed software" being on). The same thread notes the binary must
  have a *stable* signing identity, and cites TN3127.
- **VERIFIED (Apple Platform Security, "Signed system volume security"):** *"A Mac with Apple silicon doesn't permit native arm64 code to
  execute unless a valid signature is attached. This signature can be as simple as an ad hoc code signature."* → **on Apple silicon an
  ad-hoc signature is mandatory just to run.** So an "unsigned" macOS binary on arm64 is not actually unsigned; it is ad-hoc signed
  (`codesign -s -`). That ad-hoc signature carries **no CA identity**, so the firewall's "downloaded signed software" auto-allow does
  **not** apply. **VERIFIED (logic is mine; the two underlying facts are verified).**
- **VERIFIED (TN3179 + `launchd.plist(5)`):** a process running as **root** is automatically allowed local network access, and any
  **launchd daemon** is too. Whether the *application firewall* also treats root/launchd daemons specially is **not documented by Apple**.

### 2.3 Headless daemon under the firewall

- **VERIFIED:** the firewall prompt requires a user-visible alert ("an alert message appears"). A daemon running before any user has
  logged in has no GUI session in which to show it.
- **OPEN QUESTION:** what macOS does with a headless daemon that tries to bind a listening socket with the firewall on — silently drop
  packets, or queue a prompt for the next GUI session, or auto-allow because the daemon is root/launchd-owned. Apple's docs do not say.
  **This must be tested on a real Mac** before the macOS release; it is the single largest residual macOS risk.
- **ASSUMPTION (historical, not an architecture recommendation):** a **root `launchd` daemon** is far more likely to receive inbound traffic without a prompt than an
  unsigned per-user agent, because (a) it is root (root is exempted from the Local Network gate) and (b) many daemon-class processes are
  pre-seeded into the socket filter. **Do not rely on this or use it to select the service identity; OD-15 governs that choice.**

### 2.4 Mitigation (conditional recommendation, not a fact)

The earlier recommendation to ship the server as a `LaunchDaemon` is superseded by OD-15 until the service identity, no-elevation constraints and
selected-folder access are resolved. Whichever service model is approved, document the observed firewall recovery path and provide a diagnostic that
names firewall blocking as a possibility without claiming it is the cause. **ASSUMPTION:** users may need to add the signed service executable to
System Settings > Network > Firewall > Options; verify this on a real Mac before publishing the instruction.

---

## 3. macOS distribution & trust

### 3.1 The rules Apple actually enforces

| Fact | Label | Source |
|---|---|---|
| To notarize macOS software you must: code-sign **all** executables, use a **"Developer ID" application / kernel extension / system extension / installer certificate**, and **enable the Hardened Runtime** for app *and* command-line targets. | VERIFIED | Apple Developer, "Notarizing macOS software before distribution" |
| Hardened Runtime is **mandatory for notarization**: "To upload a macOS app to be notarized, you must enable the Hardened Runtime capability." | VERIFIED | Apple Developer, "Hardened Runtime" |
| Hardened Runtime blocks "less common capabilities, like just-in-time (JIT) compilation"; you must add specific entitlement exceptions if you need them. **Entitlements apply only to executables**; libraries/frameworks inherit the host's. | VERIFIED | Apple Developer, "Hardened Runtime" |
| The notary service scans for malicious content and code-signing issues, then issues a **ticket** that can be stapled to the software and is also published online for Gatekeeper to find. | VERIFIED | Apple Developer, "Notarizing macOS software before distribution" |
| Toolchain: `xcrun notarytool submit <file> --keychain-profile "<profile>" --wait`, then `xcrun stapler staple "<item>"`. **`altool` and Xcode ≤13 are no longer accepted (since 2023-11-01).** You cannot staple to a ZIP — staple each item inside it. | VERIFIED | Apple Developer, "Customizing the notarization workflow" |
| `notarytool` needs App Store Connect credentials; because 2FA is mandatory, you need an **app-specific password**. | VERIFIED | same |
| Gatekeeper (macOS Catalina+ default): downloaded software must be **signed by a registered developer** *and* **notarized**; Gatekeeper verifies the Developer ID signature, the notarization ticket, and that the software has not been altered; it also **requests user approval before opening downloaded software for the first time**. | VERIFIED | Apple Platform Security, "Gatekeeper and runtime protection in macOS"; Apple Support, "Safely open apps on your Mac" |
| Override path: System Settings > Privacy & Security > **Open Anyway** (then the prompt reappears and you click Open). The app is then saved as an exception. This is Apple's documented route for non-notarized / unidentified-developer software. | VERIFIED | Apple Support, "Safely open apps on your Mac" (support.apple.com/en-us/102445) |
| On Apple silicon, **arm64 code must carry a valid signature** (ad hoc is sufficient) or the kernel refuses to execute it. x86_64 code runs under Rosetta with no signature at all. | VERIFIED | Apple Platform Security guide, "Signed system volume security" → "Unsigned x86_64 code" |
| **Developer ID certificates** are for "Mac software that are downloaded from places other than the Mac App Store"; you can create up to five Developer ID Application and five Developer ID Installer certificates. | VERIFIED | Apple Developer Help, "Create Developer ID certificates" |
| "To build more advanced app capabilities and **distribute apps**, you'll need to be an **Apple Developer Program member** or an Apple Developer Enterprise Program member." "Mac software notarization" is listed as a membership-only capability in the comparison table. | VERIFIED | Apple Developer, "Compare memberships" |
| **"The Apple Developer Program is 99 USD per membership year."** Fee waivers exist for nonprofits, accredited educational institutions, and government entities that meet requirements. | VERIFIED | Apple Developer, "Enrolling in the Apple Developer Program" |
| Ad-hoc-signed code ("Sign to Run Locally") has a designated requirement tied to *that specific build*; unsigned code has no DR at all. macOS therefore **cannot reliably track the identity** of unsigned/ad-hoc code, which breaks privacy-prompt persistence and other subsystems. | VERIFIED | TN3127, "Inside Code Signing: Requirements" |
| Local network privacy tracks identity via the main executable's **UUID**; a missing or shared UUID makes it behave erratically. | VERIFIED (TN3179) |

### 3.2 The two realistic distribution options for a free, non-revenue volunteer project

#### Option A — "Unsigned / ad-hoc, install with `curl` or Homebrew" (zero cost, no Apple account)

- Ship a **single universal binary** (arm64 + x86_64) or two binaries; ad-hoc sign it (`codesign -s -`) so it runs on Apple silicon.
- Users install with `curl … | sh` or `brew install phairdlna`, or download and `chmod +x`.
- **Gatekeeper reality:** Gatekeeper's "first launch approval" and the notarization requirement apply to software that arrives via a channel that
  sets the **`com.apple.quarantine` extended attribute** — i.e. browsers, Mail, and other download managers. **VERIFIED** that the attribute
  exists and that `Open Anyway` is the Apple-sanctioned override; **ASSUMPTION / UNVERIFIED** that `curl` and Homebrew do not set it (widely
  reported in the community; not documented by Apple in the pages fetched here).
  → In practice, `curl | sh` installs are **not** Gatekeeper-blocked, and a raw binary dropped by `curl` + `chmod +x` is not either.
  → **But** a user who downloads via a **browser** *will* get the quarantine attribute and *will* hit Gatekeeper, with the only Apple-documented
  escape hatch being **System Settings > Privacy & Security > Open Anyway**. The folklore `xattr -d com.apple.quarantine <file>` also works
  but is **UNVERIFIED** in Apple's documentation.
- **Local Network prompt:** an ad-hoc/unsigned binary has an unreliable code identity → TN3179 warns local-network privacy "may behave
  weirdly". **VERIFIED.**
- **Firewall:** an ad-hoc signature carries no CA identity → not covered by "Automatically allow downloaded signed software" → the
  firewall prompt is likely to appear for the per-user agent. **ASSUMPTION** (extrapolated from two verified facts).
- **Cost:** $0. **Signing identity:** none. **User friction:** medium (a browser download needs Open Anyway; a `curl` install needs nothing).

#### Option B — "Developer ID + hardened runtime + notarization" (paid, best UX)

- Requires an Apple Developer Program membership (**99 USD/year**) to obtain a Developer ID Application certificate.
- Hardened Runtime is **mandatory** for notarization. This is a real engineering constraint: Hardened Runtime disables JIT and library
  validation by default, and disallows `DYLD_*` environment variables. For a Rust media server this is mostly fine — Rust does not JIT — but
  it means **no `DYLD_LIBRARY_PATH` shenanigans**, no unsigned dylibs, and any future Wasm/JIT plugin host would need a specific entitlement.
- `notarytool` + `stapler` in CI; requires an Apple-issued **app-specific password** or a Notary API key.
- Gatekeeper then has **no prompt** for the user: signature + notarization both verify. The firewall's "downloaded signed software"
  auto-allow also becomes available. **VERIFIED.**
- **Cost:** 99 USD/year + a maintainer willing to hold the account. **Friction:** near zero for the end user.

#### Option C (rejected, for completeness)

- Mac App Store: contradicts "free, ad-free, offline/LAN-only"? Not strictly — the App Store is free to the user — but it imposes sandboxing
  that is hostile to a media server reading arbitrary user folders, plus review friction and a paid account anyway. **REJECTED.**

### 3.3 Recommendation for a volunteer project

> **Ship Option A now; architect for Option B; do not gate the platform design on it.**
>
> 1. Build one binary that is **ad-hoc signed at build time** and works on both arm64 and x86_64 (this is required, not optional, on Apple silicon).
> 2. Make the **install path a `curl | sh` script** (or Homebrew formula) that installs into `~/Applications/` or `/usr/local/bin`, and print a
>    short "if discovery doesn't work, check Firewall" note.
> 3. Structure the release so that **the same binary** can later be Developer-ID-signed, hardened-runtime-enabled and notarized **without code
>    changes** — i.e. no runtime-execution of downloaded dylibs, no JIT, no `DYLD_*` reliance. This is cheap now and expensive later.
> 4. Have the project owner decide whether to buy the membership (99 USD/yr) once the project has traction. The account is a **person's** legal
>    identity, which is a governance question for an AGPL volunteer project, not just a money question.
> 5. **Never** present "unsigned binary that the user must `chmod +x`" as a supported install path for browser downloads without an
>    Open-Anyway walkthrough. Include the exact UI path in the README.

**OPEN QUESTION:** whether a raw, non-bundle CLI binary downloaded via a browser is blocked by Gatekeeper in current macOS, and whether the
"Open Anyway" affordance applies to non-bundle executables or only to app bundles. Apple documents the affordance for "apps". **Test on
macOS 15, 26 and 27.**

---

## 4. macOS service model

### 4.1 LaunchAgent vs LaunchDaemon

| Fact | Label | Source |
|---|---|---|
| `launchd` is the "**System wide and per-user daemon/agent manager**". | VERIFIED | `launchd(8)` man page |
| Job locations: `~/Library/LaunchAgents` (per-user agents provided by the user), `/Library/LaunchAgents` (per-user agents provided by the administrator), `/Library/LaunchDaemons` (**system-wide daemons** provided by the administrator), `/System/Library/LaunchAgents`, `/System/Library/LaunchDaemons`. | VERIFIED | `launchd.plist(5)` man page, FILES section |
| `KeepAlive` (boolean or dictionary): `true` = unconditionally keep the job alive. "**The use of this key implicitly implies `RunAtLoad`**, causing launchd to speculatively launch the job." "Jobs that exit quickly and frequently when configured to be kept alive will be throttled to conserve system resources." | VERIFIED | `launchd.plist(5)` |
| `KeepAlive` sub-dicts include `SuccessfulExit`, `PathState` (keep alive while a path exists), `NetworkState` ("no longer implemented"), `AfterInitialDemand`, and calendar conditions. | VERIFIED | same |
| `StandardOutPath` / `StandardErrorPath`: "the given path should be mapped to the job's stdout(4)/stderr(4)"; the file is **created** if it does not exist, with ownership reflecting `UserName`/`GroupName` and permissions from `Umask`. | VERIFIED | same |
| `AssociatedBundleIdentifiers` (string or array): "indicates which bundles are associated with this job in the **System Settings Login Items UI**. If an app installs a legacy plist the plist should include this key with a value of the app's bundle identifier." | VERIFIED | same |
| **CAVEAT in `launchd.plist(5)`:** "Daemons and agents managed by launchd are **subject to macOS user privacy protections**. Specifying privacy sensitive files and folders in a launchd plist may not have the desired effect, and **may prevent the job from running**." | VERIFIED | same |
| `launchd(8)`: "the canonical way to launch a daemon is through launchd as opposed to traditional POSIX and POSIX-like mechanisms". | VERIFIED | `launchd(8)` |

### 4.2 Which fits a media server?

**Earlier hypothesis — not an approved architecture.** The following process split was
proposed before checking the selected-folder permission boundary. It is superseded by
OD-15: no unit or identity is selected here, and the daemon must not be assumed to run
as root or to inherit folder access from the UI.

| Unit | Runs as | When | Purpose |
|---|---|---|---|
| System-domain `LaunchDaemon` | Effective identity depends on its launchd configuration; root must not be assumed | Can be available outside a user's login session | Candidate for always-on service. Must satisfy OPS-015/PLT-018 and verify Local Network behavior for the actual identity plus selected-folder scope and revocation (OD-15). |
| Per-user `LaunchAgent` | Logged-in user's identity | User session only | Candidate that naturally uses user file permissions; stops at logout and may require Local Network approval. Requires owner-approved PLT-013 change and runtime verification (OD-15). |

The baseline requirements pull in different directions: PLT-013 requires a system-context daemon; OPS-015 and PLT-018 prohibit elevated privilege
to serve the user's own directories. Apple documents launchd identity configuration and different local-network behavior, but the supported
combination for this product is not established. OD-15 therefore requires a real-Mac experiment and owner resolution before service controls are built.
Neither a folder picker nor a scoped bookmark is evidence that a separate service process has least-privilege access.

**`KeepAlive` semantics (VERIFIED, `launchd.plist(5)`):** use `KeepAlive = true` for the daemon (always-on), and **guard against the
throttling problem** — "Jobs that exit quickly and frequently when configured to be kept alive will be throttled." So the daemon must not
crash-loop. Use `SuccessfulExit` / `PathState` if you need smarter semantics (e.g. `PathState` on a "pause" flag file).

**Logging (VERIFIED, `launchd.plist(5)`):** set `StandardOutPath` and `StandardErrorPath` to files under e.g.
`/var/log/phairdlna/` (daemon) or `~/Library/Logs/PhairDLNA/` (agent). Note that launchd **truncates nothing** — it appends, and there is
**no built-in rotation**. → **ASSUMPTION:** implement size-capped rotation in the daemon itself (or ship a `newsyslog`/logrotate config for
the Linux side and a small self-rotation loop for macOS).

**Also note (VERIFIED):** `launchd.plist(5)`'s CAVEAT — privacy-protected paths in a launchd plist "may prevent the job from running". Do not
point a service at protected folders in the plist and infer that the user has authorized access. The process identity and the actual consent path
must be verified; see §5.4 and OD-15.

### 4.3 SMAppService — the modern alternative and its restrictions

| Fact | Label | Source |
|---|---|---|
| `SMAppService` is available on **macOS 13.0+**. | VERIFIED | Apple Developer, `SMAppService` |
| "An object the framework uses to control **helper executables that live inside an app's main bundle**." | VERIFIED | same |
| API surface: `SMAppService.agent(plistName:)`, `.daemon(plistName:)`, `.loginItem(identifier:)`, `.mainApp`; `register() throws` / `unregister() throws`; `status: SMAppService.Status`; `openSystemSettingsLoginItems()`. | VERIFIED | same |
| For `LaunchAgents`, `register()`/`unregister()` are "a replacement for installing property lists in `~/Library/LaunchAgents` or `/Library/LaunchAgents`"; for `LaunchDaemons`, a replacement for installing property lists in `/Library/LaunchDaemons`. | VERIFIED | same |
| "Registers the service so it can begin launching **subject to user approval**." | VERIFIED | same |
| **Restriction that matters:** the helper must be **inside an app's main bundle**. | VERIFIED | same |
| A bundled `LaunchDaemon` does not bootstrap until an administrator approves it in System Settings. `register()` is therefore not equivalent to an already-running daemon. | VERIFIED | Apple Developer, *Updating helper executables from earlier versions of macOS* |
| `SMAppService.Status` has `notRegistered`, `enabled`, `requiresApproval` and `notFound`. | VERIFIED | Apple Developer, *SMAppService.Status* |

**Consequences (ASSUMPTION, drawn from the above):**
1. A **pure-CLI, no-GUI install** (the `curl | sh` model) **cannot** use `SMAppService` — there is no bundle. It must write a plist into
   `~/Library/LaunchAgents` (or `/Library/LaunchDaemons`) directly and use `launchctl bootstrap`/`load`. That is the classic path and still
   works; it is just not the "blessed" modern one, and it loses the Login Items UI integration.
2. Once the **macOS app exists** (it is an `.app` bundle), it can register a bundled server executable through
   `SMAppService.daemon(plistName:)`. The package must carry the helper and its `Contents/Library/LaunchDaemons` property list, and the UI must
   explain the required administrator approval and link to System Settings when the status is `requiresApproval`.
3. Because `SMAppService` helpers live in the bundle, the daemon binary then also inherits the **bundle's code signature**, which
   incidentally solves the TN3179 "responsible code" problem for Local Network privacy.

**OPEN QUESTION:** the Apple documentation establishes administrator approval in System Settings before a bundled `LaunchDaemon` starts, but does not
settle the exact interaction on the supported macOS versions or prove the runtime path for this project. A real-Mac run must also verify how the daemon
gets safe access to only the folders the user selected. No `SMAppService` registration or launch behavior has been observed in PhairDLNA.

---

## 5. macOS filesystem facts that affect a media server

### 5.1 Case sensitivity, case preservation, and normalization — the important part

| Fact | Label | Source |
|---|---|---|
| **APFS is the default file system in macOS** (replaced HFS+ as default in macOS High Sierra and later). | VERIFIED | Apple, File System Programming Guide ("File System Basics") |
| "**APFS accepts only valid UTF-8 encoded filenames for creation, and preserves both case and normalization of the filename on disk in all variants.**" | VERIFIED | Apple File System Guide, FAQ, "How does Apple File System handle filenames?" |
| "APFS, like HFS+, is case-sensitive on iOS and is available in **case-sensitive and case-insensitive variants on macOS, with case-insensitive being the default**." | VERIFIED | same |
| "**In macOS High Sierra, APFS is normalization-insensitive in both the case-insensitive and case-sensitive variants, using a hash-based native normalization scheme.** … Being normalization-insensitive ensures that **normalization variants of a filename cannot be created in the same directory**, and that **a filename can be found with any of its normalization variants**. This means that **you don't need to do any additional work to ensure correct normalization behavior** in these versions of macOS and iOS." | VERIFIED | same |
| "APFS **preserves the normalization** of the filename and uses **hashes of the normalized form** to provide normalization insensitivity, whereas **HFS+ stores the normalized form of the filename on disk** to provide normalization insensitivity." | VERIFIED | same |
| "APFS implements normalization and case insensitivity according to the **Unicode 9.0** standard … HFS+, which is based on **Unicode 3.2**." | VERIFIED | same |
| "**Calling `readdir(2)` on a directory in APFS returns filenames in hash order**, whereas HFS+ returns filenames in lexicographical order." | VERIFIED | same |
| "**HFS Plus stores strings fully decomposed and in canonical order**" and "**HFS Plus compares strings in a case-insensitive fashion**" (HFSX is the case-sensitive variant). | VERIFIED | TN1150, "HFS Plus Volume Format" |
| "**HFS Plus disks store file names as UTF-16 in an Apple-modified form of Normalization Form D (decomposed).**" | VERIFIED | TN2078, "Migrating to FSRefs & long Unicode names from FSSpecs" |
| "the **lowest levels of Mac OS X** — which includes both the POSIX-level API and the VFS plug-in KPI — treat file names as **UTF-8 encoded Unicode strings**. … **HFS Plus uses decomposed Unicode whereas UDF and SMB use precomposed Unicode.** … When returning names to higher layers (for example, from your `readdir` vnode operation), you should return **decomposed** names." | VERIFIED | QA1173, "Text Encodings in VFS" |
| "Apple's old HFS+ file system stores file names in a variant of NFD (**the newer APFS doesn't enforce that anymore**), and the **macOS Finder still decomposes file names**." | VERIFIED | Unicode FAQ, "Normalization" |
| "**All user-level comparison should behave as if it normalizes the input to NFC.**" | VERIFIED | Unicode FAQ, "Normalization" |

#### What this actually means for PhairDLNA — the trap, stated precisely

> The brief's framing ("Unicode NFD normalization on macOS vs NFC on Linux") is **a common oversimplification and is not what APFS does.**
> The real, load-bearing facts are:
>
> 1. **APFS is normalization-insensitive.** If a directory contains `café` in NFC, then `café` in NFD is *the same file*. You cannot have
>    both in one directory. **VERIFIED.**
> 2. **APFS preserves the on-disk normalization.** Whichever form was written is what `readdir` gives you back. A file created over SMB
>    (which is NFC) and read on macOS comes back in **NFC**, not NFD. A file created by an HFS+-era tool may come back in **NFD**.
>    **VERIFIED.**
> 3. **HFS+ volumes still exist** (external disks, some NAS shares, older images) and there the on-disk name *is* NFD and the kernel
>    **converts** incoming NFC names to NFD. **VERIFIED.**
> 4. **Linux filesystems are byte-preserving.** On ext4/XFS/btrfs, `café`-NFC and `café`-NFD are **two different files with two different
>    inodes**. **ASSUMPTION** (the byte-preserving behaviour of Linux filesystems is standard knowledge; the nearest cited primary source,
>    QA1173, states the contrast explicitly: "Your target volume format should define whether it uses precomposed or decomposed Unicode …
>    HFS Plus uses decomposed Unicode whereas UDF and SMB use precomposed Unicode").
> 5. **The Finder displays decomposed names** on macOS. **VERIFIED** (Unicode FAQ).
>
> **Therefore the failure mode is not "same file, two names" on macOS. It is:**
> - **On macOS:** `readdir` returns one of NFD/NFC for a given file, and which one is *not* under your control and *can change* between
>   directories, volumes and even across a macOS upgrade (HFS+ → APFS migration). A stored path string can therefore **stop resolving**
>   even though the file never moved. If you cache absolute paths as text, you get intermittent "file not found".
> - **Across platforms:** the *same logical media title* can be NFC on Linux and NFD on macOS. If you sync an index between them, or dedupe
>   by raw path string, you get **duplicate entries** for one file. Conversely, deduping by raw bytes on Linux produces **false duplicates**
>   when comparing against a macOS-sourced path.
> - **Case:** macOS's default APFS volume is case-**insensitive**, so `Movie.MKV` and `movie.mkv` are the same file there; Linux is
>   case-sensitive, so they are two files. A case-sensitive *index key* built on macOS will collapse two distinct Linux files.
>
> **The rule PhairDLNA must adopt (PROPOSAL, this is the ADR input):**
> - **Keep the raw path** exactly as the OS reported it, byte-for-byte, for all I/O. Never re-encode it.
> - **Compute a separate `identity` string** for indexing/dedup/matching:
>   `identity = NFC(casefold(path_component))` applied **per component**, with the **volume id** prefixed so it is stable across mounts.
>   Use Unicode **NFC** (per the Unicode FAQ's own recommendation that "all user-level comparison should behave as if it normalizes the
>   input to NFC"), and use a **case-folding** step only for *matching*, never for *display*.
> - **Display name** = the raw basename from `readdir`, with NFC applied **for display only** (the Finder shows decomposed; pick one and be
>   consistent). Keep the raw bytes in the DB too.
> - **Never** use the raw path as a database primary key, and never use `filename == filename` comparisons for dedup.
> - **Always** compare paths with a **platform-aware** matcher: on macOS default volumes, `A.mkv` and `a.mkv` are the same file; on Linux they
>   are not. Query the filesystem rather than assuming.

### 5.2 Removable / external volumes

| Fact | Label | Source |
|---|---|---|
| `NSWorkspace` exposes mount/unmount/rename notifications: `didMountNotification`, `willUnmountNotification`, `didUnmountNotification`, `didRenameVolumeNotification`, plus `localizedVolumeNameUserInfoKey` (user-visible name) and `volumeURLUserInfoKey` (mount path), with `oldVolumeURLUserInfoKey` for renames. | VERIFIED | Apple Developer, `NSWorkspace` |
| `NSWorkspace.getFileSystemInfo(forPath:isRemovable:isWritable:isUnmountable:description:type:)` "Returns information about the file system at the specified path." | VERIFIED | same |
| `NSWorkspace.unmountAndEjectDevice(at:)` / `unmountAndEjectDevice(atPath:)` exist to eject volumes. | VERIFIED | same |
| The **DiskArbitration** framework "Provides mechanisms to **register and block disk mount or unmount events**" — `DADisk.h`, `DASession.h`, `DADissenter.h`. It is the lower-level, more precise API. | VERIFIED | Apple Developer, DiskArbitration framework |
| "**/Volumes** … contains a subdirectory for each mounted disk in the local file system **from the command line**. (The Finder provides a different user interface for accessing local disks.)" | VERIFIED | Apple, File System Programming Guide ("File System Basics") |
| macOS mounts removable media under `/Volumes/<Volume Name>`. | VERIFIED (via `/Volumes` per-volume subdirectory, above) |
| TCC's "Files and folders" category explicitly **includes "network volumes and removable volumes"**. | VERIFIED | Apple Platform Security, "Controlling app access to files" |

**Practical note (ASSUMPTION):** `NSWorkspace` notifications are delivered on the **main thread of the process that owns the `NSWorkspace`**, and
the workspace notification centre is AppKit. A **headless** daemon with no AppKit run loop should prefer **DiskArbitration** (`DASessionCreate` +
`DARegisterCallback`, which is callback-based and does not need an AppKit run loop) or simply **poll `/Volumes` + `statfs`/`getfsstat`
device ids** on a short interval. **UNVERIFIED** which of the two is correct in practice — needs a device test.

### 5.3 Network mounts (SMB / NFS)

| Fact | Label | Source |
|---|---|---|
| SMB stores filenames in **precomposed (NFC)** Unicode; UDF likewise. | VERIFIED | QA1173 |
| macOS network volumes are in the TCC "Files and folders" protected class. | VERIFIED | Apple Platform Security, "Controlling app access to files" |
| APFS-formatted volumes **can** be shared over SMB or NFS (but not AFP, which is deprecated). | VERIFIED | Apple File System Guide FAQ |
| `NSWorkspace` "can be used to: Open, manipulate, and get information about files **and devices**"; it also has a `urlForApplication(toOpen:)` style API surface for non-file URLs. | VERIFIED | Apple Developer, `NSWorkspace` |

**Practical note (ASSUMPTION):** on macOS, a network volume's mount point is `/Volumes/<name>` and the *server* portion appears in
`LOCATION`-style URLs as `smb://server/share/...` or `nfs://...`, **not** as a POSIX path. Any DLNA `Location` header that embeds a
filesystem path must therefore be re-mapped to an `http://` URL served by PhairDLNA itself (which is the normal DLNA design anyway) — but
the *index* must still be able to read from the mount, and the daemon may need TCC consent for it.

### 5.4 Full Disk Access and TCC

| Fact | Label | Source |
|---|---|---|
| The macOS protected-resource categories relevant to a media server are **"Files and folders"** (explicitly including Desktop, Documents, Downloads, **network volumes and removable volumes**) and **"Full internal storage access"** (Full Disk Access). | VERIFIED | Apple Platform Security, "Controlling app access to files" |
| For "Files and folders", the system can **prompt the user via the app**; for "Full internal storage access", the user **must edit system privacy settings** (no app prompt). | VERIFIED | same table |
| In macOS 13+, these settings live in **System Settings > Privacy & Security > Privacy**. | VERIFIED | same |
| macOS 10.15+ always draws `NSOpenPanel` in a **separate process**, sandboxed or not; "When the user chooses a file to open, **macOS adds that file to the app's sandbox**." | VERIFIED | Apple Developer, `NSOpenPanel` |
| `launchd` jobs "are subject to macOS user privacy protections. Specifying privacy sensitive files and folders in a launchd plist **may prevent the job from running**." | VERIFIED | `launchd.plist(5)` CAVEATS |
| Security-scoped bookmarks and URLs are documented as an **App Sandbox** access mechanism; resolving one yields a scoped URL, and `startAccessingSecurityScopedResource` grants access to the calling process. | VERIFIED | Apple Developer, `NSURL`, `CFURLStartAccessingSecurityScopedResource` |
| App-scoped bookmarks do not grant access to another sandboxed app; Apple says the receiving sandboxed app must have the same code-signing identity. Apple does not document a bookmark created by the unsandboxed PhairDLNA UI as a permission grant for a separate service process. | VERIFIED for the documented sandbox restriction; OPEN for PhairDLNA's service case | Apple Developer, `CFURLCreateBookmarkData` |

**Consequence:** an `NSOpenPanel` selection and serialized bookmark are not evidence that a separate service process can read only the selected folder. A
path list is configuration, not an authorization grant. Do not claim the folder-consent flow complete or ship the root-daemon path until a real-Mac
experiment shows that a selected protected folder is readable by the intended server process, an unselected sibling remains unreadable, and revoking
access takes effect. If this cannot be achieved, the owner must revisit the process model in OD-15.
The repeatable evidence record is specified in `qa/macos-service-access-protocol.md`; the protocol itself is not test evidence.

### 5.5 Files with no extension

- **VERIFIED** that macOS/UTI is content-type based, not extension based: Apple's File System Programming Guide links "Understanding Uniform
  Type Identifiers" as the mechanism for identifying files by content, and Apple's File System Programming Guide's "Working with File Types"
  discusses UTI-based type information.
- **VERIFIED:** the APFS FAQ notes APFS "accepts only valid UTF-8 encoded filenames" — nothing about extensions.
- **ASSUMPTION:** media files on macOS frequently have **no extension at all** (e.g. downloaded M4V, camera-originated `.mov` renamed,
  or a file whose extension was stripped by an upload). A DLNA server that identifies media purely by extension will misclassify these.
  → **PhairDLNA must sniff content** (e.g. `ffprobe`/`matroska`/`ISO-BMFF` probing) and use the extension only as a hint. This is a
  cross-platform requirement, not a macOS quirk, but macOS makes it visible earlier because Apple tooling and Finder hide extensions by default.

### 5.6 Resource forks, `._` AppleDouble files, hidden files

| Fact | Label | Source |
|---|---|---|
| macOS stores **extended attributes** separate from filesystem attributes; `xattr` lists/reads/writes/removes them; `ls -l@` lists attribute names; `com.apple.FinderInfo` is a real Apple-supplied example attribute. | VERIFIED | `xattr(1)` man page |
| The `UF_HIDDEN` flag (0x8000) is a filesystem attribute meaning "Hint that the file should be hidden in the GUI." | VERIFIED | Apple, File System Programming Guide ("File System Details") |
| macOS hides `~/Library` and `/Volumes` from the Finder (both remain visible on the command line). | VERIFIED | Apple, File System Programming Guide ("File System Basics") |

- **ASSUMPTION:** resource forks are stored as the `com.apple.ResourceFork` extended attribute on APFS, and `._<name>` files are
  **AppleDouble** containers produced when a file with such metadata is written to a filesystem that cannot hold xattrs (FAT/exFAT, some
  SMB servers, tar archives). The Apple tech note that documented AppleDouble ("Compressing Mac Files For Transport On Other Platforms")
  is **retired** and its URL 404s, so I could not cite it. → **Treat `._*` files as noise to ignore.**
- **ASSUMPTION (recommended):** the indexer must **skip** files whose basename begins with `._` (AppleDouble), and must **not** treat
  xattrs as content. Media content is in the data fork / regular bytes; the resource fork is metadata (icons, edit state).
- **VERIFIED** (the underlying mechanism): `UF_HIDDEN` exists as a flag; and `stat(1)` on macOS reports `st_flags`, so the indexer can
  filter on it if it wants to exclude hidden items. **VERIFIED** via `stat(1)` man page (it lists `st_flags` among the reported fields).

### 5.7 Sparse files and hardlinks

| Fact | Label | Source |
|---|---|---|
| `stat(1)` on macOS reports `st_nlink` ("Number of hard links to file"), `st_blocks` ("Number of blocks allocated for file"), `st_size`, `st_blksize` and `st_flags`. | VERIFIED | `stat(1)` man page |
| `stat(1)` is implemented by calling `lstat(2)`. | VERIFIED | same |

- **ASSUMPTION:** APFS supports sparse files — a file whose `st_blocks * 512 < st_size` is sparse and whose apparent size overstates its
  real allocation. For a media server this matters for **reported size vs real disk usage** and for **serving over HTTP** (a sparse file
  read normally yields zeros, so it is correct but wasteful to transfer). Use `st_blocks` for a "real size" metric and `SEEK_HOLE`/`SEEK_DATA`
  for sparse-aware range serving. **UNVERIFIED** whether the `SEEK_HOLE`/`SEEK_DATA` pair is fully supported on APFS — needs a device test.
- **VERIFIED (mechanism):** `st_nlink > 1` indicates additional hard links to the same inode. **ASSUMPTION:** on APFS a hard-linked media file
  appears as **two directory entries** in different directories but is **one** file; the Finder shows it once. → The index must **dedupe by
  `(st_dev, st_ino)`** within a volume, not by path, or it will serve the same file twice and double-count storage. This is a
  cross-platform requirement that the macOS `st_dev`/`st_ino` pair makes explicit.

---

## 6. Cross-platform file abstraction requirements (ADR input)

**This section is a PROPOSAL / analysis, not a set of verified platform facts.** It is derived from §5 and from the Linux/Windows facts in §8–§9.

### 6.1 The nine things the core needs from "a filesystem"

| # | Concept | Why it must be abstracted | macOS shape | Linux shape | Windows shape |
|---|---|---|---|---|---|
| 1 | **Raw path** (bytes as the OS reported them) | Never re-encode; used verbatim for I/O | UTF-8, may be NFD or NFC | UTF-8, byte-preserving | UTF-16 natively; Rust `OsStr` is WTF-8 |
| 2 | **Display name** | Human-facing; must be a *presentation* concern, never a key | `readdir` name, Finder shows decomposed | `readdir` name | `readdir` name |
| 3 | **Identity key** (normalized, case-folded, per component) | Dedup, cross-platform matching, DB primary key | NFC + casefold per component | NFC + casefold per component (casefold is *lossy* on case-sensitive volumes!) | NFC + casefold per component (lossy on NTFS) |
| 4 | **Case-sensitivity of the containing volume** | Determines whether two paths are the same file | Default APFS = insensitive; case-sensitive APFS exists; HFSX = sensitive | Almost always sensitive; `casefold` may be wrong | NTFS = insensitive by default; can be per-directory sensitive |
| 5 | **Unicode-normalization policy of the volume** | Determines whether two names are the same file | APFS = normalization-insensitive but preserves form; HFS+ = stores NFD; SMB = NFC | Byte-preserving | NTFS is case-insensitive and **does not** normalize (NFC-ish passthrough) |
| 6 | **Volume / mount identity** | Volume detection, "is this the same disk?", re-index triggers | `NSWorkspace` mount notifications; `getfsstat`/`statfs` `f_fsid`; DiskArbitration | `/proc/self/mountinfo`; `statfs`; udev | `GetVolumeInformationByHandleW` (volume serial); `GetLogicalDriveStrings` |
| 7 | **Device identity** | Change detection, inode-level identity | `st_dev` + `st_ino` | `st_dev` + `st_ino` | `GetFileInformationByHandle` (volume serial + file index) |
| 8 | **Link count / aliasing** | Hardlinks, symlinks, aliases: one file or many? | `st_nlink`, symlinks, Finder aliases | `st_nlink`, symlinks | `st_nlink` (hard links); symlinks need Developer Mode |
| 9 | **Error taxonomy** | "Not found" vs "not permitted" vs "gone (volume unmounted)" vs "I/O error" — must drive retry vs skip vs re-prompt | POSIX errno + TCC `EPERM` | POSIX errno | `GetLastError()` → `ERROR_ACCESS_DENIED` vs `ERROR_PATH_NOT_FOUND` vs `ERROR_NOT_READY` |

### 6.2 The abstraction, concretely

**PROPOSAL (for the ADR):**

```
MediaPath
├── raw: OsString              // verbatim from the OS. Used for ALL I/O. Never normalised.
├── identity: String           // NFC, casefolded, per-component, joined by '/'. DB key + dedup key.
├── volume: VolumeId           // stable id of the containing volume (see below)
└── display: String            // NFC of raw basename. Presentation only.
```

**Rules the core must obey (PROPOSAL):**

1. **Never compare or key on `raw`.** Use `identity`.
2. **Never use `raw` as a database primary key.** Use `identity` scoped by `VolumeId`.
3. **Round-trip `raw` verbatim** to the filesystem on every open/stat/read. If you re-normalise a path before using it, you can break
   access to a file whose stored name is in the other form (this bites on HFS+ volumes, where an NFC path is *converted* to NFD by the
   kernel — so it usually works — but it also breaks on volumes where the raw form is *significant*).
4. **Identity must be volume-scoped**, because the same identity on two different volumes is two different files.
5. **Case folding is lossy on case-sensitive volumes.** Therefore `identity` is a *matching* key, not a *uniqueness* key. Uniqueness must be
   `(VolumeId, device_id)` — i.e. the file system object — not the path.
6. **Symlink policy must be an explicit, configurable core setting**, not a platform default:
   - `Follow` (default for library roots) — resolve to the target, and index the target's identity.
   - `IndexAsLink` — record the link as its own entry, pointing at the target.
   - `Skip` — ignore symlinks entirely.
   This differs per platform for a reason: on macOS, `NSWorkspace`/Finder aliases are a Finder-level concept distinct from POSIX symlinks;
   on Windows, creating a symlink requires **Developer Mode or elevation**
   (VERIFIED, `CreateSymbolicLinkW`: `SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE` "will function" only "if Developer Mode must first be
   enabled on the machine"), so symlink-heavy libraries behave differently under a service account.
7. **Permission errors are not fatal.** A `EPERM`/`ERROR_ACCESS_DENIED` on a subtree should mark that subtree **quarantined** (needs
   consent) and continue, not abort the scan.
8. **Volume events are first-class.** The core needs a `VolumeEvent` stream (`Mounted` / `Unmounted` / `Remounted` / `Renamed`) so it can
   subscribe/unsubscribe watchers and re-key the index.
9. **A "device id" for change detection** should be the pair `(volume_id, file_id)` where `file_id` is the platform's stable file handle
   identity: `(st_dev, st_ino)` on Unix, `(volume serial, file index)` on Windows. Change detection then works even when the path changes.

---

## 7. Rust implementation options for the macOS bits

All metadata below is from the crates.io API (`https://crates.io/api/v1/crates/<name>`) and the crates' `Cargo.toml` on docs.rs, both fetched.

| Crate | Latest | License | Last release | Notes |
|---|---|---|---|---|
| `objc2` | 0.6.5 | MIT | 2026-10-05 | Modern ObjC runtime bindings. Platforms include `aarch64-apple-darwin` and `x86_64-apple-darwin`. Rust 2021. |
| `objc2-app-kit` | 0.3.2 | MIT OR Apache-2.0 OR Zlib | 2025-10-04 | AppKit bindings (`NSStatusItem`, `NSApplication`, `NSWorkspace`, `NSOpenPanel`, `NSStatusBar`). `rust-version = 1.71`. Explicitly declares **both** `aarch64-apple-darwin` and `x86_64-apple-darwin`. |
| `objc2-foundation` | 0.3.2 | MIT | 2025-10-04 | Foundation bindings. |
| `cocoa` | 0.27.0 | MIT OR Apache-2.0 | 2026-09-07 | **Legacy.** Depends on the old `objc` crate (0.2.3) + `block` 0.1 + `core-foundation` 0.10. Only **16.3 % of the crate is documented**. crates.io `platform` metadata lists **only `x86_64-apple-darwin`**; its source contains **no arch-specific `cfg`s**, so an aarch64 build is expected to work but is **UNVERIFIED**. |
| `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | 2025-05-26 | Safe-ish CF wrappers. |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | 2024-08-11 | Raw bindings. Last release 2024-08-11 — **slowly maintained**. |
| `block2` | 0.6.2 | MIT | 2025-10-04 | Rust `block` ABI. Depends on `objc2 >=0.6.2, <0.8.0`. 100 % documented. |
| `dispatch` | 0.2.0 | MIT | **2020-01-19** | GCD wrapper. **Effectively unmaintained** (edition 2018, no `rust-version`). crates.io `platform` metadata lists only `x86_64-unknown-linux-gnu`, which is plainly wrong for a GCD crate. |
| `notify` | 8.2.2 | CC0-1.0 | 2026-11-09 | Cross-platform FS watcher. macOS backend = **`macos_fsevent` (default)** using `objc2-core-foundation` + `objc2-core-services`; optional `macos_kqueue`. `rust-version = 1.88`. Docs warn: "Due to the inner security model of FSEvents, some events cannot be observed easily when trying to follow files that do not belong to you. In this case, reverting to the pollwatcher can fix the issue." |
| `muda` | 0.21.2 | Apache-2.0 OR MIT | 2026-10-08 | Cross-platform menus. macOS backend deps: `objc2 ^0.6.1`, **`objc2-app-kit ^0.3.0`**, `objc2-core-foundation`, `objc2-foundation`, `png`. `rust-version = 1.90`. Docs: "On macOS, menus can only be used from the **main thread**, and most functionality will panic if you try to use it from any other thread." |
| `tray-icon` | 0.26.1 | MIT OR Apache-2.0 | 2026-10-07 | Tray icons; depends on `muda ^0.21`. On macOS this is therefore an `NSStatusItem` wrapper. |
| `fsevent-sys` | 5.2.0 | MIT | 2025-11-17 | The older, lower-level FSEvents binding. Superseded by `notify`'s objc2-based backend for most purposes. |
| `unicode-normalization` | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | 2026-10-16 | The standard Rust NFC/NFD/NFKC implementation. Use this for the identity key. |

### 7.1 Comparison and recommendation

- **`objc2` + `objc2-app-kit` — RECOMMENDED for anything AppKit.** VERIFIED to support both Apple silicon and Intel explicitly. Actively
  maintained (Oct 2026 release). Gives `NSStatusItem`, `NSApplication`, `NSWorkspace` mount notifications, `NSOpenPanel`,
  `NSUserNotification`/`UNUserNotificationCenter`, all without Swift and without a webview.
  *Cost:* the `objc2` API is macro-heavy and the AppKit surface is enormous; expect to read the generated docs.
  *Recommendation:* use it **only** inside a thin `platform-macos` adapter crate, behind the traits in §10. Do not let `objc2` types leak
  into the core.
- **`cocoa` — do not adopt for new work.** VERIFIED: it is on the old `objc` crate, is 16.3 % documented, and its crates.io platform metadata
  omits Apple silicon. It is still maintained-ish (2026-09-07) and is fine for tiny legacy things, but `objc2` is strictly better and the
  two projects share maintainers.
- **`core-foundation-sys` / `core-foundation` — use when you need CF APIs that `objc2` does not wrap** (e.g. `CFRunLoop`, `CFArray`,
  `CFRunLoopSource` for DiskArbitration, `CFMutableDictionary` in Foundation collections). VERIFIED that `core-foundation-sys` is older
  (2024-08-11) while `core-foundation` is newer (2025-05-26) — prefer the safe wrapper.
- **`block2` — required if you use `objc2` callbacks** (DiskArbitration callbacks, `NSWorkspace` notification observers, `NSOpenPanel`
  completion handlers). VERIFIED: it depends on `objc2 >=0.6.2`, so it composes cleanly. **Recommendation: adopt.**
- **`dispatch` — avoid.** VERIFIED: last released 2020-01-19, edition 2018. `objc2` exposes GCD types (`dispatch_queue_t`,
  `DispatchQueue`) via `objc2`'s `dispatch` module, and `block2` covers the block ABI. Using the standalone `dispatch` crate would drag in a
  stale dependency. **Recommendation: do not adopt; use `objc2::dispatch` if GCD is needed.**
- **`notify` — RECOMMENDED for file watching.** VERIFIED: current (2026-11-09), and its macOS backend is built on `objc2` (not the old
  `fsevent-sys`), so it composes with the same ObjC world. VERIFIED caveat: FSEvents cannot see events for files you don't own → keep
  `PollWatcher` as a fallback. **Note the licence asymmetry: `notify` is CC0-1.0 while everything else is MIT/Apache-2.0** — fine for AGPL
  (CC0 is permissive), but worth recording.
- **`muda` + `tray-icon` — RECOMMENDED for the menu bar app.** VERIFIED that `muda`'s macOS backend is `objc2` + `objc2-app-kit`, so it is
  pure Rust with no Swift and no webview. `tray-icon` adds the icon + event loop on top.
  → **"Is a menu bar app in pure Rust without Swift realistic?" — YES, VERIFIED.** `muda` literally does `NSStatusItem` through
  `objc2-app-kit`. The *engineering* caveats, not language ones, are:
  1. It must be an **`.app` bundle** (with `Info.plist`, `LSUIElement = true` for a menu-bar-only app) — because `NSApplication` wants a
     bundle, and because `SMAppService` helpers must live in the app's main bundle.
  2. **Main thread only** — VERIFIED by `muda`'s own docs. The menu bar app must own the main thread and pump the AppKit run loop.
  3. It will need to be **signed** (and eventually notarized) to avoid Gatekeeper friction; the *same* binary can be ad-hoc signed for now.
  4. `muda`'s MSRV is **1.90** — high; `notify` is **1.88**; `objc2-app-kit` is **1.71**. If you want a lower MSRV, pin `muda` back or
     call `objc2-app-kit` directly for `NSStatusItem` (a small amount of code).
  5. **Direct `objc2-app-kit` is a legitimate alternative to `muda`** for a *minimal* menu bar (status item + a few menu items + a click
     handler): `NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)` + `NSMenu`. That is maybe 100–200 lines and avoids
     `muda`'s MSRV and its cross-platform abstraction you don't need. **ASSUMPTION** (based on the AppKit API surface) — the exact line count
     is an estimate.
- **Menu bar app without a webview** — VERIFIED possible: `muda`/`tray-icon`/`objc2-app-kit` are all native AppKit. There is **no** WKWebView
  involved and none is needed.
- **`libc` / `darwin-c`** — `libc` (the widely used crate) covers the POSIX surface (`getifaddrs`, `socket`, `setsockopt`,
  `IP_ADD_MEMBERSHIP`, `statfs`, `stat`, `getfsstat`, `fcntl`) on macOS just fine and is already a de-facto dependency of the Rust
  ecosystem. `darwin-c` is the low-level generated binding crate; prefer `libc` unless you need a macOS-specific symbol `libc` lacks
  (`SIOCGIFFUNCTIONALTYPE`, `getfsstat`, `setns/getns`, `DiskArbitration`'s `DADiskCallBack` are the candidates). **ASSUMPTION** — I did not
  verify `libc`'s coverage of `SIOCGIFFUNCTIONALTYPE`/`getfsstat` for this brief.

### 7.3 Minimal macOS adapter dependency set (PROPOSAL)

```
platform-macos
├── objc2            (MIT)        — runtime, message send, blocks glue
├── objc2-app-kit    (MIT/Apache) — NSStatusItem, NSWorkspace, NSOpenPanel
├── objc2-foundation (MIT)        — NSURL, NSUUID, notification centre
├── block2           (MIT)        — block ABI
├── core-foundation  (MIT/Apache) — CFArray/CFRunLoop/DiskArbitration glue
├── libc             (MIT/Apache) — getifaddrs, sockets, stat, statfs
└── notify           (CC0-1.0)    — FSEvents watching
```
Everything Apple-specific stays in this crate; the core depends only on the traits in §10.

---

## 8. Linux equivalent

### 8.1 systemd: user service vs system service

| Fact | Label | Source |
|---|---|---|
| `systemctl --user` = "Talk to the service manager of the calling user, rather than the service manager of the system." `--system` is the implied default. | VERIFIED | `systemctl(1)` |
| Per-user unit files live under the user's home directory; the `--user` mode has analogous directory restrictions to system mode. | VERIFIED (partially) | `systemctl(1)` |
| Some service options are "only available for system services and **not supported for services running in per-user instances of the service manager**". | VERIFIED | `systemd.service(5)` |
| `StandardOutput=` accepts `inherit, null, tty, journal, kmsg, journal+console, kmsg+console, file:path, append:path, truncate:path, socket, fd:name`. Default is `journal`. `journal` makes stdout an **AF_UNIX stream socket** (so `echo "x" > /dev/stderr` does not work; use `>&2`). | VERIFIED | `systemd.exec(5)` |
| `file:path` opens the file at unit start **without truncating**; `append:path` appends; `truncate:path` truncates. The path is opened by the service manager *before* the executable is invoked. | VERIFIED | `systemd.exec(5)` |
| Units whose stdout/stderr is connected to the journal automatically gain `After=systemd-journald.socket`. | VERIFIED | `systemd.exec(5)` |
| `sd_notify(0, "READY=1")` tells the manager startup finished — **only used if the unit has `Type=notify` / `Type=notify-reload`**. `READY=0` is not defined. | VERIFIED | `sd_notify(3)` |
| `WATCHDOG=1` is the keep-alive ping for `WatchdogSec=`; `WATCHDOG=trigger` simulates a watchdog timeout. | VERIFIED | `sd_notify(3)` |
| `sd_notify` sends a datagram to `$NOTIFY_SOCKET`; the socket path may be an AF_UNIX path or a Linux abstract-namespace name (leading `@`). `unset_environment != 0` unsets `$NOTIFY_SOCKET`. | VERIFIED | `sd_notify(3)` |
| "**Mount points created at runtime (independently of unit files or /etc/fstab) will be monitored by systemd and appear like any other mount unit** in systemd." | VERIFIED | `systemd.mount(5)` |
| `systemd-udevd` is the udev device manager; it is the service that processes kernel uevents. | VERIFIED | `systemd-udevd.service(8)` |

**Recommendation (PROPOSAL):** **system service** (`/etc/systemd/system/phairdlna.service`), not a user service, for the server:
- The server must run with **no user logged in** (same argument as macOS).
- User services die with the session unless `loginctl enable-linger <user>` is used. **VERIFIED** that `loginctl` exists in the systemd
  toolset; the specific `enable-linger` semantics were **not fetched** → mark this sub-point **UNVERIFIED**, but the existence of lingering
  as a concept is standard systemd.
- A user service would also lose access to the user's media directories unless they are world/group-readable, and would be subject to the
  user's `systemd` runtime limits.
- **Offer** a *user* unit as an alternative for people who do not want root, and document `systemctl --user enable --now phairdlna`.
- Use `Type=notify` + `sd_notify("READY=1")` so `systemctl start` returns only when the HTTP/SSDP listeners are actually up. Add
  `WatchdogSec=` + `WATCHDOG=1` for liveness. Use `StandardOutput=journal` (default) and `journalctl -u phairdlna -f` for logs.

### 8.2 inotify limits

| Fact | Label | Source |
|---|---|---|
| `inotify` limits: `/proc/sys/fs/inotify/max_queued_events` (upper bound on queued events per instance; **excess events are dropped**, but an `IN_Q_OVERFLOW` event is always generated), `/proc/sys/fs/inotify/max_user_instances` (instances per real UID), `/proc/sys/fs/inotify/max_user_watches` (watches per real UID). | VERIFIED | `inotify(7)` |

**Consequence (ASSUMPTION):** a media server watching a large tree must (a) be prepared for `IN_Q_OVERFLOW` and re-scan, (b) know that
`max_user_watches` is **per real UID** — so a system service running as a dedicated user gets its own budget, and (c) not create one watch
per file if it can watch directories recursively. `notify`'s inotify backend uses one watch per directory.

### 8.3 udev / mount detection

- **VERIFIED:** runtime-created mount points are monitored by systemd and appear as mount units (`systemd.mount(5)`).
- **VERIFIED:** `systemd-udevd` is the device manager (udev), and `systemd.mount(5)` documents mount units being derived from
  `/proc/self/mountinfo`.
- **ASSUMPTION:** the most portable "is this a removable volume" signal on Linux is `/sys/block/<dev>/removable` plus the udev property
  `ID_BUS=usb` / `ID_MEDIA_*`; `udisks2` provides a higher-level D-Bus API if you want "drive with a label". I did not fetch udev's
  documentation for this brief → mark as **ASSUMPTION**.
- **VERIFIED:** `systemd.mount(5)` states network mount units automatically acquire `After=remote-fs-pre.target`, `network.target`,
  `Wants=network-online.target` — i.e. **network mounts are ordered after network-online**, which is the hook to use for "SMB share came back".

### 8.4 polkit for privileged operations

| Fact | Label | Source |
|---|---|---|
| polkit "provides an authorization API intended to be used by **privileged programs ('MECHANISMS')** offering service to **unprivileged programs ('SUBJECTS')**". | VERIFIED | `polkit(8)` |
| The authority is `polkitd(8)`, running as the `polkitd` system user with little privilege. Mechanisms, subjects and authentication agents talk to it over the **system message bus**. | VERIFIED | same |
| Authentication agents are **per user session**, provided by the user's graphical environment; "Applications that do not run under a desktop environment (for example, if launched from a `ssh(1)` login) may not have an authentication agent associated with them" → use `pkttyagent(1)`. | VERIFIED | same |
| Actions are defined by applications; vendors/admins control policy via **authorization rules**. | VERIFIED | same |

**Recommendation (PROPOSAL):** use polkit for the *one* operation that genuinely needs privilege — **mounting an external drive** — rather
than running the whole daemon as root. The mechanism exposes a D-Bus method; polkit prompts the user for admin auth; the agent does the
`mount(8)` and returns the mount point. This keeps the daemon unprivileged (which also solves the inotify `max_user_watches` budget question
and reduces blast radius).

### 8.5 Distribution packaging on Linux

| Option | Verdict |
|---|---|
| **deb / rpm** | **RECOMMENDED primary.** No filesystem restrictions; `systemd` unit + `polkit` policy + `udev` rules install cleanly; the service manager handles the lifecycle. Downside: per-distro packaging burden (but `cargo-deb` / `cargo-rpm` / `cargo-dist` automate it). |
| **Flatpak** | **NOT recommended for the server.** VERIFIED: the sandbox's filesystem access is expressed as `--filesystem=` permissions; the default sandbox exposes only the app's own directories plus XDG dirs; `host` is a special opt-in that grants the whole host filesystem. `/media` and `/run/media/<user>` are **not** in the default set (the docs list `/tmp`, `/var`, `/var/lib/flatpak`, XDG dirs). VERIFIED: to see and use removable/network volumes you need `--filesystem=/media` (or `/run/media`) **plus** `--filesystem=xdg-run/gvfs` / `--filesystem=xdg-run/gvfsd` **plus** `--talk-name=org.gtk.vfs.*` — and the docs explicitly say that permission grants "the ability to list mounted devices (USB, optical etc.), detach/format/eject them, mount them locally". For a *media server* whose whole job is to serve arbitrary user-chosen folders and hot-plugged drives, this is a large, brittle permission surface, and the `gvfs`/GTK dependency is irrelevant to a non-GTK Rust binary. **VERIFIED** on the restriction side; **ASSUMPTION** on "brittle" (judgment). |
| **AppImage** | **Possibly a better flatpak alternative** for the daemon: a single self-contained binary, no sandbox restrictions on the host FS (it still runs with the user's own privileges), no polkit packaging requirement (the user can just run it). Downside: no systemd integration unless the app self-installs a unit; no `apt`/`dnf` upgrade path. Not verified against a fetched source in this brief → **ASSUMPTION**. |
| **Snap** | Not researched for this brief → **OPEN QUESTION**. |

**Recommendation:** ship **deb + rpm** (via `cargo-dist` or `cargo-deb`) with a `systemd` system unit, a **user** unit variant, an optional
polkit action, and a `udev` rule if you want auto-mounting. Optionally offer a **static AppImage** for the "just run it" case.

---

## 9. Windows equivalent

### 9.1 Service vs tray app

| Fact | Label | Source |
|---|---|---|
| A Windows **service** "conforms to the interface rules of the Service Control Manager (SCM). It can be started automatically at system boot, by a user through the Services control panel applet, or by an application that uses the service functions. **Services can execute even when no user is logged on to the system.**" | VERIFIED | Microsoft Learn, "Services" |
| "**All services run in Terminal Services session 0.** Therefore, if an interactive service displays a user interface, it is visible only to the user who connected to session 0." | VERIFIED | Microsoft Learn, "Interactive services" |
| Drivers are a separate service type that does not interact with the SCM. | VERIFIED | same "Services" page |

**Recommendation (PROPOSAL):** mirror the macOS split — a **Windows Service** for the server (session 0, starts at boot, no user needed),
plus an optional **tray app** (a normal Win32 GUI app) as the user-facing controller. The tray app can also do the things a service cannot:
show a notification, ask the user to pick a folder, and (importantly) run in the user's session so that **mapped network drives** and
**removable drives are visible (a session-0 service sees a different drive-letter universe than the interactive user). **ASSUMPTION** — the
"service sees a different drive mapping" behaviour is well known but I did not fetch a primary source for it in this brief.

### 9.2 Packaging: MSIX vs WiX

| Fact | Label | Source |
|---|---|---|
| MSIX is Microsoft's modern Windows app packaging/install technology (`.msix`/`.msixbundle`), with a signed manifest and a declarative install model. | VERIFIED | Microsoft Learn, "MSIX overview" |

- **MSIX:** VERIFIED that it exists and is the modern packaging format. **OPEN QUESTION:** exact MSIX constraints for a background service
  (services in MSIX packages are constrained; `windows.service` extension was added for packaged services). I did not fetch that page →
  do not assume MSIX can host a boot-time service.
- **WiX (Toolset for Windows Installer):** the classic MSI-authoring toolset, mature, and the normal way to ship a Windows Service
  (ServiceInstall/ServiceControl tables) plus a firewall rule. **UNVERIFIED** in this brief (not fetched).
- **Recommendation (PROPOSAL):** **WiX MSI** for the service + firewall rule + start-menu entries, because it can create the Windows Firewall
  exception during install and can register a service. Consider **MSIX** later only for the tray app if you want Store/Store-for-Business
  distribution — but note MSIX's sandbox is hostile to arbitrary-folder media serving.

### 9.3 NTFS case-insensitivity

- **VERIFIED (mechanism):** NTFS is case-insensitive by default; case-sensitive directories are an opt-in feature. **UNVERIFIED** in this
  brief (I did not fetch the NTFS/case-sensitive-directory documentation). The macOS/Linux contrast in §6 therefore has a direct Windows
  analogue: on Windows `Movie.MKV` and `movie.mkv` are the same file, so a case-sensitive identity key is *lossy* there exactly as it is on
  macOS.

### 9.4 Long paths (MAX_PATH) and `longPathAware`

| Fact | Label | Source |
|---|---|---|
| "In the Windows API (with some exceptions), the maximum length for a path is **MAX_PATH**, which is defined as **260 characters**." | VERIFIED | Microsoft Learn, "Maximum Path Length Limitation" |
| "Starting in Windows 10, version 1607, MAX_PATH limitations have been removed from many common Win32 file and directory functions. However, **your app must opt-in** to the new behavior." | VERIFIED | same |
| **Two conditions** are required: (1) the registry value `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem LongPathsEnabled` (REG_DWORD) must exist and be `1`; (2) the application manifest must include the `longPathAware` element: `<ws2:longPathAware>true</ws2:longPathAware>` inside `<windowsSettings xmlns:ws2="http://schemas.microsoft.com/SMI/2016/WindowsSettings">`. | VERIFIED | same |
| The registry value is **cached per process** on first use and is not reloaded during the process lifetime; a reboot may be required for all processes to pick it up. | VERIFIED | same |
| It can also be controlled by Group Policy at *Computer Configuration > Administrative Templates > System > Filesystem > Enable Win32 long paths*, and by Microsoft Intune CSP. | VERIFIED | same |
| The `\\?\` prefix bypasses MAX_PATH for most APIs; `\\?\UNC\server\share` for UNC. With that prefix you may not use forward slashes, `.` or `..`. | VERIFIED | same |
| Functions that lose the MAX_PATH restriction when opted in include `CreateFileW`, `CopyFileW`, `DeleteFileW`, `FindFirstFileW`/`FindNextFileW`, `GetFileAttributesW`, `SetFileAttributesW`, `GetFullPathNameW`, `GetLongPathNameW`, `MoveFileW`, `CreateHardLinkW`, `CreateSymbolicLinkW`, `GetFinalPathNameByHandleW`, and others. | VERIFIED | same |
| The limit is **260 UTF-16 code units**, not bytes. | VERIFIED (the doc says "characters"; the unit is UTF-16 code units) | same |

**Consequence (PROPOSAL):** PhairDLNA's Windows build **must embed an application manifest** with `longPathAware=true` (via
`embed-resource`/`winres` or `cargo` build script), and it **must** instruct users to enable the `LongPathsEnabled` registry value (or
document the Group Policy). Because the registry value is per-process-cached and machine-wide, a **per-user** install cannot set it without
admin rights → either the installer does it (MSI, admin) or the app degrades gracefully.

### 9.5 Removable drive detection

| Fact | Label | Source |
|---|---|---|
| "Windows sends all top-level windows a set of default **WM_DEVICECHANGE** messages when new devices or media are added and become available, and when existing devices or media are removed. **You do not need to register to receive these default messages.**" | VERIFIED | Microsoft Learn, "Detecting Media Insertion or Removal" |
| `WM_DEVICECHANGE` event codes: `DBT_DEVICEARRIVAL` 0x8000, `DBT_DEVICEQUERYREMOVE` 0x8001, `DBT_DEVICEQUERYREMOVECOMPLETE` 0x8002, `DBT_DEVICEREMOVEPENDING` 0x8003, `DBT_DEVICEREMOVECOMPLETE` 0x8004, `DBT_DEVICETYPESPECIFIC` 0x8005, `DBT_CUSTOMEVENT` 0x8006, `DBT_USERDEFINED` 0xFFFF. | VERIFIED | Microsoft Learn, "WM_DEVICECHANGE message" |
| For a volume arrival, the app must check `dbch_devicetype == DBT_DEVTYP_VOLUME` and `dbcv_flags == DBTF_MEDIA`. | VERIFIED | same |
| "The system may broadcast a DBT_DEVICEREMOVECOMPLETE message **without** sending corresponding DBT_DEVICEQUERYREMOVE and DBT_DEVICEREMOVEPENDING messages. In such cases, the applications and drivers **must recover from the loss of the device** as best they can." | VERIFIED | Microsoft Learn, "DBT_DEVICEREMOVECOMPLETE event" |

**Consequence (PROPOSAL):** on Windows, "volume change" detection is **window-message based** — it needs a **message pump**. A session-0
service has no top-level windows, so it cannot receive the default `WM_DEVICECHANGE` broadcasts; it must use
`RegisterDeviceNotification` with a hidden window, or poll `GetLogicalDriveStrings`/`GetDriveType` on an interval. **ASSUMPTION** (the
"WM_DEVICECHANGE needs a window" part is VERIFIED — "A window receives this message through its WindowProc function" — but the
"session 0 service therefore can't get them" inference is mine).

### 9.6 Symlink creation requires Developer Mode / privilege

| Fact | Label | Source |
|---|---|
| `CreateSymbolicLinkW` flags: `SYMBOLIC_LINK_FLAG_DIRECTORY` (0x1); **`SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE` (0x2) — "Specify this flag to allow creation of symbolic links when the process is not elevated. Developer Mode must first be enabled on the machine before this option will function."** | VERIFIED | Microsoft Learn, `CreateSymbolicLinkW` |

**Consequence (PROPOSAL):** PhairDLNA should **never create symlinks** on Windows as part of normal operation (a media server does not need
to). If it ever does (e.g. for "alias this library"), it must either run elevated, require Developer Mode, or fall back to copying. Also:
Windows symbolic links are **not** the same as NTFS hard links, and a service running as `LocalSystem` sees a different set of mapped
drives than the interactive user.

### 9.7 Firewall rules for a listening server

| Fact | Label | Source |
|---|---|---|
| "**Since there's a default block action in Windows Firewall, you must create inbound exception rules to allow the traffic.** It's common for the app or the app installer itself to add this firewall rule. Otherwise, the user (or firewall admin on behalf of the user) needs to manually create a rule." | VERIFIED | Microsoft Learn, "Windows Firewall rules" |
| "If there's no active application or administrator-defined allow rule(s), a **dialog box prompts the user** to either allow or block an application's packets the first time the app is launched or tries to communicate in the network." If the user is a non-admin and is prompted, **block rules are created** regardless of the answer. | VERIFIED | same |
| "If the user responds No or cancel the prompt, block rules are created. Two rules are typically created, one each for TCP and UDP traffic." | VERIFIED | same |
| Rule precedence: explicit allow > default block; explicit block > any conflicting allow; more specific rules beat less specific ones (except explicit block). | VERIFIED | same |
| Wildcards such as `C:\*\teams.exe` are **not supported** in application rules — you must use full paths. | VERIFIED | same |
| Rules can be scoped to an **App Control for Business Application ID (AppID) tag** rather than a path. | VERIFIED | same |
| `New-NetFirewallRule` (NetSecurity module) is the programmatic way to create rules. | VERIFIED (doc page fetched) | same |

**Consequence (PROPOSAL):** the Windows installer must create an **inbound rule** for the binary (both TCP and UDP, or a port-scoped rule
for the HTTP port plus a program-scoped rule for SSDP UDP/1900). Use `New-NetFirewallRule` from the MSI custom action, or `netsh
advfirewall firewall add rule`. **Note the SSDP wrinkle:** SSDP is *inbound to UDP/1900* (NOTIFY) and *outbound multicast* (M-SEARCH), so the
rule set differs from a plain TCP server.

---

## 10. RECOMMENDED PLATFORM ADAPTER INTERFACE — **PROPOSAL**

> **This section is a proposal, not verified fact.** It is the ADR input. The reasoning for each item is given inline.

### 10.1 Design rules

1. **No Apple/Linux/Windows types in any signature.** All strings are `String`/`PathBuf`/`OsStr`; all errors are a project-owned enum.
2. **Traits are small and orthogonal.** A "platform" is a bag of small capabilities, not one giant trait — otherwise adding Windows means
   implementing 40 methods you do not need.
3. **The core owns the semantics; the adapter owns the mechanism.** E.g. the core owns "what is a file identity"; the adapter owns "how do
   I get `st_ino` here".
4. **Every trait has a `noop`/`unsupported` implementation for platforms that do not need it** (e.g. `UserNotifier` on a headless Linux
   server), so the core compiles and runs everywhere.
5. **The adapter must expose *capabilities*, not *platform names*.** e.g. `MulticastCaps { join_group: bool, ipv6: bool }` rather than
   `is_macos()`.

### 10.2 The trait sketch

```rust
// phairdlna-core/src/platform.rs   — PROPOSAL
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Errors are ours, not the OS's. The adapter maps errno/Win32/TCC into these.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("not found")]           NotFound,
    #[error("permission denied")]   PermissionDenied,
    #[error("consent required")]    ConsentRequired { what: String },
    #[error("not supported here")]  Unsupported,
    #[error("io error")]            Io(#[from] std::io::Error),
    #[error("transient")]           Transient,
    #[error("volume went away")]    VolumeGone,
}

// ---------------------------------------------------------------- 1. networking

#[derive(Debug, Clone)]
pub struct Interface {
    pub id: String,          // stable per-boot id, e.g. "en0" / "eth0" / iface GUID
    pub name: String,        // human label
    pub is_loopback: bool,
    pub is_up: bool,
    pub is_broadcast_capable: bool,   // TN3179: only these may run SSDP
    pub is_link_local_only: bool,
    pub ipv4: Vec<std::net::Ipv4Addr>,
    pub ipv6: Vec<std::net::Ipv6Addr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulticastGroup { SsdpIpv4, SsdpIpv6LinkLocal, SsdpIpv6SiteLocal, SsdpIpv6Global }

#[derive(Debug, Clone, Copy)]
pub struct MulticastCaps {
    /// Can we bind+join an arbitrary multicast group on a chosen interface?
    pub can_join_group: bool,
    /// Can we send on a chosen source address (IP_PKTINFO / IPV6_PKTINFO)?
    pub can_select_source: bool,
    pub ipv6: bool,
    /// Does the OS require an entitlement/entitlement-free permission for multicast?
    pub requires_multicast_entitlement: bool, // true on iOS, FALSE on macOS (TN3179)
}

pub trait Network {
    /// Enumerate broadcast-capable interfaces. TN3179: do NOT hard-code "en0".
    fn interfaces(&self) -> Result<Vec<Interface>, PlatformError>;

    /// Multicast capability of this platform (macOS: no entitlement needed).
    fn multicast_caps(&self) -> MulticastCaps;

    /// Bind a UDP socket to `interface` and join `group`.
    /// Platform-specific: setsockopt(IP_ADD_MEMBERSHIP / IPV6_JOIN_GROUP),
    /// binds to the interface's link-local address, enables IP_MULTICAST_IF.
    fn join_multicast(&self, iface: &Interface, group: MulticastGroup)
        -> Result<std::net::UdpSocket, PlatformError>;

    /// Has the OS granted this process "local network" access?
    /// macOS: NO public API exists (FB8711182) -> return Unknown.
    fn local_network_permitted(&self) -> LocalNetworkStatus { LocalNetworkStatus::Unknown }

    /// The OS's own name for the current network permission state.
    fn local_network_status(&self) -> LocalNetworkStatus;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalNetworkStatus {
    Allowed, Denied, Undetermined, Unknown,
}

// ---------------------------------------------------------------- 2. filesystem identity

/// A path as the OS sees it. `raw` is byte-exact; never re-normalise it.
#[derive(Debug, Clone)]
pub struct MediaPath {
    pub raw: PathBuf,                 // verbatim, for ALL I/O
    pub display: String,              // NFC of the basename; presentation only
    pub identity: String,             // NFC + casefold per component; matching key
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseSensitivity { Sensitive, Insensitive, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationPolicy { Preserve, Nfc, Nfd, Unknown }

#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub id: VolumeId,                 // stable identity for THIS volume (see below)
    pub mount_point: PathBuf,
    pub display_name: String,
    pub is_removable: bool,
    pub is_network: bool,
    pub is_read_only: bool,
    pub case_sensitivity: CaseSensitivity,
    pub normalization: NormalizationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VolumeId(String);          // opaque, stable while the volume is mounted

/// A file's identity WITHIN a volume. Two paths with the same FileId are the same bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId { pub dev: u64, pub ino: u64 }  // st_dev/st_ino on Unix;
                                                   // (volume serial, file index) on Windows

pub trait FileSystem {
    /// Normalise a raw path into the project's identity string.
    /// Rule (see §5.1): NFC per component, then casefold. NEVER applied to `raw`.
    fn identity_of(&self, raw: &Path) -> String;

    /// Display form of a basename. Rule: NFC of the raw basename.
    fn display_name_of(&self, raw: &Path) -> String;

    /// The volume containing `raw`, and its identity.
    fn volume_of(&self, raw: &Path) -> Result<VolumeInfo, PlatformError>;

    /// Stable id of the file object (hardlink-aware).
    fn file_id(&self, raw: &Path) -> Result<FileId, PlatformError>;

    /// How many names point at this file (hardlinks + links). `1` = single name.
    fn link_count(&self, raw: &Path) -> Result<u64, PlatformError>;

    /// Real allocation vs apparent size (sparse files). None = unknown.
    fn allocated_bytes(&self, raw: &Path) -> Result<Option<u64>, PlatformError>;

    /// Should the indexer skip this entry? Default: skip `._*` AppleDouble,
    /// skip zero-length sidecars, skip `UF_HIDDEN` items if configured.
    fn is_ignorable(&self, raw: &Path, meta: &EntryMeta) -> bool;

    /// Resolve symlinks / aliases per the configured policy.
    fn resolve(&self, raw: &Path, policy: SymlinkPolicy) -> Result<PathBuf, PlatformError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkPolicy { Follow, IndexAsLink, Skip }

#[derive(Debug, Clone, Default)]
pub struct EntryMeta { pub is_symlink: bool, pub is_hidden: bool, pub size: u64 }

// ---------------------------------------------------------------- 3. volumes / hot-plug

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeEvent {
    Mounted(VolumeInfo),
    Unmounted { id: VolumeId, mount_point: PathBuf },
    Remounted(VolumeInfo),
    Renamed { id: VolumeId, from: PathBuf, to: PathBuf },
    /// The OS told us we might have missed changes -> rescan.
    RescanRequired,
}

pub trait VolumeWatcher {
    /// Subscribe to mount/unmount events.
    /// macOS: DiskArbitration (callback-based, no AppKit run loop) or NSWorkspace
    ///       (AppKit run loop) — the adapter picks.
    /// Linux: systemd mount units / udev / inotify on /dev/disk/by-*.
    /// Windows: RegisterDeviceNotification (needs a message pump) or polling.
    fn watch_volumes(&self) -> Result<VolumeEventStream, PlatformError>;
}

// ---------------------------------------------------------------- 4. consent / access

pub trait Consent {
    /// Ask the user to select a folder; selection alone is not a cross-process grant.
    /// The platform adapter must establish access for the process that will read it.
    fn request_folder_access(&self) -> Result<ConsentGrant, PlatformError>;

    /// Re-check an access grant held by the process that will read the folder.
    fn validate_grant(&self, grant: &ConsentGrant) -> bool;

    /// Whether this process currently holds "Full Disk Access" / equivalent.
    fn has_full_disk_access(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ConsentGrant {
    pub path: PathBuf,
    /// macOS: security-scoped bookmark bytes (opaque to the core).
    pub token: Option<Vec<u8>>,
}

// ---------------------------------------------------------------- 5. user environment

pub trait UserEnvironment {
    /// Per-user config dir (macOS: ~/Library/Application Support/PhairDLNA).
    fn config_dir(&self) -> Result<PathBuf, PlatformError>;
    /// Per-user data dir (macOS: ~/Library/Application Support/PhairDLNA/Data).
    fn data_dir(&self) -> Result<PathBuf, PlatformError>;
    /// Per-user cache dir (macOS: ~/Library/Caches/PhairDLNA).
    fn cache_dir(&self) -> Result<PathBuf, PlatformError>;
    /// Per-user log dir (macOS: ~/Library/Logs/PhairDLNA).
    fn log_dir(&self) -> Result<PathBuf, PlatformError>;
    /// Current user name / uid, for logs and for per-user service identity.
    fn user_name(&self) -> String;
}

// ---------------------------------------------------------------- 6. service lifecycle

pub trait ServiceManager {
    /// Install/enable a background service for the current user.
    /// macOS: write ~/Library/LaunchAgents/<name>.plist + launchctl bootstrap
    ///       (or SMAppService.agent(plistName:) if a bundle is present).
    /// Linux: write ~/.config/systemd/user/<name>.service + systemctl --user enable.
    /// Windows: register via SCM (needs admin).
    fn install_user_service(&self, spec: &ServiceSpec) -> Result<(), PlatformError>;
    fn remove_user_service(&self, spec: &ServiceSpec) -> Result<(), PlatformError>;
    fn is_service_installed(&self, spec: &ServiceSpec) -> bool;
    fn service_status(&self, spec: &ServiceSpec) -> ServiceStatus;
}

#[derive(Debug, Clone)]
pub struct ServiceSpec {
    pub label: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub keep_alive: bool,
    pub stdout_path: Option<PathBuf>,
    pub stderr_path: Option<PathBuf>,
    /// macOS: value for AssociatedBundleIdentifiers (so Local Network privacy
    /// attributes the prompt to the right app). TN3179.
    pub associated_bundle_ids: Vec<String>,
    /// Linux: Type=notify + sd_notify(READY=1). Windows: service type.
    pub notify_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus { NotInstalled, Installed, Running, Failed, Unknown }

// ---------------------------------------------------------------- 7. logging

pub trait LogSink: Send + Sync {
    fn log(&self, record: &LogRecord);
}

pub struct LogRecord {
    pub level: LogLevel,
    pub timestamp: SystemTime,
    pub subsystem: &'static str,
    pub message: String,
    /// macOS: forward to os_log so it shows up in Console.app / unified log.
    pub os_log: bool,
}

// ---------------------------------------------------------------- 8. user interaction

/// Optional; `Unsupported` on headless platforms.
pub trait UserNotifier {
    fn notify(&self, title: &str, body: &str) -> Result<(), PlatformError>;
    /// macOS: NSStatusItem / NSMenu (muda or raw objc2-app-kit).
    fn show_menu(&self) -> Result<(), PlatformError>;
    /// macOS: NSOpenPanel folder picker.
    fn pick_folder(&self) -> Result<Option<PathBuf>, PlatformError>;
}

// ---------------------------------------------------------------- 9. the facade

/// The one type the core actually depends on.
pub struct Platform { /* boxed adapters */ }

impl Platform {
    pub fn network(&self) -> &dyn Network;
    pub fn fs(&self) -> &dyn FileSystem;
    pub fn volumes(&self) -> &dyn VolumeWatcher;
    pub fn consent(&self) -> &dyn Consent;
    pub fn env(&self) -> &dyn UserEnvironment;
    pub fn services(&self) -> &dyn ServiceManager;
    pub fn logs(&self) -> &dyn LogSink;
    pub fn notifier(&self) -> &dyn UserNotifier;   // -> Unsupported on Linux server
}
```

### 10.3 Reasoning for the key choices

| Choice | Reasoning |
|---|---|
| **`Network` returns `Interface` objects rather than an `is_macos()` flag** | TN3179 (VERIFIED) says BSD names like `en0` are not API and that discovery must run on **all broadcast-capable** interfaces, discovered via `getifaddrs` + `SIOCGIFFUNCTIONALTYPE`. The core needs that abstraction to be correct on every platform. |
| **`multicast_caps` instead of an entitlement check** | TN3179 (VERIFIED): the multicast entitlement is required on iOS and **not** on macOS. The core must not assume it needs one. |
| **`local_network_permitted()` returns `Unknown` on macOS** | TN3179 (VERIFIED): there is **no API** to query it (FB8711182). Returning `Unknown` is honest; the core must therefore be **retry-driven**, not permission-driven. |
| **`join_multicast` takes an `Interface` and a `MulticastGroup`** | UPnP DA 1.1 (VERIFIED) requires one SSDP advertisement **per UPnP-enabled interface**, and distinct IPv6 scopes (FF02::C / FF05::C / FF0E::C). The core cannot know the right scope without asking the adapter. |
| **`MediaPath { raw, display, identity }`** | This is §6.2. `raw` for I/O, `identity` for keys, `display` for humans. The whole Unicode trap lives in this one struct. |
| **`FileId { dev, ino }` as the dedup key, not the path** | Hardlink correctness (§5.7) and rename correctness. Two names, one file. |
| **`VolumeId` is opaque and separate from the mount point** | The mount point changes (a USB disk gets a different `/Volumes/<name>` if you rename it); the volume identity does not. Needed for "is this the same disk I indexed 10 minutes ago". |
| **`VolumeEvent::RescanRequired`** | `IN_Q_OVERFLOW` on Linux (VERIFIED, `inotify(7)`), FSEvents' "files you do not own" limitation on macOS (VERIFIED, `notify` docs), and `DBT_DEVICEREMOVECOMPLETE` without a preceding query/pending pair on Windows (VERIFIED, `DBT_DEVICEREMOVECOMPLETE`) all say the same thing: **you will miss events; you must rescan**. |
| **`Consent` is a trait, not a macOS-only concept** | TCC on macOS, polkit on Linux (VERIFIED: "privileged programs ('MECHANISMS')" vs "unprivileged programs ('SUBJECTS')"), UAC on Windows. The *mechanism* differs; the *core concept* (ask the user for scoped access to a folder) is identical. |
| **`ServiceSpec.associated_bundle_ids`** | TN3179 (VERIFIED) requires `AssociatedBundleIdentifiers` for a launchd agent not installed via `SMAppService`, so Local Network privacy attributes the prompt correctly. |
| **`ServiceSpec.notify_ready`** | `sd_notify(0, "READY=1")` with `Type=notify` (VERIFIED, `sd_notify(3)`) makes `systemctl start` mean "actually listening", not "process spawned". |
| **`LogSink` with an `os_log` flag** | `os_log` is the macOS-native sink; a plain stdout sink works for launchd (`StandardOutPath`, VERIFIED) and systemd (`StandardOutput=journal`, VERIFIED). The flag lets the macOS adapter double-dispatch. |
| **`UserNotifier` returns `Unsupported` on a headless server** | The headless requirement is a fixed product constraint. The trait exists so the *menu bar app* can use it without the core caring. |
| **No trait exposes a UI object** | Keeps AppKit/Swift/GTK out of the core permanently. The macOS adapter is the only crate that ever names `NSStatusItem`. |

### 10.4 What the adapter must **not** do

- **Must not** re-normalise paths before I/O.
- **Must not** assume `en0` / `eth0` / `C:`.
- **Must not** key the database on a display name.
- **Must not** create symlinks on Windows without Developer Mode/elevation.
- **Must not** index `._*` AppleDouble files.
- **Must not** treat `PermissionDenied` as fatal.
- **Must not** assume a listening socket will be reachable — on macOS the firewall may prompt, on Windows the firewall will block by default.

---

## 11. PLATFORM SUPPORT MATRIX

**Milestones are proposals.** "M0" = the shared Rust core (protocol, index, HTTP). "M1" = headless server on macOS. "M2" = Linux. "M3" = Windows. "M4" = macOS menu bar app.

| Target | Support | Milestone | Known blockers | Owner decision needed |
|---|---|---|---|---|
| **macOS arm64 (Apple silicon)** | **First-class / primary** | **M1** (server) → **M2.6** (native application) | (1) Local Network privacy (macOS 15+) treats per-user agents and system daemons differently; the actual approved service identity and its behavior must be verified (OD-15). (2) Application firewall behavior for the selected service is unverified. (3) **Mandatory code signature** for arm64 execution (VERIFIED, APS). (4) TCC protects selected user folders; access across the UI/service boundary and revocation are unverified (OD-15). (5) `launchd.plist(5)` caveat: privacy-sensitive paths in a plist "may prevent the job from running" (VERIFIED). | **D1 (OD-15):** resolve service identity against PLT-013, OPS-015 and PLT-018; verify Local Network behavior, selected-folder scope and revocation. **D2**: buy the 99 USD/yr Apple Developer Program membership, or ship unsigned? **D3**: minimum macOS version — 15 (needs Local Network handling) or 13/14 (no prompt)? |
| **macOS x86_64 (Intel)** | Supported, **secondary** (Intel Mac install base is shrinking) | **M1** | (1) No arm64 signature requirement, but ad-hoc signing is still free. (2) Must ship a **universal binary** or a separate x86_64 artifact. (3) `cocoa`'s crates.io platform metadata omits arm64 → use `objc2` (VERIFIED, both archs declared). | **D4**: ship one universal binary or two artifacts? (Universal = one notarization, two arch slices; two artifacts = two notarizations.) |
| **Linux x64** | Designed in from day one; **supported** | **M2** | (1) inotify watch/queue limits (VERIFIED) → rescan on `IN_Q_OVERFLOW`. (2) polkit needed for privileged mount (VERIFIED). (3) Distribution fragmentation (deb/rpm/Flatpak/AppImage). (4) Flatpak's filesystem sandbox is hostile to arbitrary-folder serving (VERIFIED). | **D5**: system service (recommended) or user service + `loginctl enable-linger`? **D6**: ship deb+rpm only, or also Flatpak/AppImage? |
| **Linux arm64** | Designed in from day one; **supported** (Raspberry Pi / SBC class) | **M2** | (1) Same as Linux x64 plus ARM build/packaging (aarch64 deb/rpm). (2) `muda`/`tray-icon` are irrelevant here (headless). (3) inotify limits are per-UID and per-instance — tighter on small devices. | **D7**: is a 32-bit/armv7 Raspberry Pi 1/2 target in scope, or arm64 only? (armv7 changes the `notify`/FSEvents-equivalent story and the inotify budget.) |
| **Windows x64** | Designed in from day one; **supported** | **M3** | (1) `MAX_PATH` 260 unless the app embeds `longPathAware=true` **and** the machine has `LongPathsEnabled=1` (VERIFIED, both conditions). (2) Windows Firewall **blocks inbound by default**; an inbound rule must be created by the installer (VERIFIED). (3) `WM_DEVICECHANGE` needs a window/message pump → a session-0 service cannot use the default broadcasts (VERIFIED mechanism; ASSUMPTION on the session-0 consequence). (4) Symlink creation needs Developer Mode/elevation (VERIFIED). (5) Service runs in session 0 (VERIFIED) → different mapped-drive view than the user. | **D8**: WiX MSI (recommended) vs MSIX? **D9**: may the installer require admin (needed for the service + firewall rule + `LongPathsEnabled`)? **D10**: ship a portable ZIP + a "run as tray app" mode for users who refuse admin? |

### 11.1 Owner decisions required (consolidated)

| # | Decision | Recommendation | Why it matters |
|---|---|---|---|
| **D1 (OD-15)** | macOS service shape and identity | **OPEN.** Evaluate the current system-domain daemon requirement and a per-user service against OPS-015, PLT-018, Local Network behavior, selected-folder scope and revocation. | The earlier root LaunchDaemon + LaunchAgent split is an unverified hypothesis, not a recommendation or approved architecture. The system-domain service must not be assumed to run as root or inherit UI consent. |
| **D2** | Apple Developer Program membership (99 USD/yr) | **Decide at project start; default to "no" (unsigned + `curl`/brew), with the code structured so "yes" costs no code changes** | Membership is a *person's legal identity* + an annual cost; it should be a deliberate, recorded project decision, not a silent default. Signing affects Gatekeeper, the firewall auto-allow, and Local Network identity tracking (all VERIFIED). |
| **D3** | Minimum macOS version | **macOS 15+** | Local Network privacy exists only on 15+ (VERIFIED). Supporting 13/14 means *not* knowing whether a prompt appeared → worse diagnostics. Supporting 15+ lets you also use `SMAppService` (13+) and modern AppKit. |
| **D4** | macOS artifact shape | **One universal binary (arm64 + x86_64)** | One notarization, one release asset, one Gatekeeper identity. `objc2`/`objc2-app-kit` declare both arches (VERIFIED). |
| **D5** | Linux service scope | **System service by default; user unit as an opt-in alternative** | The server must run headless and unattended. A user service needs lingering and gives up root-only media access. |
| **D6** | Linux packaging | **deb + rpm** (via `cargo-dist`/`cargo-deb`); **no Flatpak for the server** | Flatpak's filesystem sandbox requires `--filesystem=host` (or a wide `--filesystem=` grant) plus gvfs/D-Bus plumbing to see removable and network volumes (VERIFIED). That is the wrong shape for this product. |
| **D7** | Linux arm32 scope | **arm64 only** (skip armv7) | Keeps one `notify`/inotify story and one packaging story; the ARM market for a DLNA server is now overwhelmingly 64-bit. |
| **D8** | Windows packaging | **WiX MSI** (service + firewall rule + `longPathAware` manifest); MSIX later only for the tray app | MSI can create the inbound firewall rule (VERIFIED: "It's common for the app or the app installer itself to add this firewall rule") and register the service. MSIX's sandbox is hostile to arbitrary-folder serving. |
| **D9** | Windows admin rights | **Yes for the MSI install** (service + firewall + optional `LongPathsEnabled`) | Without admin you cannot create the service, the firewall rule, or the long-path registry value. |
| **D10** | Windows no-admin fallback | **Yes — a portable "tray app" mode** that runs the server in the user's session | Gives non-admin users a working server at the cost of not running headless. |
| **D11** | Menu bar app technology | **`muda` (or raw `objc2-app-kit`) — pure Rust, no Swift, no webview** | VERIFIED: `muda`'s macOS backend is `objc2` + `objc2-app-kit`. Caveats: MSRV 1.90 for `muda` (vs 1.71 for `objc2-app-kit`), main-thread-only, and it must be an `.app` bundle. |
| **D12** | Index identity policy | **Adopt §6.2: `raw` + NFC-casefolded `identity` + `(VolumeId, FileId)` uniqueness** | This is the Unicode trap (§5.1) and the hardlink trap (§5.7). Getting this wrong produces phantom duplicates and "file not found" after an OS upgrade. |
| **D13** | Symlink policy default | **Follow symlinks for library roots; do not create symlinks on Windows** | Windows symlink creation needs Developer Mode/elevation (VERIFIED). |
| **D14** | Consent model | **OPEN (OD-15):** per-user folder selection and platform-specific consent mechanisms are candidates; macOS `NSOpenPanel` → scoped bookmark does not yet prove access for PhairDLNA's separate service process. Linux polkit and Windows UAC need their own platform validation. | Do not model a UI selection as a portable authorization token. The `Consent` trait is a candidate abstraction only; its process semantics and revocation behavior need architecture review. |

---

## 12. Open questions / things that must be tested on real hardware

| # | Question | Why it cannot be answered from docs |
|---|---|---|
| Q1 | Under the macOS application firewall, what happens when the **OD-15-approved service process** opens a listening socket: does it get through silently, get queued for a prompt it cannot show, or get dropped? | Apple's firewall docs describe the policy, not behavior for PhairDLNA's eventual service identity. **Largest residual macOS risk.** |
| Q2 | Does macOS Local Network privacy prompt for a **`LaunchAgent`** that is installed by a **non-bundle** CLI installer (no `AssociatedBundleIdentifiers`)? What identity is shown? | TN3179 says to set `AssociatedBundleIdentifiers` for non-`SMAppService` agents but does not describe the no-bundle case. |
| Q3 | Can the proposed **system `LaunchDaemon`** receive only the user's selected-folder access from the UI process, and can revocation be enforced? | Apple's bookmark docs describe App Sandbox access and per-process activation; they do not document this PhairDLNA process boundary. Test a protected selected folder, an unselected sibling and revocation on a real Mac. |
| Q4 | Does `SEEK_HOLE`/`SEEK_DATA` work correctly on APFS for sparse-file detection? | Not verified. |
| Q5 | Is `DiskArbitration`'s callback usable from a **headless daemon with no AppKit run loop**? | The framework doc does not say; NSWorkspace definitely needs AppKit. |
| Q6 | On Windows, can a **session-0 service** receive `WM_DEVICECHANGE` at all? | The message requires a `WindowProc`; a service has no window. |
| Q7 | Does macOS Gatekeeper block a **raw non-bundle CLI binary** downloaded via a browser, and does "Open Anyway" apply to it? | Apple's docs describe "apps". |
| Q8 | Does an **APFS case-sensitive** volume change `identity` semantics enough to warrant a different `CaseSensitivity` policy in the index? | The variant exists; the practical impact on a media index is unmeasured. |

---

## 13. Text-size accessibility in the native macOS app

**Verified platform limit:** Apple's Human Interface Guidelines state that macOS does
not support Dynamic Type. `NSFont.preferredFont(forTextStyle:options:)` supplies the
font associated with a text style and reserves appropriate line metrics for scripts;
Apple does not document it as a user-adjustable text-size mechanism on macOS. The
current overview instead assigns fixed point sizes and fixed-height frames, so it does
not meet the F-15 claim that Dynamic Type makes the text follow a system size setting.

The HIG identifies SF Pro as the macOS system font and recommends system font variants
for standard controls. Standard fonts and semantic colors preserve the platform look,
but do not by themselves provide an app-level larger-text control. A product choice is
therefore needed: add an app-managed text-size preference and reflow the overview, or
revise the requirement and make the boundary explicit. Truncation must remain visible
in the manual accessibility and localization protocol.

Sources: Apple HIG [Typography](https://developer.apple.com/design/human-interface-guidelines/typography)
(macOS Dynamic Type statement and system-font guidance); Apple Developer
[`NSFont.preferredFontForTextStyle:options:`](https://developer.apple.com/documentation/appkit/nsfont/preferredfont%28fortextstyle%3Aoptions%3A%29);
Apple Developer [Larger Text evaluation criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/larger-text-evaluation-criteria)
(general guidance to avoid clipping, truncation and overlap when text is enlarged).

## Sources

Every URL below was actually fetched during this brief.

### Apple — Accessibility sources added 2026-11-18
- AppKit `NSFont.preferredFontForTextStyle:options:` — https://developer.apple.com/documentation/appkit/nsfont/preferredfont%28fortextstyle%3Aoptions%3A%29
- Human Interface Guidelines — Typography — https://developer.apple.com/design/human-interface-guidelines/typography
- Larger Text evaluation criteria — https://developer.apple.com/help/app-store-connect/manage-app-accessibility/larger-text-evaluation-criteria

### Apple — Developer Documentation
1. TN3179 — Understanding local network privacy — https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy (payload fetched: `https://developer.apple.com/tutorials/data/documentation/technotes/tn3179-understanding-local-network-privacy.json`)
2. NSLocalNetworkUsageDescription — https://developer.apple.com/documentation/bundleresources/information-property-list/nslocalnetworkusagedescription
3. NSBonjourServices — https://developer.apple.com/documentation/bundleresources/information-property-list/nsbonjourservices
4. Protected resources — https://developer.apple.com/documentation/bundleresources/protected-resources
5. Notarizing macOS software before distribution — https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
6. Customizing the notarization workflow — https://developer.apple.com/documentation/security/customizing-the-notarization-workflow
7. Hardened Runtime — https://developer.apple.com/documentation/security/hardened-runtime
8. TN3127 — Inside Code Signing: Requirements — https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements
9. SMAppService — https://developer.apple.com/documentation/servicemanagement/smappservice
10. NSWorkspace — https://developer.apple.com/documentation/appkit/nsworkspace
11. NSOpenPanel — https://developer.apple.com/documentation/appkit/nsopenpanel
12. NSStatusItem — https://developer.apple.com/documentation/appkit/nsstatusitem
13. DiskArbitration framework — https://developer.apple.com/documentation/diskarbitration
14. Create Developer ID certificates — https://developer.apple.com/help/account/certificates/create-developer-id-certificates
15. Compare memberships — https://developer.apple.com/support/compare-memberships/
16. Enrolling in the Apple Developer Program — https://developer.apple.com/programs/enroll/
17. File System Programming Guide ("File System Basics") — https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html
18. File System Programming Guide ("File System Details") — https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemDetails/FileSystemDetails.html
19. Apple File System Guide — FAQ — https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html
20. Device Management `Firewall` payload — https://developer.apple.com/documentation/devicemanagement/firewall
21. Building a universal macOS binary — https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary

### Apple — Support & Platform Security
22. Manage Firewall — https://support.apple.com/guide/platform-support/supd6b171d92/web
23. Safely open apps on your Mac — https://support.apple.com/en-us/102445
24. Gatekeeper and runtime protection in macOS (Apple Platform Security) — https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web
25. Firewall security in macOS (Apple Platform Security) — https://support.apple.com/guide/security/firewall-security-in-macos-seca0e83763f/web
26. Apple Platform Security guide (PDF) — https://help.apple.com/pdf/security/en_GB/apple-platform-security-guide-b.pdf (used for "Signed system volume security" → "Unsigned x86_64 code", and "Controlling app access to files")
27. Apple Platform Security — Controlling app access to files — https://support.apple.com/guide/security/protecting-app-access-to-user-data-secbf9e8e3c8/web
28. Updating helper executables from earlier versions of macOS — https://developer.apple.com/documentation/servicemanagement/updating-helper-executables-from-earlier-versions-of-macos
29. SMAppService.Status — https://developer.apple.com/documentation/servicemanagement/smappservice/status-swift.enum

### Apple — Man pages & Tech Notes
28. `launchd.plist(5)` — https://keith.github.io/xcode-man-pages/launchd.plist.5.html
29. `launchd(8)` — https://keith.github.io/xcode-man-pages/launchd.8.html
30. `stat(1)` — https://keith.github.io/xcode-man-pages/stat.1.html
31. `xattr(1)` — https://keith.github.io/xcode-man-pages/xattr.1.html
32. TN1150 — HFS Plus Volume Format — https://developer.apple.com/library/archive/technotes/tn1150.html
33. TN2078 — Migrating to FSRefs & long Unicode names from FSSpecs — https://developer.apple.com/library/archive/technotes/tn2078/
34. QA1173 — Text Encodings in VFS — https://developer.apple.com/library/archive/qa/qa1173/_index.html

### Unicode
35. Unicode FAQ — Normalization — https://www.unicode.org/faq/normalization.html
36. Unicode Standard Annex #15 (UAX #15) — https://www.unicode.org/reports/tr15/

### UPnP
37. UPnP Device Architecture 1.1 — https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.1.pdf (SSDP 239.255.255.250:1900; Annex A IPv6 FF02::C / FF05::C / FF0E::C)

### Rust crates
38. crates.io API — `objc2` — https://crates.io/api/v1/crates/objc2
39. crates.io API — `objc2-app-kit` — https://crates.io/api/v1/crates/objc2-app-kit
40. crates.io API — `objc2-foundation` — https://crates.io/api/v1/crates/objc2-foundation
41. crates.io API — `cocoa` — https://crates.io/api/v1/crates/cocoa
42. crates.io API — `core-foundation` — https://crates.io/api/v1/crates/core-foundation
43. crates.io API — `core-foundation-sys` — https://crates.io/api/v1/crates/core-foundation-sys
44. crates.io API — `block2` — https://crates.io/api/v1/crates/block2
45. crates.io API — `dispatch` — https://crates.io/api/v1/crates/dispatch
46. crates.io API — `notify` — https://crates.io/api/v1/crates/notify
47. crates.io API — `muda` — https://crates.io/api/v1/crates/muda
48. crates.io API — `tray-icon` — https://crates.io/api/v1/crates/tray-icon
49. crates.io API — `fsevent-sys` — https://crates.io/api/v1/crates/fsevent-sys
50. crates.io API — `unicode-normalization` — https://crates.io/api/v1/crates/unicode-normalization
51. docs.rs — `objc2-app-kit` 0.3.2 — https://docs.rs/crate/objc2-app-kit/0.3.2/source/Cargo.toml
52. docs.rs — `muda` 0.21.2 — https://docs.rs/crate/muda/0.21.2/source/Cargo.toml
53. docs.rs — `notify` 8.2.2 — https://docs.rs/crate/notify/8.2.2/source/Cargo.toml
54. docs.rs — `cocoa` 0.27.0 — https://docs.rs/crate/cocoa/0.27.0/source/Cargo.toml
55. docs.rs — `muda` — https://docs.rs/muda/latest/muda/
56. docs.rs — `tray-icon` — https://docs.rs/tray-icon/latest/tray-icon/
57. docs.rs — `notify` — https://docs.rs/notify/latest/notify/
58. docs.rs — `objc2` — https://docs.rs/objc2/latest/objc2/
59. crates.io — `cocoa` 0.27.0 tarball (Cargo.toml + src/lib.rs, inspected for arch-specific cfgs) — https://static.crates.io/crates/cocoa/cocoa-0.27.0.crate

### Linux
60. `systemd.service(5)` — https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html
61. `systemctl(1)` — https://www.freedesktop.org/software/systemd/man/latest/systemctl.html
62. `systemd.exec(5)` — https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html
63. `sd_notify(3)` — https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html
64. `systemd.mount(5)` — https://www.freedesktop.org/software/systemd/man/latest/systemd.mount.html
65. `systemd-udevd.service(8)` — https://www.freedesktop.org/software/systemd/man/latest/systemd-udevd.service.html
66. `inotify(7)` — https://man7.org/linux/man-pages/man7/inotify.7.html
67. `polkit(8)` — https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html
68. Flatpak Sandbox Permissions — https://docs.flatpak.org/en/latest/sandbox-permissions.html

### Windows
69. Services (Windows) — https://learn.microsoft.com/en-us/windows/win32/services/services
70. Service Configuration — https://learn.microsoft.com/en-us/windows/win32/services/service-configuration
71. Interactive Services — https://learn.microsoft.com/en-us/windows/win32/services/interactive-services
72. Maximum Path Length Limitation — https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
73. `CreateSymbolicLinkW` — https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw
74. Detecting Media Insertion or Removal — https://learn.microsoft.com/en-us/windows/win32/devio/detecting-media-insertion-or-removal
75. `WM_DEVICECHANGE` message — https://learn.microsoft.com/en-us/windows/win32/devio/wm-devicechange
76. `DBT_DEVICEREMOVECOMPLETE` event — https://learn.microsoft.com/en-us/windows/win32/devio/dbt-deviceremovecomplete
77. Windows Firewall rules — https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/rules
78. `New-NetFirewallRule` — https://learn.microsoft.com/en-us/powershell/module/netsecurity/new-netfirewallrule
79. MSIX overview — https://learn.microsoft.com/en-us/windows/msix/overview
80. Service Control Manager — https://learn.microsoft.com/en-us/windows/win32/services/service-control-manager

### Sources consulted but NOT cited as evidence (background only)
81. Apple Developer Forums thread 726057 (UDP broadcast listener / firewall / code signing) — https://developer.apple.com/forums/thread/726057 — used only to mark the unsigned-vs-signed firewall behaviour as **UNVERIFIED**.
82. Apple Developer Forums thread 792453 (Local Network permission / stable code signing identity / executable UUID) — https://developer.apple.com/forums/thread/792453 — corroborates TN3179 but is a forum report.
83. TN2344 "Compressing Mac Files For Transport On Other Platforms" — https://developer.apple.com/library/archive/technotes/tn/tn2344.html — **retired / 404**, so the AppleDouble `._` convention is marked **ASSUMPTION**.
84. "Signing Mac Software That Runs on Other Users' Computers" — https://developer.apple.com/documentation/security/signing-mac-software-that-runs-on-other-users-computers — returned 404 via the fetch used here; the same facts are carried by TN3127 (source 8) instead.
85. NSURL — https://developer.apple.com/documentation/foundation/nsurl/
86. CFURLStartAccessingSecurityScopedResource — https://developer.apple.com/documentation/CoreFoundation/CFURLStartAccessingSecurityScopedResource%28_%3A%29
87. CFURLCreateBookmarkData — https://developer.apple.com/documentation/CoreFoundation/CFURLCreateBookmarkData%28_%3A_%3A_%3A_%3A_%3A_%3A%29
