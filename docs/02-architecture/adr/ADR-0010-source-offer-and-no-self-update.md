# ADR-0010 — The AGPL source offer is a product feature, and the server never updates itself

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect / Security Engineer
Requirements: `SEC-070`, `OPS-016`, `OPS-017`, `INT-059`, `PRD-001`, `PRD-002`

## Context

Section 13 of the AGPL says, in the words of the licence:

> "if you modify the Program, your modified version must prominently offer all users
> interacting with it remotely through a computer network … an opportunity to receive
> the Corresponding Source of your version by providing access to the Corresponding
> Source from a network server at no charge … This Corresponding Source shall include
> the Corresponding Source for any work covered by version 3 of the GNU General Public
> License that is incorporated pursuant to the following paragraph."

PhairDLNA interacts with users over a network by definition — that is what a DLNA
server *is*. A local network is a computer network; the licence does not carve out
"only if it is the internet". So the obligation applies to the binary we ship.

Three consequences follow, and each is a design decision rather than a legal detail:

1. **The offer must be reachable from the running product.** A link in a README is not
   an offer to a user who only has the device. The offer has to be a route in the
   running server.
2. **The offer must be complete.** "Corresponding Source" means everything needed to
   build and run the program: the source, the dependency tree, the lock file and the
   build recipe — not only the `.rs` files.
3. **The offer must be without authentication.** An offer that the network user
   cannot reach is not an offer.

A fourth consideration is supply-chain security: a server that downloads and applies
updates to itself is a remote code execution channel by design. Every such channel
is a compromise waiting for a CVE.

## Decision

1. **The running server serves its own source.** An unauthenticated route returns the
   corresponding source of the **exact running build**: the source tree, `Cargo.lock`,
   the dependency tree, the build instructions and the exact revision
   (`OPS-016`, `OPS-017`, `SEC-070`).
2. **The revision is reported.** The version, the commit and the build date appear
   in the status output and in the source offer, so a user can prove which build they
   are running.
3. **The server never updates itself.** There is no network self-update
   (`INT-059`). Updates arrive through the platform's package manager (Homebrew, a
   Debian or Red Hat package, an MSI) — a channel the user controls and can inspect.
4. **The offer is regenerated per release**, not per build, and it names the exact
   build it belongs to.
5. **The route is unauthenticated but harmless.** It serves source, not media, and it
   exposes nothing that the licence does not already require to be public. It is
   excluded from the administration surface's authentication requirements for exactly
   that reason.

## Consequences

- The licence's central promise is implemented as a feature rather than as a legal
  footnote — which is the honest reading of "offer the source to the network user".
- The release process must produce the source bundle as an artefact, with the
  dependency tree embedded (`cargo auditable` is the natural aid).
- No update channel exists, so no update compromise exists. The cost is that a user
  must update the way they update every other application on their machine.
- The route is a small, permanent, public surface. It is therefore kept trivially
  small: it serves static content and never touches the catalogue, the media path or
  the administration state.

## Alternatives considered

- **Pointing at the GitHub repository.** Rejected: a user who installed a build that
  differs from the repository head has no guarantee that the repository contains
  their build, and a fork or a local modification would not be covered at all.
- **Offering the source only in the release archive.** Rejected: it is not reachable
  from the running program, which is what § 13 asks for.
- **An authenticated source route.** Rejected: an authenticated offer is not an offer
  to the network user.
- **A network self-updater.** Rejected: a permanent remote code execution channel in
  a product that holds a user's private media library.

## Revisit condition

Revisit if a court or a qualified lawyer reads § 13 differently for a
LAN-only server — the implementation follows the licence text as written.
