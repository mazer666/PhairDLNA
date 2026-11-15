# F-01 — SSDP discovery and advertisement

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-001, INT-002, INT-003, INT-004, INT-005, INT-006, INT-007,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
INT-008, INT-009, INT-010, INT-011, INT-012, INT-013, INT-014, INT-015, INT-016, INT-017, INT-018, INT-019

## 1. Purpose and user story

**SSDP** (Simple Service Discovery Protocol) is how a UPnP device announces itself on a local network and how a client finds it: a small text protocol over UDP
port 1900. The device sends *advertisements* (`NOTIFY`); the client sends *searches* (`M-SEARCH`) and the server answers that one client directly.

**User story.** As a user, I want my television to find my media server automatically after I start it, so that I do not have to add a server by IP address, and
I want the server to stop announcing itself when I shut it down, so that my television stops showing a dead entry.

## 2. Scope

**In scope** — the SSDP socket, the `M-SEARCH` parser, the search-target matching table, the advertisement timer, the interface selection and the per-source
rate limit (module `ssdp`, ARCHITECTURE §4.2); the response headers required by UDA 1.1 (standards §2.4) and the `USN` identity forms (standards §2.3); IPv4
multicast `239.255.255.250:1900` and IPv6 link-local `FF02::C:1900` (standards §2.1).

**Explicitly out of scope** — the device description document (F-02; only its URL is advertised here); the UPnP Internet Gateway Device protocol, which
`INT-058` forbids and the same socket must never answer; UDA 2.0 additions (`BOOTID.UPNP.ORG`, `OPT`/`01-NLS:`, `ssdp:update`), because the baseline pins
`UPnP/1.1` in `SERVER` (INT-019); mDNS advertisement (INT-018); IPv6 site scope `FF05::C:1900`, which INT-007 does not name.

## 3. Functional requirements

**INT-001** — The server shall answer `M-SEARCH` requests on UDP 1900 within at most 1 second and shall answer with `HTTP/1.1 200 OK` and the headers `ST`,
`USN`, `LOCATION`, `CACHE-CONTROL: max-age=1800`, `SERVER`, `DATE` and `EXT`. [SPEC] These are exactly the UDA 1.1 response headers (standards §2.4, §8.1 D6).
[DESIGN] The 1-second budget applies to a *unicast* `M-SEARCH` (standards §2.2); a multicast reply is delayed inside the `MX` window (INT-008). Raised in §12.

**INT-002** — The server shall match `ST` against `ssdp:all`, `upnp:rootdevice`, the advertised MediaServer device type and the advertised service types, and
shall ignore an `ST` it does not advertise. [SPEC] The matching table is normative (standards §2.3, §8.1 D10). [DESIGN] The advertised set is `upnp:rootdevice`,
`uuid:<UDN>`, the MediaServer device type and the two service types, so an `ssdp:all` search yields five answers — the UDA formula "3+2*d+s" gives 5 for a root
device with two services.

**INT-003** — The server shall send its SSDP response unicast to the address and port of the requester. [SPEC] The unicast reply is a MUST in UDA 1.1 (standards
§2.4, §8.1 D4).

**INT-004** — The server shall send periodic `NOTIFY` advertisements with `NT: upnp:rootdevice` and `NTS: ssdp:alive`, re-advertising at an interval no longer
than a third of the advertised `max-age`, and shall send `ssdp:byebye` for each advertised `NT` on graceful shutdown. [SPEC] `ssdp:alive`/`ssdp:byebye` are the
UDA withdrawal mechanism (standards §2.5, §8.1 D11). [DESIGN] With `max-age = 1800` the bound is 600 s; the research records `max-age/4` as the conservative
recommendation (standards §2.5) — 450 s satisfies the baseline and is our value.

**INT-005** — The `USN` header shall have the form `uuid:<UDN>::<target>` where `<target>` is `upnp:rootdevice`, `urn:schemas-upnp-org:device:MediaServer:1`,
`urn:schemas-upnp-org:service:ContentDirectory:1`, `urn:schemas-upnp-org:service:ConnectionManager:1`, or the bare `uuid:<UDN>`. [SPEC] The prefix before `::`
must equal the `UDN` element of the device description (standards §2.3, §8.1 D9).

**INT-006** — The `LOCATION` header shall contain an absolute HTTP URL of the device description document that is reachable **on the network interface the
request arrived on**. [SPEC] `LOCATION` must name an address reachable on that interface, and a multi-homed device must answer on the receiving interface
(standards §2.4, §8.1 D12). [DESIGN] The URL is built from the interface-local address, never a wildcard or a loopback address.

**INT-007** — The server shall bind SSDP to the IPv4 multicast group `239.255.255.250:1900` and shall support the IPv6 link-local group `FF02::C:1900`. [SPEC]
The IPv4 group and port are reserved by IANA for SSDP; the IPv6 link-local scope is `FF02::C` on port 1900 (standards §2.1, §2.6, §8.1 D13).

**INT-008** — The server shall honour the `MX` header of an `M-SEARCH` by choosing a uniformly random delay inside `[0, min(MX, 5))` seconds, and shall ignore
an `M-SEARCH` that carries no `MX` header. [SPEC] `MX` must be at least 1 second, a device may assume a value of at most 5, and a search without `MX` must be
*silently discarded and ignored* (standards §2.2, §8.1 D2–D3).

**INT-009** — The server shall select the sending interface by the route of the incoming request and shall never answer a request that arrived on an interface
excluded by configuration. [SPEC] The receiving interface fixes the reply interface (standards §2.4, §8.1 D12). [DESIGN] The excluded set comes from the
interface configuration (CFG-006); such a request is dropped before parsing.

**INT-010** — The server shall ignore malformed `M-SEARCH` datagrams without logging them at error level and without affecting the next valid request. [DESIGN]
SSDP runs on an untrusted network (SEC-018); a malformed datagram is a normal event, not an error condition.

**INT-011** — The server shall rate-limit discovery responses per source address so that a single host cannot force an unbounded response rate. [DESIGN] This
limit is our own design; no UPnP or DLNA specification requires it. [DESIGN] A token bucket per source address: an exhausted bucket drops the answer silently (a
discovery client retries) and increments a counter. Rate and burst size are configuration values with documented defaults; the counter appears in the status
output.

**INT-012** — The server shall expose the configured `friendlyName`, `manufacturer`, `modelName` and `UDN` in the SSDP response headers consistently with the
device description. [SPEC] The `UDN` appears in `USN` and the product identity in `SERVER`; SSDP defines no headers for the other three values (standards §2.4).
[DESIGN] The reading used here: all four come from one configuration source and are the same strings the device description exposes (F-02). Raised in §12.

**INT-013** — The server shall answer `M-SEARCH` only when the `MAN` header contains the quoted token `ssdp:discover`, and shall not echo the header otherwise.
[SPEC] The `MAN` value must be exactly `"ssdp:discover"`, quoted (standards §2.2).

**INT-014** — The server shall use a stable `UDN` across restarts for the same installation unless the user resets the identity explicitly, and the `USN` prefix
shall equal that `UDN`. [SPEC] The `UDN` is persistent device state (ARCHITECTURE §9). [DESIGN] The identity lives in the configuration directory and is
regenerated only by an explicit user action.

**INT-015** — The server shall provide a CLI switch to print the exact SSDP headers it would send, so that interoperability problems can be diagnosed without
packet capture. [DESIGN] The switch prints the advertisement and one example answer byte for byte as the socket would send them; it is the same output as the
status command (OPS-005).

**INT-016** — The server shall set the multicast time-to-live of its advertisements to 2, so that they do not leave the local link. [SPEC] The IPv4 multicast
TTL should default to 2 (standards §2.1, §8.1 D14).

**INT-017** — The server shall answer `M-SEARCH` for the device type `urn:schemas-upnp-org:device:MediaServer:1` in addition to the root device and the service
types. [SPEC] This is the device type advertised in the device description (standards §1.1, §8.1 D15).

**INT-018** — The server shall not rely on mDNS for discovery; SSDP is the DLNA discovery mechanism and no mDNS advertisement shall be sent. [SPEC] mDNS (RFC
6762) is name resolution and never mentions SSDP or UPnP; DLNA device classes are defined in terms of SSDP discovery plus UPnP descriptions (standards §2.7,
§8.1 D16).

**INT-019** — The server shall expose the version of the UPnP architecture it implements in the `SERVER` header, in the form `<OS>/<version> UPnP/1.1
PhairDLNA/<version>`. [SPEC] The `SERVER` format is normative (standards §2.4, §8.1 D8); the product part carries the server version (OPS-012).

## 4. Technical requirements

- **Module.** `phairdlna-core/src/ssdp/` owns the socket, the parser, the matching table, the advertisement timer and the interface selection; it knows nothing
  about the catalogue (ARCHITECTURE §4.2). Identity and description URL arrive through a small trait, so the matching logic is testable without a socket.

- **Request shape.** The parser accepts the UDA request form — `M-SEARCH * HTTP/1.1` with `HOST`, `MAN: "ssdp:discover"`, `MX`, `ST`, `USER-AGENT` (standards
  §2.2). Header names are case-insensitive; `MAN` is matched as a quoted token. [DESIGN] Every field is length-bounded before it is copied (SEC-060, SEC-011).

- **Answer construction.** One answer per matching target: `ST` the matched target, `USN` the matching form (INT-005), `LOCATION` the interface-local
  description URL (INT-006), `CACHE-CONTROL` `max-age=1800`, `DATE` the generation time, `EXT` empty, `SERVER` per INT-019.

- **Advertisement.** A timer sends one `NOTIFY` per advertised `NT` (five targets) with `NTS: ssdp:alive`, `HOST: 239.255.255.250:1900` and the same
  `CACHE-CONTROL`, `LOCATION`, `SERVER` and `USN` as a search answer (standards §2.5). On shutdown the same five targets are sent with `NTS: ssdp:byebye` before
  the socket closes (INT-004, OPS-008).

- **Interfaces.** The reply interface is the one the request arrived on (INT-009); the advertisement is sent once per UPnP-enabled interface, each with its own
  `LOCATION` (standards §2.5, §8.1 D12).

- **Delay.** The answer is scheduled at a uniformly random instant inside `[0, min(MX, 5))` (INT-008); the random source is seeded once per process.

- **IPv6.** Where the platform provides a link-local address, the server joins `FF02::C:1900` and uses a link-local literal in `LOCATION` (standards §2.6, §8.1
  D13). [DESIGN] IPv6 sits behind a configuration flag; IPv4 is the critical path.

- **No mDNS.** No socket other than the SSDP socket and the HTTP sockets is opened for discovery (INT-018, SEC-091).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: an `M-SEARCH` is answered inside the `MX` window and, for a unicast search, within 1 s (INT-001). Method: protocol conformance test
  timestamping request and answer, ≥ 30 iterations, median and p95 reported (PERF-011).

- [DESIGN] Target: the server reaches the "ready to answer discovery" state within 2 s of start-up (PERF-002). Method: timestamp from process start to the
  first successful socket bind, on the reference hardware.

- [DESIGN] Target: idle CPU usage stays below 1 % averaged over one minute (PERF-013). Method: `top` sampling over 60 s while the advertisement timer runs.

- [DESIGN] Target: the advertisement timer performs no catalogue or filesystem access (ARCHITECTURE §4.2). Method: code review plus an assertion that no
  catalogue handle is opened.

## 6. Usability and accessibility

- The feature has no graphical surface; its user-visible effect is the device appearing in a television's server list and disappearing again when the server
  stops.

- [UX-010] The onboarding text shall say "your TV can now find this server on your network" rather than name the protocol.

- [UX-004] When discovery fails, the error the user sees shall name the interface tried and the address advertised, with the client address redacted (OPS-007,
  SEC-094).

- [UX-001] Everything here is operable without a GUI; the diagnostic switch (INT-015) is the only user-facing surface.

- [DOC-005] *M-SEARCH*, *NOTIFY*, *advertisement*, *multicast*, *unicast* and *MX* are explained at first use.

## 7. Security and privacy

- The SSDP peer is untrusted (SEC-018): the server performs no action on its behalf beyond answering it. A search causes no file access, no catalogue read and
  no outbound connection.

- Malformed and oversized datagrams are dropped before allocation (INT-010, SEC-060, SEC-011).

- The rate limit (INT-011) bounds the work one host can force; the advertisement itself is a fixed, small number of datagrams.

- The multicast TTL of 2 keeps advertisements on the local link (INT-016) and the server invisible outside the home network.

- The advertised `LOCATION` exposes the server's LAN address — the address the network needs anyway. No file name, directory layout or user data is disclosed
  (SEC-093, PLT-008).

- No outbound connection other than the multicast advertisement and the unicast answers is made (SEC-091); discovery never contacts the internet.

- Log records of answers carry the redacted client address only (OPS-007, SEC-094).

## 8. Interoperability

- Normative: UPnP Device Architecture 1.1 §2 — multicast group and port, `M-SEARCH` format, `MX` rules, the search-target matching table, the unicast-reply
  MUST, the response header set, the `USN` forms, `ssdp:alive`/`ssdp:byebye` (standards §2.1–§2.5).

- [OBSERVED] MiniDLNA, ReadyMedia and rclone all use `CACHE-CONTROL: max-age=1800` and re-advertise on that cycle (standards §2.5); our cadence is compatible.

- [OBSERVED] The OCF MediaServer template renders the ConnectionManager service identifier as `ConnectorManager`, an evident typo; every shipped implementation
  uses `ConnectionManager` (standards §3.1). We use the shipped spelling (INT-005, INT-044).

- [OBSERVED] Control points in the field send `M-SEARCH` with `ST: ssdp:all` far more often than with a specific target, so the five-answer behaviour of
  INT-002 is load-bearing.

- [OPEN] The `ssdp://239.255.255.250:1900` URI spelling was not found in normative UDA text (standards §2.1, §9 Q9). The endpoint is normative; the spelling is
  not used.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| `M-SEARCH` without `MX` | ignored silently, no answer | INT-008, standards §2.2 |
| `MAN` value not `"ssdp:discover"` | ignored, header never echoed | INT-013 |
| `ST` not advertised | ignored | INT-002 |
| Datagram that is not a valid `M-SEARCH` | dropped, no error-level log | INT-010 |
| Answer rate from one host above the limit | answer dropped, counter incremented | INT-011 |
| Request on a configuration-excluded interface | dropped before parsing | INT-009 |
| IPv6 link-local address unavailable | IPv4 discovery continues | INT-007 |
| Shutdown | `ssdp:byebye` for every advertised `NT`, then the socket closes | INT-004, OPS-008 |

No failure of this feature may take down the HTTP task or the catalogue task: a bad datagram produces a dropped packet, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a running server, *when* a client sends `M-SEARCH` with `MAN: "ssdp:discover"`, `MX: 1` and `ST: upnp:rootdevice`, *then* a `HTTP/1.1 200
OK` answer arrives carrying `ST`, `USN`, `LOCATION`, `CACHE-CONTROL: max-age=1800`, `SERVER`, `DATE` and `EXT`, and `SERVER` has the form `<OS>/<version>
UPnP/1.1 PhairDLNA/<version>`. Requirements: INT-001, INT-019 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-02** — *Given* a search sent to the multicast group, *when* the answer is observed, *then* it is a unicast datagram addressed to the source IP and port of
the request. Requirements: INT-003 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-03** — *Given* a search without an `MX` header, *when* the `MX` window would have elapsed, *then* no answer datagram was sent and no error was logged; and
*given* a search with `MX: 30`, *then* the delay lies inside `[0, 5)` seconds. Requirements: INT-008, INT-010 · Verification: protocol conformance test ·
Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-04** — *Given* a search with `ST: ssdp:all`, *when* the answers are counted, *then* exactly five answers arrive, one per advertised target:
`upnp:rootdevice`, `uuid:<UDN>`, the MediaServer device type and the two service types. Requirements: INT-002, INT-017 · Verification: protocol conformance test
· Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-05** — *Given* searches for `upnp:rootdevice`, `uuid:<UDN>`, the MediaServer device type and each service type, *when* the answers are collected, *then*
each carries the matching `ST` and the `USN` form `uuid:<UDN>::<target>`, and a search for an unadvertised `ST` produces no answer at all. Requirements:
INT-002, INT-005, INT-017 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-06** — *Given* a server with two UPnP-enabled interfaces, *when* a search arrives on interface A, *then* the `LOCATION` header names an address of
interface A and the answer is sent from interface A; and *given* a search on an interface excluded by configuration, *then* no answer is sent. Requirements:
INT-006, INT-009 · Verification: manual protocol check on a two-interface host, plus a protocol conformance test · Planned location: `qa/device-matrix.toml` and
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-07** — *Given* the advertisement timer, *when* one cycle elapses, *then* a `NOTIFY` with `NTS: ssdp:alive` is sent for every advertised `NT` and the
interval between two cycles is no longer than a third of the advertised `max-age`; and *given* a graceful shutdown, *then* one `NOTIFY` with `NTS: ssdp:byebye`
is sent for every advertised `NT` before the socket closes. Requirements: INT-004, OPS-008 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-08** — *Given* the server is started, *when* the sockets are inspected, *then* the IPv4 socket is a member of `239.255.255.250:1900` and, where the
platform provides a link-local address, an IPv6 socket is a member of `FF02::C:1900`; and *given* an advertisement capture, *then* the multicast time-to-live is
2. Requirements: INT-007, INT-016 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-09** — *Given* a datagram that is not a valid `M-SEARCH`, *when* it is delivered, *then* it is dropped, no error-level log record is produced and the next
valid search is answered normally. Requirements: INT-010 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-10** — *Given* one source address sends searches faster than the configured rate, *when* the limit is exceeded, *then* the excess answers are dropped, a
counter is incremented and a source address with its own allowance is still answered. Requirements: INT-011 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-11** — *Given* a search whose `MAN` header is not the quoted token `"ssdp:discover"`, *when* it is received, *then* no answer is sent and the `MAN` header
never appears in any response. Requirements: INT-013 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-12** — *Given* the server is restarted without an identity reset, *when* the `UDN` is read, *then* it is unchanged and the `USN` prefix equals it.
Requirements: INT-014 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-13** — *Given* the diagnostic switch is invoked, *when* it prints, *then* the output contains the exact header block of one advertisement and of one
answer, byte for byte as the socket would send them. Requirements: INT-015, OPS-005 · Verification: automated test · Planned location:
`crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-14** — *Given* the server is running, *when* the network is observed for one advertisement cycle, *then* no multicast traffic to port 5353 and no `.local.`
name advertisement is sent. Requirements: INT-018 · Verification: manual protocol check · Planned location: `qa/device-matrix.toml`

**AC-15** — *Given* the configured identity, *when* the device description is fetched from the advertised `LOCATION`, *then* the `UDN`, `friendlyName`,
`manufacturer` and `modelName` it contains are the same values the server advertises. Requirements: INT-012 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Search handling, matching table, header set | `crates/phairdlna-core/tests/ssdp_discovery.rs` |
| Parser and delay-scheduling unit tests | `crates/phairdlna-core/src/ssdp/msearch.rs` (inline `#[cfg(test)]` module) |
| Advertisement timer and byebye sequence | `crates/phairdlna-core/src/ssdp/advertise.rs` (inline `#[cfg(test)]` module) |
| Interface selection and rate limiter | `crates/phairdlna-core/src/ssdp/interfaces.rs` (inline `#[cfg(test)]` module) |
| Diagnostic switch output | `crates/phairdlna-cli/tests/cli_diagnostics.rs` |
| Device behaviour on a multi-interface host | `qa/device-matrix.toml` |

**Method.** The matching table and the answer builder are pure functions over a parsed request, so they are exercised without a socket; a UDP fixture on the
loopback interface exercises the datagram path. A second host with two interfaces is required for AC-06 and is a manual protocol check.

**Dependencies.** F-02 (the description URL that `LOCATION` points at, and the identity strings of INT-012), F-05 (the documents served behind `LOCATION`), F-07
(the client profile that may influence the answer), and the platform interface enumeration (ARCHITECTURE §12).

## 12. Open questions and decisions

1. **[OPEN] The 1-second budget of INT-001 versus the `MX` window of INT-008.** INT-001 says the server answers "within at most 1 second"; INT-008 requires a
   delay of up to `min(MX, 5)` seconds, so a multicast search with `MX: 5` cannot satisfy both readings. The UDA text supports the split used here: the 1-second
   rule is stated for a *unicast* `M-SEARCH` (standards §2.2). *Closes when:* the owner confirms the reading or amends the text. 2. **[OPEN] INT-012 names four
   values that SSDP has no headers for.** `friendlyName`, `manufacturer` and `modelName` do not appear in the UDA response header set (standards §2.4); only
   `UDN` (in `USN`) and the product identity (in `SERVER`) travel on the wire. The reading used here is "the same configured identity that the device
   description exposes". *Closes when:* the owner confirms it or amends INT-012. 3. **[OPEN] IPv6 link-local availability on macOS in CI and in the sandbox**
   (standards §9 Q10). The IPv6 path is behind a configuration flag and IPv4 is the interoperability-critical path (standards §2.6). *Closes when:* a CI host
   with a link-local address is available. 4. **[OPEN] Rate-limit values for INT-011.** The baseline fixes no rate, burst size or window. *Proposed decision,
   not in the baseline:* a default of 4 answers per source address per second with a burst of 8, configurable. Rationale: a discovery client sends a small burst
   at start-up and the limit must not break it. *Closes when:* the owner fixes the defaults. 5. **[OPEN] Advertisement cadence.** INT-004 sets the upper bound
   at `max-age/3` (600 s with `max-age=1800`); the research records `max-age/4` as the conservative upstream recommendation (standards §2.5). This document uses
   `max-age/4`. *Closes when:* the owner picks one value and states it in the configuration reference. 6. **[OPEN] Five answers to `ssdp:all`, or fewer.** The
   UDA count formula gives five for a media server with two services (standards §2.3), and the research summarises the surface as "4–5 distinct targets". We
   answer five. *Closes when:* a device round confirms that no client is confused by the bare `uuid:<UDN>` answer.
