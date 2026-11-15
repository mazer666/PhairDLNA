#!/usr/bin/env bash
# Gate QG-28: the crate dependency direction must stay one-way.
#
# Why: ADR-0001 makes the dependency direction the mechanism that keeps the core
# portable. A core that depends on a front end, or a front end that depends on
# another front end, would make the portability claim unverifiable.
#
# The allowed direction is:
#
#   phairdlna-mac  ─┐
#   phairdlna-admin ┼──▶ phairdlna-cli ──▶ phairdlna-core
#   phairdlna-cli  ─┘
#
# A crate may depend on the crate to its right in that line, and on nothing
# else. `phairdlna-core` may depend on no sibling crate at all.
#
# Exit codes: 0 = pass, 1 = fail, 77 = skipped (with a reason).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

if ! command -v python3 >/dev/null 2>&1; then
    echo "SKIPPED (exit 77) - python3 is not available, so the boundary gate cannot run"
    exit 77
fi

mapfile -t MANIFESTS < <(git ls-files 'crates/*/Cargo.toml' 2>/dev/null)

if [ "${#MANIFESTS[@]}" -eq 0 ]; then
    echo "SKIPPED (exit 77) - no crate manifests found"
    exit 77
fi

python3 - "${MANIFESTS[@]}" <<'PYTHON'
import re
import sys
from pathlib import Path

# What each crate is allowed to depend on, from the ADR-0001 diagram.
ALLOWED = {
    "phairdlna-core": set(),
    "phairdlna-cli": {"phairdlna-core"},
    "phairdlna-admin": {"phairdlna-core"},
    "phairdlna-mac": {"phairdlna-core"},
}

SIBLING = re.compile(r'^\s*(phairdlna-[a-z]+)\s*=')
PATH_DEP = re.compile(r'path\s*=\s*"\.\./([a-z0-9-]+)"')

violations = []
checked = 0

for manifest_name in sys.argv[1:]:
    path = Path(manifest_name)
    if not path.exists():
        continue
    crate = path.parent.name
    if crate not in ALLOWED:
        violations.append(f"{crate}: not a known crate of this workspace")
        continue
    text = path.read_text(encoding="utf-8")
    for line in text.splitlines():
        match = SIBLING.match(line)
        if not match:
            continue
        dependency = match.group(1)
        checked += 1
        if dependency not in ALLOWED[crate]:
            violations.append(
                f"{crate} depends on {dependency}; the allowed set is "
                + (", ".join(sorted(ALLOWED[crate])) or "(no sibling crate)")
            )
    for match in PATH_DEP.finditer(text):
        target = match.group(1)
        checked += 1
        if target not in ALLOWED[crate]:
            violations.append(
                f"{crate} has a path dependency on {target}; the allowed set is "
                + (", ".join(sorted(ALLOWED[crate])) or "(no sibling crate)")
            )

if violations:
    print("FAIL - the crate dependency direction is broken:")
    for item in violations:
        print(f"  {item}")
    print()
    print("The direction is defined in docs/02-architecture/adr/ADR-0001 and is")
    print("the mechanism that keeps phairdlna-core portable.")
    print()
    print("RESULT: FAIL")
    sys.exit(1)

print(f"RESULT: PASS ({checked} sibling dependency/dependencies checked across "
      f"{len(sys.argv) - 1} crate manifest(s))")
PYTHON
exit $?
