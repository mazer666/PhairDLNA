#!/usr/bin/env bash
# Gate QG-03: the core must not contain `unsafe` code.
#
# Why: the project forbids `unsafe` by default (docs/00-governance/PROJECT-STANDARDS.md
# section 2.6). Any use requires an ADR with a written safety argument, so the gate
# looks for the keyword and points at the ADR process rather than deciding.
#
# The scan strips line and block comments before looking for the keyword, so a
# mention of `unsafe` in a doc comment does not trip the gate. It is a lexical
# approximation, not a parser: a string literal containing the word "unsafe"
# would be a false positive. That is acceptable for a gate whose failure mode is
# "a human looks at the file".
#
# Exit codes: 0 = pass, 1 = fail, 77 = skipped (with a reason).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

if ! command -v python3 >/dev/null 2>&1; then
    echo "SKIPPED (exit 77) - python3 is not available, so the unsafe gate cannot run"
    exit 77
fi

mapfile -t FILES < <(git ls-files 'crates/**/*.rs' 'crates/*.rs' 2>/dev/null | grep -v -E '(^|/)(generated|fixtures?)/' || true)

if [ "${#FILES[@]}" -eq 0 ]; then
    echo "SKIPPED (exit 77) - no tracked Rust sources found"
    exit 77
fi

python3 - "${FILES[@]}" <<'PYTHON'
import re
import sys
from pathlib import Path

BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
LINE_COMMENT = re.compile(r"//[^\n]*")
UNSAFE = re.compile(r"\bunsafe\b")

findings = []
for name in sys.argv[1:]:
    path = Path(name)
    if not path.exists():
        continue
    text = path.read_text(encoding="utf-8", errors="replace")
    # Keep the line structure so that a finding can name a line number.
    text = LINE_COMMENT.sub("", BLOCK_COMMENT.sub("", text))
    for number, line in enumerate(text.splitlines(), 1):
        if UNSAFE.search(line):
            findings.append(f"{name}:{number}: {line.strip()[:100]}")

if findings:
    print("FAIL - unsafe code found:")
    for item in findings:
        print(f"  {item}")
    print()
    print("Any unsafe block needs an ADR with a written safety argument, and an entry")
    print("in docs/00-governance/EXCEPTIONS.md, before it is merged.")
    print()
    print("RESULT: FAIL")
    sys.exit(1)

print(f"RESULT: PASS (no unsafe code in {len(sys.argv) - 1} tracked Rust source file(s))")
PYTHON
exit $?
