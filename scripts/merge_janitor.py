#!/usr/bin/env python3
"""The merge janitor: clears what the merge train leaves behind (decision 0224).

  scripts/merge_janitor.py [--dry-run] [--step] [--budget MIN]   # exit 0: done; 2: an error

The merger routine runs it once per run, after its train passes; any session may. It acts only in
`train` mode, on the open, non-draft PRs of this repository based on `main`, except those labelled
`merge-hold` (an owner's hold against its pushes and re-marks). A catch-up can build and run the
eval, so none starts, nor a renumber, after `--budget` minutes (25); the rest waits for the next
run. A branch is *idle* for the time since its
current head was pushed (the newest push of that head in the activity API); a PR whose idle time
cannot be read is skipped. Per PR, at most one action:

- **Ejected** (`merge-train-ejected`; the reason is the merger's newest `ready-to-merge removed:`
  comment by a writer) and idle 1 h or more:
  - `conflict with main` whose merge with `main` conflicts only in the eval-gated baseline lists:
    `make catch-up` (scripts/merge_catchup.py, decision 0207);
  - `id collision`: `make renumber` (scripts/merge_renumber.py).

  The janitor comments a note naming the head it is about to push, before the push (a failed
  note leaves no unrecorded push). CI takes minutes, so it does not
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

After the PRs, escalation step 2 (`Janitor.escalate`): when the oldest stranded PR
(`merge_train_stats.stranded_since`) was marked 8 h ago and nothing merged in the last 6 h, it
files a P0 for the owner through the merge train's task filing, once per head of that PR.

Its own notes end with a `merge-janitor v1` block (head and action); only notes by writers count.
Reads go through scripts/gh_client.py and merge_ready; writes through the same client (`gh api`),
or with `--step` through `merge_step.StepClient` (decision 0169): each write, a mark's comment and
labels included, is printed as one JSON action of the merge train's schema, which the session
performs with the GitHub MCP tools and acks with `make merge-train-ack SEQ=<n>`. Git pushes stay
direct.
Pushes and marks need a checkout with no local changes; `--dry-run` reads only and lists actions.
"""

from __future__ import annotations

import argparse
import math
import os
import re
import sys
import time
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
import merge_step
import merge_train
import merge_train_filing
import merge_train_stats
from gh_client import Client, GhError, int_of, iso, label_names, text_of, when
from merge_lock import GitError, git

ROOT = Path(__file__).resolve().parent.parent
MECHANICAL_IDLE = timedelta(hours=1)
STALE_IDLE = timedelta(hours=2)
ORPHAN_IDLE = timedelta(hours=4)
# A catch-up or renumber can build and run the eval (`make eval-baseline`); none starts after
# this much of a run, and what is left waits for the next merger run (30 min later).
BUDGET = timedelta(minutes=25)
# The owner's P0 (escalation step 2, decision 0224): the oldest stranded PR this old, with no
# merge in the last QUIET.
ESCALATE_AFTER = timedelta(hours=8)
QUIET = timedelta(hours=6)
# An owner's or reviewer's hold: the janitor never touches a PR with this label (no push, no mark).
HOLD_LABEL = merge_ready.HOLD_LABEL
# Pushes older than this are not read; the newest push of a head is on the first page anyway.
ACTIVITY_WINDOW = timedelta(days=1)

# The merger's ejection comment (merge_train.Merger.unlabel and eject_red) and its reasons.
REMOVED = f"merge train: {merge_ready.READY_LABEL} removed: "
CONFLICT, COLLISION, CI_RED, OTHER = "conflict with main", "id collision", "ci-red", "other"

# The janitor's actions; all but MARK and UNLABEL are recorded in its notes.
CATCH_UP, RENUMBER, ORPHANED, REFUSED, MARK, UNLABEL = (
    "catch-up",
    "renumber",
    "orphaned",
    "refused",
    "mark",
    "unlabel",
)
ESCALATED = "escalated"
NOTE_OPEN = "<!-- merge-janitor v1"
NOTE = re.compile(r"<!-- merge-janitor v1 head: ([0-9a-f]{40}) action: ([a-z-]{1,20}) -->\Z")


def note_block(head: str, action: str) -> str:
    return f"{NOTE_OPEN} head: {head} action: {action} -->"


def parse_note(body: str) -> tuple[str, str] | None:
    """(head, action) of the note block that ends comment `body` (the session footer aside)."""
    found = NOTE.search(merge_ready.without_footer(body).rstrip())
    return (found[1], found[2]) if found else None


# The reason lines the merger writes, matched exactly (merge_train.build, id_collision, eject_red).
ID = r"(?:T-\d{4}|\d{4})"
REASONS = (
    (CONFLICT, re.compile(r"conflict with main")),
    (
        COLLISION,
        re.compile(
            rf"id collision with (?:{ID}(?:, {ID})*|a task id or decision number); renumber per decision 0121 \(2\)"
        ),
    ),
    (CI_RED, re.compile(r"train CI red, confirmed on this PR alone on main")),
)


def reason_kind(reason: str) -> str:
    """CONFLICT, COLLISION, CI_RED or OTHER for the first line of an ejection comment's reason."""
    return next((kind for kind, line in REASONS if line.fullmatch(reason)), OTHER)


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
        # The note only says the janitor pushed this head; MARK's `plan_mark(carry=True)` re-checks
        # the carry rule on it, so a head that is not a catch-up or renumber is refused there.
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
    base, a `merge-hold` label, or no ejection label and no marker)."""
    pr, head, branch = int_of(pull, "number"), text_of(pull, "head", "sha"), text_of(pull, "head", "ref")
    if (
        HOLD_LABEL in label_names(pull)
        or gh_client.field_of(pull, "draft") is not False
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
        return merge_catchup.conflicts_outside(self.repo, state.head, main) == []

    def carries(self, state: State, prev: str) -> bool:
        self.fetch(state)
        return self.repo.carry_problem(prev, state.head) is None


# Called with the merge commit's sha before it is pushed: the janitor's note goes first.
Announce = Callable[[str], None]


class Effects(Protocol):
    """The janitor's pushes, marks and P0 filing; a push returns its head, and each may raise a
    refusal."""

    def catch_up(self, state: State, announce: Announce) -> str: ...
    def renumber(self, state: State, announce: Announce) -> str: ...
    def mark(self, state: State) -> None: ...
    def file_p0(self, log: list[str], title: str, context: str) -> bool: ...  # whether it was filed


class Checkout:
    """`Effects` in the clean checkout `root`: the catch-up, the renumber, `mark-ready --carry`, and
    the merge train's task filing (a task-only PR)."""

    def __init__(self, client: Client, root: Path, by: str) -> None:
        self.client, self.root, self.by = client, root, by

    def catch_up(self, state: State, announce: Announce) -> str:
        return merge_catchup.catch_up(self.root, state.branch, before_push=announce)

    def renumber(self, state: State, announce: Announce) -> str:
        return merge_renumber.renumber(self.root, state.branch, before_push=announce)

    def file_p0(self, log: list[str], title: str, context: str) -> bool:
        return merge_train_filing.file_tasks(self.root, self.client, log, [("P0", title, context)])

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
    # `clock()` past `deadline` (both monotonic seconds) starts no new catch-up or renumber.
    deadline: float = math.inf
    clock: Callable[[], float] = time.monotonic

    def note(self, pr: int, head: str, action: str, text: str) -> None:
        self.client.comment(pr, f"merge janitor: {text}\n\n{note_block(head, action)}")

    def act(self, s: State, action: Action) -> None:
        """Performs `action` on PR `s`, or with `dry_run` only logs it."""
        if action.kind in (CATCH_UP, RENUMBER) and self.clock() >= self.deadline:
            self.log.append(f"#{s.pr}: {action.kind} left for the next run: this run's budget is spent")
            return
        self.log.append(f"#{s.pr}: {action.kind}: {action.why}")
        if self.dry_run:
            return
        if action.kind == ORPHANED:
            self.note(s.pr, s.head, ORPHANED, (
                f"this PR is orphaned ({action.why}; branch idle {hours(s.idle)}). It needs an opus run or "
                'the owner: the steward skill\'s "Orphaned agent PRs" (decision 0224).'
            ))
            return
        if action.kind == UNLABEL:
            self.client.remove_label(s.pr, merge_ready.READY_LABEL)
            self.client.comment(
                s.pr, f"merge janitor: {merge_ready.READY_LABEL} removed: {action.why}; mark again after CI and review"
            )
            return

        def announce(head: str) -> None:
            # Before the push: a note that fails leaves no push nobody recorded.
            self.note(s.pr, head, action.kind, (
                f"pushing `{head}` (`make {action.kind}`, decision 0224). Once CI is green on it, a later "
                "janitor run marks this PR ready again with the carried verdicts (`make mark-ready CARRY=1`)."
            ))

        try:
            if action.kind == MARK:
                self.effects.mark(s)
                return
            push = self.effects.catch_up if action.kind == CATCH_UP else self.effects.renumber
            self.log.append(f"#{s.pr}: pushed {push(s, announce)}")
        except REFUSALS as refusal:
            self.log.append(f"#{s.pr}: {action.kind} refused: {refusal}")
            self.note(s.pr, s.head, REFUSED, f"`{action.kind}` refused: {refusal}. Left to the PR's worker.")

    def run(self, now: datetime) -> int:
        """One pass over the open PRs, then the escalation check; returns how many failed with an
        error (each is logged, and the pass goes on with the next PR)."""
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
        try:
            self.escalate(pulls, authors, now)
        except (GhError, GitError) as error:
            errors += 1
            self.log.append(f"escalation check: error: {error}")
        return errors

    def escalate(self, pulls: Sequence[object], authors: merge_ready.Authors, now: datetime) -> None:
        """Escalation step 2 (decision 0224), checked every merger run: when the oldest stranded PR
        was marked ESCALATE_AFTER ago and nothing merged in the last QUIET, file a P0 for the
        owner once per head of that PR (a merge-janitor note records it). The steward runs only
        twice a day, so this is the check that does not wait for it."""
        stranded = merge_train_stats.stranded_since(self.client, now)
        if not stranded:
            return
        pr, marked = min(stranded, key=lambda found: found[1])
        if now - marked < ESCALATE_AFTER:
            return
        quiet_since = now - QUIET
        merged = [p for p in self.client.updated_pulls(iso(quiet_since)) if (at := when(text_of(p, "merged_at"))) and at >= quiet_since]
        if merged:
            return
        pull = next((p for p in pulls if int_of(p, "number") == pr), None)
        head = text_of(pull, "head", "sha")
        comments = [c for c in self.client.comments(pr) if authors.writer(c)]
        if any(parse_note(text_of(c, "body")) == (head, ESCALATED) for c in comments) or not gh_client.SHA.fullmatch(head):
            return
        title = f"Merge train: nothing merged in {hours(QUIET)} with #{pr} stranded {hours(now - marked)}"
        self.log.append(f"#{pr}: escalate to the owner: {title}")
        if self.dry_run:
            return
        context = (
            f"The janitor found #{pr} marked ready on its current head `{head}` since {iso(marked)}, not "
            f"ejected and unmerged, and no PR merged since {iso(quiet_since)} (decision 0224, escalation "
            "step 2). For the owner: find why the merger's passes fail; flip to `lock` only with the "
            "owner's approval."
        )
        if not self.effects.file_p0(self.log, title, context):
            return  # no note: the next run tries again
        self.note(pr, head, ESCALATED, f"filed a P0 for the owner: {title} (decision 0224).")


def hours(idle: timedelta | None) -> str:
    return f"{(idle or timedelta(0)).total_seconds() / 3600:.1f} h"


def main(argv: Sequence[str] | None = None, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="list the actions; push and write nothing")
    parser.add_argument("--by", default=os.environ.get("CLAUDE_CODE_SESSION_ID", ""), help="the marker id")
    parser.add_argument(
        "--step",
        action="store_true",
        help="print each GitHub write as a step-mode action and wait for its ack",
    )
    parser.add_argument(
        "--budget",
        type=float,
        default=BUDGET.total_seconds() / 60,
        help="minutes after which no catch-up or renumber starts (the rest waits for the next run)",
    )
    args = parser.parse_args(argv)
    if not args.dry_run and not merge_lock.valid_id(args.by):
        print("merge_janitor: no valid marker id: pass --by or set CLAUDE_CODE_SESSION_ID", file=sys.stderr)
        return 2
    real = gh_client.from_remote(root)
    if real is None:
        print("merge_janitor: origin is not a github.com remote", file=sys.stderr)
        return 2
    mode = merge_train.read_mode(root)
    if mode != merge_train.TRAIN:
        print(f"merge_janitor: the mode is {mode}; the janitor acts only in train mode")
        return 0
    # Step mode (decision 0169): every write, the marks' included, is an action the session
    # performs and acks with `make merge-train-ack`; reads and git pushes stay direct.
    client: Client = merge_step.StepClient(real, merge_step.AckFile(root)) if args.step else real
    repo = merge_ready.Repo(root)
    janitor = Janitor(client, GitFacts(client, repo), Checkout(client, root, args.by), args.dry_run)
    janitor.deadline = janitor.clock() + args.budget * 60
    try:
        errors = janitor.run(datetime.now(UTC))
    except merge_step.StepError as error:
        print("\n".join(janitor.log))
        print(f"merge_janitor: step mode aborted: {error}", file=sys.stderr)
        return 2
    except GhError as error:
        print(f"merge_janitor: {error}", file=sys.stderr)
        return 2
    print("\n".join(janitor.log) or "merge janitor: nothing to do")
    return 2 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
