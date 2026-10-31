"""The visible-parity lock as the merge tooling reads it (decision 0217).

`eval/baseline/visible-parity.txt` holds one `case<TAB>count` line per case; `rustsheet-eval`
writes it and CI's eval gate checks it. A carry-over merge (`merge_ready.Repo.carry_problem`) may
not raise a case above its parents' lines, and `make catch-up` (`merge_catchup.py`) resolves a
conflict on it with `merge_visible`.
"""

from __future__ import annotations

from collections.abc import Sequence

VISIBLE_PARITY = "eval/baseline/visible-parity.txt"


def visible_counts(text: str) -> dict[str, int]:
    """The `case<TAB>count` lines of a visible-parity lock (decision 0217) as case -> count; comment,
    blank and malformed lines are left out, and a repeated case keeps its first count."""
    counts: dict[str, int] = {}
    for line in text.splitlines():
        case, sep, count = line.partition("\t")
        if sep and case and not case.startswith("#") and count.isdigit():
            counts.setdefault(case, int(count))
    return counts


def visible_rises(head: str, parents: Sequence[str]) -> list[str]:
    """The lines of visible-parity lock `head` that raise a case above every parent's line for it,
    that no parent has (a new case's line), or that are malformed or repeat a case. A carry-over
    merge adds no case its parents did not have, so each of these is a regression (decision 0217)."""
    known = [visible_counts(p) for p in parents]
    seen: set[str] = set()
    rises = []
    for line in head.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        case, sep, count = line.partition("\t")
        allowed = [k[case] for k in known if case in k]
        if not sep or not count.isdigit() or case in seen or not allowed or int(count) > max(allowed):
            rises.append(line)
        seen.add(case)
    return sorted(rises)


def merge_visible(ours: str, theirs: str) -> str:
    """Resolves a conflict in the visible-parity lock: `theirs`' header, then each case of either
    side at the higher of its two counts, sorted as `rustsheet-eval` writes it. No line exceeds
    what a parent allowed, so `visible_rises` accepts it; the re-record and the eval gate then hold
    every case to it (decision 0217)."""
    merged = visible_counts(ours)
    for case, count in visible_counts(theirs).items():
        merged[case] = max(count, merged.get(case, 0))
    header = [line for line in theirs.splitlines() if line.startswith("#")]
    body = [f"{case}\t{n}" for case, n in sorted(merged.items()) if n > 0]
    return "".join(f"{line}\n" for line in [*header, *body])
