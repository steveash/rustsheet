#!/usr/bin/env python3
"""Catch up a PR the merge train ejected for a conflict with main, when the conflict is only in
the eval-gated known-failure lists (decision 0205).

  scripts/merge_catchup.py --pr N   # exit 0: pushed; 1: refused (nothing pushed); 2: error

In this checkout, which must have no local changes, it does the following:
- merges `main` into the PR's branch without committing;
- refuses any conflict outside the lists CI's eval gate checks (`Repo.checked_baselines`: the
  gated suites the eval job runs without a fetch);
- takes `main`'s side of each conflicted list and runs `make eval-baseline`, which re-records the
  lists and refuses regressions (decision 0093);
- refuses if the re-record changed any other file, such as quality/evidence-floor.json;
- commits everything as one merge commit and pushes it to the PR branch (never forced);
- returns to the checkout's original branch or commit.

When CI is green on the pushed head, `make mark-ready PR=N CARRY=1` re-marks the PR: the carry rule
(`merge_ready.Repo.carry_problem`) accepts exactly this shape, a merge of `main` whose only
difference from the plain merge is the content of those lists. The re-record builds and runs the
PR's merged tree with this session's credentials; forks are refused, so that tree is an agent
branch of this repository.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from collections.abc import Callable, Sequence
from pathlib import Path

import gh_client
import merge_ready
from merge_lock import GitError, git

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
RERECORD = ("make", "-s", "eval-baseline")
# The PR head is fetched here, not into a remote-tracking ref a single-branch clone may lack.
HEAD_REF = "refs/merge-catchup/head"


class Refused(Exception):
    """The PR cannot be caught up here; nothing was pushed."""


def checked_only(paths: Sequence[str], lists: set[str]) -> bool:
    """Whether every one of the conflicted `paths` is one of the eval-gated `lists` (and there is one)."""
    return bool(paths) and all(p in lists for p in paths)


def rerecord(root: Path) -> int:
    """`make eval-baseline` in `root`: re-records eval/baseline/ and refuses regressions."""
    return subprocess.run(RERECORD, cwd=root, stdin=subprocess.DEVNULL, check=False).returncode


def catch_up(root: Path, branch: str, record: Callable[[Path], int] = rerecord) -> str:
    """Merges `main` into `branch` as the module docstring says, as one merge commit that the carry
    rule accepts, and pushes it; returns the pushed head. Raises Refused when it cannot (nothing
    pushed). The checkout's branch, or detached HEAD, is restored either way."""
    branch = gh_client.branch_name(branch)
    if git("status", "--porcelain", "--untracked-files=no", root=root).stdout.strip():
        raise Refused("the checkout has local changes")
    on = git("symbolic-ref", "-q", "--short", "HEAD", root=root, check=False).stdout.strip()
    start = on or git("rev-parse", "HEAD", root=root).stdout.strip()
    repo = merge_ready.Repo(root)
    repo.fetch_base()
    main = f"refs/merge-ready/{merge_ready.BASE_BRANCH}"
    git("fetch", "-q", "--no-tags", REMOTE, f"+refs/heads/{branch}:{HEAD_REF}", root=root)
    lists = repo.checked_baselines()
    try:
        if git("merge-base", "--is-ancestor", main, HEAD_REF, root=root, check=False).returncode == 0:
            raise Refused(f"{branch} already contains {merge_ready.BASE_BRANCH}")
        git("checkout", "-q", "--detach", HEAD_REF, root=root)
        git("merge", "-q", "--no-ff", "--no-commit", main, root=root, check=False)
        conflicted = git("diff", "--name-only", "--diff-filter=U", root=root).stdout.split()
        if conflicted and not checked_only(conflicted, lists):
            outside = [p for p in conflicted if p not in lists]
            raise Refused(f"conflicts outside the eval-gated baseline lists: {', '.join(outside)}")
        for path in conflicted:
            git("checkout", "--theirs", "--", path, root=root)
            git("add", "--", path, root=root)
        if record(root) != 0:
            raise Refused("make eval-baseline refused (a regression, or the eval did not run)")
        recorded = git("diff", "--name-only", root=root).stdout.split()
        others = sorted(p for p in recorded if p not in lists)
        if others:
            raise Refused(f"the re-record changed files the carry rule does not accept: {', '.join(others)}")
        if recorded:
            git("add", "--", *recorded, root=root)
        git("commit", "-q", "--no-edit", root=root)  # one merge commit: main merged, lists re-recorded
        head = git("rev-parse", "HEAD", root=root).stdout.strip()
        git("push", "-q", REMOTE, f"HEAD:refs/heads/{branch}", root=root)
        return head
    finally:
        git("merge", "--abort", root=root, check=False)
        git("checkout", "-q", "-f", *([] if on else ["--detach"]), start, root=root, check=False)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--pr", required=True, help="the ejected PR's number")
    args = parser.parse_args(argv)
    client = gh_client.from_remote(ROOT)
    if client is None:
        print("merge_catchup: origin is not a github.com remote", file=sys.stderr)
        return 2
    try:
        pr = gh_client.number(args.pr)
        pull = client.pull(pr)
        if merge_ready.field_of(pull, "merged") is True or merge_ready.text_of(pull, "state") != "open":
            raise Refused(f"#{pr} is not open")
        if merge_ready.text_of(pull, "head", "repo", "full_name") != merge_ready.text_of(
            pull, "base", "repo", "full_name"
        ):
            raise Refused(f"#{pr} comes from a fork")
        if merge_ready.text_of(pull, "base", "ref") != merge_ready.BASE_BRANCH:
            raise Refused(f"#{pr} is not based on {merge_ready.BASE_BRANCH}")
        head = catch_up(ROOT, merge_ready.text_of(pull, "head", "ref"))
    except Refused as refusal:
        print(f"merge_catchup: not catching up #{args.pr}: {refusal}", file=sys.stderr)
        return 1
    except (gh_client.GhError, GitError) as error:
        print(f"merge_catchup: {error}", file=sys.stderr)
        return 2
    print(f"#{pr}: pushed {head}; when CI is green, run make mark-ready PR={pr} CARRY=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
