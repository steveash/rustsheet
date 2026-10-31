#!/usr/bin/env python3
"""The id registry: task ids and decision numbers are claimed atomically (decision 0215).

`make new-task` and `make new-decision` used to take one above the highest id on the working
tree and on every pushed branch. Two sessions that allocated before either pushed got the same
id, and the merge train ejected the later PR (2026-10-31: T-3467, T-3476, T-3479, 0197, 0204).

The registry is the branch `id-registry` on origin: an orphan commit with an empty tree whose
message lists the most recent claims, one per line (`T-3541`, `0213`). A claim reads it, picks
the next id with the caller's rule over everything taken *plus* the registry's claims, and
pushes the registry with the new claim appended under `--force-with-lease` (merge_lock's CAS).
Of two racing allocators exactly one push wins; the other re-reads and picks the next id. The
list keeps the newest KEEP claims: an older claim's id is long pushed, and ids only grow.

Best effort: without a network the caller falls back to the tree-and-branches scan, and the
validators in CI still catch a collision.

  scripts/id_registry.py show     # the claims, newest last
"""

from __future__ import annotations

import re
import sys
import uuid
from collections.abc import Callable
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_lock  # noqa: E402 (the sys.path line above must run first)
from merge_lock import GitError, git  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
REF = "refs/heads/id-registry"
SEEN_REF = "refs/id-registry/seen"
HEADER = "id-registry v1"
TASK = re.compile(r"T-\d{4}")
DECISION = re.compile(r"\d{4}")
CLAIM = re.compile(r"T-\d{4}|\d{4}")
KEEP = 400
ATTEMPTS = 10
IDENTITY = ("-c", "user.name=id-registry", "-c", "user.email=id-registry@invalid")


class RegistryError(Exception):
    """The registry could not be read or written; the caller falls back to the plain scan."""


def read(root: Path = ROOT) -> tuple[str, list[str]]:
    """(sha, claims oldest first) of the registry on origin; ("", []) when the branch is absent.
    Lines that are not a claim are ignored, so a malformed registry cannot block allocation."""
    fetched = git("fetch", "-q", "--no-tags", merge_lock.REMOTE, f"+{REF}:{SEEN_REF}", root=root, check=False)
    if fetched.returncode != 0:
        if "couldn't find remote ref" in fetched.stderr:
            return "", []
        raise RegistryError(f"fetch {REF}: {fetched.stderr.strip()}")
    sha = git("rev-parse", SEEN_REF, root=root).stdout.strip()
    message = git("log", "-1", "--format=%B", SEEN_REF, root=root).stdout
    return sha, [line.strip() for line in message.splitlines() if CLAIM.fullmatch(line.strip())]


def commit(claims: list[str], root: Path) -> str:
    """An orphan commit with an empty tree whose message lists `claims`. The nonce makes every
    claim's commit unique: two sessions claiming the same id in the same second would otherwise
    build byte-identical commits, and the loser's push would succeed as "up to date"."""
    tree = git("mktree", root=root).stdout.strip()
    message = "\n".join([HEADER, f"nonce: {uuid.uuid4().hex}", "", *claims[-KEEP:]])
    return git(*IDENTITY, "commit-tree", tree, "-m", message, root=root).stdout.strip()


def claim(pick: Callable[[set[str]], str], kind: re.Pattern[str], root: Path = ROOT) -> str:
    """Claims the id that `pick` chooses given the registry's claims of `kind` (TASK or DECISION),
    atomically; returns it. `pick` must choose an id of that kind outside the set it is given.
    Raises RegistryError when the registry cannot be reached or every attempt lost a race."""
    try:
        for _ in range(ATTEMPTS):
            sha, claims = read(root)
            chosen = pick({c for c in claims if kind.fullmatch(c)})
            if not kind.fullmatch(chosen) or chosen in claims:
                raise RegistryError(f"refusing to claim {chosen!r}")
            if merge_lock.push_lease(sha, commit([*claims, chosen], root), root, ref=REF):
                return chosen
    except GitError as e:
        raise RegistryError(str(e)) from e
    raise RegistryError(f"lost the race {ATTEMPTS} times")


def allocate(
    pick: Callable[[set[str]], str], kind: re.Pattern[str], fetch: bool, root: Path = ROOT
) -> tuple[str, bool]:
    """(id, reserved): `claim(pick)` when `fetch` (online); otherwise, or when the registry fails,
    `pick(set())` unreserved, with a warning: the scan of the tree and the branches still applies."""
    if fetch:
        try:
            return claim(pick, kind, root), True
        except RegistryError as e:
            print(f"warning: id registry unavailable ({e}); the id is not reserved", file=sys.stderr)
    return pick(set()), False


def note(reserved: bool) -> str:
    """How the allocators' "wrote" line ends."""
    return "reserved in the id registry" if reserved else "NOT reserved: commit and push it now to claim it"


def main(argv: list[str]) -> int:
    if argv != ["show"]:
        print(__doc__.strip().splitlines()[-1].strip())
        return 2
    try:
        _, claims = read()
    except (RegistryError, GitError) as e:
        print(f"id registry: {e}", file=sys.stderr)
        return 2
    print("\n".join(claims) or "no claims yet")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
