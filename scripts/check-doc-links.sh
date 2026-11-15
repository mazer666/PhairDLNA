#!/usr/bin/env bash
# Gate QG-11: every relative link in the documentation must resolve.
#
# Only *relative* links are checked: a link to an external site cannot be verified
# offline, and a link that cannot be verified must not be reported as passing.
#
# Exit codes: 0 = pass, 1 = fail, 77 = skipped (with a reason).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

if ! command -v python3 >/dev/null 2>&1; then
    echo "SKIPPED (exit 77) - python3 is not available, so the link gate cannot run"
    exit 77
fi

if ! type mapfile >/dev/null 2>&1; then
    mapfile() {
        local target="MAPFILE"
        if [ "$1" = "-t" ]; then shift; fi
        if [ $# -gt 0 ]; then target="$1"; fi
        eval "$target=()"
        local line
        while IFS= read -r line || [ -n "$line" ]; do
            eval "$target+=(\"\$line\")"
        done
    }
fi

# Archived snapshots are immutable historical documents: their links were valid
# when they were written and are not expected to resolve against the current tree.
mapfile -t DOCS < <(git ls-files '*.md' 2>/dev/null | grep -v '^docs/99-archive/')

if [ "${#DOCS[@]}" -eq 0 ]; then
    echo "SKIPPED (exit 77) - no tracked markdown files found"
    exit 77
fi

python3 - "${DOCS[@]}" <<'PYTHON'
import re
import sys
from pathlib import Path

LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
SKIP_SCHEMES = ("http://", "https://", "mailto:", "#", "data:")

broken = []
checked = 0

for name in sys.argv[1:]:
    path = Path(name)
    if not path.exists():
        continue
    text = path.read_text(encoding="utf-8", errors="replace")
    # Ignore fenced code blocks so that example links are not checked.
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    for target in LINK.findall(text):
        target = target.strip()
        if target.startswith(SKIP_SCHEMES) or not target:
            continue
        # Strip an anchor and any title part.
        target = target.split("#", 1)[0].split(' "', 1)[0].strip()
        if not target:
            continue
        checked += 1
        resolved = (path.parent / target).resolve()
        if not resolved.exists():
            broken.append(f"{name}: {target}")

if broken:
    print("FAIL - broken relative links:")
    for item in broken:
        print(f"  {item}")
    print(f"\nRESULT: FAIL ({len(broken)} broken of {checked} checked)")
    sys.exit(1)

print(f"RESULT: PASS ({checked} relative links checked)")
PYTHON
exit $?
