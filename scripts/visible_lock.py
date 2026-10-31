"""The visible-parity lock as the merge tooling reads it (decision 0217).

`eval/baseline/visible-parity.txt` holds one `case<TAB>count` line per case; `rustsheet-eval`
writes it and CI's eval gate checks it. A carry-over merge (`merge_ready.Repo.carry_problem`) may
add or raise a line only for a case whose case file `main` does not have, and `make catch-up`
(`merge_catchup.py`) resolves a conflict on it with `merge_visible` and re-records only the PR's
added case files as new (`added_case_files`).
"""

from __future__ import annotations

import posixpath
import tomllib
from collections.abc import Callable, Sequence
from pathlib import Path

from merge_lock import git

VISIBLE_PARITY = "eval/baseline/visible-parity.txt"
SUITES_DIR = "eval/suites"


def visible_counts(text: str) -> dict[str, int]:
    """The `case<TAB>count` lines of a visible-parity lock (decision 0217) as case -> count; comment,
    blank and malformed lines are left out, and a repeated case keeps its first count."""
    counts: dict[str, int] = {}
    for line in text.splitlines():
        case, sep, count = line.partition("\t")
        if sep and case and not case.startswith("#") and count.isdigit():
            counts.setdefault(case, int(count))
    return counts


def visible_rises(head: str, parents: Sequence[str], on_main: Callable[[str], bool]) -> list[str]:
    """The lines of visible-parity lock `head` that a carry-over merge may not record (decision 0217):
    malformed or repeated lines, and, for a case whose case file is on `main` (`on_main`), a line
    above every parent's line for that case or a line no parent has. A merge re-recorded with the
    PR's added case files as new may write any line for those cases, since `main` has no line for
    them to hide a regression behind; every other case's line must equal a parent's or be lower."""
    known = [visible_counts(p) for p in parents]
    seen: set[str] = set()
    rises = []
    for line in head.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        case, sep, count = line.partition("\t")
        allowed = [k[case] for k in known if case in k]
        if not sep or not case or not count.isdigit() or case in seen:
            rises.append(line)
        elif not (allowed and int(count) <= max(allowed)) and on_main(case):
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


def suite_roots(root: Path, commit: str) -> dict[str, str]:
    """Suite name -> its case root's repository path, from each `eval/suites/*/suite.toml` in
    `commit` (`root` defaults to `cases`, as in `rustsheet-eval`)."""
    roots: dict[str, str] = {}
    listed = git("ls-tree", "--name-only", f"{commit}:{SUITES_DIR}", root=root, check=False)
    for suite in listed.stdout.split() if listed.returncode == 0 else []:
        shown = git("show", f"{commit}:{SUITES_DIR}/{suite}/suite.toml", root=root, check=False)
        if shown.returncode != 0:
            continue
        try:
            meta = tomllib.loads(shown.stdout)
        except tomllib.TOMLDecodeError:
            continue
        name, base = meta.get("name", suite), meta.get("root", "cases")
        if isinstance(name, str) and isinstance(base, str):
            roots[name] = posixpath.normpath(f"{SUITES_DIR}/{suite}/{base}")
    return roots


def case_on(root: Path, commit: str) -> Callable[[str], bool]:
    """Whether case id `<suite>/<case>`'s case file is in `commit`. A case of a suite `commit`
    does not have is not."""
    roots = suite_roots(root, commit)

    def on(case: str) -> bool:
        suite, _, rel = case.partition("/")
        if suite not in roots or not rel:
            return False
        path = posixpath.normpath(f"{roots[suite]}/{rel}")
        return git("cat-file", "-e", f"{commit}:{path}", root=root, check=False).returncode == 0

    return on


def added_case_files(root: Path, base: str, head: str) -> list[str]:
    """The files `head` adds since its merge base with `base` (`git diff --diff-filter=A
    --name-only base...head`), for `update-baseline --new-cases`: it keeps the case files of the
    gated suites among them and ignores the rest (decision 0217)."""
    out = git("diff", "-z", "--no-renames", "--diff-filter=A", "--name-only", f"{base}...{head}", root=root)
    return [p for p in out.stdout.split("\0") if p]
