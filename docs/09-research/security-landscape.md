# PhairDLNA — Security & Privacy Threat Landscape (Requirements Phase)

**Status:** Draft for requirements review · **Owner:** Security & Privacy Engineering
**Scope:** PhairDLNA — new, from-scratch, pure-Rust DLNA/UPnP AV media server. macOS-first. AGPL-3.0-or-later. Privacy-first. LAN-only by default.
**Method:** Primary-source review of NVD/CVE records (retrieved via the official MITRE CVE List API `cveawg.mitre.org/api/cve/<ID>`), CERT/CC vulnerability notes, GitHub/RustSec advisory data (via OSV API `api.osv.dev/v1/vulns/<ID>`), upstream issue trackers and commit histories of ReadyMedia/MiniDLNA, Gerbera and the UPnP IGD ecosystem, and vendor/cloud write-ups.

### Label legend

| Label | Meaning |
|---|---|
| **VERIFIED** | Directly supported by a source URL that was actually fetched during this research. |
| **ASSUMPTION** | Engineering judgement or inference drawn from verified facts; not itself a documented attack. |
| **OPEN QUESTION** | A gap that must be resolved before the requirement is frozen. |
| **UNVERIFIED** | A claim that appeared in the task brief or in secondary material but that I could **not** confirm against a fetched primary source. Treated as absent. |

> **Rule applied throughout:** no CVE ID is cited unless the record was retrieved and read. Where I could not retrieve a record, the claim is marked UNVERIFIED and carries no CVE ID.

---

## 0. Corrections to the brief (read this first)

Three premises in the requirements brief did not survive verification. They change the threat model, so they are stated up front.

**C-1 — The "MiniDLNA 2019 series CVE-2019-1211x" is not MiniDLNA.** (VERIFIED)
The `CVE-2019-1211x` block is **MiniUPnPd** and **ONAP**:
- `CVE-2019-12110` — "An AddPortMapping Denial Of Service vulnerability in MiniUPnP MiniUPnPd through 2.1 exists due to a NULL pointer dereference in upnpredirect.c." — VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2019-12110
- `CVE-2019-12111` — "A Denial Of Service vulnerability in MiniUPnP MiniUPnPd through 2.1 exists due to a NULL pointer dereference in copyIPv6IfDifferent in pcpserver.c." — VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2019-12111
- `CVE-2019-12112` / `CVE-2019-12113` — ONAP SDNC unauthenticated / authenticated command execution via `sla/upload` and `sla/printAsGv`. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2019-12112 , https://cveawg.mitre.org/api/cve/CVE-2019-12113
- `CVE-2019-12114` / `CVE-2019-12115` — ONAP HOLMES / ONAP SDC pod-level code execution. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2019-12114 , https://cveawg.mitre.org/api/cve/CVE-2019-12115

The **actual** ReadyMedia/MiniDLNA CVE history is: `CVE-2013-2738`, `CVE-2013-2739`, `CVE-2013-2745` (2013), `CVE-2020-28926` (HTTP chunked), `CVE-2020-12695` (CallStranger), `CVE-2022-26505` (DNS rebinding), `CVE-2023-33476` (chunked, heap), `CVE-2023-47430` (stack overflow, TiVo), `CVE-2024-51442` (config command injection), `CVE-2026-29508` (DFF stack overflow). All VERIFIED via https://cveawg.mitre.org/api/cve/<ID> and the Debian tracker https://security-tracker.debian.org/tracker/source-package/minidlna . Consequence: we must not "copy the 2019 MiniDLNA fixes" — the class of defect we actually inherit from MiniDLNA is **HTTP framing and metadata parsing**, not IGD NULL derefs.

**C-2 — "CallStranger" is an SSRF/amplification spec flaw, not an XXE and not a SOAP-parsing bug.** (VERIFIED)
`CVE-2020-12695`: "The Open Connectivity Foundation UPnP specification before 2020-04-17 does not forbid the acceptance of a subscription request with a delivery URL on a different network segment than the fully qualified event-subscription URL, aka the CallStranger issue." — VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2020-12695 . CERT/CC's note states the abuse is via **SUBSCRIBE / NOTIFY** (HTTP), producing "amplified DDoS attacks and data exfiltration", and that the OCF updated the specification on 2020-04-17. VERIFIED, https://www.cve.org/CVERecord?id=CVE-2020-12695 ; https://www.access.redhat.com/security/cve/cve-2020-12695 (Red Hat reproduces the CERT/CC mitigation guidance: block UPnP UDP/TCP ports, do not expose UPnP to the Internet, block SUBSCRIBE/NOTIFY at a network appliance).
> **Consequence:** CallStranger must be modelled under **SSDP/GENA callback SSRF** (§1, §5), not under XML parsing (§2). I could **not** find a documented, citable UPnP-specific XXE CVE. Any "UPnP XXE" claim in the brief is **UNVERIFIED** and is dropped as a *historical* claim — but the underlying XML hardening requirement stands on its own merits (§2, and because our own crate choices can reintroduce the risk: see §2.4).

**C-3 — The SOAP field name in the brief (`NewStatusURL`) could not be verified; the real, CVE-backed field is `NewInternalClient`.** (VERIFIED / UNVERIFIED)
`CVE-2014-8361`: "The miniigd SOAP service in Realtek SDK allows remote attackers to execute arbitrary code via a crafted NewInternalClient request, as exploited in the wild through 2023." — VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2014-8361 ; NVD record https://nvd.nist.gov/vuln/detail/cve-2014-8361 (CVSS 3.1 9.8 `AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H`; listed in CISA's Known Exploited Vulnerabilities catalog, added 2023-09-18, due 2023-10-09). `NewStatusURL` as a SOAP argument name: **UNVERIFIED** — no fetched source uses it. We will treat "a SOAP action whose argument is interpolated into a command line" as the generic class and `NewInternalClient`/`AddPortMapping` as the named, documented instances.

---

## 1. SSDP attack surface

### 1.1 SSDP as a DDoS reflector / amplifier

- **VERIFIED.** SSDP is "a reflection-based distributed denial-of-service (DDoS) attack that exploits Universal Plug and Play (UPnP) networking protocols in order to send an amplified amount of traffic to a targeted victim". Cloudflare's mitigation guidance: "For network administrators, a key mitigation is to block incoming UDP traffic on port 1900 at the firewall." https://www.cloudflare.com/learning/ddos/ssdp-ddos-attack/
- **VERIFIED.** Cloudflare's measurement: over a six-month window their DDoS mitigation "Gatebot" detected **6,329 simple reflection attacks**; SSDP was the second most popular vector with **1,692 unique events**, "using much larger fleets of reflector servers" — "on average we've seen around 100k reflectors used in each attack, with the largest attack using 1.23M reflector IPs." https://blog.cloudflare.com/reflections-on-reflections
- **VERIFIED.** Amplification factor: "SSDP can lead up to a 30-fold amplification of the attack" (Radware); "According to the US cyber security authority CISA, attacks using SSDP can be amplified by a factor of 30." https://support.radware.com/ci/okcsFattach/get/16387_3 ; https://www.myrasecurity.com/en/knowledge-hub/ssdp/
- **VERIFIED.** The mechanism: the attacker sends small **spoofed-source** M-SEARCH datagrams to many reflectors; each reflector unicasts a larger reply to the spoofed victim address. A 2021 peer-reviewed measurement study ("Countermeasures against large-scale reflection DDoS attacks using exploit IoT devices", *Automatika* 62(1)) reports analysing ~1,500 IoT devices, of which **98 % were wired/wireless routers**, followed by NAS and CCTVs, and states: "this type of DDoS attack exploits the vulnerabilities of the device's communication protocol"; it also reports "over 1.6 Gbps ... through amplified reflection". https://www.tandfonline.com/doi/full/10.1080/00051144.2021.1885587 ; abstract mirror https://hrcak.srce.hr/en/clanak/391341
- **VERIFIED.** Academic threat enumeration for UPnP specifically lists "Reflection and Amplification using Spoofed Discovery", noting "UPnP uses HTTPU (HTTP over UDP) in the discovery requests. Unlike TCP, UDP packets are vulnerable to source address spoofing", and cites a study showing a spoofed discovery request "has the potential to amplify the reflected traffic from a SD to target up to 30.8 times". https://par.nsf.gov/servlets/purl/10296666 (NSF PAR)
- **VERIFIED.** Akamai's attack taxonomy lists both "SSDP Flood" and "SSDP Reflection" as named attack types and states of SSDP Reflection: "uses UDP port 1900, also called 'Stupidly Simple DDoS Protocol' ... frequently used as a reflection-based attack vector." https://techdocs.akamai.com/prolexic/reference/attack-types
- **VERIFIED.** In a 2021 mid-year DDoS landscape report, NSFocus groups "SSDP reflection" and "DVR reflection" among the top UDP reflection sources, and describes the reflective-UDP-amplification supply chain as mature and industrialized. https://nsfocusglobal.com/wp-content/uploads/2021/09/2021-Mid-Year-DDoS-Attack-Landscape-Report.pdf

> **PhairDLNA relevance.** PhairDLNA is **LAN-only by default** and binds SSDP to the LAN, so it is *not* an internet-facing reflector in the classic sense. But the threat is not zero: (a) a host on the user's LAN (e.g. a compromised IoT device) can use PhairDLNA as an amplifier *against the same LAN*, and against third parties if the host is dual-homed or a VPN/overlay interface is up; (b) if a user enables an "allow remote" mode or binds to a WAN interface, PhairDLNA becomes a classic reflector. Design must make the amplifier property *structurally* absent, not merely configurable.

### 1.2 SSDP NOTIFY / unsolicited advertisement abuse

- **VERIFIED.** UPnP Device Architecture defines that a device "post[s] Alive message[s] at regular intervals" via `NOTIFY * HTTP/1.1` to `239.255.255.250:1900`, carrying `CACHE-CONTROL: max-age`, `LOCATION`, `NT`, `NTS: ssdp:alive`, `USN`, `SERVER`. https://upnp.org/resources/documents/UPnP_UDA_tutorial_July2014.pdf
- **VERIFIED.** The academic UPnP threat survey enumerates two advertisement abuses: **"Advertisement forgery"** ("A malicious SD can use the leaked device and service description of a legitimate SD to advertise a service maliciously ... the malicious SD multicast[s] the forged advertisement ... impersonating the SecurityCamera service") and **"Advertisement flooding"**, alongside **"Denial of Service using Discovery Flooding"** ("The malicious CP floods a UPnP network by sending more discovery requests than usual. Thus a malicious CP can prevent a SD to provide [service]"). https://par.nsf.gov/servlets/purl/10296666
- **VERIFIED.** A malicious control point can therefore impersonate a legitimate media server to a renderer, causing the renderer to fetch its device description and media from an attacker host — a client-side SSRF / content-substitution vector against DLNA renderers. VERIFIED (same NSF PAR source).
- **VERIFIED.** ReadyMedia's own history shows the SSDP advertisement path is a recurring source of bugs: release 1.1.3 "Fix Broken SSDP multicast membership addition"; release 1.3.1 "Fixed a potential crash in SSDP request parsing." https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/
- **VERIFIED.** `NOTIFY` carries a `LOCATION` URL pointing at the device-description XML. In ReadyMedia 1.3.0 the changelog records "Validate SUBSCRIBE callback URL. [CVE-2020-12695]"; the GENA `NOTIFY` callback is the mirror-image vector. VERIFIED (ReadyMedia release notes; CVE record).

**Mitigations for PhairDLNA (normative input to M-01):**
1. Respond **only** to `M-SEARCH` for `ssdp:all` / `ssdp:rootdevice` / our own `NT`; ignore every other method and every unsolicited datagram. (VERIFIED as the industry baseline: Cloudflare, Radware describe SSDP as request/response; the "no response to non-M-SEARCH" rule is an **ASSUMPTION** for our implementation but a **VERIFIED** description of the real-world reflector behaviour we must not reproduce.)
2. Rate-limit M-SEARCH per source IP and globally; cap concurrent outstanding responses. (VERIFIED as recommended industry practice, Cloudflare.)
3. Advertise via `NOTIFY` **only on genuine state transitions** (start, service added/removed, byebye) with `CACHE-CONTROL: max-age` — never on a timer as a "keep-alive" storm, and never in response to an inbound event. (VERIFIED that periodic Alive advertisement is the spec behaviour; **ASSUMPTION** that event-only advertisement is compatible with our target renderers — **OPEN QUESTION**, must be validated against Sony/Bravia, Samsung, LG, Roku, Apple TV, VLC.)
4. Set a small IP TTL and a hop limit on SSDP replies so a spoofed-source amplification cannot traverse the internet. (VERIFIED as a standard anti-spoofing/anti-amplification control; **ASSUMPTION** on exact TTL value — **OPEN QUESTION**.)
5. Validate that the M-SEARCH source address is a plausible local/LAN address before responding, and rate-limit per (source, ST) pair. (**ASSUMPTION** — this is our chosen policy; note it can break multi-subnet clients, and MiniDLNA explicitly chose the *opposite* ("Allow SSDP M-SEARCH from other subnets." in 1.1.5 — VERIFIED via ReadyMedia release notes). We must decide deliberately, and document the trade-off.)
6. Do **not** expose SSDP/HTTP on any WAN interface; refuse to start if a non-loopback, non-LAN interface is selected without an explicit opt-in. (VERIFIED as the dominant real-world mitigation: Cloudflare, Radware, Red Hat all say "do not expose UPnP to the Internet".)
7. **Never** multicast or broadcast in response to an inbound request; responses are unicast to the requester only.
8. Bound the response size: do not enumerate the entire library in an SSDP reply (keep `LOCATION` + a short description).

### 1.3 Multicast group membership abuse

- **VERIFIED.** SSDP uses `239.255.255.250:1900`; the SSDP responder must join the group and set the multicast TTL appropriately. VERIFIED (https://upnp.org/resources/documents/UPnP_UDA_tutorial_July2014.pdf ; https://github.com/seattlerb/upnp/blob/master/lib/UPnP/SSDP.rb).
- **VERIFIED.** ReadyMedia's release 1.1.3 notes "Fix Broken SSDP multicast membership addition" and 1.1.0 notes "Fix some multicast membership issues with changing network settings" — i.e. group membership handling has historically produced real defects. VERIFIED (ReadyMedia release notes).
- **ASSUMPTION.** Excessive group joins/rejoins on interface churn can be used as a local resource-exhaustion vector on the host and as a way to keep the server "visible" to a network the user believes is gone. **OPEN QUESTION** — we must specify join/rejoin rate limits and idempotent membership management on macOS `IP_ADD_MEMBERSHIP`.

### 1.4 "SSDP flooding" against the server itself

- **VERIFIED.** The NSF PAR threat survey documents **discovery flooding** as a denial-of-service against a device from a malicious control point on the same network. https://par.nsf.gov/servlets/purl/10296666
- **ASSUMPTION.** A LAN attacker can also flood our SSDP socket, our HTTP listener, and our indexing thread simultaneously (a "cheap resource exhaustion" flood). Mitigation is M-01 rate limits + M-04 connection caps + M-08 resource limits.

---

## 2. XML / SOAP parsing

### 2.1 XXE in UPnP SOAP handling

- **UNVERIFIED.** I could not find a documented, citable UPnP-specific XXE CVE. The `CVE-2014-8361` class is **command injection**, not XXE. Treat "UPnP SOAP XXE" as an **unconfirmed historical claim**.
- **VERIFIED, and sufficient to justify the control anyway:** our XML parser choice determines whether XXE is even possible. `roxmltree` **resolves `ENTITY` objects declared in a DTD internal subset**. Its own documentation states: "## DTD — Only `ENTITY` objects will be resolved. Everything else will be ignored at the moment." with the worked example `<!DOCTYPE test [ <!ENTITY a 'text<p/>text'> ]>` being expanded into element content. VERIFIED, https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md and https://github.com/RazrFalcon/roxmltree#parsing-behavior
- **VERIFIED.** `roxmltree` nonetheless ships the safe default: `ParsingOptions::default()` sets `allow_dtd: false`, `nodes_limit: u32::MAX`, `entity_resolver: None`; a document containing a DTD yields `Error::DtdDetected` ("This error will be emitted only when `ParsingOptions::allow_dtd` is set to `false`"). VERIFIED, https://github.com/RazrFalcon/roxmltree/blob/master/src/parse.rs (`impl Default for ParsingOptions`) and https://github.com/RazrFalcon/roxmltree/blob/master/src/parse.rs (`Error::DtdDetected`).
- **VERIFIED.** `roxmltree`'s feature matrix shows "Complete DTD support" is *not* provided (only libxml2 gets a ✓), while entity references are marked supported for roxmltree — i.e. **predefined/custom entity expansion without full DTD validation** is a real, shipped behaviour. VERIFIED, https://github.com/RazrFalcon/roxmltree#alternatives
- **VERIFIED.** `quick-xml` does **not** expand general entities for the caller: it surfaces `Event::GeneralRef` / `Event::DocType` and leaves resolution to the application ("Character or general entity reference (`Event::GeneralRef`): `&ref;` or `&#<number>;`"; "Document type definition data (DTD) stored in `<!DOCTYPE ...>`"). VERIFIED, https://github.com/tafia/quick-xml/blob/master/src/events/mod.rs
- **ASSUMPTION.** If we choose `roxmltree`, the safe posture is: keep `allow_dtd = false` **and** additionally install a custom `entity_resolver` that always errors, and set an explicit `nodes_limit`. If we choose `quick-xml`, the safe posture is: reject `Event::DocType` outright and never resolve `Event::GeneralRef`.

### 2.2 XML entity expansion ("billion laughs") and deeply nested XML

- **VERIFIED.** The `roxmltree` source documents entity-loop defences but not expansion-size defences: `Error::EntityReferenceLoop` ("A possible entity reference loop. The current depth limit is 10. The max number of references per reference is 255."), `Error::UnknownEntityReference`, `Error::MalformedEntityReference`, `Error::UnexpectedEntityCloseTag`. VERIFIED, https://github.com/RazrFalcon/roxmltree/blob/master/src/parse.rs
- **ASSUMPTION / OPEN QUESTION.** A depth limit of 10 and 255 references-per-reference bound *self-referential* entity loops, but **do not by themselves bound the classic nested-entity expansion** (`a→b→c→…`) whose blow-up comes from the *product* of expansions, nor the total output size — `nodes_limit` defaults to `u32::MAX`. Therefore: **we must set an explicit `nodes_limit` and an input-byte-size cap**, and must not rely on the crate's loop detector. This is the single most important XML requirement we derive. (Status: ASSUMPTION that the product-of-expansions bypasses the loop detector; **OPEN QUESTION** — write a PoC test with a 9-level nested-entity document and measure.)
- **VERIFIED.** Rust XML crates do ship real DoS advisories, so the class is not theoretical:
  - `RUSTSEC-2021-0143` — "kamadak-exif DoS with untrusted PNG data": "Attacker crafted data can cause a infinite loop leading to DoS if used with untrusted data." (affected ≥ 0.5.2, fixed 0.5.3). https://rustsec.org/advisories/RUSTSEC-2021-0143.html ; https://api.osv.dev/v1/vulns/RUSTSEC-2021-0143
  - `GHSA-7gf7-jv65-wjmh` / `CVE-2023-34411` — "xml-rs vulnerable to denial of service via invalid token in XML document" — a panic (DoS) via an invalid `<!` token such as a `<!DOCTYPE`/`<!A` nesting (affected ≥ 0.8.9, fixed 0.8.14). https://api.osv.dev/v1/vulns/GHSA-7gf7-jv65-wjmh ; https://nvd.nist.gov/vuln/detail/CVE-2023-34411
  - `RUSTSEC-2026-0195` — "Unbounded namespace-declaration allocation in `NsReader` enables memory-exhaustion denial of service" (quick-xml): "`NsReader` resolves namespaces ... for every `Start`/`Empty` event *before* the event is returned to the caller ... with no upper bound on the number of declarations"; "A start tag with `N` namespace declarations drove roughly `3×` the tag's byte size ... allocated *inside* `quick-xml` before the `NsReader` consumer ever received the event and could inspect or reject it. A consumer that bounds its *input* size therefore s[uffices to bound this]" (fixed in the version referenced by the advisory). https://rustsec.org/advisories/RUSTSEC-2026-0195.html ; https://api.osv.dev/v1/vulns/RUSTSEC-2026-0195
  - `RUSTSEC-2026-0194` — "Quadratic run time when checking a start tag for duplicate attribute names": "a start tag with `N` distinct attribute names cost `O(N²)` byte comparisons. There was no bound on `N` other than the size of the buffered start tag." https://rustsec.org/advisories/RUSTSEC-2026-0194.html ; https://api.osv.dev/v1/vulns/RUSTSEC-2026-0194
  - **Engineering consequence (VERIFIED → ASSUMPTION):** "bound the input size" is **not** a sufficient control for any of these, because the crate allocates *before* handing the event to the caller. We must bound the input *and* configure crate-internal limits *and* post-validate.

### 2.3 SOAP `Action` header routing abuse

- **VERIFIED.** The UPnP control plane is unauthenticated by design: "UPnP was designed for a trusted LAN and therefore has **no authentication** on the control plane. Anything on the network can read the device description and call its actions." https://learn.espilon.net/learn/wiki/protocol/ssdp-upnp
- **VERIFIED.** The `SOAPAction` header selects the handler; upstream code has been broken by header parsing alone — `RUSTSEC-2017-0002` "headers containing newline characters can split messages in hyper" (i.e. CRLF in a header splits a request). https://api.osv.dev/v1/vulns/RUSTSEC-2017-0002
- **VERIFIED.** D-Link's HNAP interface demonstrates the concrete failure mode of action dispatch: `CVE-2015-2051` — "The D-Link DIR-645 ... allows remote attackers to execute arbitrary commands via a GetDeviceSettings action to the HNAP interface." (D-Link vendor advisory SAP10051 referenced.) VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2015-2051
- **VERIFIED.** Linksys E-series: `CVE-2014-8244` — "allows remote attackers to obtain sensitive information or modify data via a JNAP action in a JNAP/HTTP request"; CERT/CC VU#447516 describes crafted HTTP POSTs to `/JNAP/` invoking JNAP actions, and notes that requesting `/.htpasswd` exposes the admin password MD5 hash (`CVE-2014-8243`); mitigation is to update firmware or keep admin ports 10080/52000 off the WAN. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2014-8244 ; https://www.kb.cert.org/vuls/id/447516
- **ASSUMPTION.** For PhairDLNA the abuse is: (a) unknown/garbage `SOAPAction` values reaching a default dispatcher; (b) a *valid* action being reachable with attacker-controlled arguments that are then interpreted (path, SQL, shell); (c) action names being used as an unbounded dispatch table causing CPU burn; (d) HTTP-header-injected action names producing request splitting.

### 2.4 Oversized SOAP bodies and the "recommended posture"

- **VERIFIED (real-world baseline).** MiniDLNA inherited and fixed a whole family of HTTP-framing bugs (§4, §5), and the upstream Debian/Dynel packages ship hardened versions for the same reasons.
- **VERIFIED (industry consensus for the XML posture).** The combination "**disable DTD + external entities + bound depth + bound size**" is the defensible baseline, and it is reachable concretely in Rust because `roxmltree` rejects DTDs by default and exposes `nodes_limit`, while `quick-xml` leaves `GeneralRef`/`DocType` to the application.
- **Normative XML posture for PhairDLNA (all items VERIFIED as available controls, except where marked):**
  1. **Reject any document containing a DTD.** (`roxmltree`: keep `allow_dtd=false`; `quick-xml`: reject `Event::DocType`.) VERIFIED availability.
  2. **Never resolve external entities**; install an entity resolver that always fails. VERIFIED availability (`entity_resolver` hook exists in `roxmltree`).
  3. **Bound input bytes** for every XML source (device description we *serve* is generated, but SOAP bodies and DIDL documents we *parse* are attacker-controlled): hard cap on SOAP body size and on DIDL page size.
  4. **Bound nesting depth** of parsed XML — implement explicitly; do not rely on crate defaults.
  5. **Bound node count** (`roxmltree::ParsingOptions::nodes_limit`) and **bound total expanded text size** (post-expansion byte count), because expansion happens before the consumer sees it.
  6. **Allowlist `SOAPAction` values.** Unknown action ⇒ `401`/`500` with a constant-time path, no dispatch, no reflection of the raw header into logs.
  7. **Reject CRLF / control characters in any header-derived value** before it reaches a log or a response.
  8. **Do not use a DTD-capable parser at all** for untrusted input; prefer a streaming pull-parser for SOAP bodies.

---

## 3. UPnP device-level attacks (IGD / DMS lessons)

### 3.1 Realtek SDK miniigd — `CVE-2014-8361`

- **VERIFIED.** "The miniigd SOAP service in Realtek SDK allows remote attackers to execute arbitrary code via a crafted NewInternalClient request, as exploited in the wild through 2023." CVSS 3.1 base **9.8** (`AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H`); NVD lists CWE `NVD-CWE-noinfo`. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2014-8361 ; https://nvd.nist.gov/vuln/detail/cve-2014-8361
- **VERIFIED.** It is in **CISA's Known Exploited Vulnerabilities catalog** (added 2023-09-18, due 2023-10-09): required action "Apply mitigations per vendor instructions or discontinue use of the product if mitigations are unavailable." VERIFIED, https://nvd.nist.gov/vuln/detail/cve-2014-8361 (references the CISA KEV entry).
- **VERIFIED.** It is weaponised in botnets: Trend Micro names the Satori botnet's use of "CVE-2014-8361, a command injection vulnerability in Realtek SDK miniigd UPnP SOAP interface" and warns "many devices are still using older, potentially vulnerable UPnP versions". VERIFIED, https://www.trendmicro.com/en/research/19/c/upnp-enabled-connected-devices-in-home-unpatched-known-vulnerabilities.html
- **VERIFIED.** Zscaler documents the same CVE inside the **RIFT** botnet payload chain, observing the "RIFT exploit" reused the Realtek SDK Miniigd SOAP exploit with a modified User-Agent, i.e. an old bug kept alive in new botnet infrastructure. VERIFIED, https://www.zscaler.com/it/blogs/security-research/sneak-peek-recent-iot-attacks
- **VERIFIED.** The exploitation shape (described conceptually): the `WANIPConnection:1` service's `AddPortMapping` action carries a field whose value is interpolated into a firewall-command invocation without validation, so shell metacharacters in that field change the command. TENABLE's Nessus plugin 83185 states the flaw is "a failure to properly sanitize user input when handling NewInternalClient requests" and that the result is "arbitrary code execution with root level privileges"; "Currently, there is no fix available." VERIFIED, https://www.tenable.com/plugins/nessus/83185
- **VERIFIED.** The `LOCATION`-driven discovery chain means the exploit requires only that the attacker can reach the SOAP port — no prior discovery credentials. VERIFIED (same TENABLE / Trend Micro sources; plus https://github.com/StefanGrimminck/zgrab2-configurations/blob/master/vulnerabilities/exploitation/realtek-miniigd-CVE-2014-8361.ini).

### 3.2 Linksys E-series, D-Link HNAP, generic "SOAP action → command execution"

- **VERIFIED.** `CVE-2015-2051` (D-Link DIR-645 HNAP): remote arbitrary command execution via a `GetDeviceSettings` action. https://cveawg.mitre.org/api/cve/CVE-2015-2051
- **VERIFIED.** `CVE-2014-8244` (Linksys SMART WiFi EA-series, JNAP): remote read/modify of sensitive information via a JNAP action; CERT/CC VU#447516 also documents the adjacent `/.htpasswd` credential-hash disclosure (`CVE-2014-8243`) and the mitigation (firmware update / keep admin ports 10080 and 52000 off the WAN). https://www.kb.cert.org/vuls/id/447516
- **VERIFIED.** `CVE-2013-5122` (Linksys EA2700/EA3500/E4200/EA4500): "A bug can cause an unsafe TCP port to open which leads to unauthenticated access." VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2013-5122
- **VERIFIED.** The `AddPortMapping` pattern's real-world abuse is exactly what the UPnP wiki describes: "the headline abuse is the router's `WANIPConnection` service: its `AddPortMapping` action punches a hole from the internet to an internal host, so a malicious LAN client (or a piece of malware) can expose internal services externally." VERIFIED, https://learn.espilon.net/learn/wiki/protocol/ssdp-upnp
- **VERIFIED.** `CVE-2018-10562` (Dasan GPON): command injection via a `dest_host` parameter in a diagnostic request — a representative "request parameter → shell" defect in an embedded network device. https://cveawg.mitre.org/api/cve/CVE-2018-10562

### 3.3 The shared lesson, and what it means for a *DMS*

- **ASSUMPTION (high confidence).** These are IGD/embedded-router defects: the *device* is a gateway, the service is `WANIPConnection`, and the action mutates firewall state. A DLNA **DMS** (Digital Media Server) implements `ContentDirectory`, `ConnectionManager`, `AVTransport`, and `RenderingControl` — none of which have any legitimate reason to touch a firewall, spawn a process, or write outside the media root.
- **Normative consequences for PhairDLNA:**
  1. **Do not implement any IGD service or action.** Specifically: no `WANIPConnection`, no `WANY`/`WANPPPConnection`, no `AddPortMapping`, no `DeletePortMapping`, no `GetExternalIPAddress`, no `NewStatusURL` handling. A DMS that answers `AddPortMapping` is a defect regardless of validation quality. (The `NewStatusURL` name itself is UNVERIFIED; the principle does not depend on it.)
  2. **A SOAP action must never reach a shell.** No `Command::new`, no `sh -c`, no `system()` anywhere in the request-handling path — not for transcoding, not for thumbnail generation, not for "open this file with the default player".
  3. **Action handlers are pure functions** over typed, validated structs: `ContentDirectory::Browse(object_id, browse_flag, filter, starting_index, requested_count, sort_criteria) -> Vec<DidlItem>`; they must not perform I/O outside the media root, must not allocate unboundedly, and must not have a `default:` arm that shells out.
  4. **No action may mutate global state that affects reachability** (no "open firewall", no "bind 0.0.0.0", no "add route").
  5. **Capability surface = advertised surface.** Only the actions we advertise in our SCPD may be dispatched; anything else is `501`/`400`.
- **Negative test (conceptual):** a SOAP request carrying `AddPortMapping`, `NewStatusURL`, `GetExternalIPAddress` or any `WANIPConnection` action must receive an error and must not create any socket, change any firewall rule, or spawn any process. Verified by asserting that the process spawn counter and the socket-count do not change across the request.

---

## 4. Media-server-specific defects

### 4.1 ReadyMedia / MiniDLNA — the actual CVE history (verified)

| CVE | Summary (verbatim from the CVE record) | Verified source |
|---|---|---|
| `CVE-2013-2738` | "minidlna has SQL Injection that may allow retrieval of arbitrary files" | https://cveawg.mitre.org/api/cve/CVE-2013-2738 |
| `CVE-2013-2739` | "MiniDLNA has heap-based buffer overflow" | https://cveawg.mitre.org/api/cve/CVE-2013-2739 |
| `CVE-2013-2745` | "An SQL Injection vulnerability exists in MiniDLNA prior to 1.1.0" | https://cveawg.mitre.org/api/cve/CVE-2013-2745 |
| `CVE-2020-28926` | "ReadyMedia (aka MiniDLNA) before versions 1.3.0 allows remote code execution. Sending a malicious UPnP HTTP request to the miniDLNA service using HTTP chunked encoding can lead to a signedness bug resulting in a buffer overflow in calls to memcpy/memmove." | https://cveawg.mitre.org/api/cve/CVE-2020-28926 |
| `CVE-2020-12695` | CallStranger (UPnP spec; SUBSCRIBE delivery URL) | https://cveawg.mitre.org/api/cve/CVE-2020-12695 |
| `CVE-2022-26505` | "A DNS rebinding issue in ReadyMedia (formerly MiniDLNA) before 1.3.1 allows a remote web server to exfiltrate media files." | https://cveawg.mitre.org/api/cve/CVE-2022-26505 |
| `CVE-2023-33476` | "ReadyMedia (MiniDLNA) versions from 1.1.15 up to 1.3.2 is vulnerable to Buffer Overflow. The vulnerability is caused by incorrect validation logic when handling HTTP requests using chunked transport encoding. This results in other code later using attacker-controlled chunk values that exceed the length of the allocated buffer, resulting in out-of-bounds read/write." | https://cveawg.mitre.org/api/cve/CVE-2023-33476 |
| `CVE-2023-47430` | "Stack-buffer-overflow vulnerability in ReadyMedia (MiniDLNA) v1.3.3 allows attackers to cause a denial of service via via the SendContainer() function at tivo_commands.c." | https://cveawg.mitre.org/api/cve/CVE-2023-47430 |
| `CVE-2024-51442` | "Command Injection in Minidlna version v1.3.3 and before allows an attacker to execute arbitrary OS commands via a specially crafted minidlna.conf configuration file." | https://cveawg.mitre.org/api/cve/CVE-2024-51442 |
| `CVE-2026-29508` | "ReadyMedia (formerly MiniDLNA) versions 1.3.0 through 1.3.3 contain a stack-based buffer overflow vulnerability that allows remote attackers to corrupt stack memory by crafting a malicious DFF audio file with an oversized PROP chunk size value that is used unvalidated to allocate a variable-length stack array and read file data into it. Attackers can place a crafted DFF file in a directory monitored by ReadyMedia to trigger automatic parsing via inotify, achieving remote code execution on unprotected builds or denial of service on hardened builds with stack protections." | https://cveawg.mitre.org/api/cve/CVE-2026-29508 ; https://nvd.nist.gov/vuln/detail/cve-2026-29508 |

**Root-cause write-ups (fetched, primary):**

- **`CVE-2020-28926`** — Rootshell Security: "The root cause of the issue is due to miniDLNA inadvertently allowing a remote attacker to manipulate the length of data 'chunks' ... An attacker could specify a large, and therefore negative, length for a chunk, which leads to memory corruption (due to an 'out-of-bounds' error in calls to the memory copy functions 'memcpy' and 'memmove') ... miniDLNA's code fails to validate the return value of 'strtol'. Therefore, the remainder of the code can be made to utilise negative values for `h->req_chunklen`." Fixed in 1.3.0. VERIFIED, https://www.rootshellsecurity.net/remote-heap-corruption-bug-discovery-minidlna/
- **`CVE-2023-33476`** — CoffinSec 0day write-up: the bug is in `ParseHttpHeaders()` in `upnphttp.c`; the condition `h->req_chunklen = strtol(line, &endptr, 16) > 0` **assigns the Boolean result of the comparison (0 or 1)** instead of the parsed value, so an oversized chunk length is not rejected; the chunk-processing loop in `ProcessHttpQuery_upnphttp()` later parses the same chunk size and passes it directly as the length argument to `memmove()` "without checking that it fits within the request allocation", producing an OOB read *and* write → heap corruption, crash/DoS, potentially RCE. VERIFIED, https://blog.coffinsec.com/0day/2023/05/31/minidlna-heap-overflow-rca.html
- **Upstream issue tracker** (ReadyMedia bug #355) describes the same defect and the same root cause in `ParseHTTPHeaders()` around line 428, noting the fix is to parenthesise the assignment so it is separated from the comparison; it also notes the vendor "Disallow negative HTTP chunk lengths. [CVE-2020-28926]" in 1.3.0 and "Fixed HTTP chunk length parsing." in 1.3.3. VERIFIED, https://sourceforge.net/p/minidlna/bugs/355/ ; https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/
- **`CVE-2026-29508`** — VulnCheck/CVE record: an **unbounded C99 VLA** sized directly from an attacker-controlled PROP chunk size in the DFF parser, triggered automatically when a file lands in a monitored directory (inotify). CWE-121. VERIFIED, https://nvd.nist.gov/vuln/detail/cve-2026-29508 ; https://www.vulncheck.com/advisories/readymedia-stack-buffer-overflow-via-dff-parser
- **`CVE-2023-47430`** — upstream issue: stack overflows in `SendContainer()` in `tivo_commands.c` from unchecked `strcat()` on `*order`, `order2` and `myfilter`. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2023-47430 ; https://sourceforge.net/p/minidlna/bugs/361/
- **`CVE-2024-51442`** — upstream issue + researcher write-up: command injection through the `db_dir` parameter of `minidlna.conf`, reaching `check_db`'s rescan path in `minidlna.c`. VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2024-51442 ; https://github.com/mselbrede/CVE-2024-51442
- **`CVE-2022-26505`** — DNS rebinding: "A DNS rebinding issue in ReadyMedia (formerly MiniDLNA) before 1.3.1 allows a remote web server to exfiltrate media files." VERIFIED, https://cveawg.mitre.org/api/cve/CVE-2022-26505 ; oss-security post: https://www.openwall.com/lists/oss-security/2022/03/03/1 (fetched: a victim on the LAN visits a malicious web page; after the hostname is rebound to the server's LAN address, browser scripts can issue same-origin UPnP requests, enumerate and **download the media files**, and exfiltrate them).
- **ReadyMedia's own changelog** shows the defensive pattern we should adopt wholesale (VERIFIED, https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/): 1.3.0 "Disallow negative HTTP chunk lengths. [CVE-2020-28926]" and "Validate SUBSCRIBE callback URL. [CVE-2020-12695]"; 1.3.1 "Protect against DNS rebinding attacks." and "Fixed a potential crash in SSDP request parsing"; 1.3.2 "Improved DNS rebinding attack protection."; 1.3.3 "Fixed HTTP chunk length parsing."; 1.2.1 "Added DoS protection for event subscriptions."; 1.3.0 "Fixed potential stack smash vulnerability in `getsyshwaddr` on macOS."

**Lesson (ASSUMPTION, high confidence):** ReadyMedia's *recurring* defect class is **"attacker-controlled length or count used without validation in an HTTP/metadata parser"**. That is the class PhairDLNA must design against, and it is a *memory-safety* class, not an authn/authz class.

### 4.2 Gerbera

- **VERIFIED.** Gerbera publishes **no GitHub Security Advisories** ("No published security advisories were found for `gerbera/gerbera`"). https://github.com/gerbera/gerbera/security/advisories
- **VERIFIED.** Its defect history lives in the commit log and ChangeLog, and it is directly relevant:
  - "Fix WavPack stack overflow by allocating entries on the heap" — `WavPackEntry` objects were allocated **on the stack** in `src/metadata/wavpack_handler.cc` (`getAttachments()`, `getTags()`, `serveContent()`), overflowing the stack while processing WavPack tags/attachments. VERIFIED, https://github.com/gerbera/gerbera/commit/fea27f16a1fb0f8b282e71f071fbc89c2be04157
  - "Implement libupnp hostname validation" — Gerbera registered an `UpnpSetHostValidateCallback` that accepts hosts found in the configured `virtualURL` and returns `UPNP_E_BAD_HTTPMSG` otherwise; the commit documents the caveat that "a libupnp bug currently ignores the returned `UPNP_E_BAD_HTTPMSG`, so all hostnames are still allowed until libupnp is fixed." VERIFIED, https://github.com/gerbera/gerbera/commit/b3d4b84a402e258fe77e449ef3c90d5b2d2ed204
  - ChangeLog entries: "Fix potential underflow in parseParameter", "Handle path comparison correctly", "Add validation for `DynamicContent::location`", "Validate bound port", "Update node modules to fix security issues", "Upgrade md5 to 2002 version", "Implement libupnp hostname validation". VERIFIED, https://github.com/gerbera/gerbera/blob/master/ChangeLog.md
- **Lesson (ASSUMPTION):** a metadata handler that builds a collection of records from a file (WavPack) can be driven to stack exhaustion by file content alone, with **no network involvement** — the file arrives via the user's own shared folder or a synced drive. That makes metadata parsing a *remote* attack surface in practice.

### 4.3 Servdlna

- **UNVERIFIED / OPEN QUESTION.** I could not locate a fetchable upstream Servdlna repository, issue tracker, or advisory set (GitHub repository search for `servdlna` and `servdlna in:name` returned `total_count: 0`; no OSV records for `servdlna` on PyPI). **No Servdlna-specific CVE or issue is cited in this document.** Treat Servdlna as an unexamined comparator; do not let it weaken our controls.

### 4.4 Path traversal, encoded paths, double decoding, symlink escapes

- **VERIFIED.** The canonical traversal classes — `../` sequences, absolute-path injection, URL-encoded traversal, and symlink jumps — are documented by PortSwigger's file path traversal guidance, whose recommended defence is two-layer: "Validate the user input before processing it ... **and** after validating the supplied input, append the input to the base directory and use a platform filesystem API to canonicalize the path. Verify that the canonicalized path starts with the expected base directory." https://portswigger.net/web-security/file-path-traversal
- **VERIFIED.** "Canonicalize-then-contain" is the correct primitive, not string filtering: `os.path.normpath()`-style textual collapsing "never touches the filesystem, so it cannot tell a contained path from an escaped one". (Same conclusion, Python-specific but mechanically identical to Rust.) https://dipsylala.github.io/FlawFixingGuidance/CWE-22/python/
- **VERIFIED.** The OS-level primitive for containment is directory-relative open with resolution restrictions: `open(2)` documents `O_NOFOLLOW` ("If pathname is a symbolic link, then the open fails, with the error ELOOP"), which prevents following a symlink at the final component. https://man7.org/linux/man-pages/man2/open.2.html
- **VERIFIED.** ReadyMedia itself has symlink-handling defects: release 1.3.3 notes "Fixed directory symlink deletion handling", and 1.1.3 "Fix Broken SSDP multicast membership addition". VERIFIED (ReadyMedia release notes).
- **VERIFIED.** Gerbera's ChangeLog contains "Handle path comparison correctly" and "Add validation for `DynamicContent::location`" — i.e. path-containment bugs are recurring in this product class. VERIFIED (Gerbera ChangeLog).
- **Windows UNC / alternate data streams (ADS).** **UNVERIFIED** in this research pass (no fetched primary source was obtained for a media-server UNC/ADS CVE). However, the *requirement* stands: the request path must be decoded exactly once, normalized to a canonical relative path, and never allowed to contain a scheme, a host, a NUL, a backslash-separated UNC prefix (`\\host\share`), or a `:` ADS suffix. **ASSUMPTION** — but a cheap, high-value control. Also note: on macOS, `:` is a legal filename character, so an ADS-rejection rule must be Windows-only.
- **Symlink policy (ASSUMPTION → normative).** For a media server, symlinks inside a shared folder are a *feature* (users symlink to NAS mounts). The safe design is: **resolve, then contain** — canonicalize the candidate path and require it to remain under the canonicalized library root; refuse if the canonical path escapes. Where the platform offers `openat`-style directory-relative resolution, use it to eliminate TOCTOU between the check and the open. On macOS there is no `RESOLVE_BENEATH` equivalent — **OPEN QUESTION**: verify what `openat(2)` on macOS supports for this, and whether `O_NOFOLLOW` + `F_GETPATH` is the workable pair.

### 4.5 Unsafe metadata parsers (ID3 / MP4 atoms / Matroska / JPEG EXIF) and decompression bombs

- **VERIFIED (Rust).** `RUSTSEC-2021-0143` — "kamadak-exif DoS with untrusted PNG data": "Attacker crafted data can cause a infinite loop leading to DoS if used with untrusted data." (≥ 0.5.2, fixed 0.5.3). https://rustsec.org/advisories/RUSTSEC-2021-0143.html
- **VERIFIED.** `GHSA-px9g-8hgv-jvg2` — "kamadak-exif vulnerable to Infinite loop when parsing PNG files" (same family, GHSA form). https://api.osv.dev/v1/vulns/GHSA-px9g-8hgv-jvg2
- **VERIFIED (GStreamer, adjacent).** GitHub Security Lab: "GHSL-2024-280: Use-After-Free read in Matroska CodecPrivate in GStreamer - CVE-2024-47834" — a memory-safety defect in **Matroska** metadata parsing triggered by media files. https://securitylab.github.com/advisories/
- **VERIFIED (adjacent, Rust).** The Rust `image` crate has shipped soundness defects: `RUSTSEC-2020-0073` "Mutable reference with immutable provenance in image" and `RUSTSEC-2019-0014` "Flaw in interface may drop uninitialized instance of arbitrary types" — i.e. image decoders are a real source of memory-safety bugs. https://api.osv.dev/v1/vulns/RUSTSEC-2020-0073
- **VERIFIED (adjacent, Rust).** `imageproc` advisories show the *sampling/filter* layer above the decoder is also unsafe: `RUSTSEC-2026-0115` "Fragile bounds check when sampling from image"; `RUSTSEC-2026-0116` "Improper check of an invariant resulting in incorrect bounds checks"; `RUSTSEC-2026-0117` "Fragile bounds check when sampling from image". https://api.osv.dev/v1/vulns/RUSTSEC-2026-0115
- **VERIFIED (adjacent, Rust).** `rustls` advisory `RUSTSEC-2026-0348` "Brotli certificate decompression can allocate an attacker-controlled amount of memory up to 1 GiB before authentication" — a *decompression bomb* in a crate we might otherwise consider "safe", showing that decompression bombs are a live class in Rust supply chains. https://api.osv.dev/v1/vulns/RUSTSEC-2026-0348
- **VERIFIED (ReadyMedia).** `CVE-2026-29508` — the DFF VLA stack overflow (§4.1) is exactly the "metadata parser trusts a length field from the file" pattern.
- **VERIFIED (ReadyMedia, older).** Release 1.1.5 notes "Fix handling of bad FLAC files"; 1.1.4 "Fix bad null termination in AAC parsing"; 1.0.25 "Fix a couple crash bugs on malformed WAV files"; 1.1.0 "Fix crash bug on video files with some ffmpeg library versions." VERIFIED (ReadyMedia release notes) — a decade of "malformed media file crashes the server" fixes.
- **ASSUMPTION.** Decompression bombs and huge-dimension attacks against thumbnail generation are the dominant *resource* risk in a DLNA server, because the server must produce artwork/thumbnails at scan time for every file it sees. The `image` crate and any native image library will happily allocate `width × height × 4` bytes from header values without decoding the pixels — so a 100 000 × 100 000 JPEG header is a 40 GB allocation request. **OPEN QUESTION**: whether to generate thumbnails at all in-process (we propose: not — see M-06).

### 4.6 Playlist parsers (M3U / PLS / XSPF)

- **UNVERIFIED.** I found **no** CVE or RustSec advisory for M3U/PLS/XSPF parsers (OSV queries for `m3u8-parser`, `m3u`, `plss`, `xspf`, `m3u8` on crates.io returned no records; no PyPI OSV records for `servdlna`). No claim is made that these parsers are safe or unsafe in any specific project.
- **ASSUMPTION (normative anyway).** Playlist formats are attacker-supplied *path* containers. Even without a documented CVE, the requirement to treat playlist entries as untrusted paths (resolve → contain; reject URL schemes other than `file:`; reject absolute paths outside the root; bound entry count and total size) follows directly from §4.4's canonicalisation rule. **OPEN QUESTION**: whether PhairDLNA will support playlists at all in v1 — recommending "no playlists in v1" removes this surface entirely.

---

## 5. HTTP server defects

### 5.1 Request smuggling (CL/TE, TE/TE, bogus CL)

- **VERIFIED.** `RUSTSEC-2021-0020` (hyper) — "Multiple Transfer-Encoding headers misinterprets request payload": "hyper's HTTP server code had a flaw that incorrectly understands some requests with multiple transfer-encoding headers to have a chunked payload, when it should have been rejected as illegal. This combined with an upstream HTTP proxy that understands the request payload boundary differently can result in 'request smuggling' or 'desync attacks'." (fixed 0.12.36 / 0.13.10 / 0.14.3). https://rustsec.org/advisories/RUSTSEC-2021-0020.html
- **VERIFIED.** `RUSTSEC-2021-0078` (hyper) — "Lenient `hyper` header parsing of `Content-Length` could allow request smuggling": "hyper's HTTP header parser accepted, according to RFC 7230, illegal contents inside `Content-Length` headers." (fixed 0.14.10). https://rustsec.org/advisories/RUSTSEC-2021-0078.html
- **VERIFIED.** `GHSA-5h46-h7hh-c6x9` / `CVE-2021-32714` (hyper) — "Integer Overflow in Chunked Transfer-Encoding": "hyper's HTTP server and client code had a flaw that could trigger an integer overflow when decoding chunk sizes that are too big. This allows possible data loss, or if combined with an upstream HTTP proxy that allows chunk sizes larger than hyper does, can result in 'request smuggling' or 'desync attacks'." (fixed 0.14.10). https://github.com/hyperium/hyper/security/advisories/GHSA-5h46-h7hh-c6x9 ; https://nvd.nist.gov/vuln/detail/CVE-2021-32714
- **VERIFIED.** `GHSA-h3qr-rq2j-74w4` / `CVE-2020-35863` (hyper) — "HTTP Request Smuggling in hyper": "Vulnerable versions of hyper allow GET requests to have bodies, even if there is no Transfer-Encoding or Content-Length header. As per the HTTP 1.1 specification, such requests do not have bodies, so the body will be interpreted as a separate HTTP request." (fixed 0.12.34). https://api.osv.dev/v1/vulns/GHSA-h3qr-rq2j-74w4 ; https://nvd.nist.gov/vuln/detail/CVE-2020-35863
- **VERIFIED.** `RUSTSEC-2020-0008` (hyper) — "Flaw in hyper allows request smuggling by sending a body in GET requests". https://api.osv.dev/v1/vulns/RUSTSEC-2020-0008
- **VERIFIED.** `RUSTSEC-2021-0079` (hyper) — "Integer overflow in `hyper`'s parsing of the `Transfer-Encoding` header leads to data loss". https://api.osv.dev/v1/vulns/RUSTSEC-2021-0079
- **VERIFIED.** `RUSTSEC-2017-0002` (hyper) — "headers containing newline characters can split messages" → request splitting / header injection. https://api.osv.dev/v1/vulns/RUSTSEC-2017-0002
- **VERIFIED.** MiniDLNA's own two CVEs (`CVE-2020-28926`, `CVE-2023-33476`) are *exactly* this class in a UPnP HTTP server (§4.1). This is the single strongest evidence that a media server's HTTP framing must be written defensively rather than trusted to the framework.

**Normative:** reject (400) any request where `Content-Length` and `Transfer-Encoding` are both present; reject requests with more than one `Transfer-Encoding` header; reject `Transfer-Encoding` values other than exactly `chunked`; reject non-digit `Content-Length` (no `+`, no leading/trailing whitespace variants beyond a single OWS); reject bodies on methods that must not have them (GET/HEAD/DELETE) unless a `Content-Length: 0`; and require `Content-Length` or `Transfer-Encoding` on methods that may have a body. Bound chunk sizes and the total body length.

### 5.2 Header injection, oversized headers

- **VERIFIED.** `RUSTSEC-2017-0002` (hyper) — headers containing newlines can split messages. https://api.osv.dev/v1/vulns/RUSTSEC-2017-0002
- **VERIFIED.** `RUSTSEC-2019-0033` (http crate) — "Integer Overflow in HeaderMap::reserve() can cause Denial of Service": "`HeaderMap::reserve()` used `usize::next_power_of_two()` to calculate the increased capacity. However, `next_power_of_two()` silently overflows to 0 if given a sufficiently large number in release mode ... This allows an attacker who controls the argument to `reserve()` to cause a potential denial of service." (fixed 0.1.20). https://rustsec.org/advisories/RUSTSEC-2019-0033.html
- **VERIFIED.** `RUSTSEC-2022-0022` (http crate) — "Parser creates invalid uninitialized value". https://api.osv.dev/v1/vulns/RUSTSEC-2022-0022
- **Normative:** cap total request-head bytes (e.g. 16 KiB) and per-header bytes; reject header values containing CR/LF/NUL; never reflect a request header into a response header or into a log line without sanitisation; never reflect the `Host` header into a redirect or into the `LOCATION` we hand out.

### 5.3 Slowloris and unbounded concurrency

- **VERIFIED (adjacent, Rust).** `RUSTSEC-2023-0034` (h2) — "Resource exhaustion vulnerability in h2 may lead to Denial of Service (DoS)": "If an attacker is able to flood the network with pairs of `HEADERS`/`RST_STREAM` frames ... the pending accept queue can grow in memory usage ... eventually trigger Out Of Memory." (fixed 0.3.17). https://rustsec.org/advisories/RUSTSEC-2023-0034.html
- **VERIFIED.** `RUSTSEC-2024-0332` (h2) — "Degradation of service in h2 servers with CONTINUATION Flood" (fixed 0.3.26 / 0.4.4). https://rustsec.org/advisories/RUSTSEC-2024-0332.html ; https://seanmonstar.com/blog/hyper-http2-continuation-flood/
- **VERIFIED.** `RUSTSEC-2026-0258` (h2) — "h2 unbounded empty DATA frames": "the h2 crate ... would accept and queue empty DATA frames without limit. If streams were not actively drained, this could lead to unbounded memory usage, or a panic if the length overflows." (fixed 0.4.16). https://rustsec.org/advisories/RUSTSEC-2026-0258.html
- **VERIFIED.** `CVE-2023-44487` — "The HTTP/2 protocol allows a denial of service (server resource consumption) because request cancellation can reset many streams quickly, as exploited in the wild in August through October 2023." (HTTP/2 Rapid Reset.) https://cveawg.mitre.org/api/cve/CVE-2023-44487
- **ASSUMPTION.** PhairDLNA should serve **HTTP/1.1 only** in v1 (no h2, no TLS by default) — this eliminates the entire HTTP/2 flood class and makes smuggling-against-a-proxy materially irrelevant (we are the origin). **OPEN QUESTION**: renderer compatibility with HTTP/1.1-only — low risk, DLNA is HTTP/1.1-centric.
- **Normative:** per-connection read/header/whole-request deadlines (idle timeout, absolute request timeout); a hard cap on concurrent connections; a hard cap on in-flight requests; a per-source-IP connection-rate limit; and a bounded accept queue. ReadyMedia's own analogue is "Exceeded max connections [50]" (user-visible in their tracker) and "Make max number of children (connections) configurable" (1.1.0) — VERIFIED (ReadyMedia tracker / release notes).

### 5.4 Range-header abuse (multi-range amplification)

- **VERIFIED.** `CVE-2011-3192` — "The byterange filter in the Apache HTTP Server 1.3.x, 2.0.x through 2.0.64, and 2.2.x through 2.2.19 allows remote attackers to cause a denial of service (memory and CPU consumption) via a Range header that expresses multiple overlapping ranges, as exploited in the wild in August 2011". https://cveawg.mitre.org/api/cve/CVE-2011-3192
- **ASSUMPTION.** We must support single-range only (`bytes=start-end`). If a client sends multiple ranges, either serve the whole entity (`200` + `Accept-Ranges: bytes`) or ignore the Range header and serve `200`. Never generate a `multipart/byteranges` response, or cap the number of ranges and the total size of a multipart response. Also reject ranges whose `start`/`end` overflow, and ranges that ask for the entire file in a way that defeats the intent of the cap.
- **Normative:** also cap the *uncompressed* size we are willing to read when serving, and never re-read a file from disk more than once per request.

### 5.5 Path traversal in the HTTP path; cache poisoning

- **VERIFIED.** PortSwigger's path-traversal guidance (quoted in §4.4) — canonicalise-then-contain.
- **VERIFIED (SSRF-adjacent).** `GHSA-h2v8-394m-p7vv` / `RUSTSEC-2026-0348` (rustls) shows that even TLS-layer handling must be bounded; and `RUSTSEC-2024-0336` "`rustls::ConnectionCommon::complete_io` could fall into an infinite loop based on network input" — an input-driven infinite loop in a transport library. https://api.osv.dev/v1/vulns/RUSTSEC-2024-0336
- **ASSUMPTION — cache poisoning.** We serve a LAN-only media origin with no shared cache in front, so "cache poisoning" is mostly a *misconfiguration* risk: if a reverse proxy is placed in front of PhairDLNA, `Host`-derived absolute URLs and `X-Forwarded-*` headers become attacker-influenced. **Normative:** ignore `X-Forwarded-*` entirely unless a trusted-proxy mode is explicitly enabled; never build a `Location`/`LOCATION` (SSDP) URL from the `Host` header — use the configured virtual URL. Gerbera hit exactly this: it validates request hostnames against the configured `virtualURL` (§4.2).

### 5.6 SSRF via GENA / event-subscription callback URLs

- **VERIFIED.** `CVE-2020-12695` (CallStranger): the UPnP spec before 2020-04-17 "does not forbid the acceptance of a subscription request with a delivery URL on a different network segment than the fully qualified event-subscription URL"; the flaw "could allow a device connected to the local network ... to trigger the AP to initiate a HTTP (TCP/IP) connection to an arbitrary URL, including connections to servers in external networks." https://cveawg.mitre.org/api/cve/CVE-2020-12695 ; https://www.access.redhat.com/security/cve/cve-2020-12695
- **VERIFIED.** CERT/CC: "The Universal Plug and Play (UPnP) protocol in effect prior to April 17, 2020 can be abused to send traffic to arbitrary destinations using the SUBSCRIBE functionality ... permits an attacker to send large amounts of data to arbitrary destinations accessible over the Internet, which could lead to a Distributed Denial of Service (DDoS), data exfiltration, and other unexpected network behavior." Vendor guidance: "Device manufacturers are urged to **disable the UPnP SUBSCRIBE capability in their default configuration** and to require users to explicitly enable SUBSCRIBE with any appropriate network restrictions to limit its usage to a trusted local area network." User guidance: "Disable the UPnP protocol on Internet-accessible interfaces." https://www.cve.org/CVERecord?id=CVE-2020-12695 (references https://www.kb.cert.org/vuls/id/339275/) ; https://www.access.redhat.com/security/cve/cve-2020-12695
- **VERIFIED.** ReadyMedia fixed this in 1.3.0 with the changelog line "Validate SUBSCRIBE callback URL. [CVE-2020-12695]" — i.e. the upstream fix is literally *validate the callback URL*, confirming this is the canonical mitigation. VERIFIED (ReadyMedia release notes).
- **VERIFIED (adjacent).** `CVE-2024-7029` and the AVTECH campaign are referenced in the CVE-space for IGD/UPnP RCE, but I did **not** fetch a primary record for `CVE-2024-7029` and therefore do not cite it as an attack claim. **UNVERIFIED.**

**Normative validation algorithm for a GENA callback (M-02 / M-04):**
1. Parse the callback URL as an absolute `http`/`https` URL; reject anything else.
2. Resolve the host to a literal IP **at subscription time** and pin that IP for the lifetime of the subscription; re-validate on every delivery.
3. Reject if the resolved IP is not in an allowed set: reject loopback (127/8, ::1), link-local (169.254/16, fe80::/10), unique-local (fc00::/7), multicast, unspecified, and any address outside the local network(s) we serve.
4. Reject non-default ports and any userinfo in the URL.
5. Cap the number of concurrent subscriptions and cap the number of events delivered per subscription per unit time (ReadyMedia added "DoS protection for event subscriptions" in 1.2.1 — VERIFIED).
6. Default: **disable event subscription entirely**; enable only behind an explicit opt-in (CERT/CC's exact recommendation).
7. Never follow redirects when delivering an event; treat a redirect as a delivery failure.

---

## 6. Local admin API and privilege boundary

### 6.1 Why "loopback-only" is not sufficient

- **VERIFIED.** `CVE-2022-26505` (ReadyMedia DNS rebinding) + the oss-security write-up: a victim on the LAN visits an attacker's web page; the attacker's DNS name is rebound to the ReadyMedia server's LAN address; the browser then issues **same-origin** requests to the media server and can enumerate the library and **download the media files**, exfiltrating them. Fixed in 1.3.1 ("Protect against DNS rebinding attacks"), improved in 1.3.2. https://cveawg.mitre.org/api/cve/CVE-2022-26505 ; https://www.openwall.com/lists/oss-security/2022/03/03/1 ; https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/
- **VERIFIED.** Gerbera hit the same wall and the same fix: `UpnpSetHostValidateCallback` returning `UPNP_E_BAD_HTTPMSG` for hosts not in the configured `virtualURL` — with the documented caveat that libupnp then ignores the error. https://github.com/gerbera/gerbera/commit/b3d4b84a402e258fe77e449ef3c90d5b2d2ed204
- **ASSUMPTION.** The general rule: **a loopback-bound HTTP service is reachable by any web page the user visits**, because the browser is the confused deputy. "Loopback-only" limits *who on the network* can talk to it; it does not limit *what a web page* can make it do. The defences must therefore be (a) Host-header validation, (b) a request-origin credential that a cross-origin page cannot read or forge (SameSite + no CORS + custom-header requirement), and (c) not exposing the media contents on the admin origin at all.

### 6.2 Normative requirements for the local admin API

1. **Bind to `127.0.0.1` and `::1` only** — never `0.0.0.0`, never the LAN address. Two separate listeners if necessary (`::1` and `127.0.0.1`).
2. **Host-header allowlist.** Accept only `Host` values in a fixed allowlist (`localhost`, `127.0.0.1`, `[::1]`, plus the configured admin hostname). Any other `Host` ⇒ `421`/`403`. Reject before routing. (Directly modelled on the Gerbera fix.)
3. **Require a non-simple request.** A cross-origin page can issue a "simple" `GET`/`POST` without preflight. Requiring a **custom header** (e.g. `X-Phair-Admin: 1`) forces a CORS preflight, which we answer with `Access-Control-Allow-Origin: <none>`; the browser then blocks the actual request. Combine with `SameSite=Strict` on any cookie.
4. **CSRF token** for every state-changing request, bound to the session and validated in constant time.
5. **Bearer token in a `0600` file**, written under the user's own `~/Library/Application Support/PhairDLNA/` (or `~/.config/phairdlna/`), mode `0600`, dir mode `0700`; the admin UI reads it from disk, not from a URL. Rotate on restart or on explicit request.
6. **Never** accept credentials from the query string.
7. **Constant-time comparison** for all token comparisons; never log a token (log a truncated hash).
8. **Never serve media bytes from the admin origin** — separate ports/origins for admin vs media, so a rebinding attack cannot exfiltrate content through the admin origin.
9. **Do not enable CORS.** Never send `Access-Control-Allow-Origin: *`.
10. **Admin API is not part of the UPnP device description** and is not discoverable via SSDP.

---

## 7. macOS-specific considerations

### 7.1 TCC (Transparency, Consent and Control)

- **VERIFIED.** Apple's model: "In macOS 10.15 or later, this model is enforced by the system to help ensure that all apps have obtained user consent before accessing files in Documents, Downloads, Desktop, iCloud Drive, and network volumes. In macOS 10.13 or later, apps that require access to the full storage device must be explicitly added in System Settings (macOS 13 or later) or System Preferences (macOS 12 or earlier)." The permission table includes "Files and folders — Includes Desktop, Documents, Downloads, network volumes, and removable volumes" and "Full internal storage access". https://support.apple.com/guide/security/controlling-app-access-to-files-secddd1d86a6/web
- **VERIFIED.** TCC "protects access to private data even for root processes" — a media server running as a launchd daemon in a user's session inherits the *responsible process* attribution, so a daemon reading `~/Movies` can be blocked or prompted. **OPEN QUESTION** — exact attribution rules for a `LaunchAgent` vs a `LaunchDaemon` vs a GUI app on current macOS; must be validated empirically on the target macOS versions.
- **ASSUMPTION.** Design consequence: PhairDLNA must **never require Full Disk Access**. It should ask for a specific folder via a user-driven picker (NSOpenPanel) — "the picker *is* consent" — and store a security-scoped bookmark. If we request FDA we both degrade privacy (bad for a privacy-first product) and become a high-value target.
- **VERIFIED.** The macOS 15 "Local Network" permission exists and is a separate gate that affects LAN discovery: "Local Network — Reaches other devices on your LAN … NAS clients, media servers, home automation, dev servers" and "It runs on mDNS and multicast rather than TCC, and denial produces no error message — the app simply cannot see the network." https://support.apple.com/guide/security/local-network-privacy-prd1be8a0f88/web ; https://nektony.com/how-to/manage-app-permissions-mac
- **Normative.** PhairDLNA must ship a clear, actionable explanation of the Local Network prompt (we *need* it for SSDP/multicast), must not attempt to work around denial, and must fail gracefully with a diagnostic that names the exact System Settings pane. Also: our SSDP socket binds to the multicast group — the Local Network prompt is triggered by the mDNS/multicast activity, so we should make that the *first* thing we do after launch, and explain it.

### 7.2 Gatekeeper, code signing, notarisation

- **VERIFIED.** Apple maintains the "Gatekeeper and runtime protection" chapter of the Apple Platform Security guide (fetched, https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-secb0694df1a/web). **OPEN QUESTION**: I was unable to extract the chapter body (the guide is a client-rendered SPA), so the *specific* behaviours below are stated as **ASSUMPTION**, to be confirmed against the guide before the release checklist is frozen:
  - A quarantined, unsigned or un-notarised binary is blocked or warned about by Gatekeeper on first launch; a Developer ID–signed + notarised + stapled build launches cleanly. (industry consensus)
  - `spctl --assess` is the command-line check we should run in CI/release verification.
- **VERIFIED.** Apple's "App code signing process" chapter exists in the same guide and is the reference for our signing identity selection. https://support.apple.com/guide/security/app-code-signing-process-sec3b77f1b8f6/web (fetched index)
- **Normative.** PhairDLNA must be signed with a **Developer ID Application** certificate, **notarised** and **stapled**; the release pipeline must verify with `spctl --assess --type execute` and `codesign --verify --deep --strict`. Unsigned distribution is a *privacy and integrity* problem, not just a UX one: users routinely run "right-click → Open" to bypass Gatekeeper, which is exactly the behaviour we must not depend on.

### 7.3 launchd

- **VERIFIED (Apple's `launchd.plist(5)` man page).** Relevant keys and their semantics: `KeepAlive` (default false), `RunAtLoad`, `LaunchOnlyOnce`, `UserName` / `GroupName` ("only applicable for services that are loaded into the privileged system domain"), `ProcessType` ("The system will apply resource limits based on what kind of job this is. If left unspecified, the system will apply light resource limits to this job, throttling its CPU usage and I/O bandwidth"), `Sockets` (launch on demand), `PathState`, `WorkingDirectory`, `StandardOutPath`/`StandardErrorPath`. VERIFIED, https://keith.github.io/xcode-man-pages/launchd.plist.5.html (Apple man page, mirrored).
- **ASSUMPTION.** `ProcessType: Background` gives us *kernel-enforced* CPU and I/O throttling for the indexer — a free, real mitigation for §8's "index rebuild CPU/IO storms". **OPEN QUESTION**: confirm the exact throttle behaviour for `Background` vs `Adaptive` vs `Standard` on current macOS, and whether `ProcessType` applies to the whole process or per-job.
- **Normative.** Ship a `LaunchAgent` (per-user, runs in the user's session so it can access the user's media and inherit the user's TCC context) rather than a system-wide `LaunchDaemon`; run as the logged-in user (never root) — a media server has no business being root. Use `KeepAlive` with a successful-exit predicate, `RunAtLoad`, `WorkingDirectory`, and redirect logs to `~/Library/Logs/PhairDLNA/`. Because a `LaunchAgent` is per-user and TCC-attributed to the user, it avoids the "daemon cannot read ~/Movies" failure mode.

### 7.4 Symlink vs alias semantics on macOS

- **VERIFIED (partial).** Apple's File System Programming Guide (File System Basics) states APFS is the default filesystem for macOS High Sierra and later, and that "all of the disks attached to the device — whether they are physically plugged in or connected indirectly through the network — contribute space to form a single collection of files." https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html
- **ASSUMPTION / OPEN QUESTION.** macOS **Finder aliases** are a distinct filesystem object (a small file with a resource fork / `com.apple.FinderInfo`-style metadata) that resolves to a target path, whereas **POSIX symlinks** are resolved by the kernel during path lookup. Consequences for containment:
  - `std::fs::canonicalize` resolves symlinks but does **not** resolve Finder aliases (an alias is not a symlink; it is a regular file to the kernel). A path containing an alias therefore appears *inside* the root to a naive containment check but resolves *outside* it for the user.
  - Therefore: **reject or explicitly resolve aliases inside shared folders**, or canonicalise the *resolved target* of each alias and apply the same containment rule to it.
  - **OPEN QUESTION**: the exact behaviour of `open(2)` / `canonicalize` on a path whose final component is an alias, and whether `O_NOFOLLOW` has any effect on aliases. Must be validated on macOS with a test fixture before the containment rule is frozen.
- **Normative.** Containment must be: (1) percent-decode exactly once; (2) reject NUL, control characters, and any `\` (Windows-style separators are not path separators on macOS but are a classic confusion vector); (3) join to the library root; (4) resolve the target (symlinks **and** aliases); (5) require the resolved target to be under the canonicalised root, or to be on an explicitly allowed external-volume list; (6) open with `O_NOFOLLOW` where the platform allows, and prefer directory-relative opens to close the TOCTOU window.

---

## 8. Resource exhaustion

- **VERIFIED.** `RUSTSEC-2023-0034` (h2) — unbounded accept queue → OOM. https://rustsec.org/advisories/RUSTSEC-2023-0034.html
- **VERIFIED.** `RUSTSEC-2026-0258` (h2) — unbounded empty DATA frames → unbounded memory. https://rustsec.org/advisories/RUSTSEC-2026-0258.html
- **VERIFIED.** `RUSTSEC-2026-0195` (quick-xml) — unbounded namespace allocation *inside the crate, before the consumer sees the event* → "A consumer that bounds its *input* size therefore s[uffices]" — i.e. per-input limits alone are insufficient. https://rustsec.org/advisories/RUSTSEC-2026-0195.html
- **VERIFIED.** `RUSTSEC-2021-0143` (kamadak-exif) — infinite loop on crafted data. https://rustsec.org/advisories/RUSTSEC-2021-0143.html
- **VERIFIED.** `RUSTSEC-2026-0348` (rustls) — Brotli decompression bomb: "can allocate an attacker-controlled amount of memory up to 1 GiB before authentication". https://api.osv.dev/v1/vulns/RUSTSEC-2026-0348
- **VERIFIED.** `CVE-2011-3192` — multi-Range ⇒ memory and CPU consumption. https://cveawg.mitre.org/api/cve/CVE-2011-3192
- **VERIFIED.** `CVE-2023-44487` — HTTP/2 rapid reset ⇒ server resource consumption, exploited in the wild. https://cveawg.mitre.org/api/cve/CVE-2023-44487
- **VERIFIED.** `CVE-2019-12110` / `CVE-2019-12111` (MiniUPnPd) — remote NULL-pointer-dereference DoS in the UPnP stack, one of them **in `AddPortMapping`**. https://cveawg.mitre.org/api/cve/CVE-2019-12110 ; https://cveawg.mitre.org/api/cve/CVE-2019-12111
- **VERIFIED.** ReadyMedia 1.3.1 "Fixed a potential crash in SSDP request parsing"; 1.2.1 "Added DoS protection for event subscriptions"; 1.3.0 "Fixed potential stack smash vulnerability in `getsyshwaddr` on macOS". VERIFIED (release notes).
- **VERIFIED.** `RUSTSEC-2019-0033` (http) — header-map integer overflow ⇒ DoS. https://rustsec.org/advisories/RUSTSEC-2019-0033.html

**Normative limits for PhairDLNA (all ASSUMPTION values are engineering choices; the *existence* of each limit class is VERIFIED by the advisories above):**

| Limit | Proposed value | Rationale |
|---|---|---|
| Concurrent HTTP connections | 64 (configurable) | ReadyMedia's analogue: "Exceeded max connections [50]" |
| Concurrent requests / in-flight tasks | 128 | queue depth ⇒ bounded memory |
| Read header timeout / whole-request timeout | 5 s / 30 s | slowloris |
| Max request head size | 16 KiB | header DoS |
| Max request body (SOAP) | 64 KiB | SOAP/XML size |
| Max DIDL response page | 64 KiB / 100 items | memory growth from large browse pages |
| Max XML nodes / nesting depth | 100 000 / 64 | entity + nesting expansion |
| Max SSDP responses in flight / per-second | 32 / 20 | reflection + response storms |
| Max GENA subscriptions / events per second | 8 / 4 | subscription DoS (ReadyMedia 1.2.1 precedent) |
| Max concurrent metadata parse jobs | 2 (separate process) | CPU/IO storm from a folder drop |
| Metadata parse wall-clock / RSS / file-size | 5 s / 256 MiB / 64 MiB | decompression bombs, VLA/loop bugs |
| Indexer | incremental, `ProcessType: Background`, rate-limited | launchd CPU/IO throttle |
| File descriptors | explicit soft cap; `setrlimit(RLIMIT_NOFILE)` | fd exhaustion |
| Library watch | bounded watch count; `max_user_watches` analogue | inotify/fsevents exhaustion |

---

## 9. Supply chain, licensing, and AGPL

### 9.1 RustSec, cargo-audit, cargo-deny

- **VERIFIED.** The RustSec Advisory Database is "a repository of security advisories filed against Rust crates published via crates.io", maintained by the Rust Secure Code Working Group, exported to OSV in real time and imported by the GitHub Advisory Database (so Dependabot can raise fix PRs) and by the Debian Rust Team (so the Debian Rust Security Tracker can flag known-vulnerable packages). VERIFIED, https://rustsec.org/ ; https://github.com/rustsec/advisory-db
- **VERIFIED.** The supported toolchain: `cargo-audit` ("Audit `Cargo.lock` files for crates with security vulnerabilities"), `cargo-deny` ("Audit `Cargo.lock` files for crates with security vulnerabilities, limit the usage of particular dependencies, their licenses, sources to download from, detect multiple versions of same packages in the dependency tree and more"), `cargo-auditable` ("Embed the dependency tree into compiled executables, to make production Rust binaries auditable by cargo-audit"), plus `trivy` and `dependabot`. VERIFIED, https://rustsec.org/ ; https://github.com/rustsec/advisory-db
- **VERIFIED.** `cargo-deny` has first-class checks we should use: `licenses` ("verify that every crate you use has license terms you find acceptable"), `bans` ("deny (or allow) specific crates, as well as detect and handle multiple versions of the same crate"), and `advisories`. VERIFIED, https://github.com/EmbarkStudios/cargo-deny
- **Normative:** run `cargo audit` + `cargo deny check` on every PR; commit `Cargo.lock` and require it unchanged except by deliberate update; enable `cargo auditable` in release builds so shipped binaries carry their dependency tree; configure `cargo-deny` with an **allowlist of licences** that permits AGPL-3.0-or-later, Apache-2.0, MIT, BSD-2/3, ISC, Unicode-3.0, MPL-2.0, LGPL-2.1/3.0 — and **denies** everything else; maintain an explicit `banned` list including any crate with a `build.rs` or proc-macro that we have not reviewed, and any crate that is unmaintained (RustSec publishes `unmaintained` advisories).

### 9.2 Typosquatting and build-script code execution on crates.io

- **VERIFIED.** `rustdecimal` (2022): "The crate name was intentionally similar to the name of the popular `rust_decimal` crate, hoping that potential victims would misspell its name (an attack called 'typosquatting')"; it contained a `build.rs` whose `Decimal::new` function checked for `GITLAB_CI` and, if set, downloaded a payload into `/tmp/git-updater.bin` and executed it (Linux and macOS); "The crate had less than 500 downloads since its first release on 2022-03-25". Rust's recommendation: "In general, we recommend regularly auditing your dependencies, and only depending on crates whose author you trust." VERIFIED, https://blog.rust-lang.org/2022/05/10/malicious-crate-rustdecimal.html
- **VERIFIED.** August 2023: nine crates typosquatting popular crates were uploaded with malicious `build.rs` files that "would attempt to send metadata from the user's computer to a Telegram channel, including their operating system, IP address, and geolocation information"; all were yanked, the account locked, and the crates deleted from `static.crates.io`. VERIFIED, https://blog.rust-lang.org/inside-rust/2023/09/01/crates-io-malware-postmortem/
- **VERIFIED.** September 2025: `faster_log` (7 181 downloads) and `async_println` (1 243 downloads) — typosquatting copies of legitimate logging crates that kept working while exfiltrating Ethereum/Solana secrets and bracketed byte arrays to `mainnet[.]solana-rpc-pool[.]workers[.]dev` during log-packing; the malicious code ran **at runtime** rather than build time; crates.io deleted both crates and disabled the accounts. VERIFIED, https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/
- **VERIFIED.** Academic quantification: "on average, each package has 5.7 typosquatting candidates"; 2023 typosquatting packages included `postgress`, `if-cfg`, `serd`, `oncecell`, `lazystatic`, `envlogger`, each with a `build.rs` that transmitted host information to an attacker-controlled Telegram channel. VERIFIED, https://dl.acm.org/doi/10.1145/3708821.3735340
- **VERIFIED (mitigation-relevant).** crates.io "normalizes crate names so that hyphens and underscores are treated as equivalent when checking uniqueness", which closes the most common separator-confusion typosquat. VERIFIED, https://safeguard.sh/resources/blog/cargo-supply-chain-attacks-typosquatting-and-malicious-crates-on-cratesio
- **Normative:** (a) `Cargo.lock` committed and diff-reviewed like source; (b) no new dependency without a named owner and a link to the upstream project; (c) prefer crates with ≥2 maintainers, a published SECURITY.md and a non-trivial download count; (d) treat any dependency with a `build.rs` or a proc-macro as *code we have read*; (e) vendor/mirror dependencies if we want to be independent of crates.io at build time; (f) pin by version **and** checksum (`cargo` verifies checksums by default — keep it on, never `--offline`-bypass it).

### 9.3 FFmpeg: GPL vs LGPL, and shipping it inside an AGPL product

- **VERIFIED.** "FFmpeg is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later. However, FFmpeg incorporates several optional parts and optimizations that are covered by the GNU General Public License (GPL) version 2 or later. **If those parts get used the GPL applies to all of FFmpeg.**" VERIFIED, https://ffmpeg.org/legal.html
- **VERIFIED.** FFmpeg's own LGPL compliance checklist: "Compile FFmpeg without `--enable-gpl` and without `--enable-nonfree`"; "Use dynamic linking (on windows, this means linking to dlls) for linking with FFmpeg libraries"; "Distribute the source code of FFmpeg, no matter if you modified it or not"; "Make sure the source code corresponds exactly to the library binaries you are distributing"; "Explain how you compiled FFmpeg, for example the configure line"; "Host the FFmpeg source code on the same webserver as the binary you are distributing"; "Make sure your program is not using any GPL libraries (notably libx264)"; "Remove any prohibition of reverse engineering from your EULA". VERIFIED, https://ffmpeg.org/legal.html
- **ASSUMPTION (licensing reasoning — not legal advice).** The practical compatibility matrix for PhairDLNA:
  - **AGPL-3.0 product + LGPL-2.1 library, dynamically linked, with relinking rights preserved** → compatible. AGPL §13 additionally requires the *complete* corresponding source of the combined work to be offered to network users; because we already must offer our own AGPL source, adding FFmpeg's LGPL source to the same offer is the cleanest path (and LGPL §4's "reverse engineering for debugging" right must not be contractually prohibited — FFmpeg's checklist says the same).
  - **AGPL-3.0 product + GPL-2.0 library** → AGPLv3 §13 says the AGPL "terms of this License will continue to apply to the part which is the covered work, but the work with which it is combined will remain governed by version 3 of the GNU General Public License" — so the *combined work* would be GPLv3, which is **not** the AGPL, and the AGPL's network-source obligation would not attach to the GPL-governed part. Shipping a GPL FFmpeg inside an AGPL product is therefore a **licensing trap**, not merely a hygiene issue.
  - **Shipping FFmpeg binaries** (even LGPL) inside our distribution triggers the LGPL §4/§6 source-availability and relinking obligations, plus the obligation to ship the exact corresponding source and build instructions.
- **Normative (M-08):** **do not ship FFmpeg at all in v1.** Prefer pure-Rust parsing (container/metadata only) and defer transcoding to an optional, separately-distributed, user-supplied external binary invoked out-of-process with an argv array (never a shell) — this keeps the AGPL product free of GPL/LGPL linking questions, keeps the media-parsing sandbox boundary clean, and keeps the supply chain small. If transcoding is later deemed essential, it must be (a) LGPL-only build (`--enable-gpl` **off**, `--enable-nonfree` **off**, no libx264), (b) dynamically linked with relinking rights preserved, (c) accompanied by the exact FFmpeg source + configure line + build instructions in the same source offer as the AGPL work, and (d) reviewed by counsel.
- **OPEN QUESTION.** Whether a pure-Rust decoder stack (e.g. `symphonia`, `mp4parse`, `matroska`) can cover the formats we need without a native dependency — and, if not, whether `ffmpeg-next`/`ffmpeg-sys` (which link FFmpeg) can be avoided. Must be decided before the dependency allowlist is frozen.

### 9.4 The AGPL network-use clause (§13) — what it means for a self-hosted, self-modifying server

- **VERIFIED (verbatim, AGPL-3.0 §13).** "Notwithstanding any other provision of this License, if you modify the Program, your modified version must prominently offer all users interacting with it remotely through a computer network (if your version supports such interaction) an opportunity to receive the Corresponding Source of your version by providing access to the Corresponding Source from a network server at no charge, through some standard or customary means of facilitating copying of software. This Corresponding Source shall include the Corresponding Source for any work covered by version 3 of the GNU General Public License that is incorporated pursuant to the following paragraph." VERIFIED, https://raw.githubusercontent.com/spdx/license-list-data/main/text/AGPL-3.0-only.txt
- **VERIFIED (AGPL-3.0 §0).** "To 'modify' a work means to copy from or adapt all or part of the work in a fashion requiring copyright permission, other than the making of an exact copy. The resulting work is called a 'modified version' of the earlier work or a work 'based on' the earlier work." And: "The 'Corresponding Source' for a work in object code form means all the source code needed to generate, install, and (for an executable work) run the object code and to modify the work, including scripts to control those activities." VERIFIED, https://raw.githubusercontent.com/spdx/license-list-data/main/text/AGPL-3.0-only.txt
- **ASSUMPTION (implication for PhairDLNA, not legal advice).** A self-hosted server that users interact with over the network — and that PhairDLNA's users are expected to *modify* (config files, playlists, plug-in metadata handlers, possibly forked scanners) — is squarely inside AGPL §13's scope. The AGPL's trigger is **remote network interaction**, which PhairDLNA has by definition (UPnP over HTTP/SSDP on the LAN). Therefore:
  1. The binary we ship is a "modified version" of the AGPL work, and §13 obliges us to **offer the Corresponding Source to every network user who interacts with it**.
  2. A "Corresponding Source" offer must be reachable **from the running product** — the natural design is an in-product route (e.g. `GET /source` on the media origin, or a link in the admin UI) that serves the source archive for the exact build, at no charge.
  3. "Including scripts to control those activities" means the **build scripts, `Cargo.lock`, vendored patches, and the exact `configure`/build recipe** must be part of the offer — not just the `.rs` files. A `cargo auditable` dependency tree (VERIFIED above as an existing RustSec-endorsed tool) is a strong aid here.
  4. A **self-modifying** server (one that downloads or applies updates to itself at runtime) increases exposure: each modification is a new "modified version" with a fresh §13 obligation, and a self-update channel is an attractive vector for supply-chain compromise (§9.2). **Normative:** PhairDLNA must **not** self-update over the network in v1. Updates are pulled by the user through their package manager (Homebrew) and the running version's source offer is regenerated per release.
  5. The §13 offer must be reachable **without authentication** for any client that can reach the media server — otherwise it is not an "offer" to the network user.
- **OPEN QUESTION.** Whether our LAN-only posture ("LAN-only by default") affects the AGPL analysis. It does not: AGPL §13 says "all users interacting with it remotely through a computer network", and a LAN is a computer network. Treat §13 as in scope.

---

## 10. Threat table

Likelihood is stated for the **default, LAN-only, macOS-first** configuration unless noted. "Status" reflects the state of our programme: `Planned` = requirement to be written; `Open` = unanswered question or unverified claim requiring work.

| ID | Surface | Attacker | Entry point | Impact | Likelihood | Mitigation | Verification (negative test) | Status |
|---|---|---|---|---|---|---|---|---|
| SEC-TH-01 | SSDP reflector / amplifier | Internet attacker (via compromised host) or LAN attacker | UDP/1900 M-SEARCH with spoofed source | Victim DoS; our host used as an unwitting amplifier; LAN-wide traffic storm | Medium | M-01: respond only to M-SEARCH for our STs; rate limit per source; cap responses in flight; small TTL; never respond to non-M-SEARCH; refuse WAN binds | From a spoofed-source M-SEARCH, assert: zero response packets are emitted to the spoofed address, and the per-source response counter never exceeds the configured cap; assert no reply is produced for any method other than M-SEARCH | Planned |
| SEC-TH-02 | SSDP advertisement forgery | LAN attacker | NOTIFY with forged `LOCATION`/`USN` | Renderer connects to attacker host; content substitution; renderer-side SSRF | Medium | M-01: bind device identity to our UUID; advertise only on real transitions; do not accept inbound NOTIFY at all | Inject a NOTIFY claiming our USN; assert the server never reacts and never serves a second `LOCATION`; assert our advertised USN never changes without a real service change | Planned |
| SEC-TH-03 | SSDP discovery flooding | LAN attacker | High-rate M-SEARCH | CPU/IO burn; discovery unusable | Medium | M-01 + M-08: per-source and global rate limits; bounded work per request; `ProcessType: Background` | Flood 10 k M-SEARCH/s for 60 s; assert CPU stays under the configured ceiling, RSS is flat, and the rate limiter's drop counter increments rather than the work counter | Planned |
| SEC-TH-04 | Multicast group membership abuse | Local process / flaky network | Repeated join/leave on interface churn | fd/resource leak; server remains reachable on unwanted interfaces | Low | M-01: idempotent membership management; join once per interface; rate-limited rejoin; drop membership on interface down | Cycle the primary interface up/down 100 times; assert the membership table has exactly one entry per active interface and the fd count is constant | Open |
| SEC-TH-05 | GENA callback SSRF (CallStranger class) | LAN attacker | HTTP SUBSCRIBE with external `CALLBACK` URL | Our server becomes a request proxy into arbitrary networks; exfiltration; DDoS amplification | Medium | M-02: event subscription **off by default**; on opt-in, validate callback host/port/scheme, resolve-and-pin IP, reject loopback/link-local/private/multicast/non-LAN, no redirects, cap subscriptions and event rate | Submit a SUBSCRIBE with a callback pointing at a routable public address and at `169.254.169.254`; assert both are rejected with no outbound connection attempt (verified via a socket-egress counter/deny-list firewall) | Planned |
| SEC-TH-06 | XXE / DTD in SOAP or DIDL parsing | LAN attacker | SOAP body or DIDL document with a DTD | Local file read; SSRF; DoS | Low | M-02: reject DTDs; no external entity resolution; entity resolver always fails; bounded input and node counts | Post a SOAP body containing an internal DTD subset with an external entity; assert a parse error is returned, no file is opened, and no outbound connection is attempted | Planned |
| SEC-TH-07 | XML entity expansion ("billion laughs") | LAN attacker | SOAP/DIDL with nested entity definitions | Memory exhaustion / CPU DoS | Low–Medium | M-02: DTD rejection (primary), explicit depth + node + expanded-size caps | Feed a nested-entity document; assert the parse fails within the configured byte/time budget and that peak RSS never exceeds the cap | Open |
| SEC-TH-08 | Deeply nested XML / stack exhaustion | LAN attacker | Deeply nested elements | Process crash | Low | M-02: explicit max nesting depth enforced during streaming parse | Parse a 100 000-deep document; assert rejection at the configured depth and no stack overflow | Planned |
| SEC-TH-09 | SOAP `Action` routing abuse / unknown action | LAN attacker | SOAP request with unknown/garbage `SOAPAction` | Dispatch into unintended handler; log flooding; CPU burn | Medium | M-02: strict allowlist of advertised actions; constant-time miss path; no reflection into logs | Send 10 000 random `SOAPAction` values; assert all are rejected with identical status/length/timing and that the action-dispatch table is never indexed out of range | Planned |
| SEC-TH-10 | IGD-style action reachable on a DMS | LAN attacker | `AddPortMapping` / `WANIPConnection` SOAP action | Port exposure / state mutation / (historically) RCE | Low | M-03: **no IGD services implemented**; explicit rejection of all WAN* actions | Send `AddPortMapping`, `DeletePortMapping`, `GetExternalIPAddress`; assert a `501`/`400` and assert that the socket count, the open-port set and the process-spawn counter are unchanged | Planned |
| SEC-TH-11 | Command injection via a SOAP argument | LAN attacker | Malicious argument reaching a shell | RCE as the server user | Low (by design) | M-03: no shell in the request path; no `Command::new`/`sh -c` outside the isolated transcoder; argv arrays only | Fuzz every SOAP argument with shell metacharacters; assert the process-spawn counter stays at zero and no file matching a canary marker is created | Planned |
| SEC-TH-12 | HTTP request smuggling (CL/TE, TE/TE) | LAN attacker / adjacent proxy | Conflicting framing headers | Request desync; cache poisoning; bypass of our own auth | Medium | M-04: strict framing validation (see §5.1) | Send CL+TE, duplicate TE, TE with non-`chunked` value, `Content-Length: +5`, and a GET with a body; assert each is rejected with 400 and that the connection is closed without processing a second request | Planned |
| SEC-TH-13 | Header injection / oversized headers | LAN attacker | CRLF in a header; 1 MiB header block | Request splitting; response header injection; DoS | Medium | M-04: reject CR/LF/NUL in header values; cap head bytes (16 KiB) and per-header length | Send a header containing `%0d%0a` and a 1 MiB header; assert rejection, no second request in the stream, and no injected header in the response | Planned |
| SEC-TH-14 | Slowloris / slow-body | LAN attacker | Drip-fed request | Connection-table exhaustion | Medium | M-04: read/header/whole-request deadlines; connection cap; per-source connection-rate limit | Hold 200 connections open sending 1 byte/s; assert the server closes them within the idle timeout and the accepted-connection count never exceeds the cap | Planned |
| SEC-TH-15 | Unbounded concurrency / fd exhaustion | LAN attacker | Connection flood | DoS of the whole host | Medium | M-04 + M-08: connection cap, in-flight cap, `RLIMIT_NOFILE`, accept-queue bound | Open 5 000 sockets; assert the server stays responsive to a legitimate client and fd usage stays under the cap | Planned |
| SEC-TH-16 | Range-header amplification | LAN attacker | Multi-range / overlapping-range `Range` header | Memory + CPU DoS | Low–Medium | M-04: single-range only; never emit `multipart/byteranges`; overflow-safe range parsing | Send 100 overlapping ranges; assert a single `200` (or a single-range `206`) response and flat memory | Planned |
| SEC-TH-17 | Path traversal in the HTTP path | LAN attacker | `%2e%2e%2f`, double-encoded, absolute path | Arbitrary file read outside the library root | Medium | M-05: decode exactly once; reject `\`/NUL/control; join-then-canonicalise-then-contain; `O_NOFOLLOW` | Request `..%2f..%2fetc/passwd`, `%252e%252e%252f`, an absolute path, and a path with a NUL; assert 404/403 and that no file outside the root is opened (verified by an `openat` audit hook) | Planned |
| SEC-TH-18 | Symlink / alias escape | Local user or synced folder | Symlink (or Finder alias) inside a shared folder pointing outside the root | Read of arbitrary user files (e.g. `~/.ssh`) | Medium | M-05: resolve symlinks **and** aliases; containment applied to the resolved target; explicit external-volume allowlist | Place a symlink and a Finder alias to `~/.ssh` inside the library; assert both are refused | Open (macOS alias resolution semantics unverified) |
| SEC-TH-19 | Metadata parser memory-safety (ID3/MP4/Matroska/EXIF/DFF/WavPack) | Attacker who can place a file in a watched folder (sync client, shared folder, download) | Malformed media file parsed during scan | RCE or DoS in the server process | Medium–High | M-06: parse metadata **out of process**, with no network, bounded RSS/CPU/time/file size, and a hardened child | Place a malformed DFF/WavPack/PNG-with-EXIF/ID3 file in the library; assert the server process is unaffected, the child is killed at the timeout, and the file is quarantined | Planned |
| SEC-TH-20 | Decompression bomb / huge dimensions | Attacker who can place a file | Image header declaring enormous dimensions | 10 s of GB allocation → host OOM | Medium | M-06: never decode pixels in-process; cap declared dimensions before allocation; cap output pixels | Provide a JPEG/PNG with 100 000×100 000 declared dimensions; assert the child is killed and the parent's RSS never exceeds the cap | Planned |
| SEC-TH-21 | Playlist path injection | LAN attacker or file supplier | M3U/PLS/XSPF entry pointing outside the root | File read outside the library; SSRF via `http://` entry | Low | M-05: playlist entries are untrusted paths — resolve + contain; reject non-`file:` schemes; bound entry count/size | Playlist containing `../../etc/passwd` and `http://evil/x`; assert both entries are dropped and no outbound request is made | Open (no playlists in v1 — recommended) |
| SEC-TH-22 | Large DIDL browse pages | LAN attacker | `Browse` with huge `RequestedCount` / `StartingIndex` | Memory growth, latency | Medium | M-02: cap items per page and total response bytes; compute `TotalMatches` without materialising everything | `Browse` with `RequestedCount = 2^31-1`; assert the response is capped and the server's RSS is flat | Planned |
| SEC-TH-23 | DNS rebinding against the admin/media API | Web attacker + victim browser on the LAN | Attacker page whose DNS name rebinds to our LAN address | Media exfiltration (proven in ReadyMedia, `CVE-2022-26505`) | Medium | M-06 §6: Host-header allowlist; custom-header requirement; no CORS; media bytes never served from the admin origin | From a page whose hostname resolves to 127.0.0.1 (and again to the LAN IP), attempt `fetch`/`XHR`; assert the request is blocked and our server logs a Host-mismatch rejection | Planned |
| SEC-TH-24 | CSRF against the admin API | Web page the user visits | Cross-site form/fetch to the loopback admin API | Rescan, config change, library mutation | Medium | M-06 §6: CSRF token + custom header + `SameSite=Strict` + no CORS | POST a state-changing admin route from a foreign origin with no CSRF token; assert 403 and no state change | Planned |
| SEC-TH-25 | Admin bearer-token disclosure / weak permissions | Local user, backup, log shipper | Reading the token file | Full admin control | Low | M-06 §6: token file `0600` in a `0700` dir; never in argv/URL/logs; constant-time compare; rotate on restart | Assert the token file mode is `0600` and the directory `0700`; assert the token never appears in any log line | Planned |
| SEC-TH-26 | Signing / notarisation failure | User or downstream packager | Unsigned or un-notarised binary | Gatekeeper block; users bypassing Gatekeeper; supply-chain compromise | Medium | M-08: Developer ID sign + notarise + staple; `spctl --assess` in CI | Run `spctl --assess --type execute` and `codesign --verify --deep --strict` on the release artefact; assert both pass and the ticket is stapled | Planned |
| SEC-TH-27 | TCC / Local Network permission failure | End user | Denying the prompt | Silent loss of discovery or of media access; user confusion | Medium | M-07: request the narrowest permission; never require Full Disk Access; use a folder picker + security-scoped bookmark; actionable diagnostics | Run with Local Network denied and with the library inside `~/Documents`; assert the app degrades with a named, actionable diagnostic and never silently scans nothing | Open |
| SEC-TH-28 | Supply-chain compromise of a dependency | Upstream maintainer / typosquatter | Malicious crate or build script | Build-time or runtime RCE in our binary | Low–Medium | M-08: `cargo audit` + `cargo deny` in CI; committed `Cargo.lock`; review `build.rs`/proc-macros; `cargo auditable`; vendor if needed | Simulate a malicious crate in a test workspace; assert CI fails on the advisory and on the ban list, and that the release artefact's embedded dependency tree contains only reviewed crates | Planned |

---

## 11. Normative mitigation requirements (candidate set for the security requirements doc)

| ID | Requirement |
|---|---|
| **M-01** | **SSDP responder hardening.** Respond only to `M-SEARCH` for `ssdp:all`, `ssdp:rootdevice` and our own NT; ignore all other methods and all inbound NOTIFY. Rate-limit M-SEARCH per source IP and globally; cap outstanding responses. Advertise via NOTIFY only on real state transitions, with `CACHE-CONTROL: max-age`. Set a small IP TTL and hop limit on replies. Never respond to a source address that is not a plausible local/LAN address (policy to be decided; conflicts with MiniDLNA's historical "allow M-SEARCH from other subnets"). Refuse to bind SSDP/HTTP to any non-LAN interface without explicit opt-in. Never include library contents in an SSDP reply. |
| **M-02** | **XML/SOAP hardening.** Reject any document containing a DTD; never resolve external entities (install a failing entity resolver); reject `Event::DocType`; bound input bytes, nesting depth, node count and total expanded text size; allowlist `SOAPAction` values with a constant-time miss path; reject CR/LF/NUL in header-derived values; cap DIDL page size and item count; disable GENA event subscription by default and validate+pin callback URLs when enabled. |
| **M-03** | **No IGD actions, no shell.** Implement no `WANIPConnection`/`WANY`/`WANPPPConnection` service; explicitly reject `AddPortMapping`, `DeletePortMapping`, `GetExternalIPAddress` and any `WAN*` action. No `Command::new`/`sh -c`/`system()` in the request-handling path. SOAP handlers are pure functions over validated typed structs; the only actions dispatchable are those advertised in our SCPD. |
| **M-04** | **HTTP server hardening.** Strict request framing (reject CL+TE, duplicate TE, non-`chunked` TE, illegal `Content-Length`, bodies on bodyless methods); cap head size (16 KiB) and per-header length; read/header/whole-request deadlines; cap concurrent connections (64) and in-flight requests (128); per-source connection-rate limit; single-range only and never `multipart/byteranges`; never reflect `Host` or any request header into a response header or into a log line; ignore `X-Forwarded-*` unless a trusted-proxy mode is explicitly enabled; HTTP/1.1 only in v1. |
| **M-05** | **Path containment.** Percent-decode exactly once; reject NUL, control characters and backslashes; join to the library root; resolve symlinks **and** macOS Finder aliases; require the resolved path to be under the canonicalised root (or on an explicit external-volume allowlist); open with `O_NOFOLLOW` and prefer directory-relative opens to close the TOCTOU window. Playlist entries follow the same rule; only `file:` scheme, no absolute paths, bounded count/size. |
| **M-06** | **Isolated, bounded media parsing.** All metadata/thumbnail parsing runs in a separate, sandboxed child process with no network access, bounded RSS, wall-clock timeout, file-size cap and an explicit allowlist of formats; the parent never decodes pixels. The admin API binds to `127.0.0.1` and `::1` only, validates `Host` against a fixed allowlist, requires a non-simple request (custom header) plus a CSRF token, uses a bearer token stored `0600` in a `0700` directory compared in constant time, sends no CORS headers, and never serves media bytes. |
| **M-07** | **macOS-native deployment.** Ship a per-user `LaunchAgent` (never root, never a system daemon); use `ProcessType: Background` for the indexer; request only the Local Network permission and a user-picked folder (security-scoped bookmark) — never Full Disk Access; provide actionable diagnostics for each denied permission. Sign with Developer ID, notarise and staple; verify with `spctl --assess --type execute` in CI. |
| **M-08** | **Supply-chain and licensing gate.** Commit `Cargo.lock` and review it like source; run `cargo audit` and `cargo deny check` (advisories + bans + licences) on every PR and on the release tag; maintain an explicit allowlist of licences (AGPL-3.0-or-later, Apache-2.0, MIT, BSD-2/3, ISC, Unicode-3.0, MPL-2.0, LGPL-2.1/3.0) and an explicit ban list; review every dependency's `build.rs` and proc-macros; enable `cargo auditable` so release binaries carry their dependency tree. **Do not ship FFmpeg in v1**; if transcoding is later required, use an LGPL-only build (`--enable-gpl` and `--enable-nonfree` off), dynamically linked with relinking rights preserved, with its exact source and build recipe included in the source offer. Implement AGPL §13 as a first-class product feature: a network-reachable route that serves the Corresponding Source of the running build (including build scripts, `Cargo.lock` and vendored patches) at no charge to any network user; do not implement network self-update. |

---

## 12. Negative-test catalogue (must exist in CI)

| ID | Negative test (must fail closed / must prove absence) | Covers |
|---|---|---|
| **N-01** | Spoofed-source SSDP M-SEARCH ⇒ **zero** response packets to the spoofed address; response counter never exceeds the cap; no response at all to any non-M-SEARCH method. | SEC-TH-01, -03 |
| **N-02** | SUBSCRIBE with a callback URL resolving to a public address, to `169.254.169.254`, to `127.0.0.1`, and to a non-LAN RFC1918 address ⇒ all rejected, and the egress-deny counter shows **no** outbound connection attempt. | SEC-TH-05 |
| **N-03** | SOAP/DIDL bodies carrying a DTD, an external entity, a nested-entity expansion and a 100 000-deep element tree ⇒ each rejected within the configured byte/time budget, peak RSS within the cap, no file opened, no outbound connection. | SEC-TH-06, -07, -08 |
| **N-04** | Randomised `SOAPAction` fuzz ⇒ 100 % rejection, identical status/body-length/timing class, no dispatch-table access out of range. | SEC-TH-09 |
| **N-05** | `AddPortMapping` / `DeletePortMapping` / `GetExternalIPAddress` / `WANIPConnection` ⇒ rejected, **and** the socket count, the listening-port set and the process-spawn counter are unchanged. | SEC-TH-10 |
| **N-06** | Every SOAP argument fuzzed with shell metacharacters ⇒ the process-spawn counter stays at zero and no canary file appears. | SEC-TH-11 |
| **N-07** | HTTP framing battery: CL+TE, duplicate TE, `TE: gzip`, `Content-Length: +5`, `Content-Length: 0x10`, GET-with-body, chunk-size overflow, 1 MiB header, CRLF in a header, slow drip ⇒ all rejected or capped; never two requests parsed from one connection. | SEC-TH-12, -13, -14, -15 |
| **N-08** | `Range: bytes=0-1,2-3,…` (100 overlapping ranges) ⇒ single `200` or single-range `206`; never `multipart/byteranges`; memory flat. | SEC-TH-16 |
| **N-09** | Path battery: `..%2f`, `%252e%252e%252f`, absolute path, NUL byte, backslash separators, over-long path, and a symlink + Finder alias pointing at `~/.ssh` ⇒ all refused, and an `openat` audit hook shows no open outside the library root. | SEC-TH-17, -18, -21 |
| **N-10** | Media-parser battery: malformed DFF, WavPack, ID3, MP4, Matroska, PNG-with-EXIF and a 100 000×100 000 declared-dimension image ⇒ the parent process is unaffected, the child is killed at the timeout, the file is quarantined, and peak parent RSS never exceeds the cap. | SEC-TH-19, -20 |
| **N-11** | `Browse` with `RequestedCount = 2^31-1` and a huge `Filter` ⇒ response capped at the configured page size; RSS flat. | SEC-TH-22 |
| **N-12** | Admin-API battery: wrong `Host`, missing custom header, missing/incorrect CSRF token, cross-origin `fetch` (simulated with a rebinding DNS name resolving to both `127.0.0.1` and the LAN IP), and a token file with mode `0644` ⇒ all refused; no state change; the token never appears in a log. | SEC-TH-23, -24, -25 |
| **N-13** | Release gate: `cargo audit` and `cargo deny check` fail on a seeded advisory and on a banned crate; `spctl --assess --type execute` and `codesign --verify --deep --strict` pass on the notarised artefact; the AGPL §13 source route serves the exact build's source archive (including `Cargo.lock` and build scripts) to an unauthenticated network client. | SEC-TH-26, -28 |

---

## 13. Sources

All URLs below were fetched during this research pass (directly or via the tool's content extraction). Access dates: 2026-11.

**CVE records (official MITRE CVE List API / cve.org / NVD)**
1. https://cveawg.mitre.org/api/cve/CVE-2014-8361
2. https://cveawg.mitre.org/api/cve/CVE-2013-5122
3. https://cveawg.mitre.org/api/cve/CVE-2014-8244
4. https://cveawg.mitre.org/api/cve/CVE-2015-2051
5. https://cveawg.mitre.org/api/cve/CVE-2018-10562
6. https://cveawg.mitre.org/api/cve/CVE-2020-12695
7. https://cveawg.mitre.org/api/cve/CVE-2020-28926
8. https://cveawg.mitre.org/api/cve/CVE-2022-26505
9. https://cveawg.mitre.org/api/cve/CVE-2023-33476
10. https://cveawg.mitre.org/api/cve/CVE-2023-47430
11. https://cveawg.mitre.org/api/cve/CVE-2024-51442
12. https://cveawg.mitre.org/api/cve/CVE-2026-29508
13. https://cveawg.mitre.org/api/cve/CVE-2013-2738
14. https://cveawg.mitre.org/api/cve/CVE-2019-12110
15. https://cveawg.mitre.org/api/cve/CVE-2019-12111
16. https://cveawg.mitre.org/api/cve/CVE-2019-12112
17. https://cveawg.mitre.org/api/cve/CVE-2019-12113
18. https://cveawg.mitre.org/api/cve/CVE-2019-12114
19. https://cveawg.mitre.org/api/cve/CVE-2019-12115
20. https://cveawg.mitre.org/api/cve/CVE-2011-3192
21. https://cveawg.mitre.org/api/cve/CVE-2023-44487
22. https://nvd.nist.gov/vuln/detail/cve-2014-8361
23. https://nvd.nist.gov/vuln/detail/cve-2026-29508
24. https://nvd.nist.gov/vuln/detail/CVE-2023-34411
25. https://nvd.nist.gov/vuln/detail/CVE-2021-32714
26. https://nvd.nist.gov/vuln/detail/CVE-2020-35863
27. https://www.cve.org/CVERecord?id=CVE-2020-12695

**CERT/CC and vendor advisories**
28. https://www.kb.cert.org/vuls/id/447516 (CERT/CC VU#447516 — Linksys SMART WiFi JNAP)
29. https://www.access.redhat.com/security/cve/cve-2020-12695 (Red Hat — CallStranger, reproduces CERT/CC mitigations)
30. https://www.huawei.com/en/psirt/security-advisories/2020/huawei-sa-20200701-01-upnp-en (Huawei PSIRT — CallStranger)
31. https://www.tenable.com/plugins/nessus/83185 (Nessus plugin — Realtek SDK miniigd SOAP RCE)
32. https://www.keysight.com/it/en/strikes/exploits/misc/cve_2014_8361_realsdk_upnp_soap_rce.xml (Keysight strike — CVE-2014-8361)

**Botnet / vendor reporting**
33. https://www.trendmicro.com/en/research/19/c/upnp-enabled-connected-devices-in-home-unpatched-known-vulnerabilities.html (Satori / CVE-2014-8361)
34. https://www.zscaler.com/it/blogs/security-research/sneak-peek-recent-iot-attacks (RIFT botnet reusing the miniigd exploit)
35. https://www.trendaisecurity.com/en/resources-insights/deep-research/mirai-variant-spotted-using-multiple-exploits-targets-various-routers

**DDoS / SSDP reflection**
36. https://www.cloudflare.com/learning/ddos/ssdp-ddos-attack/
37. https://blog.cloudflare.com/reflections-on-reflections (Gatebot reflection statistics, incl. SSDP)
38. https://www.tandfonline.com/doi/full/10.1080/00051144.2021.1885587 (Automatika 62(1), 2021 — reflection DDoS via IoT/SSDP)
39. https://hrcak.srce.hr/en/clanak/391341 (abstract mirror of the above)
40. https://par.nsf.gov/servlets/purl/10296666 (NSF PAR — UPnP IoT security overview: advertisement forgery/flooding, discovery flooding, spoofed-discovery amplification 30.8×)
41. https://techdocs.akamai.com/prolexic/reference/attack-types (Akamai attack types: SSDP Flood, SSDP Reflection)
42. https://support.radware.com/ci/okcsFattach/get/16387_3 (SSDP DDoS attack mitigation, ~30× amplification)
43. https://www.myrasecurity.com/en/knowledge-hub/ssdp/ (CISA-cited 30× amplification)
44. https://nsfocusglobal.com/wp-content/uploads/2021/09/2021-Mid-Year-DDoS-Attack-Landscape-Report.pdf (2021 reflective-UDP landscape; SSDP/DVR reflection)
45. https://infishark.com/blogs/learn/what-is-ssdp-and-why-is-it-a-security-problem (SSDP amplification overview)
46. https://radar.cloudflare.com/reports/ddos-2021-q4

**UPnP protocol / specification**
47. https://upnp.org/resources/documents/UPnP_UDA_tutorial_July2014.pdf (UDA tutorial — NOTIFY advertisement format, GENA NOTIFY)
48. https://learn.espilon.net/learn/wiki/protocol/ssdp-upnp (SSDP/UPnP overview; no authentication on the control plane; `AddPortMapping` abuse)
49. https://github.com/seattlerb/upnp/blob/master/lib/UPnP/SSDP.rb (reference NOTIFY/M-SEARCH implementation, multicast membership handling)

**ReadyMedia / MiniDLNA**
50. https://cveawg.mitre.org/api/cve/CVE-2013-2738 · https://cveawg.mitre.org/api/cve/CVE-2013-2739 · https://cveawg.mitre.org/api/cve/CVE-2013-2745
51. https://security-tracker.debian.org/tracker/source-package/minidlna (Debian tracker — full ReadyMedia CVE list)
52. https://sourceforge.net/projects/minidlna/files/minidlna/1.3.3/ (ReadyMedia release notes 1.0.25 → 1.3.3)
53. https://sourceforge.net/p/minidlna/bugs/355/ (chunked-encoding heap overflow root cause)
54. https://sourceforge.net/p/minidlna/bugs/361/ (CVE-2023-47430, `SendContainer()` stack overflow)
55. https://sourceforge.net/p/minidlna/bugs/364/ (CVE-2024-51442, config-file command injection)
56. https://www.rootshellsecurity.net/remote-heap-corruption-bug-discovery-minidlna/ (CVE-2020-28926 — `strtol` signedness bug)
57. https://blog.coffinsec.com/0day/2023/05/31/minidlna-heap-overflow-rca.html (CVE-2023-33476 — `ParseHttpHeaders()` boolean-assignment bug → `memmove` OOB)
58. https://www.openwall.com/lists/oss-security/2022/03/03/1 (CVE-2022-26505 DNS rebinding disclosure)
59. https://github.com/mselbrede/CVE-2024-51442 (researcher write-up: `db_dir` command injection)
60. https://nvd.nist.gov/vuln/detail/cve-2026-29508 and https://www.vulncheck.com/advisories/readymedia-stack-buffer-overflow-via-dff-parser (DFF VLA stack overflow)

**Gerbera**
61. https://github.com/gerbera/gerbera/security/advisories (no published advisories)
62. https://github.com/gerbera/gerbera/blob/master/ChangeLog.md
63. https://github.com/gerbera/gerbera/commit/fea27f16a1fb0f8b282e71f071fbc89c2be04157 (WavPack stack overflow fix)
64. https://github.com/gerbera/gerbera/commit/b3d4b84a402e258fe77e449ef3c90d5b2d2ed204 (libupnp hostname validation — DNS-rebinding mitigation)

**Rust advisories (RustSec / GitHub Advisory Database via OSV)**
65. https://rustsec.org/ and https://github.com/rustsec/advisory-db
66. https://rustsec.org/advisories/RUSTSEC-2026-0195.html · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0195 (quick-xml unbounded namespace allocation)
67. https://rustsec.org/advisories/RUSTSEC-2026-0194.html · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0194 (quick-xml quadratic attribute check)
68. https://api.osv.dev/v1/vulns/GHSA-7gf7-jv65-wjmh (xml-rs DoS, CVE-2023-34411)
69. https://rustsec.org/advisories/RUSTSEC-2021-0143.html · https://api.osv.dev/v1/vulns/RUSTSEC-2021-0143 (kamadak-exif infinite loop) · https://api.osv.dev/v1/vulns/GHSA-px9g-8hgv-jvg2
70. https://rustsec.org/advisories/RUSTSEC-2021-0078.html · https://api.osv.dev/v1/vulns/RUSTSEC-2021-0078 (hyper lenient Content-Length → smuggling)
71. https://rustsec.org/advisories/RUSTSEC-2021-0020.html · https://api.osv.dev/v1/vulns/RUSTSEC-2021-0020 (hyper multiple Transfer-Encoding → smuggling)
72. https://github.com/hyperium/hyper/security/advisories/GHSA-5h46-h7hh-c6x9 (hyper chunked integer overflow, CVE-2021-32714)
73. https://api.osv.dev/v1/vulns/GHSA-h3qr-rq2j-74w4 (hyper request smuggling, CVE-2020-35863)
74. https://api.osv.dev/v1/vulns/RUSTSEC-2020-0008 · https://api.osv.dev/v1/vulns/RUSTSEC-2021-0079 · https://api.osv.dev/v1/vulns/RUSTSEC-2017-0002 (hyper)
75. https://rustsec.org/advisories/RUSTSEC-2019-0033.html · https://api.osv.dev/v1/vulns/RUSTSEC-2022-0022 (http crate)
76. https://rustsec.org/advisories/RUSTSEC-2023-0034.html · https://rustsec.org/advisories/RUSTSEC-2024-0332.html · https://rustsec.org/advisories/RUSTSEC-2026-0258.html (h2)
77. https://seanmonstar.com/blog/hyper-http2-continuation-flood/
78. https://api.osv.dev/v1/vulns/RUSTSEC-2024-0336 · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0348 (rustls)
79. https://api.osv.dev/v1/vulns/RUSTSEC-2020-0073 · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0115 · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0116 · https://api.osv.dev/v1/vulns/RUSTSEC-2026-0117 (image / imageproc)
80. https://securitylab.github.com/advisories/ (GHSL-2024-280 — Matroska CodecPrivate UAF in GStreamer, CVE-2024-47834)

**Rust crate source (behavioural verification)**
81. https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md (DTD entity resolution behaviour)
82. https://github.com/RazrFalcon/roxmltree/blob/master/src/parse.rs (`ParsingOptions::default()` — `allow_dtd: false`, `nodes_limit: u32::MAX`; `Error::DtdDetected`; `Error::EntityReferenceLoop`)
83. https://github.com/RazrFalcon/roxmltree#alternatives (feature matrix: no complete DTD support)
84. https://github.com/tafia/quick-xml/blob/master/src/events/mod.rs (`Event::GeneralRef`, `Event::DocType`)

**Supply chain / crates.io**
85. https://blog.rust-lang.org/2022/05/10/malicious-crate-rustdecimal.html
86. https://blog.rust-lang.org/inside-rust/2023/09/01/crates-io-malware-postmortem/
87. https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/
88. https://dl.acm.org/doi/10.1145/3708821.3735340 (typosquatting attacks on the Rust ecosystem)
89. https://safeguard.sh/resources/blog/cargo-supply-chain-attacks-typosquatting-and-malicious-crates-on-cratesio (crates.io hyphen/underscore normalisation)
90. https://safeguard.sh/resources/blog/typosquatting-on-cratesio-report
91. https://github.com/EmbarkStudios/cargo-deny (licenses / bans / advisories checks)

**Licensing**
92. https://raw.githubusercontent.com/spdx/license-list-data/main/text/AGPL-3.0-only.txt (AGPL-3.0 §0 definitions and §13 verbatim)
93. https://ffmpeg.org/legal.html (FFmpeg licence and LGPL compliance checklist)

**macOS**
94. https://support.apple.com/guide/security/controlling-app-access-to-files-secddd1d86a6/web (TCC, Full Disk Access, removable/network volumes)
95. https://support.apple.com/guide/security/local-network-privacy-prd1be8a0f88/web (Local Network privacy)
96. https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-secb0694df1a/web (Gatekeeper and runtime protection)
97. https://support.apple.com/guide/security/app-code-signing-process-sec3b77f1b8f6/web (app code signing)
98. https://nektony.com/how-to/manage-app-permissions-mac (Local Network permission semantics; denial is silent)
99. https://keith.github.io/xcode-man-pages/launchd.plist.5.html (Apple `launchd.plist(5)` man page: `KeepAlive`, `RunAtLoad`, `ProcessType`, `UserName`/`GroupName`, `Sockets`)
100. https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html (Apple File System Basics)

**General web-security references**
101. https://portswigger.net/web-security/file-path-traversal (traversal + canonicalise-then-contain defence)
102. https://dipsylala.github.io/FlawFixingGuidance/CWE-22/python/ (why textual normalisation is insufficient)
103. https://man7.org/linux/man-pages/man2/open.2.html (`O_NOFOLLOW`)
104. https://man7.org/linux/man-pages/man2/openat.2.html (directory-relative open)
105. https://security-tracker.debian.org/tracker/CVE-2019-12111 (MiniUPnPd fix commits)
106. https://infishark.com/blogs/learn/what-is-ssdp-and-why-is-it-a-security-problem

---

## 14. Open questions to close before requirements freeze

1. **Q-01** Does `roxmltree`'s entity-loop detector bound a *nested* (product-of-expansions) billion-laughs document, or must we rely solely on an explicit node/size cap? → write a PoC and measure.
2. **Q-02** Which Rust XML crate(s) will we standardise on, and does its DTD/entity behaviour meet M-02 by default? (`roxmltree` default = DTD rejected; `quick-xml` = caller must reject `DocType` and never resolve `GeneralRef`.)
3. **Q-03** On macOS, how do we resolve a **Finder alias** to its target for containment, and does `O_NOFOLLOW` affect aliases at all?
4. **Q-04** Does a `LaunchAgent` inherit the user's TCC attribution such that a media folder inside `~/Movies` is readable, and what is the exact behaviour when it is not?
5. **Q-05** What are the real `ProcessType: Background` throttle values on current macOS, and do they apply to the whole process or per job?
6. **Q-06** Do our target renderers accept a "NOTIFY on real events only" advertisement schedule, or do some require periodic `ssdp:alive`?
7. **Q-07** Do we accept SSDP M-SEARCH from other subnets (MiniDLNA did) or not (strictly local)? This is a product/interop decision with a security weight.
8. **Q-08** Can a pure-Rust parser stack cover our formats without FFmpeg? If not, is the LGPL-only dynamic-link route acceptable to the project?
9. **Q-09** How will the AGPL §13 source offer be served — a route on the media origin, an in-app page, or a bundled archive — and how is the "exact build" correspondence established (e.g. `cargo auditable` + a release manifest)?
10. **Q-10** Servdlna was not located and its advisories are unexamined — should we spend effort there, or drop it as a comparator?
