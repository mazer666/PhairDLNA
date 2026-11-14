# F-STREAM — HTTP direct play

Status: Draft.

STR-001 Serve supported media over HTTP with accurate content-type/length, GET and HEAD.
STR-002 Support valid single byte-range requests, 206 responses, Content-Range and 416 for unsatisfiable ranges; define multipart range policy explicitly.
STR-003 Handle seek, client disconnect, slow readers, cancellation and concurrent streams without leaked resources.
STR-004 Prefer direct play; never silently transcode or modify originals.
STR-005 Correctly represent client-supported subtitle resources when available.
STR-006 Bound concurrency, open file handles, network buffering and per-client resources.

Acceptance: RFC-aligned range/conditional request test vectors, playback on reference clients, interrupted/slow-client tests, 10 parallel direct-play streams on documented hardware/network (proposed target), and resource measurement. Do not imply every codec works on every device.
