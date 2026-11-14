# ADR-0009 — Transcoding is optional, isolated and never silent

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `FUN-047`, `FUN-048`, `FUN-049`, `RES-005`, `SEC-003`

## Context

A file that a client cannot decode is the most common complaint about a DLNA server.
Transcoding is the obvious answer, and it is also the most expensive answer:

- It costs processor time on a machine that may be a laptop on battery.
- It requires a codec stack, which brings licensing questions.
- It changes the product's behaviour: the user asked for their file, and the server
  decides to send something else.
- It is the classic source of the "it transcodes everything and my TV shows a
  slideshow" failure.

The project also has a hard constraint from the licence and the supply chain: the
product must not ship or link GPL code (see ADR-0002's dependency review and
`../../04-security/DEPENDENCY-LICENSE-REVIEW.md` § 1 rule 7). FFmpeg is LGPL only
when built without `--enable-gpl`; a GPL FFmpeg inside an AGPL product would make
the combined work GPLv3-governed and would detach the AGPL network-source
obligation from it.

## Decision

1. **Direct play is the default and the preferred path.** A file that is served
   directly is never transcoded.
2. **Transcoding is off by default** and is enabled per target format by an explicit
   user action.
3. **The server never transcodes silently.** A file that cannot be played directly is
   reported as incompatible, with the reason. The user may then enable a
   transcoding profile for that format or for that device (`FUN-049`).
4. **The transcoder runs in its own process**, with an explicit resource budget, a
   bounded queue and a cancellation path (`FUN-048`). It never runs inside the
   server process.
5. **No FFmpeg is shipped and no GPL code is linked.** If transcoding is enabled, the
   user supplies an external binary; the server invokes it with an argument array,
   never through a shell, and with the same sandboxing rules as the media-inspection
   child of ADR-0008.
6. **Transcoding must never starve direct play or discovery** (`RES-005`); it has
   the lowest priority class in the scheduler.

## Consequences

- The baseline is unaffected: transcoding is milestone M5 and blocks nothing.
- The product makes an honest promise — "your file plays as it is, or the server
  tells you why it does not" — which is exactly what the existing servers get wrong.
- A user who wants transcoding must install a transcoder themselves. That is more
  friction, and it is the price of not shipping GPL code and not misusing the user's
  processor without being asked.
- The resource budget of a transcoder is a product decision, not an implementation
  detail: the user must be able to say how much processor a transcode may use.

## Alternatives considered

- **Ship FFmpeg (GPL build) with the product.** Rejected: it changes the licence of
  the combined work and creates distribution obligations the project cannot meet.
- **Ship FFmpeg (LGPL build, dynamically linked).** Rejected for v1: it still brings
  a native toolchain into every build and every distribution, and it drags the
  corresponding source obligations into the release process. Revisitable if the
  owner decides transcoding is essential.
- **Link `ffmpeg-next` and transcode in-process.** Rejected on both the licensing and
  the security grounds: a native library in the server process puts a large, C
  codebase inside the trust boundary.
- **Transcode automatically when a client cannot play a file.** Rejected: it violates
  `FUN-049`, surprises the user, and burns processor time silently.

## Revisit condition

Revisit if the owner decides that built-in transcoding is a product requirement. The
decision would then need a licence assessment (LGPL-only build, dynamic linking,
corresponding source in the same offer), a resource policy, and a place in the
scheduler that keeps it below direct play.
