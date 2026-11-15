# FEATURE-XXX — <title>

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / YYYY-MM-DD
Requirements: <the requirement IDs this feature owns>

> **Template rule:** every section below must contain real content. "Not applicable"
> is acceptable only with a one-line reason. An empty section is a defect.
>
> ### How to read this specification
>
> | Tag | Meaning |
> | --- | --- |
> | `[SPEC]` | The claim comes from a published specification. The research brief and the source document are cited |
> | `[OBSERVED]` | The claim describes documented real-world behaviour, with a source |
> | `[DESIGN]` | The claim is our own decision, not someone else's rule |
> | `[OPEN]` | The question is unresolved; the section says what would close it |
>
> A citation such as `standards §8.3 C12` points into `docs/09-research/`; the
> convention is explained in `docs/09-research/README.md`.

## 1. Purpose and user story

Who needs this, why, and what is out of scope? Explain the domain terms a newcomer
would not know (SSDP, DIDL-Lite, ContentDirectory, a control point, a byte range).

## 2. Scope

**In scope** — the behaviour this feature delivers.

**Explicitly out of scope** — what it deliberately does *not* do, and which feature
does that instead. This section prevents two features from claiming the same
requirement.

## 3. Functional requirements

Numbered, uniquely identified, testable behaviours. Write each as
`**INT-030** — the server shall …` so that the ID and its elaboration sit together.
Use the exact text of the requirement in `../01-product/TECHNICAL-REQUIREMENTS.md`; do
not re-define it here.

## 4. Technical requirements

APIs, protocols, modules, persistence, error and recovery behaviour, constraints.
Name the modules that will own the behaviour (see `../02-architecture/ARCHITECTURE.md`
and `../../WHERE_TO_CHANGE.md`).

## 5. Performance requirements

Budgets, hardware and network conditions, measurement method, thresholds. A number
without a method is not a target — see `../05-quality/PERFORMANCE-BUDGETS.md`.

## 6. Usability and accessibility

Setup, error messages, defaults, accessibility, beginner experience. What does the
user see, and what does a screen reader announce?

## 7. Security and privacy

Threats, trust boundaries, mitigations, retention and permission model. Reference the
mitigations in `../04-security/SECURITY-MODEL.md` (`M-01` … `M-08`) and the threat
rows (`SEC-TH-nn`) that apply here.

## 8. Interoperability

Standards, supported clients and formats, fallback behaviour, real-device evidence.
State explicitly which client behaviour is documented and which is still unknown.

## 9. Error and recovery behaviour

A table: situation → expected behaviour → requirement. Every failure mode named in
the functional requirements must appear here.

## 10. Acceptance criteria and traceability

Given/When/Then, each with a stable `AC-nn` ID, the requirement IDs it covers, the
**verification method** (automated test / protocol fixture / manual protocol / device
test / benchmark) and the **planned** test location. A planned location is a path
that does not exist yet — never a claim that a test exists.

## 11. Test strategy and dependencies

The layers that cover this feature, the negative and boundary cases that are
mandatory, and the other features this one depends on.

## 12. Open questions and decisions

Clearly mark proposals versus approved decisions. A genuine gap in the requirement
catalogue is listed here as a **proposed** requirement with a suggested ID and a
one-line rationale, marked as not part of the baseline.
