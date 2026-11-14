# Product vision — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner
Scope: the *why* of the product. The *what* is in `PRODUCT-REQUIREMENTS.md`, the
*how* in `../02-architecture/ARCHITECTURE.md`.

---

## 1. The problem

A DLNA/UPnP media server should be boring: it starts, it shows up on the network, the
TV can browse the library, and the film plays. In practice the existing options split
into two groups that both frustrate users.

**The packaged apps** (Plex, Emby, Jellyfin) work well, but they are built around an
account, a server component and often a subscription or a licence key. They phone home.
They want to re-encode your media into their own profile. They are excellent for the
user who accepts that trade, and wrong for the user who wants a folder on their own disk
served to their own television, with nothing leaving the house.

**The classic servers** (ReadyMedia/MiniDLNA, Gerbera and their relatives) are free and
local, but they are products of an earlier decade: thin configuration, cryptic logs,
little explanation when a client refuses to play a file, no native macOS experience,
and an ecosystem of configuration that people learn by copying forum posts.

PhairDLNA exists for the user in the gap: **a local-first, privacy-first media server
that behaves like a well-made native application rather than a service.**

---

## 2. The promise

> A media server you can install on your own Mac, point at your own folders, and forget
> about — that your television finds on its own, plays reliably, and never sends a byte
> anywhere you did not ask it to.

Three commitments make that promise concrete:

1. **Reliable interoperability.** The server must work with the devices people actually
   own — smart TVs, game consoles, set-top boxes, phone apps — not only with the
   protocol specification. Compatibility is proven against real devices and recorded
   in a public matrix, including the devices that *failed*.
2. **Verifiable quality.** Every claim (speed, memory use, compatibility, security) is
   backed by a reproducible measurement or a named test. The project publishes its
   failures as readily as its successes.
3. **Radical legibility.** A new contributor can read the documentation, run the
   project, and understand *why* each piece exists. The code is small enough to read
   in a sitting and documented well enough to change without fear.

---

## 3. Who it is for

| Persona | Situation | What they need most |
| --- | --- | --- |
| **The collector** | Large, well-organised local media library on a NAS or external drive | Fast, incremental indexing; reliable long playback; no transcode tax |
| **The household** | Family TV in the living room, phone apps in the bedrooms | It just works, on every device, without an account |
| **The tinkerer** | Runs a homelab, reads logs, wants to script it | Honest configuration, readable logs, no hidden magic |
| **The privacy-conscious** | Does not want media or metadata leaving the LAN | No telemetry, no accounts, no internet dependency, auditable behaviour |
| **The newcomer** | Wants to share a folder with the TV, today | One-minute setup, a first-run wizard, clear error messages |

Non-goals for the person we are *not* building for: the rights-holding studio with a
DRM pipeline, the multi-tenant cloud provider, and the user who wants a phone app that
streams their library over the internet.

---

## 4. What "high quality" means here

Quality in this product is not a slogan; it is a set of properties we can test:

- **Correctness** — a standards-conforming server: valid SSDP, valid UPnP device
  descriptions, valid DIDL-Lite, valid HTTP range semantics.
- **Reliability** — a seven-day soak without a crash, a memory leak, an unhandled
  error or a wedged worker; a corrupt index recovers instead of failing.
- **Efficiency** — low idle footprint, fast warm start, incremental rescans, bounded
  memory regardless of library size, direct play as the default.
- **Safety** — read-only media access, path containment, no shell interpolation, no
  unbounded parsing, no silent failure, no unrequested network access.
- **Compatibility** — a real-device matrix with observed outcomes, not a certification
  claim.
- **Legibility** — beginner-readable documentation, small modules, one responsibility
  per file, a navigation file that tells a newcomer exactly where to change what.

---

## 5. Shape of the product

```
phairdlna-core      portable Rust core: config, catalogue, protocols, streaming
phairdlna-cli       headless daemon + command line (the product's core interface)
phairdlna-mac       optional native macOS menu bar app (later milestone)
phairdlna-admin     optional local administration surface (loopback only)
```

The headless server is the product. The macOS app is a front end for it — never a
replacement, and never a requirement for the server to work.

---

## 6. Explicit non-goals

- Internet-facing streaming or remote access by default.
- Cloud accounts, subscriptions, "pro" tiers or any monetisation.
- Telemetry, analytics, crash reporting to a third party, or advertising of any kind.
- DRM circumvention or protected-content handling.
- Automatic internet metadata scraping as a default (opt-in per root at most).
- A "works with everything" claim without evidence.

---

## 7. What success looks like

The first milestone is successful when a person can install the server on a Mac, point
it at one folder, and have their television browse and play a file **without a single
manual configuration step and without a single support question** — and when the
project can prove that claim with a recorded test, not an assertion.
