#!/usr/bin/env bash
# Local CI runner: runs all Quality Gates locally without cloud runners.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "=== 1. Checking format (QG-01) ==="
cargo fmt --all -- --check

echo "=== 2. Checking clippy lints (QG-02) ==="
cargo clippy --workspace --all-targets -- -D warnings

echo "=== 3. Running workspace unit and integration tests (QG-04, QG-05, QG-07) ==="
cargo test --workspace

echo "=== 4. Checking no unsafe code (QG-03) ==="
./scripts/check-no-unsafe.sh

echo "=== 5. Checking file sizes (QG-13) ==="
./scripts/check-file-size.sh

echo "=== 6. Checking crate dependency boundaries (QG-28) ==="
./scripts/check-crate-boundaries.sh

echo "=== 7. Checking documentation relative links (QG-11) ==="
./scripts/check-doc-links.sh

echo "=== 8. Checking licenses (QG-08) ==="
cargo deny check licenses

echo "=== 9. Checking bans and sources (QG-10) ==="
cargo deny check bans sources

echo "=== 10. Checking security advisories (QG-09) ==="
cargo audit

echo "=== 11. Checking requirement views synchronization ==="
python3 scripts/generate_requirement_views.py --check

echo "=== ALL LOCAL QUALITY GATES PASSED SUCCESSFULLY ==="
