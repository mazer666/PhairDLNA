# ADR-0012 — The daemon crate wires the administration surface

Status: **Accepted**
Date: 2026-11-16
Owner role: Architect
Requirements: `OPS-020`, `OPS-021`, `OPS-022`, `OPS-023`, `OPS-024`, `OPS-025`, `OPS-026`, `OPS-027`, `CFG-005`, `SEC-071`

## Context

ADR-0001 draws the crates of the workspace and puts `phairdlna-admin` beside
`phairdlna-cli` and `phairdlna-mac`, with the arrow pointing at `phairdlna-cli`.
The boundary gate QG-28 reads that drawing as it stands: the core depends on
nothing, and every other crate depends on the core and on nothing else.

The administration surface cannot live there. Two facts decide it:

- The server is **one process**. ARCHITECTURE § 3 says that the daemon runs the
  discovery, the media server and the administration surface in one process with
  a small number of long-lived tasks. A surface in a second process would need
  its own way to reach the state of the daemon — a socket, a protocol, a second
  attack surface — for no gain.
- The **crate that owns the process starts the surface**. The daemon crate
  opens the sockets, reads the configuration and stops the tasks. Whoever starts
  the surface must be able to name it, and that is the daemon crate.

Both facts point the same way: the daemon crate must be able to call into the
surface crate. The arrow of ADR-0001 therefore points the wrong way for
`phairdlna-admin`: a crate that the daemon calls cannot also be a crate that
calls the daemon.

The alternative — the surface as a separate process — was rejected: it adds a
second protocol, a second lifecycle and a second thing to secure, and it buys
nothing that a second listening socket inside the daemon does not already give.

## Decision

1. **The dependency direction is `phairdlna-cli → phairdlna-admin → phairdlna-core`.**
   The surface crate knows the core and nothing else. The daemon crate knows the
   surface crate and the core.
2. **The dependency of the daemon crate on the surface crate is optional.** It
   sits behind the cargo feature `admin`, which is on by default. A build with
   `--no-default-features` has no administration socket, no token, no
   administration route and no administration code (`OPS-025`).
3. **The surface crate knows the daemon only through a trait.** The trait
   `phairdlna_admin::AdminBackend` is the whole contact: the daemon fills it in
   with its library, its transfers and its log. The surface never sees the
   filesystem, the catalogue or the sockets of the daemon.
4. **The core stays portable.** The surface crate depends on the core and on
   the network stack, and on no sibling crate. A crate that depends on the
   daemon crate is a cycle, and QG-28 refuses it.
5. **QG-28 is amended, not weakened.** The gate now allows the edge
   `phairdlna-cli → phairdlna-admin` and still refuses `phairdlna-admin →
   phairdlna-cli`, `phairdlna-core → anything` and `phairdlna-mac → anything but
   the core`. The negative test of the gate — a core crate that depends on the
   daemon crate — still fails the gate.

## Consequences

- The drawing of ADR-0001 is amended for one crate: `phairdlna-admin` now sits
  to the *left* of `phairdlna-cli` and points at the core. The amendment is
  recorded in the decision log; the decision itself is not rewritten silently.
- The boundary gate knows one edge more, and its comment tells the reader why.
- The surface crate is a crate like any other: it has its own tests, its own
  documentation and its own rules about file size and complexity.
- A build without the feature `admin` is a build without the surface. The
  daemon starts, serves the media and stops, and nothing of the administration
  surface is in the binary.
- The daemon and the surface are one process, so the surface can answer with
  the state of the running daemon without a protocol of its own.

## Alternatives considered

- **The surface as a separate process that talks to the daemon.** Rejected: a
  second protocol, a second lifecycle and a second attack surface for a feature
  that answers three questions.
- **The surface inside the daemon crate, and no crate of its own.** Rejected:
  `OPS-025` asks for a build without the surface, and a crate of its own is
  what makes that build possible.
- **The surface in the core crate.** Rejected: the core must stay free of a
  network stack, and the surface is nothing but a network surface.
- **The surface crate depends on the daemon crate, as the drawing of ADR-0001
  says.** Rejected: the daemon cannot then start the surface, and the surface
  would be a second daemon.

## Revisit condition

Revisit if the administration surface ever has to run outside the daemon
process — for example when a second machine has to administer the server. The
decision would then need a protocol between the two, and the surface would
have to own that protocol instead of the trait.
