#!/usr/bin/env python3
"""The lock-flip rule (SPEC-0027 "The switch", decision 0157): the one change that may merge
without reviewers or a decision record is a PR that only sets the merge-train switch to `lock`.

  scripts/merge_flip.py check <pr>   # prints the head sha it checked; exit 1 when not a flip PR

A diff is a flip when, from the single merge base to the head:
- `git diff --raw --no-renames` lists exactly `M scripts/merge_train.toml`, mode 100644 on both
  sides (no symlink, no type change, no executable bit);
- both blobs parse with `tomllib` (which refuses a duplicate key), and the parsed documents are
  equal except that the head's top-level `mode` is the string "lock".

`check_governance.py` applies the rule to the local diff; `check` applies it to a PR, through the
files API (exactly one file, whose blob is the head's) and the fetched `refs/pull/<n>/head`, and
reads the PR again at the end so a head that moved during the check is refused. The merging agent
merges with `expectedHeadSha` set to the printed sha, so a later push cannot ride the check.
"""

from __future__ import annotations

import subprocess
import sys
import tomllib
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

import gh_client
import merge_ready
from gh_client import SHA, Client, GhError
from merge_lock import GitError, git

ROOT = Path(__file__).resolve().parent.parent
SWITCH = "scripts/merge_train.toml"
LOCK = "lock"
REGULAR_FILE = "100644"


@dataclass(frozen=True)
class Change:
    """One line of `git diff --raw`: modes and blob shas on both sides, status and path."""

    old_mode: str
    new_mode: str
    old_blob: str
    new_blob: str
    status: str
    path: str


def parse_raw(out: str) -> list[Change]:
    """The changes of `git diff --raw -z --no-abbrev` output (`:m m sha sha S\\0path\\0`)."""
    fields = out.split("\0")
    changes = []
    for header, path in zip(fields[::2], fields[1::2], strict=False):
        parts = header.lstrip(":").split()
        if len(parts) == 5:
            old_mode, new_mode, old_blob, new_blob, status = parts
            changes.append(Change(old_mode, new_mode, old_blob, new_blob, status, path))
    return changes


def blob_problem(old: bytes, new: bytes) -> str | None:
    """Why the switch file's change from `old` to `new` is not a flip to `lock`; None if it is."""
    try:
        before = tomllib.loads(old.decode("utf-8"))
        after = tomllib.loads(new.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        return f"{SWITCH} does not parse on both sides: {error}"
    if after.get("mode") != LOCK:
        return f'the head does not set the top-level mode = "{LOCK}"'
    if {k: v for k, v in before.items() if k != "mode"} != {k: v for k, v in after.items() if k != "mode"}:
        return f"{SWITCH} changes more than the top-level mode"
    return None


def diff_problems(changes: Sequence[Change]) -> list[str]:
    """Why a diff is not exactly `M scripts/merge_train.toml` between two regular files."""
    shown = ", ".join(f"{c.status} {c.path}" for c in changes) or "nothing"
    if len(changes) != 1 or changes[0].status != "M" or changes[0].path != SWITCH:
        return [f"the diff is not exactly `M {SWITCH}`: {shown}"]
    change = changes[0]
    if (change.old_mode, change.new_mode) != (REGULAR_FILE, REGULAR_FILE):
        return [f"{SWITCH} is not a regular file on both sides: {change.old_mode} -> {change.new_mode}"]
    return []


def blob(root: Path, sha: str) -> bytes:
    """A blob's bytes (read as bytes: the rule, not git, decides whether they are UTF-8)."""
    result = subprocess.run(
        ["git", "-C", str(root), "cat-file", "blob", sha], stdin=subprocess.DEVNULL, capture_output=True, check=False
    )
    if result.returncode != 0:
        raise GitError(f"git cat-file blob {sha}: {result.stderr.decode('utf-8', 'replace').strip()}")
    return result.stdout


def local_problems(root: Path, base: str, head: str) -> list[str]:
    """Why the diff from the single merge base of `base` and `head` to `head` is not a flip; empty
    when it is. A git failure is a problem too: the rule fails closed."""
    try:
        bases = git("merge-base", "--all", base, head, root=root).stdout.split()
        if len(bases) != 1:
            return [f"{len(bases)} merge bases, not one"]
        raw = git(
            "diff", "--raw", "-z", "--no-abbrev", "--no-renames", bases[0], head, "--", root=root
        ).stdout
        changes = parse_raw(raw)
        problems = diff_problems(changes)
        if problems:
            return problems
        old, new = (blob(root, sha) for sha in (changes[0].old_blob, changes[0].new_blob))
    except GitError as error:
        return [f"git: {error}"]
    problem = blob_problem(old, new)
    return [problem] if problem else []


def head_of(pull: dict[str, object]) -> str:
    head = pull.get("head")
    sha = head.get("sha") if isinstance(head, dict) else None
    return gh_client.checked(SHA, sha if isinstance(sha, str) else "", "head sha")


def pr_problems(client: Client, pr: int, root: Path = ROOT) -> tuple[str, list[str]]:
    """The head sha checked and why PR `pr` is not an open flip PR to `main` at that head."""
    pull = client.pull(pr)
    head = head_of(pull)
    base = pull.get("base")
    problems = []
    if pull.get("state") != "open" or pull.get("merged"):
        problems.append("the PR is not open")
    if not isinstance(base, dict) or base.get("ref") != merge_ready.BASE_BRANCH:
        problems.append(f"the PR's base is not {merge_ready.BASE_BRANCH}")
    files = client.pull_files(pr)
    named = [(f.get("status"), f.get("filename"), f.get("sha")) for f in files if isinstance(f, dict)]
    if len(files) != 1 or len(named) != 1 or named[0][:2] != ("modified", SWITCH):
        problems.append(f"the files API does not list exactly `modified {SWITCH}`: {named}")
    repo = merge_ready.Repo(root)
    try:
        fetched = repo.fetch_head(pr)
        repo.fetch_base()
        blob = git("rev-parse", "--verify", f"{head}:{SWITCH}", root=root, check=False).stdout.strip()
    except GitError as error:
        return head, [*problems, f"git: {error}"]
    if fetched != head:
        problems.append(f"refs/pull/{pr}/head is {fetched}, not the API's head {head}")
    elif len(named) == 1 and named[0][2] != blob:
        problems.append("the files API's blob is not the head's")
    problems += local_problems(root, f"refs/merge-ready/{merge_ready.BASE_BRANCH}", head)
    if head_of(client.pull(pr)) != head:
        problems.append("the head moved during the check")
    return head, problems


def main(argv: list[str], root: Path = ROOT) -> int:
    if len(argv) != 2 or argv[0] != "check":
        print("usage: merge_flip.py check <pr>", file=sys.stderr)
        return 2
    client = gh_client.from_remote(root)
    if client is None:
        print("merge_flip: origin is not a github.com repository", file=sys.stderr)
        return 2
    try:
        head, problems = pr_problems(client, gh_client.number(argv[1]), root)
    except GhError as error:
        print(f"merge_flip: {error}", file=sys.stderr)
        return 2
    if problems:
        print(f"not a flip PR (head {head}):")
        print("\n".join(f"  {p}" for p in problems))
        return 1
    print(head)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
