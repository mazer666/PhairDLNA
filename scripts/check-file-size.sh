#!/usr/bin/env bash
# Gate QG-13: file and function size limits.
#
# Why this script exists: no maintained Rust tool measures "maximum file length"
# against a project budget, so the project writes its own. The rules come from
# docs/00-governance/PROJECT-STANDARDS.md:
#
#   * Rust source file: target <= 300 physical lines, hard limit > 500 lines
#   * A hard-limit breach must be recorded in docs/00-governance/EXCEPTIONS.md
#
# Exit codes: 0 = pass, 1 = fail, 77 = skipped (with a reason).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

SOFT_LIMIT=300
HARD_LIMIT=500

if ! command -v python3 >/dev/null 2>&1; then
    echo "SKIPPED (exit 77) - python3 is not available, so the size gate cannot run"
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

# Excluded from the gate: generated code, fixtures and non-Rust files.
# The list holds the tracked sources *and* the sources that are not tracked
# yet: a file that is new in the working tree is measured like every other,
# so that the gate cannot be passed by not committing yet.
mapfile -t FILES < <(git ls-files -co --exclude-standard 'crates/**/*.rs' 'crates/*.rs' 2>/dev/null | sort -u | grep -v -E '(^|/)(generated|fixtures?)/' || true)

if [ "${#FILES[@]}" -eq 0 ]; then
    echo "SKIPPED (exit 77) - no tracked Rust sources found"
    exit 77
fi

status=0
warned=0

for file in "${FILES[@]}"; do
    [ -f "$file" ] || continue
    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt "$HARD_LIMIT" ]; then
        echo "FAIL  $file: $lines lines (hard limit $HARD_LIMIT)."
        echo "      A file above the hard limit needs a recorded exception in"
        echo "      docs/00-governance/EXCEPTIONS.md, or it must be split."
        status=1
    elif [ "$lines" -gt "$SOFT_LIMIT" ]; then
        echo "WARN  $file: $lines lines (target $SOFT_LIMIT, hard limit $HARD_LIMIT)."
        warned=$((warned + 1))
    fi
done

if [ "$status" -ne 0 ]; then
    echo
    echo "RESULT: FAIL"
    exit 1
fi

echo "RESULT: PASS"
if [ "$warned" -gt 0 ]; then
    echo "        ($warned file(s) above the $SOFT_LIMIT-line target; split them when touched.)"
fi
exit 0
