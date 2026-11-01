"""The visible-parity lock as the merge tooling reads it (decision 0219).

`eval/baseline/visible-parity.txt` holds one `case<TAB>count` line per case; `rustsheet-eval`
writes it and CI's eval gate checks it. `make catch-up` (`merge_catchup.py`) resolves a
conflict on it three-way per case (`merge_visible`) and re-records only the PR's added case files
as new (`added_case_files`). Neither it nor a carry-over merge (`merge_ready.Repo.carry_problem`)
may record a line above `main`'s for a case whose case file `main` has (`visible_rises`). The lock
replaced `eval/baseline/visible-parity.json` (`RETIRED_LOCK`), which PRs from before it still
modify; a merge with `main` resolves that by deleting it.
"""

from __future__ import annotations

import posixpath
import re
from collections.abc import Callable
from pathlib import Path

from merge_lock import git
from suite_meta import suites_on

VISIBLE_PARITY = "eval/baseline/visible-parity.txt"
# The single-number lock it replaced (decisions 0088, 0219).
RETIRED_LOCK = "eval/baseline/visible-parity.json"
# ASCII digits only, as `rustsheet-eval` reads them: `str.isdigit` would take "²", and no sign.
COUNT = re.compile(r"[0-9]+")


def is_entry(line: str) -> bool:
    """Whether `line` is a lock line rather than a blank line or a `#` comment."""
    return bool(line.strip()) and not line.startswith("#")


def parse_line(line: str) -> tuple[str, int] | None:
    """(case, count) of lock line `line`, or None when it is not `case<TAB>count`."""
    case, sep, count = line.partition("\t")
    return (case, int(count)) if sep and case and COUNT.fullmatch(count) else None


def visible_counts(text: str) -> dict[str, int]:
    """The lock lines of `text` as case -> count; blank, comment and malformed lines are left out,
    and a repeated case keeps its first count. A case without a line counts 0."""
    counts: dict[str, int] = {}
    for line in text.splitlines():
        parsed = parse_line(line) if is_entry(line) else None
        if parsed:
            counts.setdefault(*parsed)
    return counts


def three_way(base: int, pr: int, main: int) -> int:
    """One case's count merged from the merge base's, the PR's and `main`'s (0 for no line): the
    side that changed it wins, and when both did, the lower. So a lowering on `main` (the steward's
    daily run) survives a PR that did not touch the case, and a line `main` dropped stays dropped."""
    if pr == base:
        return main
    if main == base:
        return pr
    return min(pr, main)


def merged_counts(base: str, pr: str, main: str) -> dict[str, int]:
    """`three_way` of every case of the three locks, cases at 0 left out."""
    b, p, m = visible_counts(base), visible_counts(pr), visible_counts(main)
    merged = {c: three_way(b.get(c, 0), p.get(c, 0), m.get(c, 0)) for c in {*b, *p, *m}}
    return {c: n for c, n in merged.items() if n > 0}


def merge_visible(base: str, pr: str, main: str) -> str:
    """Resolves a conflict in the lock (decision 0219): `main`'s header, then each case at its
    `three_way` count, sorted as `rustsheet-eval` writes it. The re-record and the eval gate then
    hold every case to it, and `visible_rises` accepts it."""
    header = [line for line in main.splitlines() if line.startswith("#")]
    body = [f"{case}\t{n}" for case, n in sorted(merged_counts(base, pr, main).items())]
    return "".join(f"{line}\n" for line in [*header, *body])


def visible_rises(head: str, main: str, on_main: Callable[[str], bool]) -> list[str]:
    """The lines of lock `head`, a merge of a PR and `main` (whose lock is `main`), that the merge
    tooling may not record without the reviewers (decision 0219): malformed or repeated lines, and,
    for a case whose case file `main` has (`on_main`), a line above `main`'s line for it (0 for
    none). So a merge never undoes a lowering on `main`, and a PR's sanctioned raise of one of
    `main`'s cases (`--allow-regressions`) is merged by hand and reviewed again. A case the PR adds
    may have any line: `main` has none for it to hide a regression behind, and the catch-up
    re-records it as new."""
    allowed = visible_counts(main)
    seen: set[str] = set()
    rises = []
    for line in filter(is_entry, head.splitlines()):
        parsed = parse_line(line)
        if parsed is None or parsed[0] in seen:
            rises.append(line)
        elif parsed[1] > allowed.get(parsed[0], 0) and on_main(parsed[0]):
            rises.append(line)
        seen.add(parsed[0] if parsed else line)
    return sorted(rises)


def in_commit(root: Path, commit: str, path: str) -> bool:
    """Whether `commit` has a file at `path`."""
    return git("cat-file", "-e", f"{commit}:{path}", root=root, check=False).returncode == 0


def case_on(root: Path, commit: str) -> Callable[[str], bool]:
    """Whether case id `<suite>/<case>`'s case file is in `commit`. A case of a suite `commit`
    does not have is not."""
    roots = {s.name: s.root for s in suites_on(root, commit) if s.root is not None}

    def on(case: str) -> bool:
        suite, _, rel = case.partition("/")
        if suite not in roots or not rel:
            return False
        return in_commit(root, commit, posixpath.normpath(f"{roots[suite]}/{rel}"))

    return on
def added_case_files(root: Path, base: str, head: str) -> list[str]:
    """The files `head` adds since its merge base with `base` (`git diff --diff-filter=A
    --name-only base...head`), for `update-baseline --new-cases`: it keeps the case files of the
    gated suites among them and ignores the rest (decision 0219)."""
    out = git("diff", "-z", "--no-renames", "--diff-filter=A", "--name-only", f"{base}...{head}", root=root)
    return [p for p in out.stdout.split("\0") if p]
