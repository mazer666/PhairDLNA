# Security model — initial threat model

Assets: media contents, filesystem paths, credentials/configuration, host resources, local network and media metadata.

Attack surfaces: SSDP multicast, UPnP XML/SOAP, HTTP requests and ranges, malformed media metadata, symlinks, local admin API, external transcoders and update/distribution chain.

Controls:
- LAN-only discovery by default; explicit interface selection; no automatic router port mapping.
- Admin API loopback-only by default; never equate LAN membership with authentication.
- Read-only media roots, canonicalization and traversal defenses, symlink and TOCTOU tests.
- Bound request sizes, XML nesting, media probing time, concurrency, memory and CPU.
- Restrict and isolate external executables; never interpolate shell commands.
- Redact sensitive paths and tokens from logs; no telemetry.
- Dependency auditing, fuzzing, negative tests, provenance/checksums for releases.
- Atomic state updates, recoverable index and explicit storage retention policies.
- Security issues handled privately via SECURITY.md.

Before release: threat-model review, verified test evidence, no known unmitigated critical/high findings.
