#!/usr/bin/env python3
"""Decision-record numbering gate (decision 0023).

Every record in docs/decisions/ (except README.md) must be named `NNNN-slug.md`, and no two may
share a number: parallel PRs that both take "the next number" otherwise land a clash (0021, T-0032
and T-0033, 2026-10-05). Needs no git or base ref, so it runs in `make hygiene` and on main.

  scripts/check_decisions.py [--root <dir>]   # <dir>: the tree to judge (default: this checkout)
"""

from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path

DECISIONS = Path(__file__).resolve().parent.parent / "docs" / "decisions"
NAME = re.compile(r"^(\d{4})-[a-z0-9-]+\.md$")


def problems(names: list[str]) -> list[str]:
    """Badly named records and numbers used more than once, as human-readable lines."""
    records = [n for n in names if n != "README.md"]
    out = [f"bad record name (want NNNN-slug.md): {n}" for n in sorted(records) if not NAME.match(n)]
    numbers = Counter(m.group(1) for n in records if (m := NAME.match(n)))
    for num in sorted(k for k, c in numbers.items() if c > 1):
        clash = ", ".join(sorted(n for n in records if n.startswith(f"{num}-")))
        out.append(f"decision number {num} used twice (renumber the newer record and its references): {clash}")
    return out


# Placeholder that every `make new-decision` template field starts with (scripts/new_decision.py);
# the colon keeps prose that names the placeholder, as decision 0016 does, from matching.
UNFILLED = "TODO(fill):"


def unfilled(texts: dict[str, str]) -> list[str]:
    """Records that still hold a template placeholder, as human-readable lines."""
    return [f"{n} still has `{UNFILLED}` lines from `make new-decision`" for n, t in sorted(texts.items()) if UNFILLED in t]


def main(argv: list[str]) -> int:
    # `--root <dir>` judges another tree: the merge train runs `main`'s copy of this script
    # against a train worktree (SPEC-0027 section 4 step 4, decision 0066).
    decisions = DECISIONS
    if argv:
        if len(argv) != 2 or argv[0] != "--root" or not Path(argv[1]).is_dir():
            print("usage: check_decisions.py [--root <dir>]", file=sys.stderr)
            return 2
        decisions = Path(argv[1]).resolve() / "docs" / "decisions"
    names = [p.name for p in decisions.glob("*.md")]
    if len(names) <= 1:
        print(f"FAIL decisions: no records found in {decisions}")
        return 1
    texts = {p.name: p.read_text() for p in decisions.glob("*.md") if p.name != "README.md"}
    if found := problems(names) + unfilled(texts):
        for line in found:
            print(f"FAIL decisions: {line}")
        return 1
    print(f"decisions ok ({len(names) - 1} records)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
