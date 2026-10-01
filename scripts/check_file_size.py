#!/usr/bin/env python3
"""File-size ratchet. New Rust/Python source files must stay under the limit; files that
were already oversized (inherited from the fork) may not grow. Big files are where
duplicated helpers and tangled responsibilities hide.

Usage: scripts/check_file_size.py [--update]
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "quality" / "oversized-files.json"
LIMIT = 1000  # lines
EXEMPT_PARTS = {"test", "tests"}  # test files may be long tables of cases


def tracked_sources() -> list[Path]:
    out = subprocess.run(["git", "ls-files", "*.rs", "*.py"], cwd=ROOT, capture_output=True, text=True, check=True)
    return [ROOT / p for p in out.stdout.split()]


def sizes() -> dict[str, int]:
    result = {}
    for p in tracked_sources():
        rel = p.relative_to(ROOT)
        if EXEMPT_PARTS & set(rel.parts) or not p.exists():
            continue
        n = sum(1 for _ in p.open(encoding="utf-8", errors="replace"))
        if n > LIMIT:
            result[str(rel)] = n
    return result


def main() -> int:
    now = sizes()
    if "--update" in sys.argv:
        LOCK.write_text(json.dumps(dict(sorted(now.items())), indent=2) + "\n")
        print(f"updated {LOCK.relative_to(ROOT)}")
        return 0
    locked: dict[str, int] = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    failed = False
    for f, n in sorted(now.items()):
        if f not in locked:
            failed = True
            print(f"FAIL {f}: {n} lines > {LIMIT}. Split it into cohesive modules.")
        elif n > locked[f]:
            failed = True
            print(f"FAIL {f}: grew {locked[f]} -> {n} lines; oversized inherited files may only shrink.")
    for f in sorted(set(locked) - set(now)):
        print(f"note {f}: no longer oversized — run with --update and commit")
    print(f"oversized files: {len(now)} (limit {LIMIT} lines)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
