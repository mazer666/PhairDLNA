# M4.5 — Security Review of the Whole Surface — 2026-11-26

Status: **Independent security review complete — all 8 normative mitigations verified**  
Owner role: Security & Privacy Engineer  
Version/date: 1.0 / 2026-11-26  
Requirements: `SEC-001` … `SEC-096`, `INT-057` … `INT-059`, `INT-071` … `INT-072`, `OPS-020` … `OPS-027`  
Baseline: `docs/04-security/SECURITY-MODEL.md`

---

## 1. Executive Summary

As part of Milestone 4.5, a systematic security audit of the PhairDLNA attack surface
and trust boundaries was conducted against `SECURITY-MODEL.md`.

All 8 normative security mitigations (`M-01` through `M-08`) were evaluated against the
current codebase, test suite, and configuration constraints. No open critical, high,
or medium security issues were identified.

---

## 2. Review of the Eight Normative Mitigations

| Mitigation | Goal | Primary Requirements | Verification & Evidence | Status |
| --- | --- | --- | --- | --- |
| **M-01: Untrusted multicast & discovery** | SSDP packet validation, drop spoofed unicast replies, ignore invalid datagrams | `SEC-012`, `SEC-063`, `INT-004`, `INT-010` | `crates/phairdlna-core/tests/ssdp_hardening.rs`, `crates/phairdlna-core/tests/parser_fuzz_bounds.rs` | **PASS** |
| **M-02: Untrusted XML & SOAP** | Refuse DTDs, external entities, nested entity bombs ("billion laughs"), enforce input bounds | `SEC-010`, `SEC-060`, `SEC-061` | `crates/phairdlna-core/tests/xml_hardening.rs`, `xml_reader.rs`, `soap_negative.rs` | **PASS** |
| **M-03: No shell execution** | Zero subshell invocations, no command string interpolation | `SEC-005`, `INT-058`, `INT-059` | `scripts/check-no-unsafe.sh` (QG-03) and absence of `std::process::Command` in media/streaming core | **PASS** |
| **M-04: HTTP framing & delivery limits** | Strict HTTP/1.1 framing, single/multi-range bounds, reject overlapping/amplification attacks | `SEC-011`, `SEC-065`, `PERF-001` | `crates/phairdlna-core/tests/http_framing.rs`, `http_range.rs`, `http_negative.rs` | **PASS** |
| **M-05: Path containment** | Strict root containment, symlink/alias checks, traversal defense (`../`, `..\\`, null bytes) | `SEC-001` … `SEC-004`, `SEC-068`, `SEC-069` | `crates/phairdlna-core/tests/path_containment.rs`, `symlink_policy.rs`, `parser_fuzz_bounds.rs` | **PASS** |
| **M-06: Local administration surface** | Restrict to loopback interface, mandatory random secret token, CSRF token on POST, non-standard custom header, no CORS | `SEC-071`, `SEC-072`, `OPS-020` … `OPS-027` | `crates/phairdlna-admin/tests/admin_surface.rs`, `admin_api.rs`, `crates/phairdlna-cli/tests/admin.rs` | **PASS** |
| **M-07: Bounded resource consumption** | Explicit bounds on DIDL response size, connection caps, read/write deadlines, file descriptors | `SEC-011`, `SEC-012`, `SEC-060` … `SEC-065` | `crates/phairdlna-core/tests/resource_bounds.rs`, `resource_exhaustion.rs`, `limit/tests.rs` | **PASS** |
| **M-08: Supply chain & provenance** | Exact build recipes, SBOM generation, unauthenticated HTTP source offer | `SEC-070`, `SUP-001` … `SUP-008`, `OPS-016` | `scripts/generate-release-artefacts.sh`, cargo audit (QG-09), cargo-deny license & ban check (QG-08, QG-10) | **PASS** |

---

## 3. Trust Boundary Analysis

1. **Multicast Socket (`0.0.0.0:1900`):**
   - Pure parser rejects malformed inputs without panicking.
   - Bounded reply delay and background task worker limit prevent socket flooding.
2. **Media Delivery HTTP Server (`0.0.0.0:8200`):**
   - Media root path containment refuses path traversals before filesystem access.
   - HTTP Range parsing validates range intervals against actual file sizes (`SEC-065`).
   - Capped listener enforces concurrency and timeout bounds.
3. **Administration Surface (`127.0.0.1:8210`):**
   - Bound strictly to loopback by default (`SEC-071`).
   - Rejects any DNS-rebinding attacks via Host header validation.
   - Enforces `X-PhairDLNA-Admin` custom header and CSRF token on state modifications (`SEC-072`).
4. **Local File System & Media Metadata:**
   - Image decoder bounds dimensions before buffer allocation (`SEC-067`).
   - Corrupt files and invalid playlists are quarantined/ignored without damaging media originals (`REL-001`).

---

## 4. Conclusion & Milestone Exit

All requirements for `M4.5` are satisfied. Milestone 4 (**Interoperability and Hardening**)
has met all exit criteria defined in `docs/07-project/ROADMAP.md`.
