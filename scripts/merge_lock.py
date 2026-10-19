#!/usr/bin/env python3
"""The merge lock: agents merge to `main` one at a time (decision 0075).

  scripts/merge_lock.py acquire --pr N [--holder ID] [--ttl MINUTES]  # exit 0: held; 1: busy
  scripts/merge_lock.py release [--holder ID]                         # frees only your own lock
  scripts/merge_lock.py status

The lock is the branch `merge-lock` on origin: an orphan commit with an empty tree whose message
names the holder and the PR, or says `state: free`. Pushes carry `--force-with-lease`, so the server
creates the branch only while it is absent and replaces it only while it still holds the commit the
caller read: of two racing acquirers exactly one wins. Release writes the free marker instead of
deleting the branch, because the session git proxy refuses branch deletion (HTTP 403; T-1938). A lock
older than its TTL (the holder died) may be taken over the same way. The empty tree keeps task and
decision files off the branch (tasks.py scans every origin branch), and CI does not run on it (ci.yml
runs on pull requests and on `main`).

The holder is `CLAUDE_CODE_SESSION_ID` unless `--holder` is given.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
LOCK_REF = "refs/heads/merge-lock"
# Where a read keeps the lock commit, so its sha and message always come from one fetch.
SEEN_REF = "refs/merge-lock/seen"
# A holder merges `main` in, waits for CI (about 5 minutes) and merges; 30 minutes is a dead holder.
TTL_MINUTES = 30
# The committer of lock commits, so a clone with no git identity can still take the lock.
IDENTITY = ("-c", "user.name=merge-lock", "-c", "user.email=merge-lock@invalid")


class GitError(Exception):
    """A git command failed for a reason other than a lost race."""


@dataclass(frozen=True)
class Lock:
    """The lock commit as origin holds it."""

    sha: str
    holder: str
    pr: str
    taken_at: int
    # A released lock: the branch holds the free marker, which anyone may replace.
    free: bool = False

    def age_minutes(self, now: float) -> float:
        return (now - self.taken_at) / 60

    def is_stale(self, now: float, ttl_minutes: float) -> bool:
        return self.age_minutes(now) >= ttl_minutes


def git(*args: str, root: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(root), *args], stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False
    )
    if check and result.returncode != 0:
        raise GitError(f"git {' '.join(args)}: {result.stderr.strip()}")
    return result


def read_lock(root: Path = ROOT) -> Lock | None:
    """The lock commit on origin (held, or the free marker), or None when the branch is absent."""
    fetched = git("fetch", "-q", "--no-tags", REMOTE, f"+{LOCK_REF}:{SEEN_REF}", root=root, check=False)
    if fetched.returncode != 0:
        if "couldn't find remote ref" in fetched.stderr:
            return None
        raise GitError(f"fetch {LOCK_REF}: {fetched.stderr.strip()}")
    body = git("log", "-1", "--format=%H%n%ct%n%B", SEEN_REF, root=root).stdout.splitlines()
    try:
        fields = dict(line.split(": ", 1) for line in body[2:] if ": " in line)
        return Lock(
            sha=body[0],
            holder=fields.get("holder", "?"),
            pr=fields.get("pr", "?"),
            taken_at=int(body[1]),
            free=fields.get("state") == "free",
        )
    except (IndexError, ValueError) as e:
        raise GitError(f"malformed lock commit on {LOCK_REF}") from e


def held(lock: Lock | None) -> Lock | None:
    """`lock` if someone holds it, None when it is absent or free."""
    return lock if lock is not None and not lock.free else None


def lock_commit(holder: str | None, pr: str = "", root: Path = ROOT) -> str:
    """A new orphan commit with an empty tree that names `holder` and `pr`, or the free marker when
    `holder` is None."""
    tree = git("mktree", root=root).stdout.strip()
    message = f"merge-lock\n\nholder: {holder}\npr: {pr}\n" if holder is not None else "merge-lock\n\nstate: free\n"
    return git(*IDENTITY, "commit-tree", tree, "-m", message, root=root).stdout.strip()


def push_lease(expected: str, new: str, root: Path = ROOT) -> bool:
    """Sets the lock to the commit `new` only if origin still has `expected` there ("" for absent).
    False when another agent moved it first."""
    lease = f"--force-with-lease={LOCK_REF}:{expected}"
    result = git("push", "-q", lease, REMOTE, f"{new}:{LOCK_REF}", root=root, check=False)
    return result.returncode == 0


def describe(lock: Lock, now: float, ttl_minutes: float) -> str:
    stale = " (stale: may be taken over)" if lock.is_stale(now, ttl_minutes) else ""
    return f"held by {lock.holder} for PR {lock.pr}, {lock.age_minutes(now):.0f} min ago{stale}"


def acquire(holder: str, pr: str, ttl_minutes: float = TTL_MINUTES, root: Path = ROOT) -> int:
    """Takes the lock, or renews it when `holder` already has it. 0 when held, 1 when busy."""
    now = time.time()
    lock = read_lock(root)
    current = held(lock)
    mine = current is not None and current.holder == holder
    if current is not None and not mine and not current.is_stale(now, ttl_minutes):
        print(f"merge lock busy: {describe(current, now, ttl_minutes)}")
        return 1
    if push_lease(lock.sha if lock else "", lock_commit(holder, pr, root), root):
        print(f"merge lock {'renewed' if mine else 'acquired'} for PR {pr}")
        return 0
    winner = held(read_lock(root))
    print(f"merge lock busy: {describe(winner, now, ttl_minutes) if winner else 'lost a race'}")
    return 1


def release(holder: str, root: Path = ROOT) -> int:
    """Frees the lock if `holder` has it, by writing the free marker. 0 when the lock is no longer
    `holder`'s, 1 when another agent holds it (it is never released for them)."""
    current = held(read_lock(root))
    if current is None:
        print("merge lock: not held")
        return 0
    if current.holder != holder:
        print(f"merge lock not yours: {describe(current, time.time(), TTL_MINUTES)}")
        return 1
    if push_lease(current.sha, lock_commit(None, root=root), root):
        print("merge lock released")
        return 0
    print("merge lock: taken over before release")
    return 0


def status(root: Path = ROOT) -> int:
    current = held(read_lock(root))
    print(f"merge lock {describe(current, time.time(), TTL_MINUTES)}" if current else "merge lock: free")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="command", required=True)
    take = commands.add_parser("acquire")
    take.add_argument("--pr", required=True, type=int)
    take.add_argument("--ttl", type=float, default=TTL_MINUTES, help="minutes before a lock is stale")
    drop = commands.add_parser("release")
    for sub in (take, drop):
        sub.add_argument("--holder", default=os.environ.get("CLAUDE_CODE_SESSION_ID"))
    commands.add_parser("status")
    args = parser.parse_args(argv)
    try:
        if args.command == "status":
            return status()
        if not args.holder:
            print("merge lock: no holder (set --holder or CLAUDE_CODE_SESSION_ID)", file=sys.stderr)
            return 2
        if args.command == "acquire":
            return acquire(args.holder, str(args.pr), args.ttl)
        return release(args.holder)
    except GitError as e:
        print(f"merge lock: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
