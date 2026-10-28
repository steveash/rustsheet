#!/usr/bin/env python3
"""Decides whether a PR's changed files need the rust, eval and python CI jobs (decision 0049).

Reads the changed paths NUL-separated on stdin (`git diff -z --no-renames --name-only`) and prints
`code=true` or `code=false`. `code=false` means every path is documentation that nothing reads at
build or test time, so the heavy jobs may be skipped. Anything else, including empty or
undecodable input, is `code=true`: a wrong skip looks exactly like a pass, so doubt runs CI.

With `--merge-base A B` it lists the paths itself: the diff from the single merge base of A and B
to B (decision 0147). A pull request passes its `base.sha` and `head.sha`, so only the PR's own
changes count even when its branch is behind `main`; a merge-train push passes `origin/main` and
`HEAD`. Several merge bases, none, a failed git command, a shallow clone or a malformed revision
give `code=true`. Any other arguments also give `code=true`.

Standard library only: CI runs the copy from the PR's base commit, or from a train's merge base
(decision 0066), so a PR can never change how its own files are classified.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# Documentation directories: changing files under them alone does not run the heavy jobs.
DOC_DIRS = ("docs/", ".claude/")

# Paths under DOC_DIRS that code reads at build or test time, so they count as code. A new reader
# must add its path here in the same PR; `scripts/check_docs_readers.py` (run by `make hygiene`)
# fails until it does. docs/excel-semantics/: the eval harness validates DEVIATION anchors in it
# (crates/rustsheet-eval/src/deviations.rs). docs/milestones.md, docs/progress.md and
# docs/progress/: the progress report generator reads them (tools/src/rustsheet_tools/
# progress_report/, SPEC-0022, decision 0120), so a steward PR that changes only the page or its
# history still runs the python job.
# An entry ending in `/` is a directory and matches every path under it; any other entry is one
# file and matches only itself, so docs/progress.md does not also claim docs/progress.mdx.
CODE_READ_PATHS = ("docs/excel-semantics/", "docs/milestones.md", "docs/progress.md", "docs/progress/")
CODE_READ_DIRS = tuple(p for p in CODE_READ_PATHS if p.endswith("/"))


def is_code_read(path: str) -> bool:
    """Whether `path` is one of CODE_READ_PATHS or under one of its directories."""
    return path in CODE_READ_PATHS or path.startswith(CODE_READ_DIRS)


def is_code(path: str) -> bool:
    """Whether changing `path` needs the heavy CI jobs."""
    if is_code_read(path):
        return True
    if path.startswith(DOC_DIRS):
        return False
    # Top-level Markdown (README.md, CLAUDE.md) is read by people and agents only.
    return "/" in path or not path.endswith(".md")


def classify(raw: bytes) -> bool:
    """`code` for NUL-separated paths: true unless there is at least one path and none is code."""
    try:
        paths = [p for p in raw.decode("utf-8").split("\0") if p]
    except UnicodeDecodeError:
        return True
    return not paths or any(is_code(p) for p in paths)


# A revision CI passes: a sha, `HEAD` or a ref name. Nothing that git could read as an option, and
# no whitespace, so the value is one argument with one meaning.
REVISION = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_./-]*")


def _git(cwd: Path | None, *args: str) -> bytes | None:
    """stdout of `git args` in `cwd`, or None when git fails or cannot run."""
    try:
        run = subprocess.run(["git", *args], cwd=cwd, capture_output=True, check=False, timeout=120)
    except (OSError, subprocess.SubprocessError):
        return None
    return run.stdout if run.returncode == 0 else None


def classify_range(base: str, head: str, cwd: Path | None = None) -> bool:
    """`code` for the diff from the single merge base of `base` and `head` to `head`.

    Diffing from the merge base, not from `base`, leaves out whatever `base`'s branch gained
    since `head` branched off it: on PR #2119 (run 40622986699) `base.sha` lagged GitHub's test
    merge, so `main`'s newer code files made a docs-only PR `code=true` (decision 0147).
    """
    if not all(REVISION.fullmatch(rev) for rev in (base, head)):
        return True
    # A shallow clone can hide the true merge base or report a wrong one; so can a failed check.
    if _git(cwd, "rev-parse", "--is-shallow-repository") != b"false\n":
        return True
    bases = _git(cwd, "merge-base", "--all", base, head)
    if bases is None or len(bases.split()) != 1:
        return True
    merge_base = bases.decode("ascii", "replace").strip()
    # --no-renames lists both sides of a rename, so moving code into docs/ still counts as code;
    # -z keeps unusual file names intact.
    changed = _git(cwd, "diff", "-z", "--no-renames", "--name-only", merge_base, head, "--")
    if changed is None:
        return True
    # File names are attacker-controlled: log only their count (workflow-command injection).
    print("changed files:", changed.count(b"\0"), file=sys.stderr)
    return classify(changed)


def main() -> int:
    args = sys.argv[1:]
    if not args:
        code = classify(sys.stdin.buffer.read())
    elif len(args) == 3 and args[0] == "--merge-base":
        code = classify_range(args[1], args[2])
    else:
        code = True
    print(f"code={'true' if code else 'false'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
