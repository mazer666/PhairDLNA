# OD-18 security review — 2026-11-19

Status: **Targeted independent review complete; M-02 security review remains open**

## Scope

Read-only review of the current OD-18 implementation: the `quick-xml` 0.42.0
adapter and project-owned XML policy; both `tower-http` timeout placements; admin
socket deadlines and connection cap; the existing CLI `DeadlineIo`; and the
dependency and status claims. This report does not certify the complete parser,
HTTP surface, dependency graph or M-02 milestone.

## Inspected paths

- `crates/phairdlna-core/src/upnp/xml_reader.rs`
- `crates/phairdlna-core/src/upnp/xml_syntax.rs`
- `crates/phairdlna-core/tests/xml_reader.rs`
- `crates/phairdlna-core/tests/xml_reader/truncation.rs`
- `crates/phairdlna-admin/src/routes/timeout.rs`, `routes.rs`, `listener.rs`,
  `server.rs` and `guard.rs`
- `crates/phairdlna-admin/tests/admin_surface/timeout.rs`
- `crates/phairdlna-cli/src/serve/http.rs`, `guard.rs`, `limit.rs` and
  `setup/startup.rs`
- `Cargo.toml`, `Cargo.lock`, crate manifests, `deny.toml`,
  `docs/04-security/DEPENDENCY-LICENSE-REVIEW.md`, ADR-0002, OD-18 and F-16

## Findings verified in source

- The XML byte limit is checked before `Reader` construction. Application-owned
  tree construction retains the configured depth, node and text bounds. DTDs
  and custom entities are refused; the five predefined entities and legal XML
  1.0 numeric character references are accepted.
- Numeric references are checked against XML 1.0's legal character ranges.
  Tests reject forbidden references in text and attributes. A boundary test
  confirms that `max_depth` includes the root at depth zero.
- The admin `TimeoutLayer` is inside the guard and bounds handler-future
  completion. The CLI router applies the same handler-future timeout inside its
  guard. These layers do not replace socket I/O deadlines or time response-body
  streaming after the handler returns.
- `AdminListener` shares a semaphore connection budget across its listeners;
  each accepted connection holds a permit until its I/O wrapper is dropped.
  Its `DeadlineIo` resets read and write silence deadlines only on progress.
  The CLI's existing `CappedListener` / `DeadlineIo` remains in place with the
  same reset-on-progress behavior.
- `tower-http` tracing is not wired. The dependency review correctly leaves its
  privacy and redaction policy open. It also records the need to assess the
  complete resolved dependency graph and the parser's internal allocations.

## Test evidence

- **Observed in this review:**
  `cargo test -p phairdlna-core --test xml_reader --locked --offline` — **22
  passed**. Coverage includes XML 1.0 reference validity, depth boundary,
  resource bounds and eleven watchdog-bounded malformed/truncated inputs with
  valid-document recovery.
- **Parent-reported:** the loopback integration test
  `timeout::a_partial_header_is_closed_by_the_socket_read_deadline` passed when
  run with the required elevated socket access. This review's sandboxed attempt
  could not bind either loopback address (`Operation not permitted`), so that
  local attempt is not a pass or a code failure.
- Admin `DeadlineIo` unit tests cover idle reads, blocked writes and permit
  release. These are wrapper-level tests; they do not establish full end-to-end
  write-deadline behavior through a running server and client.

## Remaining blockers

1. Assess and record the complete resolved lock graph and its licences.
2. Review `quick-xml`'s hostile-input behavior and internal allocation bounds;
   add any missing parser-policy coverage beyond the targeted XML reader suite.
3. Define and review a redaction-safe `tower-http` tracing policy before adding
   the accepted tracing role.
4. Record full socket read/write deadline regression evidence, including the
   end-to-end write-deadline path and slow-but-progressing transfers.
5. Complete an independent security review of the integrated changes and all
   required tests before closing M-02.

**Conclusion:** no implementation blocker was found in the reviewed adapter or
timeout placements. This is a targeted review only. OD-18's implementation
decision is recorded, but **M-02 is not complete** and this document must not be
read as a full security approval.
