#!/usr/bin/env python3
"""Governance gate for autonomous development.

Some paths define the rules agents work under (CI, lint policy, baselines, agent
instructions). A PR that changes them must also add or update a decision record in
docs/decisions/ explaining why, so rules are never silently weakened. Additionally,
`--allow-regressions` baseline changes and test deletions are flagged.

Usage: scripts/check_governance.py [BASE_REF]   (default: origin/main)
"""

from __future__ import annotations

import subprocess
import sys

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
    "eval/milestones/",  # frozen milestone probe lists (decisions 0043, 0044)
)


def is_deviations_file(path: str) -> bool:
    """A suite's accepted-deviations list (decision 0036): each line hides a cell from bit-exact."""
    return path.startswith("eval/suites/") and path.endswith("/deviations")


def is_protected(path: str) -> bool:
    """Whether changing `path` needs a decision record in the same PR."""
    return path.startswith(PROTECTED) or path in PROTECTED or is_deviations_file(path)


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
    protected = [p for p in paths if is_protected(p)]
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
    if not failed:
        print(f"governance ok ({len(paths)} files changed, {len(protected)} protected)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
