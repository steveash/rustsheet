#!/usr/bin/env python3
"""The merge train (SPEC-0027 section 4): batched merges of ready PRs under the merge lock.

  scripts/merge_train.py mode    # the mode on origin/main: lock, shadow or train

The mode is read from `scripts/merge_train.toml` on a freshly fetched `origin/main`, never from
the checkout, so a branch cannot switch its own flow. The read fails closed: a missing file, a
TOML error (a duplicate key included), or a missing, non-string or unknown `mode` reads as `lock`.
"""

from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path

from merge_lock import GitError, git

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
SWITCH = "scripts/merge_train.toml"
LOCK, SHADOW, TRAIN = "lock", "shadow", "train"
MODES = (LOCK, SHADOW, TRAIN)
# The id checks of step 4; each takes `--root <tree>`. They run from `main`'s scripts (M0).
ID_CHECKS = (("tasks.py", "validate"), ("check_decisions.py",))
ID_CHECK_TIMEOUT = 300
BATCH_MAX = 5
# The committer of train merge commits, so a clone with no git identity can still build.
IDENTITY = ("-c", "user.name=merge-train", "-c", "user.email=merge-train@invalid")
# Ids that an id-check failure line names; only these, never its file names, reach a reason.
COLLIDING_ID = re.compile(r"\b(?:duplicate id (T-\d{4})|decision number (\d{4}))\b")


def parse_mode(text: str | None) -> str:
    """The mode a switch file's text sets; LOCK for anything that is not exactly a valid mode."""
    if text is None:
        return LOCK
    try:
        mode = tomllib.loads(text).get("mode")
    except tomllib.TOMLDecodeError:
        return LOCK
    return mode if isinstance(mode, str) and mode in MODES else LOCK


def read_mode(root: Path = ROOT) -> str:
    """The mode on a freshly fetched origin/main; LOCK when the fetch or the read fails."""
    try:
        git("fetch", "-q", "--no-tags", REMOTE, "+refs/heads/main:refs/remotes/origin/main", root=root)
        shown = git("show", f"refs/remotes/origin/main:{SWITCH}", root=root, check=False)
    except GitError:
        return LOCK
    return parse_mode(shown.stdout if shown.returncode == 0 else None)


def id_check_failures(scripts: Path, tree: Path) -> list[str]:
    """The FAIL lines of M0's id checks on the train worktree `tree` (SPEC-0027 section 4 step 4).

    `scripts` is M0's scripts directory, in a worktree outside `tree`. `python3 -I` with the
    working directory there imports no module from `tree`, so a PR cannot change how its own tree
    is judged (decision 0066). A check that cannot run counts as one failure.
    """
    failures = []
    for script, *args in ID_CHECKS:
        argv = [sys.executable, "-I", str(scripts / script), *args, "--root", str(tree)]
        try:
            run = subprocess.run(
                argv, cwd=scripts, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                timeout=ID_CHECK_TIMEOUT, check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            failures.append(f"{script}: {error.__class__.__name__}")
            continue
        lines = [line for line in run.stdout.splitlines() if line.startswith("FAIL")]
        if run.returncode != 0:
            failures += lines or [f"{script}: exit {run.returncode}"]
    return failures


@dataclass(frozen=True)
class Member:
    """PR `pr` at head `head`, merged into the train as merge commit `merge`."""

    pr: int
    head: str
    merge: str


@dataclass
class Train:
    """A built train on `base` (M0): its members in order, and the PRs left out with the reason."""

    base: str
    members: list[Member] = field(default_factory=list)
    deferred: list[tuple[int, str]] = field(default_factory=list)
    ejected: list[tuple[int, str]] = field(default_factory=list)

    @property
    def tip(self) -> str:
        return self.members[-1].merge if self.members else self.base


def independent(worktree: Path, base: str, head: str, members: Sequence[Member]) -> Member | None:
    """The first member whose head shares history with `head` beyond `base`, or None.

    Every merge base (`--all`, so criss-cross histories count) of a member's head and `head` must
    be an ancestor of `base`; otherwise squashing them one after the other would not give the
    train's trees (SPEC-0027 section 5)."""
    for member in members:
        bases = git("merge-base", "--all", member.head, head, root=worktree, check=False).stdout.split()
        for merge_base in bases:
            if git("merge-base", "--is-ancestor", merge_base, base, root=worktree, check=False).returncode != 0:
                return member
    return None


def id_collision(failures: Sequence[str]) -> str:
    ids = sorted({name for line in failures for pair in COLLIDING_ID.findall(line) for name in pair if name})
    named = ", ".join(ids) or "a task id or decision number"
    return f"id collision with {named}; renumber per decision 0121 (2)"


def build(worktree: Path, scripts: Path, base: str, heads: Sequence[tuple[int, str]]) -> Train:
    """Steps 3 and 4 of SPEC-0027 section 4 in the scratch worktree `worktree`, checked out at
    `base`: merge each (PR, head) in order, deferring dependent or intra-batch conflicting PRs
    and ejecting those that conflict with `base` or collide on an id. `scripts` is M0's scripts
    directory (outside `worktree`). The heads must already be fetched."""
    train = Train(base)
    git("checkout", "-q", "--detach", base, root=worktree)
    if id_check_failures(scripts, worktree):
        raise GitError("the id checks fail on main itself")
    for pr, head in heads:
        if len(train.members) == BATCH_MAX:
            break
        other = independent(worktree, base, head, train.members)
        if other is not None:
            train.deferred.append((pr, f"shares history with #{other.pr} beyond main"))
            continue
        merged = git(*IDENTITY, "merge", "-q", "--no-ff", "--no-edit", head, root=worktree, check=False)
        if merged.returncode != 0:
            git("merge", "--abort", root=worktree, check=False)
            git("checkout", "-q", "-f", "--detach", train.tip, root=worktree)
            alone = git("merge-tree", "--write-tree", base, head, root=worktree, check=False)
            if alone.returncode == 0:
                train.deferred.append((pr, "conflicts with an earlier PR of this train"))
            else:
                train.ejected.append((pr, "conflict with main"))
            continue
        failures = id_check_failures(scripts, worktree)
        if failures:
            # The tip passed the checks, so this PR adds the later copy.
            git("checkout", "-q", "-f", "--detach", train.tip, root=worktree)
            train.ejected.append((pr, id_collision(failures)))
            continue
        merge = git("rev-parse", "HEAD", root=worktree).stdout.strip()
        train.members.append(Member(pr, head, merge))
    return train


def main(argv: list[str], root: Path = ROOT) -> int:
    if argv != ["mode"]:
        print("usage: merge_train.py mode", file=sys.stderr)
        return 2
    print(read_mode(root))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
