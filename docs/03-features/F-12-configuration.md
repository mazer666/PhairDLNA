# F-12 — Configuration

Status: Draft
Owner role: Product/Requirements
Version/date: 0.3 / 2026-11-19
Purpose: Specify the typed, validated and versioned PhairDLNA configuration.
References: `../00-governance/PROJECT-STANDARDS.md`, `../00-governance/REQUIREMENTS-METHOD.md`, `../02-architecture/adr/ADR-0004-typed-versioned-configuration.md`, `../07-project/OPEN-DECISIONS.md` (OD-16), `F-14`, `F-15`.
Requirements: CFG-001, CFG-002, CFG-003, CFG-004, CFG-005, CFG-006, CFG-007, CFG-008,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
CFG-009, CFG-010, CFG-011, CFG-012, CFG-013, CFG-014, CFG-015

## 1. Purpose and user story

A media server is configured once and then mostly forgotten. The configuration decides
which folders are shared, which network interfaces are used, how many streams may run
at once, which clients may browse, and whether the administration interface is reachable
from the network. Getting one of those wrong is not a cosmetic problem: a typo in a
setting name can silently switch off a protection, and a wrong path can make the server
publish a directory the user never meant to share.

The design rule of this feature is therefore **fail loudly, at start-up**. The
configuration is read exactly once, parsed into typed values, validated as a whole, and
either accepted completely or rejected with a message that names the field, the file, the
expected type and the accepted values. Nothing is validated "where it is used", because a
value that is only checked when a client happens to request it fails at the worst
possible moment.

The file is a **TOML** document — a format designed to be edited by hand and to carry
comments — and it carries a `schema_version`, so that a later release can change what a
setting means without silently reinterpreting an old file (ADR-0004).

**User story.** As a user, I want to change one setting in one file and have the server
tell me immediately — at start-up — if I got it wrong, so that I never discover a
misconfiguration hours later, when my television cannot see the server.

## 2. Scope

**In scope**

- The `config` module of `phairdlna-core`: the schema, its defaults, its validation and
  its migrations (ARCHITECTURE §4.1, ADR-0004).
- The location of the configuration file and the precedence between the command-line
  switch, the environment variable and the platform default path (CFG-001).
- The typed schema, the start-up validation, the error format and the `schema_version`
  migration path (CFG-002, CFG-003, CFG-004, CFG-013).
- The settings that decide reachability: the administration bind address (CFG-005), the
  network interfaces used for discovery and for media delivery (CFG-006), the
  access-control lists (CFG-009) and the trusted-client list (CFG-010).
- The per-root symbolic-link policy (CFG-007) and the resource limits (CFG-008).
- Secrets referenced by path rather than stored in the file (CFG-014), and the rule that
  no value is expanded from the environment without an explicit opt-in (CFG-015).
- The configuration commands: validate, export, import (CFG-011, CFG-012).

**Explicitly out of scope**

- The administration interface itself, its token handling and its HTTP surface (F-14
  owns OPS-020 … OPS-027); this feature only decides where it binds.
- The command line as a whole (F-13 owns the exit codes and the sub-command set);
  this feature defines what the configuration commands do.
- The device description and the SSDP advertisement, which *consume* configured values
  such as `friendlyName` (F-01, F-02).
- Runtime reloading of the configuration: ADR-0004 rejects it for the baseline, and a
  `reload` command is the acceptable later step.
- The catalogue, the media pipeline and the platform adapters; the configuration only
  describes them.

## 3. Functional requirements

**CFG-001** — The server shall read its configuration from a single, versioned
configuration file whose path is given by an environment variable or a command-line
switch. [DESIGN] One file, one reader: only the `config` module opens the file or reads
an environment variable (ADR-0004, ARCHITECTURE §4.1). The path comes from the
`--config <path>` switch, else from the `PHAIRDLNA_CONFIG` environment variable, else
from the platform's per-user configuration directory (platform §12). The command-line
switch wins over the environment variable.

**CFG-002** — The configuration schema shall be typed and validated at start-up; an
invalid value shall abort start-up with a message naming the field, the file, the
expected type and the accepted values. [DESIGN] Validation happens once, on the whole
document, before any listener is opened (ADR-0004). The error is one block:

```
configuration error in /Users/u/.config/phairdlna/config.toml
  field:    limits.max_concurrent_streams
  expected: integer
  accepted: 1..=256
  what now: set a value in that range, or remove the key to use the default (32)
```

**CFG-003** — The configuration file shall carry a `schema_version`, and the server
shall migrate an older known schema version automatically and warn about it. [DESIGN] The
migration runs in memory and is logged with the version it came from and the version it
produced; the file on disk is not rewritten (CFG-012 exports a file in the new shape if
the user wants one). A file whose `schema_version` is *newer* than the software
understands is refused, with a message naming both versions — the server will not guess
what a future setting means.

**CFG-004** — The server shall provide documented defaults for every setting such that an
empty configuration file yields a working, secure server. [DESIGN] Every field has a
default; a zero-byte file is valid. The defaults are the safe ones: the server serves
the LAN, binds no administration port to a non-loopback address, contacts nothing
outside the machine, and creates its index and identity in the user's own directories
(PRD-009). No default requires the user to edit anything to get a first working server.

**CFG-005** — The server shall refuse to start when the configuration would expose an
administration interface to a non-loopback address, unless the user explicitly overrides
it. [DESIGN] The administration bind address is validated against the loopback range at
start-up; a non-loopback value aborts the start with a message naming the field and the
switch that overrides it. The override is a separate, explicit act — it is not implied
by setting the value (OPS-020, F-14).

**CFG-006** — The server shall support selecting the network interfaces used for
discovery and for media delivery. [DESIGN] One interface table, with a `discovery` and a
`media` flag per interface, so the two uses can differ — a common setup is "discover on
Wi-Fi, deliver over Ethernet". The default is every broadcast-capable interface that is
up and is not loopback. Interfaces are named by name and address, never by index:
platform interface names such as `en0` are not an API and the Wi-Fi interface is not
always `en0` (platform §1.2). The excluded set is what the SSDP module refuses to answer
on (INT-009) and what the HTTP module binds to (F-06).

**CFG-007** — The server shall support a per-root setting for symbolic link following,
disabled by default. [DESIGN] Each root carries `follow_symlinks = false` by default.
With the setting off, a symbolic link inside a root is not followed and is not indexed
as an entry of its own. With it on, the link is resolved and the *target* is indexed —
and only if the resolved target is still inside the root (SEC-003, SEC-006). *Whether
the setting should be richer than a boolean is raised in §12.*

**CFG-008** — The server shall support resource limits: maximum concurrent streams,
maximum cache size, maximum metadata extraction time, maximum index workers. [DESIGN]
Four settings under one table. The values are ceilings, not hints: the server refuses
further streams rather than exhausting memory (FUN-043), evicts cache entries rather
than growing the disk (SEC-064), abandons a file that exceeds the extraction time
(SEC-062) and bounds the number of concurrent index workers. The metadata extraction
time may only *tighten* the sandbox's own limit, never raise it (SEC-066) — raised in
§12.

**CFG-009** — The server shall support an optional access-control list of client
addresses allowed to browse or to stream. [DESIGN] Two independent lists, `browse` and
`stream`, each a list of addresses or CIDR prefixes. An absent list means "no
restriction beyond the LAN-only rule"; a present list means "only these". A client
address that is not on the relevant list is refused with `403 Forbidden` and a generic
message that reveals nothing else (SEC-013). The list does not change what the server
advertises: SSDP discovery still happens, because hiding the server would look like a
broken product. *Whether the advertisement should also be restricted is raised in §12.*

**CFG-010** — The server shall support a "trusted client" list whose quirks are handled
by a specific device profile. [DESIGN] Each entry names a client address and the device
profile to use for it; the profile itself lives in the client-compatibility registry
(F-07, INT-100). An entry naming a profile that does not exist is a configuration error
(CFG-002), not a silently ignored line. A "trusted" client is trusted only in the sense
of "we know its quirks and apply the documented workaround" — it grants no extra access
and no bypass of CFG-009.

**CFG-011** — The server shall provide a command that validates a configuration file
without starting the server. [DESIGN] The command reads, parses, migrates and validates
the file, prints what it found, and exits with the configuration exit code when the
file is invalid (OPS-014). It opens no socket and starts no task, so it is safe to run
in a script or in CI.

**CFG-012** — The server shall provide a command that exports the effective
configuration and one that imports it, preserving the schema version. [DESIGN] The
export writes the *effective* configuration — every default resolved to its value — as
a TOML document with its `schema_version`. The import reads such a document, validates
it and writes it to the target path, preserving the schema version. Export → import →
export is byte-identical. Secrets are never exported: the export carries the *path* of
the secret file, never its content (CFG-014).

**CFG-013** — Unknown configuration keys shall be reported as errors rather than
ignored, so that a typo never silently disables a security setting. [DESIGN] The
deserialiser is configured to deny unknown fields (ADR-0004). The error names the
unknown key and the file, and names the known keys of that table when that list is
short enough to be useful.

**CFG-014** — Secrets (such as an administration token) shall be readable from a file
with owner-only permissions and shall never appear in the configuration file itself.
[DESIGN] The configuration names the *path* of a secret file. The server refuses to use
a secret file that is group- or world-readable, and it never writes a secret into the
configuration file, into the exported configuration, into a log line or onto a command
line (SEC-TH-25, M-06).

**CFG-015** — The configuration file shall never contain a value that is expanded from an
environment variable without an explicit opt-in, to keep the file's meaning unambiguous.
[DESIGN] `${HOME}` in a TOML value is a literal string by default. Expansion exists only
when the user turns it on explicitly, and the log then names the variable that was
expanded without logging its value.

## 4. Technical requirements

- **Module.** `phairdlna-core/src/config/` owns the schema, the defaults, the validation
  and the migrations. No other module reads an environment variable or a configuration
  file (ARCHITECTURE §4.1, ADR-0004). Every other module receives its settings as typed
  values.
- **Format.** TOML, deserialised into Rust structs with `deny_unknown_fields`. The
  document carries `schema_version` at the top level. A commented example file is
  shipped and is the documentation (CFG-004, DOC-009).
- **Structure.** `[server]` (identity, version strings, log level), `[network]`
  (interfaces, `discovery`/`media` flags), `[[roots]]` (name, path, `follow_symlinks`,
  exclusions), `[limits]` (streams, cache size, extraction time, index workers),
  `[access]` (browse and stream lists), `[[clients]]` (trusted-client profiles) and
  `[secrets]` (paths to secret files).
- **Validation.** One pass over the whole document at start-up, before any socket is
  bound and before the index is opened. Cross-field rules — "the administration bind
  address must be loopback", "the interface list must not be empty" — live in the
  `config` module and are checked there, not at the point of use (ADR-0004).
- **Error formatter.** One formatter produces the "what happened / why / what to do"
  shape (UX-004, ARCHITECTURE §11). The same formatter is used by the command line and
  by the macOS application, so the wording is identical everywhere (ADR-0004).
- **Secrets.** The secret file is opened read-only; its mode is checked before it is
  read; its content is held in memory for the lifetime of the process and compared in
  constant time (SEC-TH-25). It is never passed on a command line, never logged and
  never exported.
- **Instance identity.** The configuration file may name the *instance* it belongs to,
  so that a second instance can run next to the first with its own data directory, its
  own ports and its own device identity (OPS-010, F-13).
- **No runtime reload.** The file is read once. ADR-0004 rejects runtime reload for the
  baseline because it multiplies the state combinations that must be tested.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.
Measurements follow qa-performance §4 and use the reference profile of
qa-performance §2.

- [DESIGN] Target: loading and validating the configuration takes under 50 ms. Method:
  `criterion` over the shipped example file and over a maximal example file, ≥ 30
  iterations, median and p95 reported. The configuration is read once, so this cost is
  paid once per process start.
- [DESIGN] Target: the configuration contributes no measurable cost to the idle CPU
  budget (PERF-013) and to the start-up budget (PERF-002). Method: the start-up
  benchmark of qa-performance §5.3, with and without a configuration file.
- [DESIGN] Target: validating a configuration file with the validation command
  completes in under 200 ms. Method: `hyperfine` (qa-performance §4), ≥ 30 iterations.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible surface is the file the user
  edits, the error the user reads when the file is wrong, and the two commands that
  check and export it.
- [UX-004] Every error message states what happened, why it happened and what the user
  can do next — the four-line block of CFG-002 is the shape.
- [UX-006] Every sub-command of the configuration group has its own `--help`
  (UX-006, F-13).
- [UX-010] The onboarding text says "point it at a folder" and never names a TOML key
  before the user asks for the file (PRD-009, UX-002).
- The shipped example file is commented in plain language, and every key states its
  default and its accepted values — a user should never have to read the source to learn
  what a setting does (DOC-005).
- A migration produces a warning the user can understand: "your configuration was
  written for version 2 and has been migrated to version 3 in memory; run
  `phairdlna config export` to save the new shape".

## 7. Security and privacy

- A typo in a key name is an error, not a no-op (CFG-013). This is the single most
  important property of the feature: a mistyped `follow_symlinks = true` must never
  become a silently ignored line.
- The administration surface binds to loopback unless the user explicitly overrides it
  (CFG-005, OPS-020, SEC-017).
- Secrets live in owner-only files and never in the configuration file, in the exported
  configuration, in a log line or in `argv` (CFG-014, SEC-TH-25).
- Values are not expanded from the environment unless the user opts in (CFG-015), so the
  meaning of the file does not depend on the environment it happens to run in.
- The configuration file may contain absolute paths, which are personal data in the
  sense of SEC-093. It stays on the machine, in a user-owned directory, and it is
  included in the diagnostic report only in redacted form (OPS-007, SEC-094).
- The access-control lists are evaluated against the *peer address* of the request, never
  against a header the client controls (SEC-018).

## 8. Interoperability

- The configuration carries the values that the network layer advertises:
  `friendlyName`, `manufacturer`, `modelName` and the device identity appear in the
  device description (F-02) and in the SSDP headers, and they must be the same strings
  in both (INT-012, F-01).
- The interface selection decides which addresses the server advertises as `LOCATION`;
  the advertised URL must be reachable on the interface the request arrived on
  (INT-006), so the configuration and the SSDP module must agree about which interfaces
  are in use.
- The resource limits decide what the server *advertises* it can do: the `Source` list
  of the ConnectionManager must not name more simultaneous streams than the limit
  allows (INT-051, F-04).
- The trusted-client list selects device profiles, which are documented with the
  evidence that justified each workaround (INT-100, INT-102, F-07).

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| The configuration file does not exist | the server starts with the defaults and says so in the first-run output | CFG-004, PRD-009 |
| The file is not valid TOML | start-up aborts with the parse position and the file name | CFG-002 |
| A value has the wrong type | start-up aborts naming field, file, expected type and accepted values | CFG-002 |
| A key is unknown | start-up aborts naming the key and the file | CFG-013 |
| The file is empty | the server starts with every default | CFG-004 |
| `schema_version` is older and known | the file is migrated in memory, a warning is logged, the start continues | CFG-003 |
| `schema_version` is newer than the software | start-up aborts naming both versions; nothing is guessed | CFG-003 |
| The administration bind address is not loopback | start-up aborts unless the explicit override is present | CFG-005 |
| A secret file is missing or world-readable | start-up aborts naming the path and the required mode | CFG-014 |
| A trusted-client entry names an unknown profile | start-up aborts naming the entry and the known profile ids | CFG-010 |
| The interface list is empty | start-up aborts; a server that cannot advertise is not a server | CFG-006 |
| The validation command is run against a bad file | it prints the same error block and exits with the configuration exit code | CFG-011, OPS-014 |

No configuration error may produce a panic message without context (UX-005); every one
of them is a rendered error with the three-part shape.

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a configuration file named by `--config`, *when* the server starts, *then* that file is used; *given* only `PHAIRDLNA_CONFIG`, *then* that file is used; *given* both, *then* the switch wins. Requirements: CFG-001 · Verification: automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-02** — *Given* a configuration file with a value of the wrong type, *when* the server starts, *then* the start aborts and the message names the field, the file, the expected type and the accepted values. Requirements: CFG-002, UX-004 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-03** — *Given* an empty configuration file, *when* the server starts, *then* it starts successfully, binds no administration port to a non-loopback address and creates its index and identity in the user's own directory. Requirements: CFG-004, PRD-009 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-04** — *Given* a configuration file with a known older `schema_version`, *when* the server starts, *then* the file is migrated in memory, a warning naming the migration is logged, and the file on disk is unchanged. Requirements: CFG-003 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_migrations.rs`

**AC-05** — *Given* a configuration file with a `schema_version` newer than the software understands, *when* the server starts, *then* the start aborts with a message naming both versions and no setting is guessed. Requirements: CFG-003 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_migrations.rs`

**AC-06** — *Given* a configuration file containing an unknown key, *when* the server starts, *then* the start aborts naming the key and the file, and the setting is not silently ignored. Requirements: CFG-013 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-07** — *Given* a configuration file with an administration bind address that is not a loopback address, *when* the server starts, *then* the start aborts unless the explicit override is present, and with the override the start continues. Requirements: CFG-005 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-08** — *Given* a configuration file that selects one interface for discovery and another for media, *when* the server starts, *then* discovery is announced only on the first, the media listener is bound only on the second, and the interfaces command reports the same split. Requirements: CFG-006, INT-009, OPS-004 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-09** — *Given* a root with `follow_symlinks` disabled and a symbolic link inside it, *when* the catalogue is built, *then* the link is not followed, the target is not indexed through it and no file outside the root is opened; *given* the setting enabled and the link pointing inside the root, *then* the target is indexed. Requirements: CFG-007, SEC-006 · Verification: automated test (filesystem audit hook) · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-10** — *Given* a configuration file with the four resource limits set, *when* the server runs, *then* the stream count never exceeds the configured maximum, the cache never exceeds the configured size, a file that exceeds the extraction time is abandoned, and the number of concurrent index workers never exceeds the configured maximum. Requirements: CFG-008, SEC-062, SEC-064, FUN-043 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_limits.rs`

**AC-11** — *Given* an access-control list naming one client address, *when* that address browses the catalogue, *then* the browse succeeds; *when* another address browses, *then* the request is refused with `403` and a message revealing nothing beyond the refusal; *given* no list at all, *then* every LAN client may browse. Requirements: CFG-009, SEC-013 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/config_access.rs`

**AC-12** — *Given* an access-control list naming one address for streaming, *when* another address requests a media object, *then* the request is refused with `403` and no byte of the media is written. Requirements: CFG-009, SEC-013 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/config_access.rs`

**AC-13** — *Given* a trusted-client entry naming a known device profile and a second entry naming an unknown profile, *when* the server starts, *then* the start aborts naming the unknown profile, and with only the valid entry the quirks of that client are handled by the named profile. Requirements: CFG-010, INT-100 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

**AC-14** — *Given* the validation command and a configuration file with an invalid value, *when* the command is run, *then* it prints the error block, opens no socket and exits with the configuration exit code. Requirements: CFG-011, OPS-014 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-15** — *Given* the export command and a running configuration, *when* it is exported, imported and exported again, *then* the two exports are identical, the `schema_version` is preserved, and no secret value appears in either file. Requirements: CFG-012, CFG-014 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-16** — *Given* a configuration file that references a secret file, *when* the secret file is readable by the group or by others, *then* the start aborts naming the path and the required mode; *given* a mode of `0600` inside a `0700` directory, *then* the start continues and the secret never appears in a log line, in the exported configuration or in a process argument list. Requirements: CFG-014, SEC-TH-25 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_secrets.rs`

**AC-17** — *Given* a configuration value containing `${HOME}` and expansion not enabled, *when* the server starts, *then* the value is the literal string `${HOME}`; *given* expansion explicitly enabled, *then* the value is expanded and the log names the variable without logging its value. Requirements: CFG-015 · Verification: automated test · Planned location: `crates/phairdlna-core/tests/config_schema.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Schema, defaults, validation, unknown keys | `crates/phairdlna-core/tests/config_schema.rs` |
| Migrations from every known older version | `crates/phairdlna-core/tests/config_migrations.rs` |
| Resource limits | `crates/phairdlna-core/tests/config_limits.rs` |
| Access-control lists | `crates/phairdlna-core/tests/config_access.rs` |
| Secret files and their permissions | `crates/phairdlna-core/tests/config_secrets.rs` |
| Schema unit tests | `crates/phairdlna-core/src/config/validate.rs`, `crates/phairdlna-core/src/config/migrate.rs` (inline `#[cfg(test)]` modules) |
| Configuration commands | `crates/phairdlna-cli/tests/cli_config.rs` |

**Method.** The whole schema is validated from a table of examples — one example per
field, per accepted value and per rejection — and the migration path from every older
version is exercised separately, because a migration is a function from one document shape
to another (ADR-0004). The shipped example file is itself a test case: it must parse,
validate and export identically (DOC-009).

**Dependencies.** F-13 (the command line and the exit codes), F-14 (the administration
bind address and the token file), F-07 (the device profiles named by CFG-010), F-01 and
F-02 (the identity values the configuration carries), F-09 and F-10 (the limits that
bound their work) and the platform trait that locates the per-user configuration
directory (ARCHITECTURE §12).

## 12. Open questions and decisions

1. **[OPEN] User instance or system instance?** OPS-010 asks for a service identity so
   that a second instance can run beside the first, and PLT-013 makes the macOS
   always-on server a system-context launchd daemon. A daemon and a per-user instance
   do not share a configuration directory, a data directory or a secret file. *Proposed
   reading:* the configuration file is per instance, its path is supplied explicitly, and
   the platform adapter offers a per-user default path only for the user instance.
   *Closes when:* F-13 fixes the per-instance directory layout (OPS-009, OPS-010).
2. **[OPEN] Does a migration rewrite the file on disk?** ADR-0004 says the migration
   happens in memory. The alternative — rewriting the file, with a backup — is friendlier
   but touches a file the user wrote. *Proposed reading:* in memory only, with the
   export command as the way to obtain the new shape (CFG-012). *Closes when:* the owner
   confirms.
3. **[OPEN] May the user raise the metadata extraction time above the sandbox limit?**
   CFG-008 lists a "maximum metadata extraction time" as a user setting, while SEC-062
   and SEC-066 bound it in the sandbox. *Proposed reading:* the setting may only lower
   the limit, never raise it; the configured value is clamped and the clamp is logged.
   *Closes when:* the owner confirms the direction of the clamp.
4. **[OPEN] Does the access-control list affect discovery?** CFG-009 speaks of browsing
   and streaming. *Proposed reading:* it does not affect SSDP; the server keeps
   advertising and refuses at the HTTP layer, because a server that disappears from the
   network is indistinguishable from a broken one. *Closes when:* the owner confirms.
5. **[OPEN] Boolean or three-valued symbolic-link policy?** CFG-007 asks for a per-root
   setting that is "disabled by default"; the platform research proposes three policies
   — follow the target, index the link as its own entry, skip the link (platform §6.2
   rule 6). *Proposed reading:* implement the boolean required by CFG-007, and consider
   the third value ("index as link") later. *Closes when:* the owner decides.
6. **[OPEN] Is environment expansion worth having at all?** CFG-015 requires that
   expansion not happen without an opt-in, which leaves the opt-in open. *Proposed
   reading:* implement the opt-in, restricted to a documented subset of variables and to
   string values; otherwise a user who needs an environment-specific path can set it per
   instance. *Closes when:* the owner decides.
7. **[OPEN] The shape of the `schema_version` number.** The baseline says the file
   "shall carry a `schema_version`" without fixing its type. *Proposed reading:* a
   positive integer, incremented when a setting changes meaning; the migrations are
   named after the version they produce. *Closes when:* the owner fixes the scheme.
8. **[DECIDED DIRECTION, DEC-079; implementation gated by OD-15] How the native application bootstraps and activates saved settings.** ADR-0004
   rejects runtime configuration reload in the baseline. The app creates the initial typed config locally with restrictive ownership/permissions
   and an atomic write; later settings updates use the authenticated API, validate the complete candidate, persist atomically and require a
   controlled service restart. The app reports saved and effective state separately. *Implementation closes when:* F-14/F-15 define and verify
   validation, atomicity, conflicts, restart failure and recovery, and the approved service consumes the saved roots.

### Revision history

| Version/date | Change |
| --- | --- |
| 0.3 / 2026-11-19 | Records DEC-079's local first-config bootstrap and authenticated API update/restart direction, gated on OD-15's service evidence. |
| 0.2 / 2026-11-18 | Links app-driven configuration activation to OD-16 without changing ADR-0004's no-runtime-reload baseline. |
