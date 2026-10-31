#!/usr/bin/env python3
"""Fitted-constant ratchet (decision 0188 R4.3).

Every empirically fitted constant, cut-point or table in the Rust sources carries one doc-comment
line `FITTED(T-NNNN, <n> cells, <range>)`: the task that fitted it, how many Excel cells it was
measured on and the input range those cells cover (`range unknown` when the evidence does not say).
This check rejects a malformed tag or one citing a task that has no file, and compares the count
per source file with quality/fitted.json: a count may only fall. A new fit raises it, which needs
`--update` in a PR that cites the fit task and its hold-out result (decision 0188 R4).

Usage: scripts/check_fitted.py [--update]
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "quality" / "fitted.json"
MARK = "FITTED("
TAG = re.compile(r"^\s*//[/!]?\s*FITTED\(T-(\d{4,}), (\d+) cells?, (\S(?:.*\S)?)\)$")


def task_ids() -> set[str]:
    return {p.name.split("-")[1] for p in (ROOT / "docs" / "tasks").glob("*/T-*.md")}


def scan(files: dict[str, str], tasks: set[str]) -> tuple[dict[str, int], list[str]]:
    """Per-file tag counts and the problems (malformed tags, unknown tasks) in `files` (path -> text)."""
    counts: dict[str, int] = {}
    problems: list[str] = []
    for path, text in sorted(files.items()):
        for n, line in enumerate(text.splitlines(), 1):
            if MARK not in line:
                continue
            m = TAG.match(line)
            if m is None:
                problems.append(f"{path}:{n}: malformed tag, want `/// FITTED(T-NNNN, <n> cells, <range>)`")
            elif m.group(1) not in tasks:
                problems.append(f"{path}:{n}: FITTED cites T-{m.group(1)}, which has no task file")
            else:
                counts[path] = counts.get(path, 0) + 1
    return counts, problems


def compare(now: dict[str, int], locked: dict[str, int]) -> tuple[list[str], list[str]]:
    """Failures (a file whose count rose) and notes (a file whose count fell) against the lock."""
    fails = [
        f"{f}: {locked.get(f, 0)} -> {n} FITTED tags; a new fit needs --update in a PR citing its hold-out result"
        for f, n in sorted(now.items())
        if n > locked.get(f, 0)
    ]
    notes = [
        f"{f}: {n} -> {now.get(f, 0)} FITTED tags — run with --update and commit"
        for f, n in sorted(locked.items())
        if now.get(f, 0) < n
    ]
    return fails, notes


def sources() -> dict[str, str]:
    out = subprocess.run(["git", "ls-files", "*.rs"], cwd=ROOT, capture_output=True, text=True, check=True)
    return {
        p: (ROOT / p).read_text(encoding="utf-8", errors="replace")
        for p in out.stdout.split()
        if (ROOT / p).exists()
    }


def main(argv: list[str]) -> int:
    now, problems = scan(sources(), task_ids())
    for p in problems:
        print(f"FAIL {p}")
    if "--update" in argv:
        if problems:
            return 1
        LOCK.write_text(json.dumps(now, indent=2, sort_keys=True) + "\n")
        print(f"updated {LOCK.relative_to(ROOT)}")
        return 0
    locked: dict[str, int] = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    fails, notes = compare(now, locked)
    for f in fails:
        print(f"FAIL {f}")
    for n in notes:
        print(f"note {n}")
    print(f"fitted constants: {sum(now.values())} FITTED tags (lock {sum(locked.values())})")
    return 1 if problems or fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
