#!/usr/bin/env python3
"""Decision-record numbering gate (decision 0023).

Every record in docs/decisions/ (except README.md) must be named `NNNN-slug.md`, and no two may
share a number: parallel PRs that both take "the next number" otherwise land a clash (0021, T-0032
and T-0033, 2026-10-05). Needs no git or base ref, so it runs in `make hygiene` and on main.
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


def main() -> int:
    names = [p.name for p in DECISIONS.glob("*.md")]
    if len(names) <= 1:
        print(f"FAIL decisions: no records found in {DECISIONS}")
        return 1
    if found := problems(names):
        for line in found:
            print(f"FAIL decisions: {line}")
        return 1
    print(f"decisions ok ({len(names) - 1} records)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
