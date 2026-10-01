#!/usr/bin/env python3
"""Copy-paste / duplicated-logic ratchet for Rust and Python sources.

Normalizes code (strips comments, whitespace, string literal contents and numbers),
hashes sliding windows of WINDOW lines, and reports blocks that occur in more than one
place. The total number of duplicated lines may not increase (quality/duplication.json).
This is the mechanical half of the "no redundant definitions" rule; the quality reviewer
agent covers semantic duplication (two helpers doing the same thing differently).

Usage: scripts/check_duplication.py [--update] [--report N]
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "quality" / "duplication.json"
WINDOW = 10
SKIP_PARTS = {"test", "tests", "upstream"}
STRING = re.compile(r'"(?:[^"\\]|\\.)*"')
NUMBER = re.compile(r"\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b")


def normalize(line: str, lang: str) -> str:
    comment = "//" if lang == "rs" else "#"
    line = STRING.sub('""', line)
    if comment in line:
        line = line.split(comment, 1)[0]
    line = NUMBER.sub("0", line)
    return re.sub(r"\s+", "", line)


def files() -> list[Path]:
    out = subprocess.run(["git", "ls-files", "*.rs", "*.py"], cwd=ROOT, capture_output=True, text=True, check=True)
    return [ROOT / p for p in out.stdout.split() if not SKIP_PARTS & set(Path(p).parts)]


def scan() -> tuple[int, list[tuple[int, list[tuple[str, int]]]]]:
    windows: dict[str, list[tuple[str, int]]] = defaultdict(list)
    for path in files():
        if not path.exists():
            continue
        lang = path.suffix[1:]
        lines = []
        for i, raw in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            norm = normalize(raw, lang)
            if len(norm) > 3:  # skip braces / trivial lines
                lines.append((i, norm))
        for k in range(len(lines) - WINDOW + 1):
            chunk = "\n".join(n for _, n in lines[k : k + WINDOW])
            digest = hashlib.blake2b(chunk.encode(), digest_size=12).hexdigest()
            windows[digest].append((str(path.relative_to(ROOT)), lines[k][0]))
    dup_lines: set[tuple[str, int]] = set()
    groups = []
    for occ in windows.values():
        if len(occ) > 1:
            for f, start in occ:
                dup_lines.update((f, start + j) for j in range(WINDOW))
            groups.append((len(occ), occ))
    groups.sort(key=lambda g: -g[0])
    return len(dup_lines), groups


def main() -> int:
    total, groups = scan()
    if "--report" in sys.argv:
        n = int(sys.argv[sys.argv.index("--report") + 1])
        for count, occ in groups[:n]:
            print(f"{count}x: " + ", ".join(f"{f}:{line}" for f, line in occ[:6]))
    if "--update" in sys.argv:
        LOCK.write_text(json.dumps({"duplicated_lines": total, "window": WINDOW}, indent=2) + "\n")
        print(f"updated {LOCK.relative_to(ROOT)}: {total}")
        return 0
    locked = json.loads(LOCK.read_text())["duplicated_lines"] if LOCK.exists() else 0
    print(f"duplicated lines: {total} (baseline {locked})")
    if total > locked:
        print("FAIL duplication increased. Extract a shared helper instead of copying code;"
              " run with --report 20 to see where.")
        return 1
    if total < locked:
        print("note duplication decreased — run with --update and commit quality/duplication.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
