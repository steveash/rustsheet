#!/usr/bin/env python3
"""Fails when code reads a docs-only path that the CI classifier would skip (decision 0066).

A PR that changes only `docs/` or `.claude/` files skips the rust, eval and python jobs
(decision 0049, `scripts/ci_changes.py`). That is safe only while no build or test reads those
files. This check scans code under crates/, tools/, bindings/ and fuzz/ for string literals that
name a repository-root `docs/` or `.claude/` path (`"docs/x"`, `"../../docs/x"`, `"docs"`) and
fails unless the path is under `CODE_READ_PATHS`, which the classifier treats as code.

Mentions in comments and prose do not count: a literal is flagged only when its text starts with
the path. A flagged literal that does not name the repository's docs/ (a crate's own docs/
directory) goes in NOT_REPO_DOCS with the reason. scripts/ is exempt: `make hygiene` always runs.

It also guards the Rust shards' skip (decision 0235): a literal in crates/, bindings/ or fuzz/
that climbs out of its crate (`"../../eval/x"`, or `"/../../eval/x"` after `CARGO_MANIFEST_DIR`)
into `RUST_INERT_DIRS` must name a path under `RUST_READ_PATHS`. A path built by joining
separate components (`root.join("eval")`) is not seen; decision 0235 records the strace audit
behind the list.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

from ci_changes import DOC_DIRS, RUST_INERT_DIRS, is_code_read, is_rust_read

ROOT = Path(__file__).resolve().parent.parent
CODE_DIRS = ("crates/", "tools/", "bindings/", "fuzz/")
# Code built and tested by the Rust shards (decision 0235).
RUST_DIRS = ("crates/", "bindings/", "fuzz/")
SUFFIXES = (".rs", ".py", ".sh", ".toml")

# (file, literal text) pairs that look like repository docs/ paths but are not, with the reason.
NOT_REPO_DOCS: dict[tuple[str, str], str] = {
    ("bindings/python/run_examples.sh", "docs/examples"): "the script runs in bindings/python",
}

# A one-line string literal in single or double quotes; Rust's `\"` escapes are rare enough here.
LITERAL = re.compile(r"\"([^\"\n]*)\"|'([^'\n]*)'")
COMMENT_PREFIXES = ("//", "#", "/*", "*")
DOC_ROOTS = tuple(d.rstrip("/") for d in DOC_DIRS)


def repo_doc_path(text: str) -> str | None:
    """The docs path a literal names (`../` and `./` stripped), or None if it names none."""
    while text.startswith(("../", "./")):
        text = text[text.index("/") + 1 :]
    if text in DOC_ROOTS or text.startswith(DOC_DIRS):
        return text
    return None


def rust_inert_path(text: str) -> str | None:
    """The Rust-inert repository path a literal climbs to (`../` or `/../` first), or None."""
    if text.startswith("/../"):
        text = text[1:]
    if not text.startswith("../"):
        return None
    while text.startswith(("../", "./")):
        text = text[text.index("/") + 1 :]
    roots = tuple(d.rstrip("/") for d in RUST_INERT_DIRS)
    if text in roots or text.startswith(RUST_INERT_DIRS):
        return text
    return None


def violations(path: str, source: str) -> list[str]:
    """`file:line: literal` for each docs path in `source` that the classifier would skip."""
    found = []
    for lineno, line in enumerate(source.splitlines(), 1):
        if line.lstrip().startswith(COMMENT_PREFIXES):
            continue
        for match in LITERAL.finditer(line):
            text = match.group(1) if match.group(1) is not None else match.group(2)
            inert = rust_inert_path(text) if path.startswith(RUST_DIRS) else None
            # A directory literal may come without its trailing slash; a format placeholder
            # (`cases/{name}.xlsx`) stays inside the directory it starts in.
            if inert is not None and not (is_rust_read(inert) or is_rust_read(inert + "/")):
                found.append(f"{path}:{lineno}: {text!r} (Rust reads it: RUST_READ_PATHS)")
                continue
            doc = repo_doc_path(text)
            if doc is None or (path, text) in NOT_REPO_DOCS:
                continue
            if not (is_code_read(doc) or is_code_read(doc + "/")):
                found.append(f"{path}:{lineno}: {text!r}")
    return found


def tracked_code_files() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "-z", "--", *CODE_DIRS],
        cwd=ROOT,
        capture_output=True,
        check=True,
    ).stdout.decode()
    return [p for p in out.split("\0") if p.endswith(SUFFIXES) or p.endswith("/Makefile")]


def main() -> int:
    found = []
    for path in tracked_code_files():
        try:
            source = (ROOT / path).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        found.extend(violations(path, source))
    if found:
        print("FAIL docs readers: code reads paths whose changes CI would skip (decisions 0049, 0235):")
        for line in found:
            print(f"  {line}")
        print("Add the path to CODE_READ_PATHS (or, for a Rust reader, RUST_READ_PATHS) in")
        print("scripts/ci_changes.py (a protected path, so with a decision record), or to")
        print("NOT_REPO_DOCS in scripts/check_docs_readers.py if it is not the repository's docs/.")
        return 1
    print("docs readers ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
