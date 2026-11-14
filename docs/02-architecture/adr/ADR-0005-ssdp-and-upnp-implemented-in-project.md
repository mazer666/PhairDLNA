# ADR-0005 — SSDP and the UPnP control layer are implemented in this project

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `INT-001` … `INT-029`, `INT-050` … `INT-056`, `SEC-010`, `SEC-018`

## Context

UPnP is an old protocol family, and Rust crates for it exist. The question is whether
to build on one of them or to implement the parts we need ourselves.

The parts we need are small and sharply defined: an SSDP responder and advertiser, an
HTTP device description, a SOAP dispatcher with five actions, and a DIDL-Lite writer.
The parts we would inherit from a general UPnP stack are large and mostly unused: the
full device architecture with embedded devices and eventing, GENA, the IGD profile,
plugin systems, and the UPnP certification-oriented model of the specification.

## Decision

We implement the SSDP and UPnP control layer **in this project**, on top of a general
HTTP server and a general XML parser.

Reasons:

1. **No Rust UPnP stack is production-ready for our purpose.** The candidates are
   small, largely unmaintained, or oriented at the IGD / embedded-device use case
   rather than at a media server. Adopting one would mean carrying a dependency we
   cannot influence and a compatibility surface we do not control.
2. **The surface we need is small.** Five SOAP actions, one SSDP exchange and one XML
   document format. Owning them means owning the client-quirk behaviour that makes or
   breaks compatibility.
3. **The security boundary is ours.** Parsing untrusted XML and answering untrusted
   multicast is exactly where we do not want third-party behaviour (ADR-0009 in the
   security model; the parser policy in `SEC-010`).
4. **The specification is old but stable.** The parts of UPnP AV that we implement
   have not changed for more than a decade; the risk of a spec change is lower than
   the risk of an unmaintained dependency.

We use established crates **underneath**: a maintained HTTP server, a streaming XML
parser with DTD processing disabled, and a SQLite binding. We do not implement HTTP
or XML ourselves.

## Consequences

- We own the compatibility behaviour, which is the product's differentiator.
- We also own the bugs. Every part must be tested against fixtures, including
  malformed input.
- The XML policy (`SEC-010`: no DTD, no external entities, bounded depth and size)
  is enforced by configuration of the chosen parser, and a negative test proves it.
- If a mature, maintained UPnP stack appears later, adopting it would mean replacing
  the `upnp` and `ssdp` modules behind unchanged trait boundaries — the modularity
  in `ARCHITECTURE.md` § 4.2 and § 4.3 is what keeps that option open.

## Alternatives considered

- **Adopt an existing Rust UPnP/SSDP crate.** Rejected for the reasons above; the
  research behind this decision is recorded in the dependency review
  (`../../04-security/DEPENDENCY-LICENSE-REVIEW.md`).
- **Implement the full UPnP Device Architecture including eventing and GENA.**
  Rejected: it multiplies the compatibility surface without improving the user
  experience. Eventing is optional in our scope.
- **Implement HTTP or XML ourselves.** Rejected without discussion: both are solved
  problems with mature implementations, and both carry real security history.

## Revisit condition

Revisit if the project needs UPnP eventing, embedded devices, or IGD-style features,
or if a maintained UPnP stack with an acceptable licence and maintenance record
appears and demonstrates better client compatibility than our implementation.
