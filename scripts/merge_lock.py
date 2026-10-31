#!/usr/bin/env python3
"""The merge lock: agents merge to `main` one at a time, first-in, first-out (decisions 0075, 0145).

  scripts/merge_lock.py acquire --pr N | --train ID [--prs N ...] [--renew] [--holder ID] [--ttl MINUTES]
  scripts/merge_lock.py wait    --pr N | --train ID [--prs N ...] [--timeout MINUTES]  # acquire every 30 s
  scripts/merge_lock.py release [--holder ID]
  scripts/merge_lock.py status

Exit codes of `acquire` and `wait`: 0 the lock is yours; 1 busy (you are in the queue), or `wait`
timed out; 2 an error; 3 PR N is already merged or closed; 4 lost, for `acquire --renew` only:
you held the lock and it was taken over; 5 refused, `--pr N` in `train` mode (below).

In `train` mode only merge trains (`--train`) and the emergency-flip PR (labelled
`merge-train-stop`) take the lock; any other `--pr N` claim exits 5 and names `make mark-ready`
(decision 0204). A failed read of the mode (it reads as `lock`) or of the PR's labels lets the
claim through, so a GitHub outage never blocks an emergency flip.

The lock is the branch `merge-lock` on origin: an orphan commit with an empty tree whose message
(format `version: 2`, SPEC-0027 section 1) names the holder and the PR, or says `state: free`, and
lists the waiters in queue order. Pushes carry `--force-with-lease`, so the server creates the
branch only while it is absent and replaces it only while it still holds the commit the caller
read: of two racing writers exactly one wins, and one CAS updates the holder and the queue
together. Release writes the free marker instead of deleting the branch, because the session git
proxy refuses branch deletion (HTTP 403; T-1938). The empty tree keeps task and decision files off
the branch (tasks.py scans every origin branch), and CI does not run on it (ci.yml runs on pull
requests and on `main`).

A caller that finds the lock held joins the queue. A free, stale (TTL since `renewed`: the holder
died) or finished lock goes to the head of the queue, or to anyone when the queue is empty. A
holder that releases and acquires again joins at the tail. A PR that is already merged or closed
never holds the lock (decision 0097): `acquire --pr N` refuses (exit 3) when PR N is finished, and
the lock of a finished PR may be taken over whatever its age. That state is one GitHub read
through scripts/gh_client.py (decision 0148); when it fails the PR counts as open.

The holder is `CLAUDE_CODE_SESSION_ID` unless `--holder` (or `--train`) is given.
"""

from __future__ import annotations

import argparse
import dataclasses
import math
import os
import re
import subprocess
import sys
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import NamedTuple

import gh_client

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
LOCK_REF = "refs/heads/merge-lock"
# Where a read keeps the lock commit, so its sha and message always come from one fetch.
SEEN_REF = "refs/merge-lock/seen"
# A holder merges `main` in, waits for CI (about 5 minutes) and merges; 30 minutes is a dead holder.
TTL_MINUTES = 30
# The committer of lock commits, so a clone with no git identity can still take the lock.
IDENTITY = ("-c", "user.name=merge-lock", "-c", "user.email=merge-lock@invalid")

# SPEC-0027 section 1. Every limit below bounds what one lock commit can make a reader do.
VERSION = "2"
TRAIN = "train"
# What a held lock with a missing or malformed holder reads as: held, never free. It is not a valid
# holder id, so nobody can renew or release it; it goes stale after the TTL like any other.
UNKNOWN = "unknown"
ID = re.compile(r"[A-Za-z0-9._-]{1,64}")
PR = re.compile(r"[1-9][0-9]{0,8}")
UNIX = re.compile(r"[0-9]{1,12}")
BATCH_MAX = 5
QUEUE_MAX = 20
# A timestamp this far past the lock commit's committer time is clamped to it.
CLOCK_SLACK = 60
# A waiter that has not refreshed for this long has stopped waiting.
WAITER_TTL = 10 * 60
# A waiter rewrites its `seen` only when it is this old, to limit pushes.
REFRESH = 3 * 60
# A live head polls every 30 s, so a head that has not claimed a freed lock in this time has
# stopped waiting.
HEAD_CLAIM = 2 * 60
# `wait` polls every POLL seconds, for WAIT_MINUTES by default.
POLL = 30
WAIT_MINUTES = 45
# A PR's state is re-read at most every READ_EVERY polls: about 12 reads an hour per PR, against
# 60 an hour for unauthenticated API access.
READ_EVERY = 10
# `wait` retries git errors and an unreadable marker until its timeout, but gives up (exit 2) when
# the marker stays unreadable for this many polls in a row (a persistent format error).
UNREADABLE_POLLS = 5
# Waiters rewrite the commit when they refresh, so a holder's write can lose to a waiter's.
ATTEMPTS = 10

# What `busy_message` says to a caller that could not join a full queue.
QUEUE_FULL = f"queue full ({QUEUE_MAX} waiters)"
# The start of what `busy_message` says to a caller that is in the queue.
IN_QUEUE = "you are waiter"

HELD, BUSY, ERROR, DONE, LOST, REFUSED = 0, 1, 2, 3, 4, 5


class GitError(Exception):
    """A git command failed for a reason other than a lost race."""


class LockFormatError(ValueError):
    """A value that the lock commit format does not allow (a bad id, PR number or count)."""


@dataclass(frozen=True)
class Waiter:
    """One line of the queue."""

    id: str
    pr: str
    registered: int
    seen: int


@dataclass(frozen=True)
class Lock:
    """The lock commit as origin holds it (`sha` is "" for one not yet written)."""

    sha: str
    holder: str | None  # None when free
    pr: str = ""
    prs: tuple[str, ...] = ()
    renewed: int = 0
    freed: int = 0
    waiters: tuple[Waiter, ...] = ()

    @property
    def free(self) -> bool:
        return self.holder is None

    def age_minutes(self, now: float) -> float:
        return (now - self.renewed) / 60

    def is_stale(self, now: float, ttl_minutes: float) -> bool:
        return self.age_minutes(now) >= ttl_minutes


def valid_id(value: str) -> bool:
    return value != UNKNOWN and ID.fullmatch(value) is not None


def valid_pr(value: str, train: bool = False) -> bool:
    return PR.fullmatch(value) is not None or (train and value == TRAIN)


def git(*args: str, root: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(root), *args], stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False
    )
    if check and result.returncode != 0:
        raise GitError(f"git {' '.join(args)}: {result.stderr.strip()}")
    return result


def parse_lock(sha: str, committed: int, message: str) -> Lock:
    """The lock a commit message describes. Malformed lines are dropped one by one; a commit
    without `version` is version 1 (no queue, and the commit time is `renewed`)."""
    fields: dict[str, str] = {}
    waiter_lines: list[str] = []
    for line in message.splitlines()[1:]:
        key, sep, value = line.partition(": ")
        if not sep:
            continue
        if key == "waiter":
            waiter_lines.append(value)
        else:
            fields.setdefault(key, value)

    version_2 = fields.get("version") == VERSION
    renewed = clamp_stamp(fields.get("renewed"), committed, committed) if version_2 else committed
    waiters = parse_waiters(waiter_lines, committed) if version_2 else ()
    holder = fields.get("holder")
    if holder is None and fields.get("state") == "free":
        return Lock(sha, None, freed=clamp_stamp(fields.get("freed"), committed, committed), waiters=waiters)
    # Fail closed: a lock that is not plainly free is held, by `unknown` when the holder is bad.
    pr = fields.get("pr", "")
    pr = pr if valid_pr(pr, train=True) else ""
    prs = tuple(fields.get("prs", "").split())
    if pr != TRAIN or len(prs) > BATCH_MAX or not all(valid_pr(p) for p in prs):
        prs = ()
    holder = holder if holder is not None and valid_id(holder) else UNKNOWN
    return Lock(sha, holder, pr, prs, renewed, waiters=waiters)


def clamp_stamp(value: str | None, committed: int, fallback: int) -> int:
    """A timestamp read from a lock commit with committer time `committed`, or `fallback` when it
    is missing, malformed or more than CLOCK_SLACK past `committed`. Never clamped to the read
    time: that would keep a bad lock fresh forever."""
    if value is None or not UNIX.fullmatch(value) or int(value) > committed + CLOCK_SLACK:
        return fallback
    return int(value)


def parse_waiters(lines: Sequence[str], committed: int) -> tuple[Waiter, ...]:
    """The queue: at most QUEUE_MAX well-formed lines, one per id. An out-of-range timestamp
    reads as 0, so that waiter has expired."""
    waiters: dict[str, Waiter] = {}
    for line in lines[:QUEUE_MAX]:
        parts = line.split(" ")
        if len(parts) != 4 or not valid_id(parts[0]) or not valid_pr(parts[1], train=True):
            continue
        registered, seen = (clamp_stamp(p, committed, 0) for p in parts[2:])
        waiters.setdefault(parts[0], Waiter(parts[0], parts[1], registered, seen))
    return tuple(waiters.values())


def render(lock: Lock) -> str:
    """The commit message of `lock`, after checking every value against the format: a newline or
    any other character outside the regexes cannot reach the branch."""
    if len(lock.waiters) > QUEUE_MAX:
        raise LockFormatError(f"queue of {len(lock.waiters)} > {QUEUE_MAX}")
    lines = ["merge-lock", "", f"version: {VERSION}"]
    if lock.holder is None:
        lines += ["state: free", f"freed: {int(lock.freed)}"]
    else:
        # A holder or PR that read as malformed is written back as it read, so the lock stays held.
        check_claim(None if lock.holder == UNKNOWN else lock.holder, lock.pr or None, lock.prs)
        lines += [f"holder: {lock.holder}"] + ([f"pr: {lock.pr}"] if lock.pr else [])
        lines += [f"prs: {' '.join(lock.prs)}"] if lock.pr == TRAIN else []
        lines.append(f"renewed: {int(lock.renewed)}")
    for w in lock.waiters:
        check_claim(w.id, w.pr)
        lines.append(f"waiter: {w.id} {w.pr} {int(w.registered)} {int(w.seen)}")
    return "\n".join(lines) + "\n"


def check_claim(holder: str | None, pr: str | None, prs: Sequence[str] = ()) -> None:
    """Raises LockFormatError unless `holder` may hold or wait for the lock for `pr` (and `prs`).
    A field given as None is absent and not checked."""
    if holder is not None and not valid_id(holder):
        raise LockFormatError(f"bad holder id {holder!r}: want {ID.pattern}, not {UNKNOWN!r}")
    if pr is not None and not valid_pr(pr, train=True):
        raise LockFormatError(f"bad PR {pr!r}: want a positive number or {TRAIN!r}")
    if (prs and pr != TRAIN) or len(prs) > BATCH_MAX or not all(valid_pr(p) for p in prs):
        raise LockFormatError(f"bad prs {list(prs)!r}: a train lists at most {BATCH_MAX} PR numbers")


def read_lock(root: Path = ROOT) -> Lock | None:
    """The lock commit on origin (held, or the free marker), or None when the branch is absent."""
    fetched = git("fetch", "-q", "--no-tags", REMOTE, f"+{LOCK_REF}:{SEEN_REF}", root=root, check=False)
    if fetched.returncode != 0:
        if "couldn't find remote ref" in fetched.stderr:
            return None
        raise GitError(f"fetch {LOCK_REF}: {fetched.stderr.strip()}")
    body = git("log", "-1", "--format=%H%n%ct%n%B", SEEN_REF, root=root).stdout
    try:
        sha, committed, message = body.split("\n", 2)
        return parse_lock(sha, int(committed), message)
    except ValueError as e:
        raise GitError(f"malformed lock commit on {LOCK_REF}") from e


def held(lock: Lock | None) -> Lock | None:
    """`lock` if someone holds it, None when it is absent or free."""
    return lock if lock is not None and not lock.free else None


def lock_commit(lock: Lock, root: Path = ROOT) -> str:
    """A new orphan commit with an empty tree whose message is `lock`, validated."""
    message = render(lock)
    tree = git("mktree", root=root).stdout.strip()
    return git(*IDENTITY, "commit-tree", tree, "-m", message, root=root).stdout.strip()


def push_lease(expected: str, new: str, root: Path = ROOT) -> bool:
    """Sets the lock to the commit `new` only if origin still has `expected` there ("" for absent).
    False when another agent moved it first."""
    lease = f"--force-with-lease={LOCK_REF}:{expected}"
    result = git("push", "-q", lease, REMOTE, f"{new}:{LOCK_REF}", root=root, check=False)
    return result.returncode == 0


def write(lock: Lock | None, new: Lock, root: Path) -> bool:
    """Replaces `lock` by `new` with one CAS. True when written, or when nothing changed."""
    if lock is not None and dataclasses.replace(new, sha=lock.sha) == lock:
        return True
    return push_lease(lock.sha if lock else "", lock_commit(new, root), root)


def describe(lock: Lock, now: float, ttl_minutes: float) -> str:
    stale = " (stale: may be taken over)" if lock.is_stale(now, ttl_minutes) else ""
    pr = f"PR {lock.pr or '?'}" + (
        f" ({' '.join(lock.prs) or 'between batches'})" if lock.pr == TRAIN else ""
    )
    return f"held by {lock.holder} for {pr}, {lock.age_minutes(now):.0f} min ago{stale}"


def train_mode_refusal(pr: str, root: Path = ROOT) -> str | None:
    """Why a `--pr` claim is refused in `train` mode, or None when it may proceed (decision 0204).

    Sessions still on the per-PR flow after the switch to `train` queued ahead of the trains and
    stranded the ready PRs (2026-10-31: 27 per-PR holds against about 12 train holds). Only the
    emergency-flip PR merges under the lock in `train` mode (`steward` skill, "Emergency flip to
    `lock`" step 4). The mode is `make merge-mode`'s read; unreadable reads as `lock`, and an
    unreadable label lets the claim through, so neither failure can block a flip."""
    import merge_ready  # imports this module
    import merge_train  # imports this module

    if merge_train.read_mode(root) != merge_train.TRAIN:
        return None
    client = gh_client.from_remote(root, REMOTE)
    if client is None or not valid_pr(pr):
        return None
    try:
        labels = merge_ready.label_names(client.pull(pr))
    except gh_client.GhError:
        return None
    if merge_ready.STOP_LABEL in labels:
        return None
    return (
        f"merge lock refused: the merge train is on (`make merge-mode` says train), so PR {pr} does not "
        f"take the lock. Mark it ready with `make mark-ready PR={pr}` and move on; the train merges it "
        f"(work-loop step 3, decision 0204). Only the emergency-flip PR, labelled "
        f"{merge_ready.STOP_LABEL}, merges under the lock in train mode."
    )


def pr_finished(pr: str, root: Path = ROOT) -> bool | None:
    """Whether PR `pr` of origin's repository is merged or closed: True or False, None when the
    GitHub read fails (no network, no `gh`, a non-GitHub remote). The read is gh_client's."""
    client = gh_client.from_remote(root, REMOTE)
    if client is None or not valid_pr(pr):
        return None
    return client.pr_finished(pr)


def lock_finished(lock: Lock, finished: Callable[[str], bool]) -> bool:
    """Whether `lock`'s PR is merged or closed. A train lock is finished only when its `prs` are
    non-empty and all finished: between batches the merger holds it with none, and must not be
    taken over mid-run."""
    if lock.pr == TRAIN:
        return bool(lock.prs) and all(finished(p) for p in lock.prs)
    return finished(lock.pr)


def drop_head(lock: Lock, now: int) -> Lock:
    """`lock` without its head waiter. `freed` restarts, so the next head gets its own HEAD_CLAIM."""
    return dataclasses.replace(lock, waiters=lock.waiters[1:], freed=now if lock.free else lock.freed)


def prune(lock: Lock, now: int) -> Lock:
    """`lock` without expired waiters, and without a head that has not claimed a freed lock within
    HEAD_CLAIM."""
    lock = dataclasses.replace(lock, waiters=tuple(w for w in lock.waiters if now - w.seen <= WAITER_TTL))
    return drop_head(lock, now) if lock.free and lock.waiters and now - lock.freed > HEAD_CLAIM else lock


def join(lock: Lock, holder: str, pr: str, now: int) -> tuple[Lock, int | None]:
    """`lock` with `holder` in the queue (at the tail, or refreshed in place when its `seen` is
    REFRESH old), and its place, 1-based; None when the queue is full."""
    waiters = list(lock.waiters)
    for k, w in enumerate(waiters):
        if w.id == holder:
            if w.pr != pr or now - w.seen >= REFRESH:
                waiters[k] = dataclasses.replace(w, pr=pr, seen=now)
            return dataclasses.replace(lock, waiters=tuple(waiters)), k + 1
    if len(waiters) >= QUEUE_MAX:
        return lock, None
    waiters.append(Waiter(holder, pr, now, now))
    return dataclasses.replace(lock, waiters=tuple(waiters)), len(waiters)


def acquire(
    holder: str,
    pr: str,
    ttl_minutes: float = TTL_MINUTES,
    *,
    root: Path = ROOT,
    prs: Sequence[str] | None = None,
    renew: bool = False,
) -> int:
    """Takes the lock, renews it when `holder` already has it, or queues `holder`. 0 when held,
    1 when busy, 3 when PR `pr` is already merged or closed, 4 when a renewal lost the lock."""
    code, message = attempt(holder, pr, prs=prs, ttl_minutes=ttl_minutes, root=root, renew=renew)
    print(message)
    return code


class Read(NamedTuple):
    """One PR-finished read, and the poll it was made at."""

    poll: int
    finished: bool


class FinishedReads:
    """PR-finished reads cached per PR. Within one `acquire` (one poll) each PR is read once, out
    of the CAS retries; across `wait`'s polls a PR is re-read only every READ_EVERY polls, so a new
    head or holder is read at once and an unchanged one every 10th poll (SPEC-0027 section 1)."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.poll = 0
        self.read_at: dict[str, Read] = {}

    def __call__(self, pr: str) -> bool:
        last = self.read_at.get(pr)
        if last is None or self.poll - last.poll >= READ_EVERY:
            last = Read(self.poll, valid_pr(pr) and bool(pr_finished(pr, self.root)))
            self.read_at[pr] = last
        return last.finished

    def next_poll(self) -> None:
        self.poll += 1


def attempt(
    holder: str,
    pr: str,
    *,
    prs: Sequence[str] | None,
    ttl_minutes: float,
    root: Path,
    renew: bool,
    finished: FinishedReads | None = None,
) -> tuple[int, str]:
    """One `acquire`: its exit code and what to print. `prs` (a train's) replaces the lock's when
    given. `renew` says the caller holds the lock, so a lock that is not theirs is lost (4), not
    busy. `finished` carries cached reads from earlier polls."""
    check_claim(holder, pr, prs or ())
    finished = finished or FinishedReads(root)
    if pr != TRAIN and finished(pr):
        return DONE, f"merge lock refused: PR {pr} is already merged or closed"
    renewing = renew
    for _ in range(ATTEMPTS):
        lock = read_lock(root)
        now = int(time.time())
        pruned = prune(lock or Lock("", None), now)
        if pruned.holder == holder:
            renewing = True
            kept = pruned.prs if pr == TRAIN and prs is None else tuple(prs or ())
            new = dataclasses.replace(pruned, pr=pr, prs=kept, renewed=now)
            if write(lock, new, root):
                return HELD, f"merge lock renewed for PR {pr}"
            continue
        if renewing:
            current = describe(pruned, now, ttl_minutes) if pruned.holder else "free"
            return LOST, f"merge lock lost: it was taken over ({current})"
        live = (
            not pruned.free and not pruned.is_stale(now, ttl_minutes) and not lock_finished(pruned, finished)
        )
        head = pruned.waiters[0] if pruned.waiters else None
        if not live and head is not None and head.id != holder and head.pr != TRAIN and finished(head.pr):
            pruned = drop_head(pruned, now)
            head = pruned.waiters[0] if pruned.waiters else None
        if not live and (head is None or head.id == holder):
            queue = tuple(w for w in pruned.waiters if w.id != holder)
            new = Lock("", holder, pr, tuple(prs or ()), now, waiters=queue)
            if write(lock, new, root):
                taken = f", taking over from {pruned.holder}" if pruned.holder else ""
                return HELD, f"merge lock acquired for PR {pr}{taken}"
            continue
        queued, place = join(pruned, holder, pr, now)
        if place is not None and not write(lock, queued, root):
            continue
        return BUSY, busy_message(pruned, live, place, len(queued.waiters))
    return BUSY, f"merge lock busy: lost the race {ATTEMPTS} times"


def busy_message(lock: Lock, live: bool, place: int | None, waiting: int) -> str:
    """Who the lock is for, and the caller's place in the queue."""
    head = lock.waiters[0] if lock.waiters else None
    if live:
        ahead = f"held by {lock.holder} for PR {lock.pr or '?'}"
    else:
        ahead = f"reserved for {head.id if head else '?'} (PR {head.pr if head else '?'})"
    where = f"{IN_QUEUE} {place} of {waiting}" if place else QUEUE_FULL
    return f"merge lock busy: {ahead}; {where}"


def wait(
    holder: str,
    pr: str,
    *,
    prs: Sequence[str] | None = None,
    timeout_minutes: float = WAIT_MINUTES,
    ttl_minutes: float = TTL_MINUTES,
    root: Path = ROOT,
) -> int:
    """`acquire` every POLL seconds until the lock is held (0), the PR is finished (3), or the
    timeout (1). Prints only changed messages. A bad claim, or a queue that is full when the
    waiter first tries to join, exits 2 at once. A git error, a lost race or an unreadable marker
    (a racing write) is printed and retried at the next poll, keeping the queue place; the
    UNREADABLE_POLLS-th unreadable marker in a row is raised (exit 2)."""
    check_claim(holder, pr, prs or ())
    finished = FinishedReads(root)
    deadline = time.time() + timeout_minutes * 60
    last, unreadable, queued = "", 0, False
    while True:
        try:
            code, message = attempt(
                holder, pr, prs=prs, ttl_minutes=ttl_minutes, root=root, renew=False, finished=finished
            )
            unreadable = 0
        except GitError as e:
            code, message = BUSY, f"merge lock: {e} (retrying)"
        except LockFormatError as e:
            unreadable += 1
            if unreadable >= UNREADABLE_POLLS:
                raise
            code, message = BUSY, f"merge lock: {e} (retrying)"
        if code == BUSY and message.endswith(QUEUE_FULL) and not queued:
            print(message, flush=True)
            return ERROR
        queued = queued or (code == BUSY and IN_QUEUE in message)
        if message != last:
            print(message, flush=True)
            last = message
        if code != BUSY:
            return code
        if time.time() + POLL > deadline:
            print(f"merge lock: still waiting after {timeout_minutes:g} min; giving up", flush=True)
            return BUSY
        time.sleep(POLL)
        finished.next_poll()


def release(holder: str, root: Path = ROOT) -> int:
    """Frees the lock if `holder` has it, by writing the free marker and keeping the queue, so the
    head waiter takes it next. 0 when the lock is no longer `holder`'s, 1 when another agent holds
    it (it is never released for them)."""
    for attempt_no in range(ATTEMPTS):
        lock = read_lock(root)
        current = held(lock)
        if current is None:
            print("merge lock: not held")
            return 0
        if current.holder != holder:
            if attempt_no == 0:
                print(f"merge lock not yours: {describe(current, time.time(), TTL_MINUTES)}")
                return 1
            print("merge lock: taken over before release")
            return 0
        now = int(time.time())
        if write(lock, Lock("", None, freed=now, waiters=prune(current, now).waiters), root):
            print("merge lock released")
            return 0
    print(f"merge lock: release lost the race {ATTEMPTS} times", file=sys.stderr)
    return ERROR


def status(root: Path = ROOT) -> int:
    now = time.time()
    lock = read_lock(root)
    current = held(lock)
    print(f"merge lock {describe(current, now, TTL_MINUTES)}" if current else "merge lock: free")
    for k, w in enumerate(prune(lock, int(now)).waiters if lock else (), 1):
        print(f"  {k}. {w.id} for PR {w.pr}, waiting {(now - w.registered) / 60:.0f} min")
    return 0


def minutes(value: str) -> float:
    """An argparse type: a finite, non-negative number of minutes (no NaN, inf or negatives)."""
    number = float(value)
    if not (math.isfinite(number) and number >= 0):
        raise argparse.ArgumentTypeError(f"want a finite number of minutes >= 0, not {value!r}")
    return number


def main(argv: list[str], root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="command", required=True)
    take = commands.add_parser("acquire")
    take.add_argument("--renew", action="store_true", help="you hold the lock: exit 4 if it was taken over")
    queue = commands.add_parser("wait")
    queue.add_argument(
        "--timeout", type=minutes, default=WAIT_MINUTES, help="minutes before giving up (exit 1)"
    )
    for sub in (take, queue):
        claim = sub.add_mutually_exclusive_group(required=True)
        claim.add_argument("--pr", type=int)
        claim.add_argument("--train", metavar="ID", help="a merge train: holder ID, PR `train`")
        sub.add_argument(
            "--prs", type=int, nargs="*", help="a train's PRs still unmerged (replaces the lock's)"
        )
        sub.add_argument("--ttl", type=minutes, default=TTL_MINUTES, help="minutes before a lock is stale")
    drop = commands.add_parser("release")
    for sub in (take, queue, drop):
        sub.add_argument("--holder", default=os.environ.get("CLAUDE_CODE_SESSION_ID"))
    commands.add_parser("status")
    args = parser.parse_args(argv)
    try:
        if args.command == "status":
            return status(root)
        holder = getattr(args, "train", None) or args.holder
        if not holder:
            print("merge lock: no holder (set --holder or CLAUDE_CODE_SESSION_ID)", file=sys.stderr)
            return ERROR
        if args.command == "release":
            return release(holder, root)
        pr = TRAIN if args.train else str(args.pr)
        check_claim(holder, pr)
        if pr != TRAIN and (refusal := train_mode_refusal(pr, root)) is not None:
            print(refusal, file=sys.stderr)
            return REFUSED
        prs = None if args.prs is None else [str(p) for p in args.prs]
        if args.command == "wait":
            return wait(holder, pr, prs=prs, timeout_minutes=args.timeout, ttl_minutes=args.ttl, root=root)
        return acquire(holder, pr, args.ttl, root=root, prs=prs, renew=args.renew)
    except (GitError, LockFormatError) as e:
        print(f"merge lock: {e}", file=sys.stderr)
        return ERROR


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
