#!/usr/bin/env python3
"""The merge train (SPEC-0027 section 4): batched merges of ready PRs under the merge lock.

  scripts/merge_train.py mode    # the mode on origin/main: lock, shadow or train

The mode is read from `scripts/merge_train.toml` on a freshly fetched `origin/main`, never from
the checkout, so a branch cannot switch its own flow. The read fails closed: a missing file, a
TOML error (a duplicate key included), or a missing, non-string or unknown `mode` reads as `lock`.
"""

from __future__ import annotations

import subprocess
import sys
import tomllib
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


def main(argv: list[str], root: Path = ROOT) -> int:
    if argv != ["mode"]:
        print("usage: merge_train.py mode", file=sys.stderr)
        return 2
    print(read_mode(root))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
