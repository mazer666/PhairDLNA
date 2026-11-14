# F-LIB — media library and indexing

Status: Draft.

LIB-001 Add/remove multiple local or mounted roots without altering source media.
LIB-002 Index supported media with metadata and stable browse identifiers; use incremental updates and explicit rescan.
LIB-003 Missing/offline drives do not silently delete user configuration or corrupt the index.
LIB-004 Recover from abrupt termination and corrupt index with documented repair procedure.
LIB-005 Follow an explicit symlink policy; prevent traversal beyond authorized roots.
LIB-006 Support at least 100,000 items under a defined test dataset without unbounded memory growth (proposed target).
LIB-007 New files appear within proposed 30 seconds on reliable filesystem-event sources; fallback periodic scans documented.
LIB-008 Provide scan progress, errors, pause/cancel and privacy-preserving diagnostics.

Acceptance: repeatable fixture-based scan, rename/delete/mount/unmount, symlink race, corrupted database and SIGKILL tests; assert originals unchanged by checksum.
