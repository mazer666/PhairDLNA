# Security policy

## Reporting a vulnerability

**Please do not report an exploitable vulnerability in a public issue, a discussion or
a pull request.**

Use GitHub's private vulnerability reporting
("Security" → "Report a vulnerability") when it is available for this repository. If it
is not, open an issue that contains **only** a one-line description such as
"security report — please open a private channel" and no technical detail. The
maintainer will arrange a private channel.

Please include, when you can:

- the affected version or commit,
- the platform and its version,
- what an attacker can achieve (not a working exploit if you can avoid it),
- the steps to reproduce.

**No personal media, no secrets, no identifying file paths** in a report. Use
synthetic paths such as `/media/example.mkv`.

## What we consider a vulnerability

- Reading a file outside the configured media roots.
- Executing code on the server through any input.
- Bypassing the loopback restriction of the administration surface.
- Making the server send traffic to an address the user did not intend.
- Denial of service that a device on the local network can trigger at will.
- Any transmission of data that the privacy model says does not leave the machine.

The threat model with the mitigations and the verification for each is in
[docs/04-security/SECURITY-MODEL.md](docs/04-security/SECURITY-MODEL.md).

## Supported versions

**No release exists yet.** There is therefore no supported version, and no security
advisory can be issued for one. Once a release exists, this section will name the
supported versions and their lifetime.

Until then, every commit on `main` is the current state of development, and the
security model applies to it.

## What we do not consider a vulnerability

- The server answering SSDP discovery requests: that is its purpose.
- The server serving files from a configured root to a client on the local network:
  that is its purpose.
- A client that cannot play a particular format: that is a compatibility matter, not a
  security matter. Please report it as an interoperability issue.
- Running the server on a network the user did not intend to expose it to. The server
  binds the local interfaces; exposing it to the internet is a network configuration
  matter.

## Hardening requirements — the target state, not present behaviour

**No server exists yet.** The repository contains a requirements baseline and a Rust
workspace scaffold, not a running server. The properties below are **requirements
that the implementation must satisfy**, each with a negative test that will prove it
once the code exists. None of them is a claim about software you can download today.

| Property | Requirement | Delivered by | State |
| --- | --- | --- | --- |
| The administration surface binds to the loopback interface only and validates the `Host` header against an allowlist | `OPS-020`, `SEC-071` | F-14 | specified, not implemented |
| Media paths are canonicalised and verified to be inside a configured root, including after symbolic links and aliases are resolved | `SEC-001` … `SEC-004`, `SEC-069` | F-08, F-16 | specified, not implemented |
| XML is parsed without DTD processing or external entity resolution, with bounded size and depth | `SEC-010`, `SEC-060` | F-16 | specified, not implemented |
| No SOAP action ever reaches a shell, and no Internet Gateway Device action exists | `SEC-005`, `INT-058` | F-03, F-16 | specified, not implemented |
| Media metadata is parsed out of process, with a bounded memory and time budget | `SEC-066`, `SEC-067` | F-10, F-16 | specified, not implemented |
| The product sends no telemetry and makes no outbound connection other than the local network | `SEC-090` … `SEC-093` | F-16 | specified, not implemented |

The threat model, the eight mitigations and the verification that will prove each one
are in [docs/04-security/SECURITY-MODEL.md](docs/04-security/SECURITY-MODEL.md). A
security claim will only be published once its negative test has been observed.

## Licence

PhairDLNA is licensed under AGPL-3.0-or-later. The complete licence text is in
[LICENSE](LICENSE). Section 13 of that licence obliges the running server to offer its
corresponding source to the users who interact with it over a network — PhairDLNA
implements that offer as a product feature.
