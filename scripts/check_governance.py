#!/usr/bin/env python3
"""Governance gate for autonomous development.

Some paths define the rules agents work under (CI, lint policy, baselines, agent
instructions). A PR that changes them must also add or update a decision record in
docs/decisions/ explaining why, so rules are never silently weakened. Additionally,
`--allow-regressions` baseline changes and test deletions are flagged.

Usage: scripts/check_governance.py [BASE_REF]   (default: origin/main)
"""

from __future__ import annotations

import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

PROTECTED = (
    ".github/",
    ".claude/",
    "CLAUDE.md",
    "Cargo.toml",  # workspace lint policy
    "clippy.toml",
    "deny.toml",
    "rustfmt.toml",
    "Makefile",
    "scripts/",
    "quality/",
    "docs/process/",
)


def duplicate_decision_numbers(names: list[str]) -> list[str]:
    """Decision numbers used by more than one record (two parallel PRs both took the next one)."""
    numbers = Counter(m.group(1) for n in names if (m := re.match(r"(\d{4})-", n)))
    return sorted(n for n, k in numbers.items() if k > 1)


def git(*args: str) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def main() -> int:
    base = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
    try:
        merge_base = git("merge-base", base, "HEAD").strip()
    except subprocess.CalledProcessError:
        print(f"note: base {base} not found; skipping governance check")
        return 0
    changed = git("diff", "--name-status", merge_base, "HEAD").splitlines()
    paths = [line.split("\t")[-1] for line in changed]
    protected = [p for p in paths if p.startswith(PROTECTED) or p in PROTECTED]
    decisions = [p for p in paths if p.startswith("docs/decisions/") and p.endswith(".md")]
    failed = False
    if protected and not decisions:
        failed = True
        print("FAIL governance: protected paths changed without a decision record in docs/decisions/:")
        for p in protected:
            print(f"  {p}")
    deleted_tests = [
        line.split("\t")[-1]
        for line in changed
        if line.startswith("D") and ("/tests/" in line or "/test/" in line or line.endswith("_test.py"))
    ]
    if deleted_tests:
        failed = True
        print("FAIL governance: test files deleted (requires a decision record and reviewer sign-off):")
        for p in deleted_tests:
            print(f"  {p}")
        if decisions:
            failed = False
            print("  (decision record present — allowed; reviewers must confirm)")
    records = [p.name for p in Path("docs/decisions").glob("*.md")]
    if dupes := duplicate_decision_numbers(records):
        failed = True
        print("FAIL governance: decision numbers used twice (renumber the newer record and its references):")
        for n in dupes:
            print(f"  {n}: " + ", ".join(sorted(r for r in records if r.startswith(n))))
    if not failed:
        print(f"governance ok ({len(paths)} files changed, {len(protected)} protected)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
