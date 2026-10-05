#!/usr/bin/env python3
"""Writes a new decision record with a collision-free number (decision 0029).

  scripts/new_decision.py --title "..." [--no-fetch]   # docs/decisions/NNNN-<slug>.md

NNNN is one above the highest number on the working tree and on every `origin/*` branch (read
after a fetch), so a record pushed by another session is never reused. Push the record with the
first commit on the branch so other sessions see the number. Allocation only makes clashes rare:
`check_decisions.py` and the merge-time up-to-date check (decision 0028) still catch the rest.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_decisions  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402

# check_decisions.py fails a record that still holds this placeholder.

TEMPLATE = """# {number} — {title}

- **Status:** accepted (TODO(fill): date, task id)
- **Context:** TODO(fill): what forced the decision; evidence.
- **Decision:** TODO(fill): the rule, in one or two sentences.
- **Consequences:** TODO(fill): what changes for agents, gates and later work.
"""


def numbers_in_tree(directory: Path) -> set[str]:
    """Record numbers in the working tree's `directory`."""
    return {m.group(1) for p in directory.glob("*.md") if (m := check_decisions.NAME.match(p.name))}


def next_number(taken: set[str]) -> str:
    """One above the highest taken number; gaps are never reused."""
    return f"{max((int(n) for n in taken), default=0) + 1:04d}"


def new_decision(title: str, fetch: bool = True) -> int:
    if not title or "\n" in title:
        print('usage: new_decision.py --title "<one-line title>" [--no-fetch]')
        return 2
    directory = check_decisions.DECISIONS
    taken = numbers_in_tree(directory) | tasks.git_branch_names(
        str(directory.relative_to(tasks.ROOT)), check_decisions.NAME, fetch
    )
    number = next_number(taken)
    path = directory / f"{number}-{tasks.slug(title, 'decision')}.md"
    path.write_text(TEMPLATE.format(number=number, title=title))
    print(
        f"wrote {path.relative_to(tasks.ROOT)}: fill in the TODO(fill) lines, then commit and push it now to claim {number}"
    )
    return 0


def main() -> int:
    return new_decision(tasks.arg("--title") or "", fetch="--no-fetch" not in sys.argv)


if __name__ == "__main__":
    sys.exit(main())
