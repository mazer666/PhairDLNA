# Research inputs — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / QA
Purpose: the evidence base that the requirements and the feature specifications cite.

---

## 1. What this directory is

The feature specifications cite their sources with a short prefix and a section
number, for example `standards §8.3 C12` or `security-landscape §5.1`. **Those
prefixes name the files in this directory.** They are the research briefs that were
produced while the requirements were written, and they are committed here so that
every `[SPEC]`, `[OBSERVED]` and `[DESIGN]` label in the specifications can be
checked against its origin.

| Cited as | File | Content |
| --- | --- | --- |
| `standards` | [standards.md](standards.md) | The UPnP and DLNA standards: SSDP, the device description, ContentDirectory, ConnectionManager, DIDL-Lite, HTTP delivery, the DLNA layer, with the primary source documents and URLs |
| `platform-integration` | [platform-integration.md](platform-integration.md) | macOS, Linux and Windows: the local-network permission, the firewall, code signing and notarisation, launchd, the Unicode and case-sensitivity model, the Rust options, and the service models |
| `security-landscape` | [security-landscape.md](security-landscape.md) | The attack surface of this product class: SSDP amplification, the GENA callback SSRF, XML and SOAP hardening, the CVE history of the existing servers, the HTTP framing defects, the local administration boundary and the supply chain |
| `qa-performance` | [qa-performance.md](qa-performance.md) | The test toolchain, protocol conformance, fuzzing, the measurement protocol and the gate matrix that this project's matrix grew out of |
| `ecosystem` | [ecosystem.md](ecosystem.md) | The existing DLNA servers, their strengths and weaknesses, and the competitive position |
| `clients-*` | [clients-a…](clients-a-samsung-lg-philips-panasonic.md) · [clients-b…](clients-b-sony-xbox-playstation.md) · [clients-d-foobar2000…](clients-d-foobar2000.md) · [clients-d-windows-media-center…](clients-d-windows-media-center.md) | The client behaviour research per device family |

## 2. What these documents are, and are not

- They are **research**: what the sources say, with the URL that was actually
  fetched. **Each brief declares its own tags in its own "how to read this document"
  section**, and they are not the same set:

  | Brief | Its tags |
  | --- | --- |
  | `standards.md` | `STANDARD`, `OBSERVED`, `ASSUMPTION`, `OPEN` |
  | `platform-integration.md` | `VERIFIED`, `ASSUMPTION`, `OPEN` |
  | `security-landscape.md` | `VERIFIED`, `ASSUMPTION`, `OPEN` |
  | `qa-performance.md` | `VERIFIED`, `ASSUMPTION`, `OPEN` |
  | `ecosystem.md` | `OBSERVED`, `ASSUMPTION`, `OPEN` |

  The project's own labels — `[SPEC]`, `[OBSERVED]`, `[DESIGN]`, `[OPEN]` — are
  defined in `../03-features/FEATURE-TEMPLATE.md` and mean something slightly
  different: they classify a *requirement*, not a piece of research.
- They are **not normative**. A requirement in
  `../01-product/TECHNICAL-REQUIREMENTS.md` is the normative text. Where a research
  document and a requirement disagree, the requirement wins and the research
  document is wrong.
- They are **immutable in substance**. A correction is added as a note, not by
  rewriting the finding, so that a citation still points at what was actually read.

## 3. The labels used across the project

| Label | Meaning |
| --- | --- |
| `[SPEC]` | The claim comes from a specification. The research document names the document and the URL |
| `[OBSERVED]` | The claim describes documented real-world behaviour, with a source |
| `[DESIGN]` | The claim is our own decision |
| `[OPEN]` | The question is unresolved; the research document says what would close it |

## 4. The identifier key

The research briefs use their own short identifiers, and one of them collides with
the project's own vocabulary. Here is the key.

| Identifier | Means | Lives in |
| --- | --- | --- |
| `M-01` … `M-08` | The eight **security mitigations** (SSDP hardening, XML and SOAP hardening, no shell, strict HTTP framing, path containment, isolation of untrusted work, bounded resources, supply chain) | `security-landscape.md` §4 and `../04-security/SECURITY-MODEL.md` §4 |
| `SEC-TH-nn` | A row of the threat table | `../04-security/SECURITY-MODEL.md` §5 |
| `D-n`, `P-n`, `C-n`, `M-n` (single letter), `L-n`, `H-n`, `E-n`, `N-n` | Obligation lists inside the standards brief (discovery, device description, ContentDirectory, ConnectionManager, DIDL-Lite, HTTP, GENA, DLNA layer) | `standards.md` §8 |
| `Q1` … `Q12` | The open questions of the standards brief | `standards.md` §9 |
| `PC-nn` | A protocol conformance check | `../05-quality/INTEROPERABILITY-MATRIX.md` §2 |
| `AC-nn` | An acceptance criterion | the feature specifications |
| `QG-nn` | A quality gate | `../05-quality/QUALITY-GATES.md` §3 |
| `G0` … `G6` | A milestone gate | `../05-quality/QUALITY-GATES.md` §4 |
| `M0` … `M5` | A **milestone** | `../07-project/ROADMAP.md` |
| `R-nn` | A risk | `../07-project/RISK-REGISTER.md` |
| `OD-nn` | An open owner decision | `../07-project/OPEN-DECISIONS.md` |
| `DEC-nnn` | A decision-log entry | `../00-governance/DECISION-LOG.md` |
| `ADR-nnnn` | An architecture decision record | `../02-architecture/adr/` |

**The collision.** `M-01` … `M-08` (mitigations, hyphenated) and `M0` … `M5`
(milestones, unhyphenated) look alike. Read them with their prefixes and their
spelling: a hyphen means a mitigation, no hyphen means a milestone. When a
mitigation is meant in prose, it is written as "mitigation `M-04`".

## 5. Paths that are still planned

The specifications also cite paths that do not exist yet, because they describe where
evidence *will* live. They are listed here so that a reader is not left hunting:

| Cited path | What it will be | Owner |
| --- | --- | --- |
| `qa/device-matrix.toml` | The machine-readable form of the interoperability matrix, so that the matrix can be diffed and linted | QA / Interoperability |
| `qa/macos-usability-protocol.md` | The script the usability study follows | Documentation reviewer |
| `qa/corpus/` | The captured SSDP and SOAP request corpus | QA / Interoperability |
| `qa/xsd/` | The vendored DIDL-Lite XSD, with rewritten `schemaLocation` | QA |

Until they exist, these are **planned locations**, and no gate depends on them
today.
