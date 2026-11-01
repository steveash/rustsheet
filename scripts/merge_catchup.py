#!/usr/bin/env python3
"""Catch up a PR the merge train ejected for a conflict with main, when the conflict is only in
the eval-gated known-failure lists (decision 0207).

  scripts/merge_catchup.py --pr N   # exit 0: pushed; 1: refused (nothing pushed); 2: error

In this checkout, which must have no local changes, it does the following:
- merges `main` into the PR's branch without committing;
- refuses any conflict outside the lists CI's eval gate checks (`Repo.checked_baselines`: the
  gated suites the eval job runs without a fetch, and the visible-parity lock);
- takes `main`'s side of each conflicted known-failure list, resolves a conflicted visible-parity
  lock three-way per case from the merge base (`visible_lock.merge_visible`, decision 0219), and
  runs `make eval-baseline NEW_CASES=<the case files the PR adds>`, which re-records the lists,
  refuses regressions (decision 0093) and records visible-parity lines only for those cases
  (0219);
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
import tempfile
from collections.abc import Callable, Sequence
from pathlib import Path

import gh_client
import merge_ready
import visible_lock
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


def rerecord(root: Path, new_cases: Path) -> int:
    """`make eval-baseline` in `root`: re-records eval/baseline/ and refuses regressions. Only the
    cases `new_cases` lists (the PR's added case files) may get a new visible-parity line or a
    higher one (`update-baseline --new-cases`, decision 0219)."""
    command = (*RERECORD, f"NEW_CASES={new_cases}")
    return subprocess.run(command, cwd=root, stdin=subprocess.DEVNULL, check=False).returncode


# What `catch_up` merges `main` into: given the checkout, the PR head and `main` (both fetched), the
# commit to check out before the merge and the merge commit's message. `make renumber` passes one
# that renumbers the PR's colliding ids first (scripts/merge_renumber.py, decision 0224).
Rewrite = Callable[[Path, str, str], tuple[str, str]]


def plain(root: Path, head: str, main: str) -> tuple[str, str]:
    """The catch-up's own `Rewrite`: the PR head itself."""
    return head, f"Merge {merge_ready.BASE_BRANCH} (make catch-up, decision 0207)"


def catch_up(
    root: Path,
    branch: str,
    record: Callable[[Path, Path], int] = rerecord,
    rewrite: Rewrite = plain,
    record_clean: bool = True,
) -> str:
    """Merges `main` into `branch` as the module docstring says, as one merge commit that the carry
    rule accepts, and pushes it; returns the pushed head. Raises Refused when it cannot (nothing
    pushed). The checkout's branch, or detached HEAD, is restored either way. `rewrite` gives the
    commit the merge starts from (the merge commit's parents are still the PR head and `main`);
    without `record_clean`, the lists are re-recorded only when one conflicted."""
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
    head, main_sha = (git("rev-parse", ref, root=root).stdout.strip() for ref in (HEAD_REF, main))
    try:
        if git("merge-base", "--is-ancestor", main, HEAD_REF, root=root, check=False).returncode == 0:
            raise Refused(f"{branch} already contains {merge_ready.BASE_BRANCH}")
        start_at, message = rewrite(root, head, main_sha)
        git("checkout", "-q", "--detach", start_at, root=root)
        git("merge", "-q", "--no-ff", "--no-commit", main, root=root, check=False)
        if git("rev-parse", "-q", "--verify", "MERGE_HEAD", root=root, check=False).returncode:
            raise Refused(f"git could not start merging {merge_ready.BASE_BRANCH} into {branch}")
        conflicted = git("diff", "--name-only", "--diff-filter=U", root=root).stdout.split()
        # A PR from before decision 0219 modified the retired json lock, which main deleted.
        if not visible_lock.in_commit(root, main, visible_lock.RETIRED_LOCK):
            lists = lists | {visible_lock.RETIRED_LOCK}
        if conflicted and not checked_only(conflicted, lists):
            outside = [p for p in conflicted if p not in lists]
            raise Refused(f"conflicts outside the eval-gated baseline lists: {', '.join(outside)}")
        for path in conflicted:
            if path == visible_lock.RETIRED_LOCK:
                git("rm", "-q", "--", path, root=root)
                continue
            if path == visible_lock.VISIBLE_PARITY:
                # main's side alone would drop the PR's lines, and either side's higher count would
                # undo main's lowerings: merge each case three-way (decision 0219). Stage 1 is the
                # merge base (absent when both sides added the file), 2 the PR, 3 main.
                base, pr, main_side = (git("show", f":{n}:{path}", root=root, check=False).stdout for n in (1, 2, 3))
                (root / path).write_text(visible_lock.merge_visible(base, pr, main_side), encoding="utf-8")
            else:
                git("checkout", "--theirs", "--", path, root=root)
            git("add", "--", path, root=root)
        # The PR's own case files, from before the merge: after it, the tree's baseline lists the
        # PR's cases too, so only this list tells them from main's (decision 0219).
        added = visible_lock.added_case_files(root, main, HEAD_REF)
        if conflicted or record_clean:
            with tempfile.TemporaryDirectory() as tmp:
                listed = Path(tmp) / "new-cases.txt"
                listed.write_text("".join(f"{p}\n" for p in added), encoding="utf-8")
                if record(root, listed) != 0:
                    raise Refused("make eval-baseline refused (a regression, or the eval did not run)")
        # The carry rule's visible-parity check, before anything is pushed: a line above main's
        # for one of main's cases (a PR's sanctioned raise) needs a merge by hand (decision 0219).
        lock_path = root / visible_lock.VISIBLE_PARITY
        lock = lock_path.read_text(encoding="utf-8") if lock_path.exists() else ""
        main_lock = git("show", f"{main}:{visible_lock.VISIBLE_PARITY}", root=root, check=False).stdout
        raised = visible_lock.visible_rises(lock, main_lock, visible_lock.case_on(root, main))
        if raised:
            raise Refused(f"{visible_lock.VISIBLE_PARITY} has a line above main's ({raised[0]}): merge by hand")
        recorded = git("diff", "--name-only", root=root).stdout.split()
        others = sorted(p for p in recorded if p not in lists)
        if others:
            raise Refused(f"the re-record changed files the carry rule does not accept: {', '.join(others)}")
        if recorded:
            git("add", "--", *recorded, root=root)
        # One merge commit of the PR head and main: main merged, lists re-recorded (and, from
        # `make renumber`, ids renumbered). A push the branch moved under is refused, not forced.
        tree = git("write-tree", root=root).stdout.strip()
        pushed = git("commit-tree", tree, "-p", head, "-p", main_sha, "-m", message, root=root).stdout.strip()
        git("push", "-q", REMOTE, f"{pushed}:refs/heads/{branch}", root=root)
        return pushed
    finally:
        git("merge", "--abort", root=root, check=False)
        git("checkout", "-q", "-f", *([] if on else ["--detach"]), start, root=root, check=False)


def pr_branch(client: gh_client.Client, pr: int) -> str:
    """The branch of PR `pr`; Refused unless the PR is open, from this repository and based on
    `main` (forks are refused: the tools build and push the PR's tree with this session's
    credentials)."""
    pull = client.pull(pr)
    if merge_ready.field_of(pull, "merged") is True or merge_ready.text_of(pull, "state") != "open":
        raise Refused(f"#{pr} is not open")
    if merge_ready.text_of(pull, "head", "repo", "full_name") != merge_ready.text_of(pull, "base", "repo", "full_name"):
        raise Refused(f"#{pr} comes from a fork")
    if merge_ready.text_of(pull, "base", "ref") != merge_ready.BASE_BRANCH:
        raise Refused(f"#{pr} is not based on {merge_ready.BASE_BRANCH}")
    return merge_ready.text_of(pull, "head", "ref")


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
        head = catch_up(ROOT, pr_branch(client, pr))
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
