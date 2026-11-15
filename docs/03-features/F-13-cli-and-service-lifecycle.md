# F-13 — Command line and service lifecycle

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: OPS-001, OPS-002, OPS-003, OPS-004, OPS-005, OPS-006, OPS-007, OPS-008,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
OPS-009, OPS-010, OPS-011, OPS-012, OPS-013, OPS-014, OPS-015, LIFE-001, LIFE-002,
LIFE-003, LIFE-004, LIFE-005, LIFE-006, LIFE-007, LIFE-008, LIFE-009, LIFE-011

## 1. Purpose and user story

PhairDLNA is a server, and a server is something you start, watch and stop. The
**command line** is its primary interface: the product must be complete and usable
without any graphical application being present or running (PRD-006, UX-001). The
**menu-bar application** on macOS is a convenience, not a prerequisite.

Everything a user needs must therefore exist as a command:

- run the daemon and watch what it does,
- ask it how it is doing,
- tell it to rescan now,
- see which network interfaces it will use and why,
- print the exact bytes it would advertise, for interoperability debugging,
- export a diagnostic report for a bug report,
- and stop it cleanly when the machine shuts down.

A server that is killed rather than stopped leaves a television showing a dead entry and
may leave a half-written index behind. The **service lifecycle** is therefore part of
the product, not an afterthought: on macOS a `launchd` **daemon**, on Linux a `systemd`
unit, on Windows a Windows service — each of them starting the daemon, restarting it
when it dies and stopping it politely.

**User story.** As a user, I want to run the server from a terminal, see what it is
doing, and stop it without leaving a mess behind; and I want it to run unattended as a
system service, so that my television can find it after a reboot without me logging in.

## 2. Scope

**In scope**

- The `phairdlna-cli` crate: the command parser, the sub-commands and the process
  lifecycle (ARCHITECTURE §2, §3).
- The foreground daemon command and its structured log output on standard error
  (OPS-001, OPS-006).
- The status, scan, interfaces, description, diagnostics and configuration commands
  (OPS-002 … OPS-005, OPS-013, CFG-011, CFG-012).
- Log redaction (OPS-007) and the error format (UX-004).
- Signal handling and the graceful shutdown sequence (OPS-008).
- The service integration on macOS, Linux and Windows (OPS-009) and the instance
  identity that lets a second instance run beside the first (OPS-010).
- The service lifecycle states and the network-recovery behaviour of the daemon
  (LIFE-001 … LIFE-009, LIFE-011): starting, serving, sleeping and waking, an
  interface that disappears, a volume that goes away, a crash, and the recovery
  after it.
- The help and version output (OPS-011) and the exit-code scheme (OPS-014).

**Explicitly out of scope**

- The administration HTTP surface, its token and its CSRF protection (F-14 owns
  OPS-020 … OPS-027).
- The AGPL **source offer** — the route that serves the running build's source and the
  exact source revision (OPS-016, OPS-017) — which F-14 owns. This feature only prints
  the route's location in the diagnostic report and the revision in `--version`.
- The macOS menu-bar application (F-15), which runs as a `launchd` **agent** beside the
  daemon (PLT-013).
- Packaging and installation (F-16); the configuration schema itself (F-12).
- The transcoder's command surface (F-17).

## 3. Functional requirements

**OPS-001** — The server shall provide a command that runs the daemon in the foreground
with structured log output on standard error. [DESIGN] `phairdlna serve` runs the
server in the foreground until it is stopped. Log records go to standard error; standard
output is reserved for command *results* (the exported configuration, the description
document), so that `phairdlna config export > config.toml` produces a valid file. A
service manager runs the very same command in the background — there is no separate
"daemon mode" binary and no double-fork.

**OPS-002** — The server shall provide a command that reports the current status:
version, uptime, configured roots, catalogue size, active transfers and last scan
result. [DESIGN] `phairdlna status` asks the running daemon over a local control
channel and prints the result as text or, with a switch, as a structured document. When
no daemon is running, the command says so plainly and exits with the "nothing to do"
exit code (OPS-014). *How the command reaches a running daemon is raised in §12.*

**OPS-003** — The server shall provide a command that scans a root on demand and
reports progress and errors. [DESIGN] `phairdlna scan [root]` asks the running daemon to
scan, or performs the scan in its own process when no daemon is running. It reports the
number of files seen, added, changed and skipped, and every error with its reason. The
command can be cancelled with `Ctrl-C`, and the scan stops at the next batch boundary
(UX-013).

**OPS-004** — The server shall provide a command that lists the known network
interfaces and which ones would be used for discovery. [DESIGN] `phairdlna interfaces`
lists every interface the platform reports with its addresses and flags, and marks each
one "used for discovery", "used for media" or "excluded", naming the reason for an
exclusion (for example "excluded by configuration"). The output is what the
configuration would cause to happen, not a guess.

**OPS-005** — The server shall provide a command that prints the device description and
the SSDP headers it would advertise, for debugging. [DESIGN] `phairdlna describe` prints
the device description document and the exact header block of one advertisement and of
one answer — byte for byte as the socket would send them (INT-015, F-01). The output is
produced without opening a socket, so it works on a machine with no network.

**OPS-006** — The server shall write logs with a timestamp, a level, a component and a
structured message. [DESIGN] One record per line: timestamp, level, component (the
module that produced it — `catalogue`, `ssdp`, `http`, `media`, `config`), and the
message with its named fields. The default rendering is readable by a human; a switch
selects one-line JSON for machines. The level is a configuration setting.

**OPS-007** — Log output shall be redactable: file system paths, client addresses and
identifiers shall be replaceable by a stable pseudonym when redaction is enabled.
[DESIGN] Redaction replaces the three categories with short tokens; the same input maps
to the same token throughout a log file, so the user can still correlate "the same
client" or "the same folder" across records. The mapping is not stored, so it cannot be
reversed; it is derived from a per-installation salt (SEC-094, SEC-TH-25). *Whether the
salt survives a restart is raised in §12.*

**OPS-008** — The server shall shut down gracefully on `SIGINT` and `SIGTERM`, sending
`ssdp:byebye` and closing open transfers within a bounded time. [DESIGN] The order is
the one in ARCHITECTURE §3: the SSDP task stops first and sends `ssdp:byebye` for every
advertised target; the HTTP task then drains the open transfers for a bounded time; the
catalogue task flushes the index; the process exits with code 0. A second signal
abandons the drain and exits at once. *The budget is raised in §12.*

**OPS-009** — The server shall support running as a background service on macOS
(launchd), Linux (systemd user service) and Windows (service). [DESIGN] macOS: the
always-on server is a `launchd` **daemon** (`/Library/LaunchDaemons`), because the
local-network permission exempts daemons but not agents (PLT-013, platform §1.2, §4.2);
the user interface is a `launchd` **agent** in the user's session (F-15). The job file
uses `KeepAlive`, sets `StandardOutPath` and `StandardErrorPath` for the logs, and
`ProcessType: Background` so that the indexer does not compete with the user's work
(platform §4.2). Linux: a `systemd` unit with `Type=notify`; the daemon calls
`sd_notify(READY=1)` once the listeners are up, so that `systemctl start` returns only
when the server can actually answer (platform §8.1). Windows: a service that follows
the Service Control Manager interface (platform §9.1). *The unit scope — user or system
— is raised in §12.*

**OPS-010** — The server shall support a configuration file that sets the service
identity, so that a second instance can run next to the first. [DESIGN] The instance
name in the configuration selects the data directory, the configuration path, the log
file and the device identity. Two instances must not collide on the media port, on the
device identity (the `UDN` differs per instance — F-01, INT-014), on the index file or
on the control channel.

**OPS-011** — The command line shall provide `--help` and `--version` output that names
the version, the licence and the documentation location. [DESIGN] `--version` prints the
product name, the version, the licence (`AGPL-3.0-or-later`) and the documentation
location; `--help` lists the sub-commands with a one-line description each. Every
sub-command has its own `--help` (UX-006).

**OPS-012** — The server shall expose its version in the `SERVER` header and in the
device description. [DESIGN] One constant feeds both, so that they can never disagree
(INT-019, F-01; INT-020, F-02).

**OPS-013** — The server shall write a structured diagnostic report (anonymised) that a
user can attach to a bug report. [DESIGN] `phairdlna diagnostics` writes one document:
version and source revision, platform, the configuration summary with values redacted,
the interface list, the catalogue statistics, the last scan result, the quarantine list,
a redacted excerpt of the recent log and the location of the source offer (F-14,
OPS-016). Redaction is on by default in the report; the user can turn it off
deliberately (UX-011).

**OPS-014** — The command line shall return a distinct exit code for configuration
errors, runtime failures and "nothing to do", so that scripts can react. [DESIGN] The
codes are stable and documented: success, runtime failure, configuration error, nothing
to do. A script that runs `phairdlna status` against a stopped server can therefore
tell "not running" from "misconfigured" without parsing text. *The numbering is raised
in §12.*

**OPS-015** — The server shall never require root privileges to serve media from the
user's own directories. [DESIGN] The foreground daemon and the per-user service instance
run as the invoking user and open no privileged port. On macOS the always-on daemon
runs in a system context so that it is available at boot and exempt from the
local-network prompt — but root does **not** bypass the macOS privacy prompts for
protected folders, so folder access is granted through the user agent (platform §5.4,
PLT-017, PLT-018). *How the two reconcile on macOS is raised in §12.*

## 4. Technical requirements

- **Crate.** `phairdlna-cli` owns the process lifecycle and depends on
  `phairdlna-core`; it depends on no other crate (ARCHITECTURE §2).
- **Command set.** `serve`, `status`, `scan`, `interfaces`, `describe`, `diagnostics`,
  and the `config` group of F-12 (`check`, `export`, `import`). Each is a thin
  argument-parsing shell over a typed function in the core or in the CLI crate.
- **Streams.** Log records go to standard error; command results go to standard output;
  diagnostics go to standard output or to a file named by the user.
- **Signals.** `SIGINT` and `SIGTERM` start the shutdown sequence (OPS-008); `SIGCHLD`
  reaps the sandbox children of F-10. A second `SIGINT`/`SIGTERM` exits immediately.
- **Shutdown order.** SSDP stops and sends `ssdp:byebye` → HTTP drains open transfers
  for the bounded time → the catalogue flushes the index → the process exits with code
  0 (ARCHITECTURE §3). Each step has a deadline; the deadline that is exceeded is
  logged, and the next step starts anyway.
- **Control channel.** The status, scan and diagnostics commands talk to a running
  daemon over a local channel owned by the instance: a domain socket on Unix and a
  named pipe on Windows, created with owner-only permissions inside the instance's data
  directory. The channel carries the same bearer token as the administration surface and
  is compared in constant time (SEC-TH-25). It never listens on a network address.
- **Log records.** A record is a timestamp, a level, a component and a message with
  named fields (OPS-006). Redaction is applied at the point of formatting, so a redacted
  value cannot reach a log file through another path (OPS-007, SEC-094).
- **Service files.** The repository ships the `launchd` job files, the `systemd` unit
  and the Windows service definition, and a test checks that each one starts the same
  binary with the same arguments and the same instance name.
- **Instance identity.** The instance name from the configuration (F-12) selects the
  data directory, the ports, the device identity and the control channel name
  (OPS-010).

### 4.1 Service lifecycle and network recovery

A server is not a program that runs: it is a thing with a life. It starts, it serves,
the operating system puts it to sleep, the network changes under it, a volume
disappears, somebody kills it — and it has to come back honestly. The states below are
the vocabulary the daemon, the status command and the administration surface share.

**LIFE-001** — The server shall expose the distinct states *Starting*, *Ready*,
*Degraded*, *Suspended*, *Recovering*, *Stopping*, *Stopped* and *Failed*, and every
transition shall carry a reason, a timestamp and a user-facing explanation. [DESIGN]
The state is one value, never a set of flags: the server is *Ready* or it is not. A
transition is written to the log as one record carrying the new state, the reason, the
timestamp and the sentence the user would read (UX-004), and it is reflected in the
status output (OPS-002) and in the administration surface (F-14).

**LIFE-002** — The server shall handle a sleep/wake cycle without spinning, leaking
resources or reporting a "ready" state it does not have; on wake it shall refresh the
active interfaces, rebind its listeners as required and re-advertise only on permitted
interfaces. [DESIGN] The sleep and the wake are events reported by the platform layer;
how they are detected belongs to the adapter, not to the core. On the way down the
server releases its workers and file handles predictably (RES-009) and stops
answering; on the way up it re-reads the interface list, rebinds what has to be
rebound, and advertises again — on the permitted interfaces only, never on all of them.

**LIFE-003** — The server shall handle a change of interface, address, subnet or
route, and shall never widen its network exposure beyond the configured policy.
[DESIGN] The configured interface policy (CFG-006) is re-evaluated after every change:
an excluded interface stays excluded, and an interface the user did not configure is
never adopted. Exposure is a decision of the configuration; a change in the network is
not a reason to change it.

**LIFE-004** — When a configured interface disappears the server shall enter the
*Degraded* or *Recovering* state and shall not automatically expose itself on another
interface. [DESIGN] The disappearance is a transition with a reason — "the interface
`en0` disappeared" — and not a silent degradation. The server keeps whatever it still
has and advertises nothing new.

**LIFE-005** — The server shall detect the loss and the return of a network or
removable volume, shall preserve the configured roots and the index, and shall mark
the affected entries as unavailable rather than deleting them. [DESIGN] The volume
events are the first-class events of F-08 (FUN-017, PLT-011): the root stays in the
configuration, the index rows stay, and the entries carry an *unavailable* marker
until the volume comes back (REL-002).

**LIFE-006** — A transfer interrupted by a real sleep or a real network loss shall end
in a way that is truthful for the protocol; the server shall not promise a transparent
continuation that the client cannot achieve. [DESIGN] The connection ends the way a
client can recognise: a truncated response or a closed connection, never a body that
simply stops in the middle without saying so. The transfer slot is released
(INT-089, FUN-046), and the client can ask for the bytes it did not receive with an
ordinary byte-range request (INT-081). Nothing is "resumed" behind the client's back.

**LIFE-007** — The server shall retry discovery and rebinding with a bounded backoff
and jitter, and shall not produce a multicast storm or a tight retry loop. [DESIGN]
The retry schedule has a documented first delay, a documented maximum delay and a
random jitter, and it is reset only when the interface comes back. The bounds are
published so that the recovery time is predictable; the numbers are raised in §12.

**LIFE-008** — The server shall recover from an abnormal termination with an atomic
state and either a consistent catalogue or a safe repair, and shall not require a user
interface to be running. [DESIGN] The headless daemon alone is enough: the index is
checked at start-up and rebuilt if it is damaged (FUN-010), the media originals are
never touched (REL-001), and the recovery is reported in the log and in the status
output. The user interface may be absent.

**LIFE-009** — The administration surface and the user interface shall reflect the
effective interface, address, listener state and the last recovery event, with the
technical detail available on demand. [DESIGN] The status command (OPS-002) and the
administration surface (F-14) show the *effective* values — the address the server is
really reachable at, not the one it was configured with — and the last recovery event
in plain language; the technical detail (the raw interface table, the retry counters)
is available on request.

**LIFE-011** — The server shall keep a local, redacted timeline of network changes and
recovery events that the user may export after reviewing it. [DESIGN] The timeline is a
bounded local file in the instance's data directory, in the same redacted form as the
rest of the log (OPS-007, SEC-094): paths, addresses and identifiers appear as
pseudonyms. The user reads it before it is exported, and the export is a diagnostic
document like any other (OPS-013, UX-011).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.
Measurements follow qa-performance §4 and use the reference profile of
qa-performance §2.

- [DESIGN] Target: the daemon reaches the "ready to answer discovery" state within 2 s
  of process start (PERF-002). Method: timestamp from process start to the first
  successful SSDP advertisement, which is how qa-performance §5 defines the measurement.
- [DESIGN] Target: a status, interfaces or describe command answers within 200 ms,
  including process start-up. Method: `hyperfine` (qa-performance §4), ≥ 30 iterations,
  median and p95 reported.
- [DESIGN] Target: the graceful shutdown completes within its budget (OPS-008) even
  with open transfers, and the log names each step's elapsed time. Method: a scripted
  run that opens a transfer, sends the signal and measures the time to exit.
- [DESIGN] Target: the idle CPU usage of the daemon stays below 1 % averaged over one
  minute (PERF-013). Method: `top` sampling over 60 s while the advertisement timer and
  the log writer run.

## 6. Usability and accessibility

- The command line is the product's primary interface; every capability of the product
  must be reachable from it (PRD-006, UX-001). A capability that only exists in the
  macOS application is a defect.
- [UX-006] Every sub-command has its own `--help`, and every error message states what
  happened, why and what to do next (UX-004).
- [UX-013] The long-running commands — `serve` and `scan` — report progress and can be
  cancelled; `scan` prints a summary line at the end.
- [UX-010] The onboarding text avoids protocol jargon: the status output says "this
  server is reachable at 192.168.1.20" rather than naming the device type.
- [UX-003] The first-run output names the device name, the address the server is
  reachable at and how to check it from another device.
- [UX-011] The diagnostic report is one command and one file, copyable in one action,
  with redaction on by default.
- [DOC-005] *daemon*, *foreground*, *service*, *exit code* and *redaction* are explained
  at first use.

## 7. Security and privacy

- The daemon needs no privileges: it opens the media port and, if the user asked for
  it, the loopback administration port — never a privileged port and never a
  non-loopback administration port (OPS-015, SEC-017, CFG-005).
- The control channel is local-only and owner-only, and its bearer token is never in
  `argv`, in an environment variable visible to other processes, in a log line or in a
  crash dump (SEC-TH-25).
- Logs are the one place where personal data is most likely to leak: paths, client
  addresses and device identifiers are redactable (OPS-007, SEC-094) and the
  diagnostic report is anonymised by default (OPS-013, UX-011).
- The server makes no outbound connection other than the SSDP announcements and the
  media connections it serves (SEC-091); the command line makes none either — every
  command in this feature is local.
- The `--version` output names the licence (AGPL-3.0-or-later) and the documentation
  location (OPS-011, PRD-001).

## 8. Interoperability

- The description command prints the same device description and the same SSDP header
  block that the server would put on the wire (INT-015, F-01; F-02). This is the
  documented way to diagnose a discovery problem without a packet capture.
- The version string in the `SERVER` header and in the device description comes from one
  constant (OPS-012, INT-019).
- The service integration exists so that the server behaves identically on macOS, Linux
  and Windows (PLT-010): the same command, the same configuration, the same log format.
- The diagnostic report points at the AGPL source offer (F-14, OPS-016), which the
  network-use clause of the licence requires the running server to provide.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| The configuration is invalid | the daemon aborts before binding any socket and prints the configuration error block | CFG-002, OPS-014 |
| A signal arrives | the shutdown sequence of OPS-008 runs; a second signal exits at once | OPS-008 |
| The drain deadline is exceeded | the open transfers are closed, the step is logged with its elapsed time, the shutdown continues | OPS-008 |
| `status` is run with no daemon running | the command says so and exits with the "nothing to do" code | OPS-002, OPS-014 |
| `scan` is run with no daemon running | the command scans in its own process and reports the same summary | OPS-003 |
| The control channel cannot be reached | the command reports that the daemon is not answering and exits with the "nothing to do" code | OPS-002 |
| `interfaces` finds no usable interface | the command lists the interfaces it found and says that the server would not advertise | CFG-006 |
| The log file cannot be written | the daemon reports it on standard error and continues; the log goes to standard error instead | OPS-001 |
| The sandbox child of F-10 dies unexpectedly | the parent records it and quarantines the file; the daemon continues | SEC-066 |
| The machine sleeps | the server suspends its work and releases workers and file handles; it does not report *Ready* while it is suspended | LIFE-002, RES-009 |
| The machine wakes | the interfaces are refreshed, the listeners are rebound and discovery is re-advertised on the permitted interfaces only | LIFE-002 |
| An interface, an address or a route changes | the configured interface policy is re-evaluated and the listeners rebound; the exposure never widens beyond the configuration | LIFE-003 |
| A configured interface disappears | the server enters *Degraded* or *Recovering* with a reason and does not expose itself on another interface | LIFE-004 |
| A removable or network volume disappears | the root and its index rows are preserved and the entries are marked unavailable; the other roots keep serving | LIFE-005, REL-002 |
| A transfer is interrupted by a sleep or a network loss | the connection ends truthfully for the protocol and the transfer slot is released | LIFE-006, INT-089 |
| The daemon is killed | on restart the index is consistent or is rebuilt and reported; no user interface is needed and no media original is touched | LIFE-008, REL-001 |

No failure of this feature may produce a panic message without context (UX-005), and no
command may leave a half-written index behind (FUN-009).

## 10. Acceptance criteria and traceability

The criteria below are *planned*. The ones that M1.11 implemented say so and name the
test that proves them; the others still have no test file.

**AC-01** — *Given* the `serve` command, *when* it is started, *then* the daemon runs in the foreground, its log records
are written to standard error as structured lines, and standard output carries no log record. Requirements:
OPS-001, OPS-006 · Verification: automated CLI test · **State: implemented** — the command starts the daemon in the foreground, the daemon answers a
client on the network and the log records go to standard error
(`crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`; the format of a record is proven by
`crates/phairdlna-cli/src/serve/log.rs::tests::a_known_moment_is_read_correctly`). The switch `--port` names the port of the HTTP server; the
discovery socket and the identity file follow the configuration and the platform.

**AC-02** — *Given* a running daemon, *when* `status` is executed, *then* the output names the version, the uptime, the
configured roots, the catalogue size, the number of active transfers and the result of the last scan.
Requirements: OPS-002 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_commands.rs`

**AC-03** — *Given* no running daemon, *when* `status` is executed, *then* the command says that the server is not
running and exits with the "nothing to do" exit code, which is different from the configuration-error code.
Requirements: OPS-002, OPS-014 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_commands.rs`

**AC-04** — *Given* a library with changed and unchanged files, *when* `scan` is executed, *then* the command reports
the number of files seen, changed and skipped, and the same numbers appear in the daemon's status
afterwards. Requirements: OPS-003 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_commands.rs`

**AC-05** — *Given* a host with several network interfaces, *when* `interfaces` is executed, *then* every interface the
platform reports is listed with its addresses and flags, and each one is marked as used for discovery, used
for media or excluded, with the reason for an exclusion. Requirements: OPS-004, CFG-006 · Verification:
automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_commands.rs`

**AC-06** — *Given* the `describe` command, *when* it is executed on a host with no network, *then* it prints the device
description document and the exact header block of one advertisement and of one answer, byte for byte as the
SSDP socket would send them. Requirements: OPS-005, INT-015 · Verification: automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-07** — *Given* the log output, *when* a record is written, *then* it carries a timestamp, a level, a component and
a message with named fields, and the JSON rendering carries the same fields. Requirements: OPS-006 ·
Verification: automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_logging.rs`

**AC-08** — *Given* redaction enabled and a log record that contains a file system path, a client address and an object
identifier, *when* the record is written, *then* each of the three appears as a pseudonym, the same input
yields the same pseudonym throughout the log, and the original values appear nowhere in the file.
Requirements: OPS-007, SEC-094 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_logging.rs`

**AC-09** — *Given* a running daemon with an open transfer, *when* it receives `SIGINT`, *then* it sends `ssdp:byebye`
for every advertised target, closes the open transfer within the shutdown budget, flushes the index, exits
with code 0, and the same happens for `SIGTERM`. Requirements: OPS-008, INT-004 · Verification: automated
CLI test · **State: partly implemented** — the daemon withdraws its advertisements, stops the discovery, drains the HTTP server and exits with the code 0
after a `SIGTERM` (`crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`,
`crates/phairdlna-cli/tests/discovery.rs::the_advertisements_are_withdrawn`). The **flush of the index** does not exist, because the index on disk
(`FUN-009`) is not implemented; the drain budget of 10 s is not measured. Planned location for the rest:
`crates/phairdlna-cli/tests/cli_shutdown.rs`

**AC-10** — *Given* a daemon that is shutting down and that receives a second signal, *when* the second signal arrives,
*then* the process exits immediately without waiting for the drain deadline. Requirements: OPS-008 ·
Verification: automated CLI test · **State: implemented, not yet proven by a test** — the second signal ends the daemon at once
(`crates/phairdlna-cli/src/serve/mod.rs::run`); the test that sends two signals is still planned. Planned location:
`crates/phairdlna-cli/tests/cli_shutdown.rs`

**AC-11** — *Given* the macOS `launchd` job files in the repository, *when* they are inspected, *then* the always-on
server is a daemon (`/Library/LaunchDaemons`) and the user interface is an agent (`~/Library/LaunchAgents`),
each starting the same binary with `KeepAlive` and a log path. Requirements: OPS-009, PLT-013 ·
Verification: automated test over the shipped job files · Planned location:
`crates/phairdlna-cli/tests/service_units.rs`

**AC-12** — *Given* the Linux `systemd` unit in the repository, *when* it is inspected, *then* it declares
`Type=notify`, and the daemon sends the readiness notification only after the listeners are up.
Requirements: OPS-009 · Verification: automated test over the shipped unit file plus a manual service check
· Planned location: `crates/phairdlna-cli/tests/service_units.rs` and `qa/device-matrix.toml`

**AC-13** — *Given* two instances with different instance names, *when* both are started, *then* they use different data
directories, different ports, different control channels and different device identities, and neither
interferes with the other. Requirements: OPS-010, INT-014 · Verification: automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_instances.rs`

**AC-14** — *Given* `--version` and `--help`, *when* they are executed, *then* the version output names the version, the
licence and the documentation location, and the help output lists the sub-commands; and every sub-command
has its own `--help`. Requirements: OPS-011, UX-006 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_commands.rs`

**AC-15** — *Given* the running server, *when* the version is read from the `SERVER` header of an SSDP answer and from
the device description, *then* both carry the same version string. Requirements: OPS-012, INT-019 ·
Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-16** — *Given* the `diagnostics` command, *when* it is executed, *then* it writes one document containing the
version, the platform, the redacted configuration summary, the interface list, the catalogue statistics, the
last scan result and the quarantine list, and it contains no unredacted path, client address or identifier.
Requirements: OPS-013, UX-011, SEC-094 · Verification: automated CLI test · Planned location:
`crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-17** — *Given* a command that fails, *when* it exits, *then* the exit code distinguishes a configuration error from
a runtime failure from "nothing to do", and the distinction is stable across releases. Requirements: OPS-014
· Verification: automated CLI test · Planned location: `crates/phairdlna-cli/tests/cli_commands.rs`

**AC-18** — *Given* the daemon started as an ordinary user on a machine with the media in the user's own directories,
*when* it serves a file, *then* it opened no privileged port, required no root and read no file outside the
configured roots. Requirements: OPS-015, PLT-018, PRD-007 · Verification: automated test plus a manual
protocol check · Planned location: `crates/phairdlna-cli/tests/cli_commands.rs` and `qa/device-matrix.toml`

**AC-19** — *Given* a daemon that starts, serves, is interrupted and stops, *when* each transition happens, *then*
the server reports exactly one of the eight documented states, and every transition is written to the log with the
new state, a reason, a timestamp and a user-facing explanation. Requirements: LIFE-001, UX-004 · Verification:
automated CLI test · Planned location: `crates/phairdlna-cli/tests/service_lifecycle.rs`

**AC-20** — *Given* a running server, *when* the platform reports a sleep and later a wake, *then* the server does
not report the *Ready* state while it is suspended, it releases its workers and file handles on the way down, it
refreshes the interfaces and rebinds its listeners on the way up, and it re-advertises only on the permitted
interfaces; no listener and no task is leaked. Requirements: LIFE-002, RES-009, REL-003 · Verification: automated
test with a scripted sleep/wake event source, plus the manual trial of the resilience plan · Planned location:
`crates/phairdlna-core/tests/network_recovery.rs` and `docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md` (case
LIFE-A02, result `NOT RUN`)

**AC-21** — *Given* a configuration that selects the interfaces used for discovery and for media, *when* the
address, the subnet or the route of one of them changes, *then* the server rebinds what it must, the configured
policy is re-evaluated, and the server is not reachable on an interface the configuration excludes. Requirements:
LIFE-003, CFG-006, INT-009 · Verification: automated test with a scripted interface change, plus the manual trial
of the resilience plan · Planned location: `crates/phairdlna-core/tests/network_recovery.rs` and
`docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md` (case LIFE-A01, result `NOT RUN`)

**AC-22** — *Given* a configured interface that disappears, *when* the server notices, *then* it enters the
*Degraded* or *Recovering* state with a reason naming the interface, and it does not begin advertising on another
interface. Requirements: LIFE-004, LIFE-003 · Verification: automated test with a scripted interface removal ·
Planned location: `crates/phairdlna-core/tests/network_recovery.rs`

**AC-23** — *Given* a configured root on a removable or network volume, *when* the volume disappears and later
returns, *then* the root stays in the configuration, the index rows are preserved, the entries are marked
unavailable while the volume is gone, and they are served again when it returns. Requirements: LIFE-005, REL-002,
FUN-017 · Verification: automated test with a scripted volume event, plus the manual trial of the resilience plan
· Planned location: `crates/phairdlna-core/tests/network_recovery.rs` and
`docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md` (case LIFE-A03, result `NOT RUN`)

**AC-24** — *Given* an active transfer, *when* a real sleep or a real network loss interrupts it, *then* the
connection ends truthfully for the protocol, the transfer slot is released, and the client can obtain the missing
bytes with a byte-range request. Requirements: LIFE-006, INT-089, INT-081 · Verification: fault injection ·
Planned location: `crates/phairdlna-core/tests/transfer_interruption.rs`

**AC-25** — *Given* an interface that does not come back, *when* the server retries discovery and rebinding,
*then* the delay between attempts grows to the documented bound, carries a random jitter, and the number of
advertisement datagrams sent per unit of time stays within the documented bound; neither a tight retry loop nor a
multicast storm occurs. Requirements: LIFE-007 · Verification: automated test with a scripted clock · Planned
location: `crates/phairdlna-core/tests/retry_backoff.rs`

**AC-26** — *Given* a daemon that was killed, *when* it is started again with no user interface running, *then* the
index is consistent or is rebuilt automatically and reported, and no media original was modified. Requirements:
LIFE-008, REL-001, FUN-010 · Verification: fault injection, plus the manual trial of the resilience plan ·
Planned location: `crates/phairdlna-cli/tests/service_lifecycle.rs` and
`docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md` (case LIFE-A04, result `NOT RUN`)

**AC-27** — *Given* a server that has recovered from an interface or a volume event, *when* the status output and
the administration surface are read, *then* they show the effective interface, the address the server is actually
reachable at, the listener state and the last recovery event in plain language, and the technical detail is
available on demand. Requirements: LIFE-009, OPS-002, UX-004 · Verification: automated CLI test · Planned
location: `crates/phairdlna-cli/tests/cli_status.rs`

**AC-28** — *Given* a series of network changes and recovery events, *when* the local timeline is inspected,
*then* it holds the events in order with their reasons, it contains no unredacted path, address or identifier,
and the user can export it after reviewing it. Requirements: LIFE-011, SEC-094, UX-011 · Verification: automated
CLI test · Planned location: `crates/phairdlna-cli/tests/recovery_timeline.rs`

## 11. Test strategy and dependencies

**Test locations** (the ones that exist are marked):

| Purpose | Path | State |
| --- | --- | --- |
| Sub-command behaviour, exit codes, help and version | `crates/phairdlna-cli/tests/cli.rs` | exists |
| The `serve` command and the graceful shutdown | `crates/phairdlna-cli/tests/serve_command.rs` | exists |
| The daemon that answers a client | `crates/phairdlna-cli/tests/serve.rs` | exists |
| The discovery of the daemon | `crates/phairdlna-cli/tests/discovery.rs` | exists |
| Logging format and redaction | `crates/phairdlna-cli/tests/cli_logging.rs` | planned |
| Description and diagnostic output | `crates/phairdlna-cli/tests/cli_diagnostics.rs` |
| Two instances side by side | `crates/phairdlna-cli/tests/cli_instances.rs` |
| Shipped service files | `crates/phairdlna-cli/tests/service_units.rs` |
| Lifecycle states and transitions, crash recovery | `crates/phairdlna-cli/tests/service_lifecycle.rs` |
| Sleep/wake, interface change, volume loss and recovery | `crates/phairdlna-core/tests/network_recovery.rs` |
| Retry backoff and the advertisement rate | `crates/phairdlna-core/tests/retry_backoff.rs` |
| A transfer interrupted by a sleep or a network loss | `crates/phairdlna-core/tests/transfer_interruption.rs` |
| Recovery-event timeline and its export | `crates/phairdlna-cli/tests/recovery_timeline.rs` |
| Effective interface, address and listener state in the status output | `crates/phairdlna-cli/tests/cli_status.rs` |
| Signal, exit-code and format unit tests | `crates/phairdlna-cli/src/main.rs`, `crates/phairdlna-cli/src/commands/` (inline `#[cfg(test)]` modules) |
| Behaviour on a real service manager | `qa/device-matrix.toml` |
| Behaviour across a real sleep/wake cycle and a real network change | `docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md` (cases LIFE-A01 … LIFE-A04, all `NOT RUN`) |

**Method.** The commands are thin wrappers over typed functions, so most of them are
exercised by calling the function and inspecting the rendered output. The shutdown
sequence is exercised by starting the daemon as a child process, opening a transfer against
it, sending the signal and asserting the order of the observable effects — the
`ssdp:byebye` datagram, the closed transfer, the flushed index and the exit code. The
shipped `launchd`, `systemd` and Windows service files are checked by a test that
parses them, so that a broken unit file fails the build rather than the user's machine.
The lifecycle and the recovery are exercised by driving the same interface the
platform adapter implements with scripted events — a sleep, a removed interface, a lost
volume — so that the recovery logic is testable without unplugging a cable; the real
sleep/wake cycle and the real network change remain manual checks (the resilience
acceptance plan, cases LIFE-A01 … LIFE-A04, all `NOT RUN`).

**Dependencies.** F-01 (the SSDP headers the description command prints, and the
`ssdp:byebye` of the shutdown), F-02 (the device description), F-09 (the catalogue
statistics and the last scan result), F-12 (the configuration and the instance name),
F-14 (the source offer referenced by the diagnostic report, and the token of the
control channel) and the platform trait for interface enumeration and per-user
directories (ARCHITECTURE §12). The lifecycle and recovery cases depend on the
volume events of F-08 and on the resilience acceptance plan
(`../05-quality/RESILIENCE-ACCEPTANCE-PLAN.md`, cases LIFE-A01 … LIFE-A04), whose
results are all `NOT RUN`.

## 12. Open questions and decisions

1. **[OPEN] The Linux service scope.** OPS-009 names a "systemd **user** service",
   while the platform research recommends a **system** service for the server, because a
   user service dies with the session and would need `loginctl enable-linger` to
   survive it, and because the server must run with no user logged in (platform §8.1).
   *Proposed reading:* the project ships both units — a system unit as the primary and a
   user unit as the alternative for people who do not want a system install — and both
   use `Type=notify`. *Closes when:* the owner fixes the primary distribution form.
2. **[OPEN] The macOS daemon contradicts one research mitigation.** PLT-013 and the
   platform research both put the always-on server in a system-context `launchd`
   **daemon**, because the local-network permission exempts daemons but not agents
   (platform §1.2, §4.2). The security research's mitigation M-07 says the opposite —
   "ship a per-user LaunchAgent (never root, never a system daemon)". The baseline
   requirement and the platform facts agree with each other, so this document follows
   the baseline; the mitigation text needs correcting. *Closes when:* the owner
   reconciles M-07 with PLT-013.
3. **[OPEN] How the commands reach a running daemon.** A separate `status` process and
   a background daemon have to talk. *Proposed reading:* a local, owner-only control
   channel in the instance's data directory, authenticated with the same bearer token as
   the administration surface; the status command never falls back to reading the index
   directly, because two readers of one SQLite file is a source of confusion.
   *Closes when:* the owner confirms the channel.
4. **[OPEN] The shutdown budget.** OPS-008 requires a *bounded* time without naming
   one. *Proposed:* 10 s to drain open transfers and 5 s to flush the index, after
   which the process exits anyway. *Closes when:* the owner fixes the numbers.
5. **[OPEN] Does the redaction salt survive a restart?** "A stable pseudonym"
   (OPS-007) can mean stable within one log file or stable across restarts. *Proposed
   reading:* the salt lives in the instance's data directory, so a pseudonym is stable
   across restarts and across the diagnostic report, but the mapping still cannot be
   reversed. *Closes when:* the owner confirms.
6. **[OPEN] The exit-code numbering.** OPS-014 requires distinct codes but fixes no
   values. *Proposed:* `0` success, `1` runtime failure, `2` configuration error, `3`
   nothing to do. *Closes when:* the owner fixes the numbers and they are published in
   the command reference.
7. **[OPEN] Root on macOS and the privacy prompts.** The daemon runs as a daemon in a
   system context, but root does not bypass the macOS privacy prompts for protected
   folders (platform §5.4), and the research leaves open whether a root daemon can use
   a security-scoped bookmark created by the user agent. *Closes when:* a device test
   settles it; until then the design assumes the agent mediates folder access.
8. **[OPEN] The recovery backoff bounds.** LIFE-007 requires a bounded backoff with
   jitter and names no numbers. *Proposed reading:* the first retry waits 1 s, each
   further attempt doubles the delay up to a maximum of 60 s, and every attempt carries
   a random jitter of up to 20 % of the current delay; the schedule is reset when the
   interface comes back. The same numbers govern the rebinding attempts and the
   advertisement re-sends. *Closes when:* the owner fixes the numbers and they are
   published next to the state machine.
