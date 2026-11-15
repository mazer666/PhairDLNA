# Competitive parity — against Elmedia Media Player

Status: **Approved (v1.0 baseline, 2026-11-15)** · Owner role: Product Owner
Requirement: `PRD-021` · Decision: `DEC-024` in `../00-governance/DECISION-LOG.md`

> **The owner's direction:** *we have to be better than the Elmedia media player.*
>
> This document turns that direction into criteria that can be **tested**, because
> "better" is otherwise a slogan that no review can check.

---

## 1. What Elmedia actually is

Elmedia Media Player is a **commercial macOS media player** by Eltima/Elmedia. The
facts below come from the vendor's own feature pages and app-store listings, retrieved
2026-11-15. **They are vendor claims, not measurements** — the same rule applies to
this document as to every other claim in the project.

| Capability | Vendor claim | Source |
| --- | --- | --- |
| Plays almost any container | AVI, MP4, FLV, SWF, WMV, MKV, M4V, 3GP, ASF, DivX, VOB, MXF, WebM and more | elmedia-video-player.com/features, app-store listing |
| Audio | MP3, M4A, WMA, AAC, AC3, AIFF, APE, FLAC, MKA, OGG, WAV, DTS | app-store listing |
| Subtitles | utf, srt, ass, smi, txt, ssa, smil, jss, rt, idx, sub, vtt — external, embedded and burned in during transcoding | app-store listing, vendor feature page |
| Sends to devices | DLNA (Samsung, Sony, LG Smart TVs), AirPlay 2, Chromecast, Roku, Apple TV | vendor feature page |
| Real-time transcoding | "Streams any format, transcoding it if needed" | vendor feature page |
| Online subtitles | Integrated OpenSubtitles.com search | app-store listing |
| Playlists | Create playlists for uninterrupted streaming | app-store listing |
| Playback control while streaming | Play, pause, seek, stop, volume | app-store listing |
| Hardware decoding | VideoToolbox, smooth 4K | vendor page |
| Price | Free tier plus a PRO tier (a one-off purchase) | app-store listing |

## 2. Why the comparison is not one-to-one

Elmedia is a **player and a sender**: the user picks a file in the app, and the app
pushes it to a device. PhairDLNA is a **server**: the device comes to the library and
pulls whatever it wants.

That makes two different comparisons, and only one of them is a competition:

| Comparison | Who wins today | Why it matters to us |
| --- | --- | --- |
| **As a DLNA *source* for a television** | Elmedia wins on polish, we have nothing yet | This is the real gap. A user who today opens Elmedia to push a film to the TV must instead find the file in the TV's own DLNA app — and the TV must offer it |
| **As the *native macOS experience* around a library** | Elmedia wins on polish | Our M2 menu bar application must feel at least as good on a Mac |
| **As a *privacy-respecting* tool** | **We win already, by design** | Elmedia searches OpenSubtitles.com — an internet service. We do not talk to the internet at all |
| **As an *auditable* tool** | **We win already, by design** | Elmedia is closed-source; PhairDLNA is AGPL and every line is readable |
| **As a *free* tool** | We win, but Elmedia's free tier exists | Elmedia's PRO is a one-off purchase, not a subscription — that is a fair competitor and it must be treated as one |

**The honest reading of the owner's direction:** PhairDLNA must be the better
*macOS media server*, and it must not be the worse *macOS media tool*. The criteria
below are split accordingly.

## 3. Criteria we must beat Elmedia on

Each criterion is written so that a test could decide it. "Better" without a test is
marketing.

### C-01 — The whole library, not one file at a time

Elmedia pushes the file the user picked. PhairDLNA publishes the entire library, with
folders, metadata and artwork, and the device browses it.

**Test:** a television finds the server, browses three levels deep and starts a file
that the user never touched in the Mac. **Criterion:** the device reaches any file in
the library without the Mac being involved at any point.

### C-02 — Subtitles that the television can actually read

Elmedia lists twelve subtitle formats for its **own** playback. A server has a harder
job: it must hand the file to a device that never asked for it.

**Test:** for each supported format, a recorded `Browse` result carries a `res`
element with the correct MIME type, and a real device displays the subtitles.
**Criterion:** SRT, WebVTT, SSA/ASS, SMIL and SUB/IDX are offered as separate
resources; the charset is one the device can read (UTF-8 with or without BOM, with a
Latin-1 fallback); and a device that ignores subtitles is never *broken* by them.

### C-03 — Files larger than four gigabytes

This is where DLNA servers traditionally fail: a 32-bit offset silently truncates, and
the device plays the first four gigabytes and then stops. Elmedia streams 4K files
from a Mac; a user with a 4K remux will have files well past that mark.

**Test:** a file larger than 4 GiB is served with correct ranges across the 4 GiB
boundary and at its end.
**Criterion:** the byte offsets are 64-bit throughout, and a range request that starts
beyond 4 GiB returns the right bytes.

### C-04 — More than one audio or subtitle track

Elmedia lets the user switch the audio and subtitle track. On a server, the choice
belongs to the device, so the server must *offer* the tracks.

**Test:** a video with two audio tracks produces two `res` elements with distinct
MIME types.
**Criterion:** each additional track is a separate resource, and the primary resource
is the one the device plays by default.

### C-05 — A real-time transcoding fallback

Elmedia transcodes on the fly when the device cannot play the format.

**Test:** with transcoding explicitly enabled, a device that cannot play the source
format receives a stream it can play; with it disabled, the device receives the
original and the server says plainly that the file is incompatible.
**Criterion:** the fallback is never silent, never automatic, and never burns the
user's processor without being asked (`FUN-049`).

### C-06 — No internet, no account, no advertising

Elmedia searches OpenSubtitles.com and offers a PRO tier. PhairDLNA does neither.

**Test:** the outbound-connection audit during a full session.
**Criterion:** zero outbound connections; no account, no purchase, no advertising, no
telemetry — and the full source of the running build is one HTTP request away
(`SEC-070`).

### C-07 — It tells you *why* it did not work

Elmedia is a sender: if the television refuses the stream, the user gets a spinner.
A server sees the refusal, knows the client's profile, and can name the cause.

**Test:** a scripted failure — discovery, browse, codec, seek, subtitles, permissions —
produces a classified diagnosis with a proposed remedy.
**Criterion:** the failure is classified, the confidence is stated, and the remedy is
offered without being applied behind the user's back (`PDOC-001` … `PDOC-006`, the
proposed Playback Doctor).

### C-08 — A native macOS experience that is not an afterthought

Elmedia is a Mac application first. So must our front end be.

**Test:** the menu bar application passes a keyboard-only and a VoiceOver audit, and
the interface never blocks while the server works.
**Criterion:** every task is reachable by keyboard, the status is available as text
(`UX-018`), and the server keeps running when the interface quits (`REL-006`).

### C-09 — Honest about what it does not know

Elmedia claims "plays virtually any video or audio file". A project that measures
cannot write that sentence — and must not.

**Test:** the documentation.
**Criterion:** no compatibility claim without a device row behind it; an unknown
capability stays unknown (`COMP-002`).

## 4. Where we are *not* competing, and why

| Elmedia capability | Our answer |
| --- | --- |
| Web playback of YouTube | Out of scope. We serve the user's own files; we do not become a browser or an advertising surface |
| Built-in equaliser, audio effects, playback speed | A player feature. We are a server |
| Video editing, frame stepping, screenshots | A player feature |
| DVD/Blu-ray playback | A player feature. We serve what is already on the disk |
| A PRO tier | There is no tier. The project is AGPL and free, permanently (`DEC-001`) |

## 5. The order of work this changes

Nothing in the baseline is abandoned. The criteria map onto existing work:

| Criterion | Delivered by | Milestone |
| --- | --- | --- |
| C-01 | F-08, F-09, F-03 | M1 |
| C-02 | F-11 (`FUN-026`, `FUN-027`) | M2 |
| C-03 | F-06 (`FUN-028`) | M1 |
| C-04 | F-05, F-11 (`FUN-029`, `INT-105`) | M2 |
| C-05 | F-17 (optional, deferred) | M5 |
| C-06 | F-16 (`SEC-090` … `SEC-096`, `SEC-070`) | M1 |
| C-07 | F-21 (proposed Playback Doctor) | after the baseline |
| C-08 | F-15 | M2 |
| C-09 | F-07, F-19 | M1, M4 |

**What the comparison adds to the baseline:** four new requirements — `FUN-026`
(subtitle formats and MIME types), `FUN-027` (subtitle charset handling),
`FUN-028` (files beyond 4 GiB, 64-bit ranges) and `FUN-029` plus `INT-105` (multiple
tracks as separate resources). Everything else was already required; this document
gives it a competitive reason and a named test.

## 6. What would make us *worse* than Elmedia

Stated plainly, so that a reviewer can hold us to it:

- **A server that works, and an application that does not.** If the Mac application
  is clumsy while Elmedia's is polished, the user keeps Elmedia for the interface and
  uses us only as a fallback source.
- **Subtitles that do not arrive.** A film without subtitles on the television is a
  worse experience than a film the user can play on the Mac.
- **A silence that is not explained.** "It did not work" with no reason is worse than
  a clear "this television cannot play this codec".
- **A first run that takes ten minutes.** Elmedia installs and plays. A media server
  that needs a configuration file before the first film is a worse tool.

## 7. How this document is kept honest

- Every criterion names a test. A criterion without a test is not a criterion.
- The comparison is refreshed when the research is refreshed
  (`../09-research/ecosystem.md`), and the date of the refresh is recorded.
- The competitor's claims are labelled as vendor claims, never as facts.
- **No "faster than Elmedia" claim** will ever be published: the two products do not
  perform the same work, and a comparison without a shared workload is a slogan.
