# Test strategy — draft

Test pyramid: unit tests for core/domain; integration tests for SSDP/UPnP, catalog and streaming; protocol conformance and negative input suites; cross-platform builds; real-device compatibility; load and seven-day soak tests before stable.

Required negative tests include malformed SSDP/SOAP/XML, oversized requests, path traversal, symlink changes, partial/invalid ranges, inaccessible shares, corrupt indexes, interrupted clients, hostile metadata and resource exhaustion.

Each test records environment, expected/observed result and reproducibility. Hardware performance budgets are goals, not guarantees.
