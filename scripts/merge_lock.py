#!/usr/bin/env python3
"""The merge lock: agents merge to `main` one at a time (decision 0075).

  scripts/merge_lock.py acquire --pr N [--holder ID] [--ttl MINUTES]  # exit 0: held; 1: busy
  scripts/merge_lock.py release [--holder ID]                         # drops only your own lock
  scripts/merge_lock.py status

The lock is the branch `merge-lock` on origin: an orphan commit with an empty tree whose message
names the holder and the PR. Pushes carry `--force-with-lease`, so the server creates the branch
only while it is absent, and replaces or deletes it only while it still holds the commit the caller
read: of two racing acquirers exactly one wins. A lock older than its TTL (the holder died) may be
taken over the same way. The empty tree keeps task and decision files off the branch (tasks.py scans
every origin branch), and CI does not run on it (ci.yml runs on pull requests and on `main`).

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
# A holder merges `main` in, waits for CI (about 5 minutes) and merges; 30 minutes is a dead holder.
TTL_MINUTES = 30
# The committer of lock commits, so a clone with no git identity can still take the lock.
IDENTITY = ("-c", "user.name=merge-lock", "-c", "user.email=merge-lock@invalid")


class GitError(Exception):
    """A git command failed for a reason other than a lost race."""


@dataclass(frozen=True)
class Lock:
    """The lock as origin holds it."""

    sha: str
    holder: str
    pr: str
    taken_at: int

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
    """The lock on origin, or None when nobody holds it."""
    listing = git("ls-remote", REMOTE, LOCK_REF, root=root).stdout.split()
    if not listing:
        return None
    sha = listing[0]
    git("fetch", "-q", REMOTE, LOCK_REF, root=root)
    body = git("log", "-1", "--format=%ct%n%B", sha, root=root).stdout.splitlines()
    fields = dict(line.split(": ", 1) for line in body[1:] if ": " in line)
    return Lock(sha=sha, holder=fields.get("holder", "?"), pr=fields.get("pr", "?"), taken_at=int(body[0]))


def lock_commit(holder: str, pr: str, root: Path = ROOT) -> str:
    """A new orphan commit with an empty tree that names `holder` and `pr`."""
    tree = git("mktree", root=root).stdout.strip()
    message = f"merge-lock\n\nholder: {holder}\npr: {pr}\n"
    return git(*IDENTITY, "commit-tree", tree, "-m", message, root=root).stdout.strip()


def push_lease(expected: str, new: str, root: Path = ROOT) -> bool:
    """Sets the lock to `new` (a sha, or "" to delete it) only if origin still has `expected` there
    ("" for absent). False when another agent moved it first."""
    source = f"{new}:{LOCK_REF}" if new else f":{LOCK_REF}"
    result = git(
        "push", "-q", f"--force-with-lease={LOCK_REF}:{expected}", REMOTE, source, root=root, check=False
    )
    return result.returncode == 0


def describe(lock: Lock, now: float, ttl_minutes: float) -> str:
    stale = " (stale: may be taken over)" if lock.is_stale(now, ttl_minutes) else ""
    return f"held by {lock.holder} for PR {lock.pr}, {lock.age_minutes(now):.0f} min ago{stale}"


def acquire(holder: str, pr: str, ttl_minutes: float = TTL_MINUTES, root: Path = ROOT) -> int:
    """Takes the lock, or renews it when `holder` already has it. 0 when held, 1 when busy."""
    now = time.time()
    lock = read_lock(root)
    if lock is not None and lock.holder != holder and not lock.is_stale(now, ttl_minutes):
        print(f"merge lock busy: {describe(lock, now, ttl_minutes)}")
        return 1
    if push_lease(lock.sha if lock else "", lock_commit(holder, pr, root), root):
        taken = "renewed" if lock is not None and lock.holder == holder else "acquired"
        print(f"merge lock {taken} for PR {pr}")
        return 0
    lost_to = read_lock(root)
    print(f"merge lock busy: {describe(lost_to, now, ttl_minutes) if lost_to else 'lost a race'}")
    return 1


def release(holder: str, root: Path = ROOT) -> int:
    """Drops the lock if `holder` has it. 0 when the lock is no longer `holder`'s, 1 when another
    agent holds it (it is never released for them)."""
    lock = read_lock(root)
    if lock is None:
        print("merge lock: not held")
        return 0
    if lock.holder != holder:
        print(f"merge lock not yours: {describe(lock, time.time(), TTL_MINUTES)}")
        return 1
    if push_lease(lock.sha, "", root):
        print("merge lock released")
        return 0
    print("merge lock: taken over before release")
    return 0


def status(root: Path = ROOT) -> int:
    lock = read_lock(root)
    print(f"merge lock {describe(lock, time.time(), TTL_MINUTES)}" if lock else "merge lock: free")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="command", required=True)
    take = commands.add_parser("acquire")
    take.add_argument("--pr", required=True)
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
            return acquire(args.holder, args.pr, args.ttl)
        return release(args.holder)
    except GitError as e:
        print(f"merge lock: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
