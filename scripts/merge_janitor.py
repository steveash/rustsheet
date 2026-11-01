#!/usr/bin/env python3
"""The merge janitor: clears what the merge train leaves behind (decision 0224).

  scripts/merge_janitor.py [--dry-run]   # exit 0: done (with --dry-run: the actions listed); 2: error

The merger routine runs it after every pass; any session may. It acts only in `train` mode, on the
open, non-draft PRs of this repository based on `main`. A branch is *idle* for the time since its
current head was pushed (the newest push of that head in the activity API); a PR whose idle time
cannot be read is skipped. Per PR, at most one action:

- **Ejected** (`merge-train-ejected`; the reason is the merger's newest `ready-to-merge removed:`
  comment by a writer) and idle 1 h or more:
  - `conflict with main` whose merge with `main` conflicts only in the eval-gated baseline lists:
    `make catch-up` (scripts/merge_catchup.py, decision 0207);
  - `id collision`: `make renumber` (scripts/merge_renumber.py).

  The janitor then comments a note naming the head it pushed. CI takes minutes, so it does not
  wait: a later run finds the note, and once CI is green on that head, marks the PR again with the
  carried verdicts (`make mark-ready CARRY=1`, through `merge_ready.plan_mark`).
- **Orphaned**: an ejected PR the janitor cannot clear (`ci-red`, a real conflict, a refused
  catch-up, renumber or marking, or red CI on its own push), idle 4 h or more, gets one comment per
  head saying it needs an opus run or the owner (the steward skill's "Orphaned agent PRs").
- **Stale marker**: not ejected, and the newest marker by a writer names an older head, idle 2 h
  or more. When the head is a merge of `main` onto the marked head that the carry rule accepts
  (`merge_ready.Repo.carry_problem`, catch-ups and renumbers included) and CI is green on it, the
  PR is marked again with `CARRY`; when the carry rule refuses it, CI is red or a marking was
  refused, `ready-to-merge` is removed, as V2 does. Pending CI waits.

Its own notes end with a `merge-janitor v1` block (head and action); only notes by writers count.
Reads go through scripts/gh_client.py and merge_ready; writes through the same client (`gh api`).
Pushes and marks need a checkout with no local changes; `--dry-run` reads only and lists actions.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Protocol

import gh_client
import id_renumber
import merge_catchup
import merge_lock
import merge_ready
import merge_renumber
import merge_train
import visible_lock
from gh_client import Client, GhError, int_of, label_names, text_of
from merge_lock import GitError, git
from merge_train_stats import iso, when

ROOT = Path(__file__).resolve().parent.parent
MECHANICAL_IDLE = timedelta(hours=1)
STALE_IDLE = timedelta(hours=2)
ORPHAN_IDLE = timedelta(hours=4)
# Pushes older than this are not read; the newest push of a head is on the first page anyway.
ACTIVITY_WINDOW = timedelta(days=1)

# The merger's ejection comment (merge_train.Merger.unlabel and eject_red) and its reasons.
REMOVED = f"merge train: {merge_ready.READY_LABEL} removed: "
CONFLICT, COLLISION, CI_RED, OTHER = "conflict with main", "id collision", "ci-red", "other"

# The janitor's actions; the first five are recorded in its notes.
CATCH_UP, RENUMBER, ORPHANED, REFUSED, MARK, UNLABEL = (
    "catch-up",
    "renumber",
    "orphaned",
    "refused",
    "mark",
    "unlabel",
)
NOTE_OPEN = "<!-- merge-janitor v1"
NOTE = re.compile(r"<!-- merge-janitor v1 head: ([0-9a-f]{40}) action: ([a-z-]{1,20}) -->\Z")


def note_block(head: str, action: str) -> str:
    return f"{NOTE_OPEN} head: {head} action: {action} -->"


def parse_note(body: str) -> tuple[str, str] | None:
    """(head, action) of the note block that ends comment `body` (the session footer aside)."""
    found = NOTE.search(merge_ready.without_footer(body).rstrip())
    return (found[1], found[2]) if found else None


def reason_kind(reason: str) -> str:
    """CONFLICT, COLLISION, CI_RED or OTHER for an ejection comment's reason."""
    if reason.startswith(CONFLICT):
        return CONFLICT
    if reason.startswith(COLLISION):
        return COLLISION
    return CI_RED if reason.startswith("train CI red") or "reason: ci-red" in reason else OTHER


@dataclass
class State:
    """What the janitor read about one PR."""

    pr: int
    branch: str
    head: str
    labels: list[str]
    idle: timedelta | None
    reason: str = ""
    marker: merge_ready.Marker | None = None
    # The actions of the janitor's notes on the current head.
    done: set[str] = field(default_factory=set)

    @property
    def ejected(self) -> bool:
        return merge_ready.EJECTED_LABEL in self.labels


@dataclass(frozen=True)
class Action:
    pr: int
    kind: str
    why: str


class Facts(Protocol):
    """The git and CI reads `decide` needs, made only when it needs them."""

    def ci(self, state: State) -> str: ...  # merge_ready.GREEN, PENDING or RED on the head
    def lists_only(self, state: State) -> bool: ...  # the merge with main conflicts only in the lists
    def carries(self, state: State, prev: str) -> bool: ...  # the carry rule accepts head over prev


def decide(s: State, facts: Facts) -> Action | None:
    """The one action for PR `s` (module docstring), or None."""
    if s.idle is None:
        return None
    if s.ejected:
        return decide_ejected(s, facts)
    if s.marker is None or s.marker.head == s.head or s.idle < STALE_IDLE:
        return None
    labelled = merge_ready.READY_LABEL in s.labels
    if REFUSED not in s.done and facts.carries(s, s.marker.head):
        state = facts.ci(s)
        if state == merge_ready.GREEN:
            return Action(s.pr, MARK, f"the head is a carry-over merge onto the marked {s.marker.head}")
        if state == merge_ready.PENDING:
            return None
    return Action(s.pr, UNLABEL, f"readiness invalidated by a push (now {s.head})") if labelled else None


def decide_ejected(s: State, facts: Facts) -> Action | None:
    idle = s.idle or timedelta(0)
    pushed = s.done & {CATCH_UP, RENUMBER}
    kind = reason_kind(s.reason)
    if pushed and REFUSED not in s.done:
        state = facts.ci(s)
        if state == merge_ready.GREEN:
            return Action(s.pr, MARK, f"CI is green on the janitor's {'/'.join(sorted(pushed))} push")
        if state == merge_ready.PENDING:
            return None
        why = "CI is red on the janitor's push"
    elif not pushed and REFUSED not in s.done and idle >= MECHANICAL_IDLE and kind == COLLISION:
        return Action(s.pr, RENUMBER, s.reason)
    elif (
        not pushed
        and REFUSED not in s.done
        and idle >= MECHANICAL_IDLE
        and kind == CONFLICT
        and facts.lists_only(s)
    ):
        return Action(s.pr, CATCH_UP, "the conflicts with main are only in the eval-gated baseline lists")
    else:
        why = "a refused janitor step" if REFUSED in s.done else f"ejected: {s.reason or 'no reason found'}"
    if ORPHANED in s.done or idle < ORPHAN_IDLE:
        return None
    return Action(s.pr, ORPHANED, why)


def idle_for(client: Client, branch: str, head: str, now: datetime) -> timedelta | None:
    """How long ago `head` was last pushed to `branch` (the activity API); None when unknown."""
    try:
        pushes = client.activity(branch, iso(now - ACTIVITY_WINDOW))
    except GhError:
        return None
    times = [at for p in pushes if text_of(p, "after") == head and (at := when(text_of(p, "timestamp")))]
    return now - max(times) if times else None


def read_state(client: Client, pull: object, authors: merge_ready.Authors, now: datetime) -> State | None:
    """The State of one open PR, or None when the janitor leaves it alone (a draft, a fork, another
    base, or no ejection label and no marker)."""
    pr, head, branch = int_of(pull, "number"), text_of(pull, "head", "sha"), text_of(pull, "head", "ref")
    if (
        gh_client.field_of(pull, "draft") is not False
        or text_of(pull, "base", "ref") != merge_ready.BASE_BRANCH
        or text_of(pull, "head", "repo", "full_name") != text_of(pull, "base", "repo", "full_name")
        or not gh_client.SHA.fullmatch(head)
    ):
        return None
    comments = [c for c in client.comments(pr) if authors.writer(c)]
    markers = merge_ready.writer_markers(comments, authors)
    labels = label_names(pull)
    if merge_ready.EJECTED_LABEL not in labels and not markers:
        return None
    reasons = [text_of(c, "body")[len(REMOVED) :] for c in comments if text_of(c, "body").startswith(REMOVED)]
    done = {note[1] for c in comments if (note := parse_note(text_of(c, "body"))) and note[0] == head}
    try:
        branch = gh_client.branch_name(branch)
    except GhError:
        return None
    return State(
        pr,
        branch,
        head,
        labels,
        idle_for(client, branch, head, now),
        reasons[-1].split("\n", 1)[0].strip() if reasons else "",
        markers[-1] if markers else None,
        done,
    )


class GitFacts:
    """`Facts` from the checkout `root` and the client (merge_ready's reads)."""

    def __init__(self, client: Client, repo: merge_ready.Repo) -> None:
        self.client, self.repo, self.fetched = client, repo, False

    def fetch(self, state: State) -> None:
        if not self.fetched:
            self.repo.fetch_base()
            self.fetched = True
        if self.repo.fetch_head(state.pr) != state.head:
            raise GitError(f"#{state.pr}'s head moved while the janitor read it")

    def ci(self, state: State) -> str:
        self.fetch(state)
        return merge_ready.head_state(self.client, self.client.pull(state.pr), state.head, self.repo)[1]

    def lists_only(self, state: State) -> bool:
        self.fetch(state)
        main = f"refs/merge-ready/{merge_ready.BASE_BRANCH}"
        merged = id_renumber.merge_tree(self.repo.root, state.head, main)
        lists = self.repo.checked_baselines() | {visible_lock.RETIRED_LOCK}
        return merged is not None and all(p in lists for p in merged[1])

    def carries(self, state: State, prev: str) -> bool:
        self.fetch(state)
        return self.repo.carry_problem(prev, state.head) is None


class Effects(Protocol):
    """The janitor's pushes and marks; each returns the pushed head or raises a refusal."""

    def catch_up(self, state: State) -> str: ...
    def renumber(self, state: State) -> str: ...
    def mark(self, state: State) -> None: ...


class Checkout:
    """`Effects` in the clean checkout `root`: the catch-up, the renumber, and `mark-ready --carry`."""

    def __init__(self, client: Client, root: Path, by: str) -> None:
        self.client, self.root, self.by = client, root, by

    def catch_up(self, state: State) -> str:
        return merge_catchup.catch_up(self.root, state.branch)

    def renumber(self, state: State) -> str:
        return merge_renumber.renumber(self.root, state.branch)

    def mark(self, state: State) -> None:
        """`make mark-ready CARRY=1` with the PR head checked out; the checkout is restored."""
        if git("status", "--porcelain", "--untracked-files=no", root=self.root).stdout.strip():
            raise merge_ready.Refusal("the checkout has local changes")
        on = git("symbolic-ref", "-q", "--short", "HEAD", root=self.root, check=False).stdout.strip()
        start = on or git("rev-parse", "HEAD", root=self.root).stdout.strip()
        repo = merge_ready.Repo(self.root)
        try:
            git("checkout", "-q", "--detach", repo.fetch_head(state.pr), root=self.root)
            plan = merge_ready.plan_mark(self.client, state.pr, repo, None, True, self.by)
            merge_ready.apply(self.client, plan)
        finally:
            git("checkout", "-q", "-f", *([] if on else ["--detach"]), start, root=self.root, check=False)


REFUSALS = (merge_catchup.Refused, merge_ready.Refusal)


@dataclass
class Janitor:
    client: Client
    facts: Facts
    effects: Effects
    dry_run: bool = False
    log: list[str] = field(default_factory=list)

    def note(self, pr: int, head: str, action: str, text: str) -> None:
        self.client.comment(pr, f"merge janitor: {text}\n\n{note_block(head, action)}")

    def act(self, s: State, action: Action) -> None:
        """Performs `action` on PR `s`, or with `dry_run` only logs it."""
        self.log.append(f"#{s.pr}: {action.kind}: {action.why}")
        if self.dry_run:
            return
        if action.kind == ORPHANED:
            self.note(
                s.pr,
                s.head,
                ORPHANED,
                (
                    f"this PR is orphaned ({action.why}; branch idle {hours(s.idle)}). It needs an opus run or "
                    'the owner: the steward skill\'s "Orphaned agent PRs" (decision 0224).'
                ),
            )
            return
        if action.kind == UNLABEL:
            self.client.remove_label(s.pr, merge_ready.READY_LABEL)
            self.client.comment(
                s.pr, f"merge janitor: {merge_ready.READY_LABEL} removed: {action.why}; mark again after CI and review"
            )
            return
        step: Callable[[State], object] = {
            CATCH_UP: self.effects.catch_up,
            RENUMBER: self.effects.renumber,
            MARK: self.effects.mark,
        }[action.kind]
        try:
            pushed = step(s)
        except REFUSALS as refusal:
            self.log.append(f"#{s.pr}: {action.kind} refused: {refusal}")
            self.note(s.pr, s.head, REFUSED, f"`{action.kind}` refused: {refusal}. Left to the PR's worker.")
            return
        if isinstance(pushed, str):
            self.log.append(f"#{s.pr}: pushed {pushed}")
            self.note(
                s.pr,
                pushed,
                action.kind,
                (
                    f"pushed `{pushed}` (`make {action.kind}`, decision 0224). Once CI is green on it, a later "
                    "janitor run marks this PR ready again with the carried verdicts (`make mark-ready CARRY=1`)."
                ),
            )

    def run(self, now: datetime) -> int:
        """One pass over the open PRs; returns how many failed with an error (each is logged, and
        the pass goes on with the next PR)."""
        pulls = self.client.pulls()
        owner_type = text_of(pulls[0], "base", "repo", "owner", "type") if pulls else ""
        authors = merge_ready.Authors(self.client, owner_type == "User")
        errors = 0
        for pull in pulls:
            try:
                state = read_state(self.client, pull, authors, now)
                action = decide(state, self.facts) if state is not None else None
                if state is not None and action is not None:
                    self.act(state, action)
            except (GhError, GitError, id_renumber.RenumberError) as error:
                errors += 1
                self.log.append(f"#{int_of(pull, 'number')}: error: {error}")
        return errors


def hours(idle: timedelta | None) -> str:
    return f"{(idle or timedelta(0)).total_seconds() / 3600:.1f} h"


def main(argv: Sequence[str] | None = None, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="list the actions; push and write nothing")
    parser.add_argument("--by", default=os.environ.get("CLAUDE_CODE_SESSION_ID", ""), help="the marker id")
    args = parser.parse_args(argv)
    if not args.dry_run and not merge_lock.valid_id(args.by):
        print("merge_janitor: no valid marker id: pass --by or set CLAUDE_CODE_SESSION_ID", file=sys.stderr)
        return 2
    client = gh_client.from_remote(root)
    if client is None:
        print("merge_janitor: origin is not a github.com remote", file=sys.stderr)
        return 2
    mode = merge_train.read_mode(root)
    if mode != merge_train.TRAIN:
        print(f"merge_janitor: the mode is {mode}; the janitor acts only in train mode")
        return 0
    repo = merge_ready.Repo(root)
    janitor = Janitor(client, GitFacts(client, repo), Checkout(client, root, args.by), args.dry_run)
    try:
        errors = janitor.run(datetime.now(UTC))
    except GhError as error:
        print(f"merge_janitor: {error}", file=sys.stderr)
        return 2
    print("\n".join(janitor.log) or "merge janitor: nothing to do")
    return 2 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
