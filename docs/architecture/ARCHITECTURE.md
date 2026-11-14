# Architecture — draft

Rust workspace with independently testable components:
1. Core: configuration, task lifecycle, diagnostics and policy.
2. Discovery: SSDP, UPnP descriptors and service lifecycle.
3. Catalog: filesystem scan, SQLite index, metadata, virtual browsing.
4. Streaming: HTTP media delivery, range support, backpressure and cancellation.
5. Compatibility: declarative device profiles and conformance fixtures.
6. Transcoding (optional): isolated adapters, bounded queues and resource budgets.
7. Interfaces: headless CLI, local admin API, native platform shell.

Candidate libraries: Tokio, Axum/Hyper, SQLite (via a maintained Rust binding). These are proposals, not approved dependencies.

Storage classes: disposable cache, persistent recovery state and user-owned artifacts. Use opaque internal names, atomic writes and fsync where appropriate. Never promise secure deletion. No background network telemetry.

Platform-specific adapters must not leak into the core. Every external dependency requires license, maintenance and security assessment.
