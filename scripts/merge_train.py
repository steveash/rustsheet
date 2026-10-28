#!/usr/bin/env python3
"""The merge train (SPEC-0027 section 4): batched merges of ready PRs under the merge lock.

  scripts/merge_train.py mode        # the mode on origin/main: lock, shadow or train
  scripts/merge_train.py --dry-run   # steps 2-4: collect, build, id checks; prints, writes nothing

The mode is read from `scripts/merge_train.toml` on a freshly fetched `origin/main`, never from
the checkout, so a branch cannot switch its own flow. The read fails closed: a missing file, a
TOML error (a duplicate key included), or a missing, non-string or unknown `mode` reads as `lock`.
"""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
import tomllib
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path

import gh_client
import merge_ready
from gh_client import Client, GhError
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


@dataclass
class Collected:
    """Step 2: the ready PRs oldest marker first, and what the merger would do about the rest."""

    ready: list[merge_ready.Verdict] = field(default_factory=list)
    unlabel: list[tuple[int, str]] = field(default_factory=list)
    keep: list[tuple[int, str]] = field(default_factory=list)


def collect(client: Client, repo: merge_ready.Repo) -> Collected:
    """Runs V1-V6 on every open PR labelled ready-to-merge (SPEC-0027 section 4, step 2)."""
    found = Collected()
    for pull in client.pulls():
        labels = merge_ready.field_of(pull, "labels")
        names = [merge_ready.text_of(label, "name") for label in labels] if isinstance(labels, list) else []
        if merge_ready.READY_LABEL not in names:
            continue
        pr = gh_client.number(merge_ready.int_of(pull, "number"))
        verdict = merge_ready.verify(client, pr, repo)
        actions = {f.action for f in verdict.failures}
        reasons = "; ".join(f.reason for f in verdict.failures)
        if merge_ready.UNLABEL in actions:
            found.unlabel.append((pr, reasons))
        elif merge_ready.KEEP in actions:
            found.keep.append((pr, reasons))
        elif verdict.ready:
            if merge_ready.text_of(client.pull(pr), "mergeable_state") == "dirty":
                found.unlabel.append((pr, "conflict with main"))
            else:
                found.ready.append(verdict)
    found.ready.sort(key=lambda v: (v.updated_at, v.comment_id or 0))
    return found


def dry_run(client: Client, root: Path = ROOT) -> str:
    """Steps 2-4 against a fresh origin/main in scratch worktrees; returns the report. No writes:
    no push, label, comment or lock (the write policy of SPEC-0027 section 4)."""
    repo = merge_ready.Repo(root)
    repo.fetch_base()
    base = git("rev-parse", f"refs/merge-ready/{merge_ready.BASE_BRANCH}", root=root).stdout.strip()
    found = collect(client, repo)
    heads = [(v.pr, v.marker.head) for v in found.ready[:BATCH_MAX] if v.marker is not None]
    with tempfile.TemporaryDirectory(prefix="merge-train-") as scratch:
        train_tree, scripts_tree = Path(scratch) / "train", Path(scratch) / "m0"
        try:
            for tree in (train_tree, scripts_tree):
                git("worktree", "add", "-q", "--detach", str(tree), base, root=root)
            train = build(train_tree, scripts_tree / "scripts", base, heads)
        finally:
            for tree in (train_tree, scripts_tree):
                git("worktree", "remove", "--force", str(tree), root=root, check=False)
    lines = [f"dry run on main {base}: {len(heads)} candidate(s); nothing was written"]
    lines.append("batch: " + (", ".join(f"#{m.pr} ({m.head[:12]})" for m in train.members) or "none"))
    lines += [f"deferred #{pr}: {reason}" for pr, reason in train.deferred]
    lines += [f"would eject #{pr}: {reason}" for pr, reason in train.ejected]
    lines += [f"would unlabel #{pr}: {reason}" for pr, reason in found.unlabel]
    lines += [f"skipped, label kept #{pr}: {reason}" for pr, reason in found.keep]
    lines += [f"waiting for a later batch #{v.pr}" for v in found.ready[BATCH_MAX:]]
    return "\n".join(lines)


def main(argv: list[str], root: Path = ROOT) -> int:
    if argv == ["mode"]:
        print(read_mode(root))
        return 0
    if argv != ["--dry-run"]:
        print("usage: merge_train.py mode | --dry-run", file=sys.stderr)
        return 2
    client = gh_client.from_remote(root, REMOTE)
    if client is None:
        print("merge_train: origin is not a github.com repository", file=sys.stderr)
        return 2
    try:
        print(dry_run(client, root))
    except (GhError, GitError) as error:
        print(f"merge_train: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
