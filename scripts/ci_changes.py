#!/usr/bin/env python3
"""Decides whether a PR's changed files need the rust, eval and python CI jobs (decision 0049).

Reads the changed paths NUL-separated on stdin (`git diff -z --no-renames --name-only`) and prints
`code=true` or `code=false`. `code=false` means every path is documentation that nothing reads at
build or test time, so the heavy jobs may be skipped. Anything else, including empty or
undecodable input, is `code=true`: a wrong skip looks exactly like a pass, so doubt runs CI.

Standard library only: CI runs the copy from the PR's base commit (decision 0066), so a PR can
never change how its own files are classified.
"""

from __future__ import annotations

import sys

# Documentation directories: changing files under them alone does not run the heavy jobs.
DOC_DIRS = ("docs/", ".claude/")

# Paths under DOC_DIRS that code reads at build or test time, so they count as code. A new reader
# must add its path here in the same PR; `scripts/check_docs_readers.py` (run by `make hygiene`)
# fails until it does. docs/excel-semantics/: the eval harness validates DEVIATION anchors in it
# (crates/rustsheet-eval/src/deviations.rs).
CODE_READ_PATHS = ("docs/excel-semantics/",)


def is_code(path: str) -> bool:
    """Whether changing `path` needs the heavy CI jobs."""
    if path.startswith(CODE_READ_PATHS):
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


def main() -> int:
    code = classify(sys.stdin.buffer.read())
    print(f"code={'true' if code else 'false'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
