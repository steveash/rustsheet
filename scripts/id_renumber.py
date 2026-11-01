"""Renumbering a PR's colliding task and decision files, as one definition (decision 0224).

`make renumber` (scripts/merge_renumber.py) gives each task or decision file a PR adds, whose id
`main` also has, a fresh id from the id registry, and the carry rule
(`merge_ready.Repo.carry_problem`) accepts the result without the reviewers. Both build the
renumbered PR head with `renumbered_commit`, so the rule cannot drift from what the tool does:

- the PR's own file `docs/tasks/<M>/T-OLD-<slug>.md` (or `docs/decisions/OLD-<slug>.md`) moves to
  the new id, nothing else in its path changing;
- the old id becomes the new one in every file the PR adds (the renamed files included, so the
  task's `id:` and the record's `# NNNN —` title follow) and in the lines the PR adds to files it
  modifies (`added_lines`, from `git diff -U0` of the merge base and the PR head);
- no other byte changes: a line `main` owns keeps the old id, which names `main`'s own file.

The renumbered head (a throwaway commit whose parent is the PR head) is then merged with `main`.
The carry rule recomputes that merge from the head's parents and the renames the head shows, and
compares trees (`renumbered_merge`), so a hand edit anywhere is refused.

Task ids are matched as `T-NNNN` not inside a longer word; decision numbers as four digits not
inside a longer number or word (so `0.0221` and `T-0221` never match).
"""

from __future__ import annotations

import os
import re
import subprocess
import tempfile
from collections.abc import Mapping
from pathlib import Path

from merge_lock import GitError, git

TASK_PATH = re.compile(r"^(docs/tasks/(?:[^/]+/)*)(T-\d{4})(-[^/]*\.md)$")
DECISION_PATH = re.compile(r"^(docs/decisions/)(\d{4})(-[a-z0-9-]+\.md)$")
ID_DIRS = ("docs/tasks", "docs/decisions")
TASK_ID = re.compile(r"T-\d{4}")
DECISION_ID = re.compile(r"\d{4}")
# A pinned diff, so the merger and the renumbering session number the PR's added lines alike.
DIFF = (
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--no-color",
    "--no-renames",
    "--diff-algorithm=myers",
    "-U0",
)
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")
IDENTITY = ("-c", "user.name=merge-renumber", "-c", "user.email=merge-renumber@invalid")
REGULAR = ("100644", "100755")


class RenumberError(Exception):
    """A renumber that is not allowed (the reason); nothing was written."""


def id_path(path: str) -> tuple[str, str, str] | None:
    """(prefix, id, suffix) of a task or decision file path, or None for any other path."""
    match = TASK_PATH.match(path) or DECISION_PATH.match(path)
    return (match[1], match[2], match[3]) if match else None


def same_kind(old: str, new: str) -> bool:
    """Whether `old` and `new` are both task ids or both decision numbers."""
    return any(kind.fullmatch(old) and kind.fullmatch(new) for kind in (TASK_ID, DECISION_ID))


def id_pattern(old: str) -> str:
    """The regex of one id where it stands as a whole id (module docstring)."""
    if TASK_ID.fullmatch(old):
        return rf"(?<![\w-]){re.escape(old)}(?!\d)"
    return rf"(?<![\w.,-]){re.escape(old)}(?!\w|[.,]\d)"


def substitute(text: str, mapping: Mapping[str, str]) -> str:
    """`text` with every old id of `mapping` replaced by its new id, all at once (no chains)."""
    if not mapping:
        return text
    pattern = re.compile("|".join(f"(?:{id_pattern(old)})" for old in sorted(mapping)))
    return pattern.sub(lambda m: mapping[m[0]], text)


def raw(root: Path, *args: str, data: bytes | None = None, env: Mapping[str, str] | None = None) -> bytes:
    """`git -C root <args>`'s stdout as bytes (blobs need not be text); GitError when it fails."""
    done = subprocess.run(
        ["git", "-C", str(root), *args],
        input=data if data is not None else b"",
        capture_output=True,
        check=False,
        env=None if env is None else {**os.environ, **env},
    )
    if done.returncode != 0:
        raise GitError(f"git {' '.join(args)}: {done.stderr.decode('utf-8', 'replace').strip()}")
    return done.stdout


def merge_base(root: Path, a: str, b: str) -> str:
    """The single merge base of `a` and `b`; RenumberError when there is not exactly one."""
    bases = git("merge-base", "--all", a, b, root=root, check=False).stdout.split()
    if len(bases) != 1:
        raise RenumberError("the PR head and main do not have exactly one merge base")
    return bases[0]


def changed(root: Path, base: str, head: str, *paths: str) -> dict[str, str]:
    """path -> status (A, M, D, T) of `head` against `base`, renames off."""
    out = git(
        "diff-tree", "-r", "-z", "--no-renames", "--name-status", base, head, "--", *paths, root=root
    ).stdout
    parts = out.split("\0")
    return {path: status for status, path in zip(parts[0::2], parts[1::2], strict=False) if path}


def ids_in(root: Path, commit: str) -> set[str]:
    """The task ids and decision numbers of the files in `commit`'s id directories."""
    out = git("ls-tree", "-r", "-z", "--name-only", commit, "--", *ID_DIRS, root=root).stdout
    return {found[1] for path in out.split("\0") if (found := id_path(path))}


def added_files(root: Path, base: str, head: str) -> list[str]:
    """The paths `head` adds since `base`."""
    return sorted(p for p, status in changed(root, base, head).items() if status == "A")


def colliding(root: Path, base: str, head: str, main: str) -> dict[str, str]:
    """old id -> path of each task or decision file `head` adds since `base` whose id `main` has."""
    on_main = ids_in(root, main)
    found = {}
    for path in added_files(root, base, head):
        parts = id_path(path)
        if parts and parts[1] in on_main:
            found[parts[1]] = path
    return found


def added_lines(root: Path, base: str, head: str, path: str) -> set[int]:
    """The 1-based numbers of the lines of `path` in `head` that `head` adds since `base`."""
    out = git(*DIFF, base, head, "--", path, root=root).stdout
    lines: set[int] = set()
    for line in out.splitlines():
        hunk = HUNK.match(line)
        if hunk:
            start, count = int(hunk[1]), int(hunk[2] if hunk[2] is not None else 1)
            lines.update(range(start, start + count))
    return lines


def renamed(path: str, mapping: Mapping[str, str]) -> str:
    """`path` with its id replaced when it is a task or decision file whose id `mapping` moves."""
    parts = id_path(path)
    return f"{parts[0]}{mapping[parts[1]]}{parts[2]}" if parts and parts[1] in mapping else path


def check_mapping(root: Path, base: str, head: str, main: str, mapping: Mapping[str, str]) -> None:
    """RenumberError unless each old id is the id of exactly one file `head` adds and each new id
    is a fresh id of the same kind: on neither `main` nor `head`, and given once."""
    added = [parts[1] for p in added_files(root, base, head) if (parts := id_path(p))]
    taken = ids_in(root, main) | ids_in(root, head)
    if len(set(mapping.values())) != len(mapping):
        raise RenumberError("two ids renumbered to one")
    for old, new in mapping.items():
        if added.count(old) != 1:
            raise RenumberError(f"{old} is not the id of exactly one file the PR adds")
        if not same_kind(old, new) or new in taken:
            raise RenumberError(f"{old} renumbered to {new}, which is not a fresh id of its kind")


def renumbered_files(
    root: Path, base: str, head: str, mapping: Mapping[str, str]
) -> tuple[dict[str, tuple[str, bytes]], list[str]]:
    """What the renumber changes in `head`'s tree: path -> (mode, content) to write, and the paths
    it removes (the renamed files' old paths)."""
    pattern = re.compile("|".join(id_pattern(old) for old in mapping))
    writes: dict[str, tuple[str, bytes]] = {}
    removed: list[str] = []
    entries = git("ls-tree", "-r", "-z", head, root=root).stdout.split("\0")
    modes = {e.split("\t", 1)[1]: e.split(" ", 1)[0] for e in entries if "\t" in e}
    for path, status in changed(root, base, head).items():
        if status not in ("A", "M") or modes.get(path) not in REGULAR:
            continue
        data = raw(root, "cat-file", "blob", f"{head}:{path}")
        target = renamed(path, mapping) if status == "A" else path
        text = decoded(data)
        new = data
        if text is not None and pattern.search(text):
            if status == "A":
                new = substitute(text, mapping).encode("utf-8")
            else:
                numbers = added_lines(root, base, head, path)
                lines = text.split("\n")
                new = "\n".join(
                    substitute(line, mapping) if n in numbers else line for n, line in enumerate(lines, 1)
                ).encode("utf-8")
        if target != path:
            removed.append(path)
        if target != path or new != data:
            writes[target] = (modes[path], new)
    return writes, removed


def decoded(data: bytes) -> str | None:
    """`data` as UTF-8 text, or None for a binary or undecodable blob (left as it is)."""
    if b"\0" in data:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def renumbered_commit(root: Path, base: str, head: str, mapping: Mapping[str, str]) -> str:
    """A throwaway commit whose parent is `head` and whose tree is `head` renumbered (module
    docstring). It is built in a private index, so the checkout is not touched."""
    writes, removed = renumbered_files(root, base, head, mapping)
    with tempfile.TemporaryDirectory() as tmp:
        env = {"GIT_INDEX_FILE": str(Path(tmp) / "index")}
        raw(root, "read-tree", head, env=env)
        for path in removed:
            raw(root, "update-index", "--force-remove", "--", path, env=env)
        for path, (mode, data) in writes.items():
            blob = raw(root, "hash-object", "-w", "--stdin", data=data).decode().strip()
            raw(root, "update-index", "--add", "--cacheinfo", f"{mode},{blob},{path}", env=env)
        tree = raw(root, "write-tree", env=env).decode().strip()
    message = "renumber " + ", ".join(f"{old} -> {new}" for old, new in sorted(mapping.items()))
    return raw(root, *IDENTITY, "commit-tree", tree, "-p", head, "-m", message).decode().strip()


def merge_tree(root: Path, a: str, b: str) -> tuple[str, list[str]] | None:
    """(tree, conflicted paths) of the merge of commits `a` and `b` (`git merge-tree
    --write-tree`, conflicts written with markers); None when it cannot be computed."""
    merged = git("merge-tree", "--write-tree", "--name-only", a, b, root=root, check=False)
    if merged.returncode not in (0, 1):
        return None
    lines = merged.stdout.split("\n")
    # After the tree: the conflicted paths, a blank line, then informational messages.
    rest = lines[1:]
    return lines[0].strip(), rest[: rest.index("")] if "" in rest else rest


def implied_mapping(root: Path, base: str, prev: str, tree: str, head: str) -> dict[str, str]:
    """old id -> new id of the renames `head` shows against the plain merge `tree`: a file `prev`
    adds since `base` that `head` lacks, and the one file `head` has in its place under another
    id. Empty when there are none. Only a hint: `renumbered_merge` checks the whole tree."""
    diff = changed(root, tree, head, *ID_DIRS)
    gone = [p for p, status in diff.items() if status == "D"]
    new = [p for p, status in diff.items() if status == "A"]
    ours = set(added_files(root, base, prev))
    mapping: dict[str, str] = {}
    for path in gone:
        parts = id_path(path)
        if parts is None or path not in ours:
            continue
        twins = [
            p for p in new if (q := id_path(p)) and q[0] == parts[0] and q[2] == parts[2] and q[1] != parts[1]
        ]
        if len(twins) == 1:
            mapping[parts[1]] = id_path(twins[0])[1]  # type: ignore[index]
    return mapping


def renumbered_merge(root: Path, parents: list[str], tree: str, head: str) -> tuple[str, list[str]] | None:
    """The carry rule's expected merge for a renumber (decision 0224): (tree, conflicted paths) of
    the PR head `parents[0]`, renumbered as `head`'s renames imply, merged with `main`'s
    `parents[1]`. None when `head` renames nothing against the plain merge `tree`; RenumberError
    when the renames are not a renumber `make renumber` could have made."""
    prev, main = parents
    base = merge_base(root, prev, main)
    mapping = implied_mapping(root, base, prev, tree, head)
    if not mapping:
        return None
    check_mapping(root, base, prev, main, mapping)
    return merge_tree(root, renumbered_commit(root, base, prev, mapping), main)
