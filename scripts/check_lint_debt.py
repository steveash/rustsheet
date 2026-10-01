#!/usr/bin/env python3
"""Lint-debt ratchet: the LINT-DEBT allow-lists in forked crates may only shrink.

Usage: scripts/check_lint_debt.py [--update]
Reads every Cargo.toml with a `# LINT-DEBT-BEGIN` block and compares its entries with
quality/lint-debt.json. New entries fail the check. Removed entries are reported; run
with --update to lock the smaller set in (commit the JSON).
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "quality" / "lint-debt.json"
BLOCK = re.compile(r"# LINT-DEBT-BEGIN.*?\n(.*?)# LINT-DEBT-END", re.S)


def current() -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for manifest in sorted(ROOT.glob("**/Cargo.toml")):
        if "target" in manifest.parts:
            continue
        text = manifest.read_text()
        entries: list[str] = []
        for section, block in zip(re.findall(r"\[lints\.(\w+)\]", text), BLOCK.findall(text), strict=False):
            for line in block.splitlines():
                key = line.split("=")[0].strip()
                if key and not key.startswith("#"):
                    entries.append(f"{section}::{key}")
        if entries:
            out[str(manifest.relative_to(ROOT))] = sorted(entries)
    return out


def main() -> int:
    now = current()
    if "--update" in sys.argv:
        LOCK.write_text(json.dumps(now, indent=2) + "\n")
        print(f"updated {LOCK.relative_to(ROOT)}")
        return 0
    locked: dict[str, list[str]] = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    failed = False
    for manifest, entries in now.items():
        allowed = set(locked.get(manifest, []))
        added = sorted(set(entries) - allowed)
        removed = sorted(allowed - set(entries))
        if added:
            failed = True
            print(f"FAIL {manifest}: new lint-debt allows are forbidden: {', '.join(added)}")
        if removed:
            print(f"note {manifest}: paid down {', '.join(removed)} — run with --update and commit")
    total = sum(len(v) for v in now.values())
    print(f"lint debt: {total} allow entries across {len(now)} crates")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
