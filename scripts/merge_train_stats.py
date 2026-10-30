#!/usr/bin/env python3
"""`make merge-train-stats [SINCE=<date>]`: the merge train's metrics (SPEC-0027 "Metrics" and
"Daily thresholds"), the shadow agreement score (section 4, "When a shadow prediction agrees") and
T-2991's evidence and load bar (SPEC-0027 "Rollout", "Shadow evidence for T-2991"); T-3076.

Every read goes through scripts/gh_client.py. Force-pushed branches keep no history, so the train
and lock histories come from the activity API (each push's `after` commit, read through the commits
API), and CI from the workflow-run and job lists. Whether a later PR head is a clean merge of `main`
or renumbers an id file is a git question, answered from `refs/pull/<n>/head` in the local clone.

Definitions the spec leaves open, fixed here:
- a merged PR's *start* is its first green `pull_request` ci.yml run on the head that merged; its
  *arrival hour* is the clock hour (UTC) of that start, bucketed by how many merged PRs started in it;
- *merge-stage CI runs* are the ci.yml runs created while the merge lock was held for the PR (on its
  branch) or for a train (on `merge-train/current`), per PR merged in the window;
- a *train* is one batch on one `main`: the runs of a train branch on the same base whose batch is a
  subset of the first run's. Its extra runs are bisection runs (or one infrastructure retry);
- a prediction whose 24 h have not settled it either way is `unsettled` and, like `defer`, `no
  result` and `eject:invalidated`, not scored.
"""

from __future__ import annotations

import argparse
import math
import re
import subprocess
import sys
from collections import Counter
from collections.abc import Iterable, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Protocol

import gh_client
import merge_lock
import merge_ready
from gh_client import SHA, Client, GhError
from merge_ready import field_of, int_of, text_of
from merge_train import TRAIN_BRANCH
from merge_train_shadow import NO_RESULT, PREDICTION_HEADER, SHADOW_BRANCH

ROOT = Path(__file__).resolve().parent.parent
DAY = timedelta(hours=24)
LOCK_BRANCH = "merge-lock"
TRAIN_HEADER = re.compile(r"merge train on main ([0-9a-f]{40})")
BATCH_LINE = re.compile(r"#([1-9]\d{0,8}) ([0-9a-f]{40})")
OUTCOME_LINE = re.compile(r"#([1-9]\d{0,8}) (merge|defer|eject:[a-z-]+|no result)")
TREE_LINE = re.compile(r"tree-check: (match|mismatch|n/a)")
RENUMBERED = ("docs/tasks/", "docs/decisions/")

# Section 4, "When a shadow prediction agrees": the scored predictions, and the verdicts.
SCORED = ("merge", "eject:ci-red", "eject:conflict", "eject:id-collision")
AGREE, DISAGREE, UNSETTLED = "agree", "disagree", "unsettled"
# How a PR commit after a predicted head changed it.
CLEAN, CONFLICT, RENUMBER, CHANGE = "clean merge", "conflict merge", "renumber", "change"

# "Shadow evidence for T-2991": the bar.
BAR_TRAINS = 10
BAR_FRESH = timedelta(hours=2)
BAR_AGREE = 0.9
LOAD_BEFORE = timedelta(hours=48)
LOAD_WALL = 1.2
LOAD_QUEUE = 60.0
# "Metrics" and "Daily thresholds".
STRANDED_AFTER = timedelta(hours=2)
LATENCY_P90_MAX = 60.0
TRAIN_RUNS_MAX = 6
TRAINS_OVER_MAX = 2
BUCKETS = (("under 3/h", 0, 2), ("3-6/h", 3, 6), ("over 6/h", 7, math.inf))


def when(value: str) -> datetime | None:
    """A GitHub timestamp (`2026-10-29T21:58:36Z`) as an aware datetime; None when malformed."""
    try:
        moment = datetime.fromisoformat(value)
    except ValueError:
        return None
    return moment if moment.tzinfo is not None else None


def iso(moment: datetime) -> str:
    return moment.astimezone(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def percentile(values: Sequence[float], q: float) -> float | None:
    """The nearest-rank `q` percentile (0 < q <= 1); None for no values."""
    if not values:
        return None
    ordered = sorted(values)
    return ordered[max(0, math.ceil(q * len(ordered)) - 1)]


def minutes(seconds: float | None) -> str:
    return "n/a" if seconds is None else f"{seconds / 60:.1f} min"


# Predictions.
@dataclass(frozen=True)
class Prediction:
    """One `merge train shadow prediction [skip ci]` commit (decision 0155)."""

    sha: str
    at: datetime
    heads: dict[int, str]
    outcomes: dict[int, str]
    tree_check: str

    @property
    def approximate(self) -> bool:
        """After a bisection, shadow re-tests the rest on M0 without the green prefix (decision 0155)."""
        values = set(self.outcomes.values())
        return "eject:ci-red" in values and "merge" in values


def parse_prediction(sha: str, at: datetime, message: str) -> Prediction | None:
    lines = message.splitlines()
    if not lines or lines[0] != PREDICTION_HEADER:
        return None
    heads: dict[int, str] = {}
    outcomes: dict[int, str] = {}
    tree = "n/a"
    for line in lines[1:]:
        if line.startswith("shadow-input: "):
            heads = {int(pr): head for pr, head in BATCH_LINE.findall(line)}
        elif found := OUTCOME_LINE.fullmatch(line):
            outcomes.setdefault(int(found[1]), found[2])
        elif found := TREE_LINE.fullmatch(line):
            tree = found[1]
    return Prediction(sha, at, heads, outcomes, tree)


# What happened to a PR after a prediction.
class History(Protocol):
    """The git questions about a PR's later heads."""

    def clean_merge(self, pr: int, prev: str, head: str) -> bool: ...

    def renumbers(self, pr: int, prev: str, head: str) -> bool: ...


class GitHistory:
    """`History` from the local clone: `refs/pull/<n>/head` and `main`, fetched once each."""

    def __init__(self, root: Path = ROOT) -> None:
        self.repo = merge_ready.Repo(root)
        self.fetched: set[int] = set()

    def fetch(self, pr: int) -> None:
        if pr not in self.fetched:
            if not self.fetched:
                self.repo.fetch_base()
            self.repo.fetch_head(pr)
            self.fetched.add(pr)

    def clean_merge(self, pr: int, prev: str, head: str) -> bool:
        self.fetch(pr)
        return self.repo.carry_problem(prev, head) is None

    def renumbers(self, pr: int, prev: str, head: str) -> bool:
        self.fetch(pr)
        out = self.repo.git("diff", "-M", "--name-status", "-z", prev, head, "--", *RENUMBERED, check=False)
        return any(part.startswith("R") and part[1:].isdigit() for part in out.split("\0"))


@dataclass
class Later:
    """A PR's later life: when and on which head it merged, its commits (oldest first, as (sha,
    kind of change from the commit before, committed)) and its heads' ci.yml run conclusions."""

    merged_at: datetime | None
    merged_head: str
    steps: list[tuple[str, str, datetime]]
    conclusions: dict[str, list[str]] = field(default_factory=dict)

    def after(self, head: str, until: datetime) -> list[tuple[str, str]] | None:
        """The changes after `head` committed by `until`; None when `head` is not a PR commit (the
        branch was rewritten), which counts as a non-merge change."""
        shas = [sha for sha, _, _ in self.steps]
        if head not in shas:
            return None
        return [(sha, kind) for sha, kind, at in self.steps[shas.index(head) + 1 :] if at <= until]


def score(outcome: str, head: str, at: datetime, later: Later) -> str | None:
    """The verdict on one prediction (section 4), or None when the outcome is not scored."""
    if outcome not in SCORED:
        return None
    until = at + DAY
    steps = later.after(head, until)
    if steps is None:
        steps = [("", CHANGE)]
    clean = [head]
    for sha, kind in steps:
        if kind != CLEAN:
            break
        clean.append(sha)
    rest = [kind for _, kind in steps[len(clean) - 1 :]]
    merged = later.merged_at is not None and later.merged_at <= until
    merged_clean = merged and later.merged_head in clean
    runs = later.conclusions
    if outcome == "merge":
        if merged_clean:
            return AGREE if runs.get(later.merged_head, [])[-1:] == ["success"] else DISAGREE
        return DISAGREE if rest or merged else UNSETTLED
    if outcome == "eject:ci-red":
        if any("failure" in runs.get(sha, []) for sha in clean) or rest:
            return AGREE
        return DISAGREE if merged_clean else UNSETTLED
    if CONFLICT in rest or RENUMBER in rest:
        return AGREE
    return DISAGREE if merged else UNSETTLED


@dataclass(frozen=True)
class Run:
    """One ci.yml workflow run."""

    id: int
    branch: str
    sha: str
    event: str
    conclusion: str
    created: datetime
    started: datetime
    updated: datetime

    @property
    def wall(self) -> float:
        return (self.updated - self.started).total_seconds()


def runs_of(items: Iterable[object]) -> list[Run]:
    found = []
    for item in items:
        times = [when(text_of(item, key)) for key in ("created_at", "run_started_at", "updated_at")]
        if text_of(item, "path") != merge_ready.CI_WORKFLOW or None in times or int_of(item, "id") <= 0:
            continue
        created, started, updated = (t for t in times if t is not None)
        found.append(
            Run(
                int_of(item, "id"),
                text_of(item, "head_branch"),
                text_of(item, "head_sha"),
                text_of(item, "event"),
                text_of(item, "conclusion"),
                created,
                started,
                updated,
            )
        )
    return sorted(found, key=lambda run: run.created)


@dataclass(frozen=True)
class Hold:
    """One holding of the merge lock: by PR `pr` or by a train for `prs`."""

    pr: str
    prs: tuple[str, ...]
    start: datetime
    end: datetime
    wait: float


def holds_of(pushes: Sequence[tuple[datetime, merge_lock.Lock]], now: datetime) -> list[Hold]:
    """The holds in a lock history (oldest first). A hold's wait runs from its holder's queue entry
    in the lock commit just before it (0 when it took a free lock unqueued)."""
    holds: list[Hold] = []
    current: tuple[str, str, tuple[str, ...], datetime, float] | None = None
    previous: merge_lock.Lock | None = None
    for at, lock in pushes:
        key = (lock.holder or "", lock.pr)
        if current is not None and key != current[:2]:
            holds.append(Hold(current[1], current[2], current[3], at, current[4]))
            current = None
        if current is None and lock.holder is not None:
            queued = [w.registered for w in (previous.waiters if previous else ()) if w.id == lock.holder]
            wait = max(0.0, at.timestamp() - queued[0]) if queued else 0.0
            current = (lock.holder, lock.pr, lock.prs, at, wait)
        previous = lock
    if current is not None:
        holds.append(Hold(current[1], current[2], current[3], now, current[4]))
    return holds


@dataclass
class Train:
    """The runs of one batch on one base (module docstring)."""

    branch: str
    base: str
    batch: frozenset[int]
    runs: list[Run] = field(default_factory=list)


def trains_of(runs: Sequence[tuple[Run, str, frozenset[int]]]) -> list[Train]:
    """Groups (run, base, batch) of a train branch, oldest first, into trains."""
    trains: list[Train] = []
    open_: dict[tuple[str, str], Train] = {}
    for run, base, batch in runs:
        train = open_.get((run.branch, base))
        if train is None or not batch <= train.batch:
            train = Train(run.branch, base, batch)
            trains.append(train)
            open_[(run.branch, base)] = train
        train.runs.append(run)
    return trains


class Stats:
    """Reads the history from GitHub and prints the report."""

    def __init__(self, client: Client, history: History, now: datetime, since: datetime) -> None:
        self.client, self.history, self.now, self.since = client, history, now, since
        self.out: list[str] = []
        self.laters: dict[int, Later] = {}
        self.conclusions: dict[str, list[str]] = {}
        # Job queue delays (s) of the window's completed ci.yml runs: `merge_stage` reads them (one
        # call per run) and `load` reuses them, so `report` calls `merge_stage` first.
        self.delays: list[float] = []

    # Reads.
    def pushes(self, branch: str, since: datetime) -> list[tuple[datetime, str]]:
        """(time, sha) of the pushes to `branch`, oldest first, back to the last one before `since`."""
        found = []
        for item in self.client.activity(branch, iso(since)):
            at, sha = when(text_of(item, "timestamp")), text_of(item, "after")
            if at is not None and SHA.fullmatch(sha) and sha != "0" * 40:
                found.append((at, sha))
        found.sort()
        older = [i for i, (at, _) in enumerate(found) if at < since]
        return found[older[-1] :] if older else found

    def message(self, sha: str) -> str:
        return text_of(self.client.commit(sha), "message")

    def runs(self, start: datetime, end: datetime) -> list[Run]:
        return runs_of(self.client.workflow_runs(created=f"{iso(start)}..{iso(end)}"))

    def head_conclusions(self, sha: str) -> list[str]:
        """The conclusions of a PR head's completed `pull_request` ci.yml runs, oldest first."""
        if sha not in self.conclusions:
            runs = [r for r in runs_of(self.client.workflow_runs(head_sha=sha)) if r.event == "pull_request"]
            self.conclusions[sha] = [r.conclusion for r in runs if r.conclusion]
        return self.conclusions[sha]

    def later(self, pr: int) -> Later:
        if pr in self.laters:
            return self.laters[pr]
        pull = self.client.pull(pr)
        merged_at = when(text_of(pull, "merged_at")) if field_of(pull, "merged") else None
        steps: list[tuple[str, str, datetime]] = []
        prev = ""
        for item in self.client.pull_commits(pr):
            sha = text_of(item, "sha")
            parents = [text_of(p, "sha") for p in field_of(item, "parents") or [] if isinstance(p, dict)]
            at = when(text_of(item, "commit", "committer", "date")) or self.now
            if not SHA.fullmatch(sha):
                continue
            if prev and len(parents) == 2 and parents[0] == prev:
                kind = CLEAN if self.history.clean_merge(pr, prev, sha) else CONFLICT
                kind = RENUMBER if self.history.renumbers(pr, prev, sha) else kind
            elif prev:
                kind = RENUMBER if self.history.renumbers(pr, prev, sha) else CHANGE
            else:
                kind = CHANGE
            steps.append((sha, kind, at))
            prev = sha
        later = Later(merged_at, text_of(pull, "head", "sha"), steps)
        for sha, _, _ in steps:
            later.conclusions[sha] = self.head_conclusions(sha)
        self.laters[pr] = later
        return later

    def predictions(self) -> tuple[list[Prediction], datetime | None]:
        """Every shadow prediction since the shadow branch was created, and that creation time."""
        found = []
        pushes = self.pushes(SHADOW_BRANCH, datetime.fromtimestamp(0, UTC))
        for at, sha in pushes:
            parsed = parse_prediction(sha, at, self.message(sha))
            if parsed is not None:
                found.append(parsed)
        return found, (pushes[0][0] if pushes else None)

    def emit(self, line: str = "") -> None:
        self.out.append(line)

    # The report.
    def report(self) -> str:
        window = (self.now - self.since).total_seconds()
        self.emit(f"merge-train-stats {iso(self.since)} .. {iso(self.now)}")
        predictions, shadow_start = self.predictions()
        runs = self.runs(self.since, self.now)
        lock = [
            (at, merge_lock.parse_lock(sha, int(at.timestamp()), self.message(sha)))
            for at, sha in self.pushes(LOCK_BRANCH, self.since)
        ]
        holds = holds_of(lock, self.now)
        merged = self.merged()
        self.throughput(merged, window)
        latencies = self.latency(merged)
        self.merge_stage(merged, holds, runs)
        self.lock_use(holds, window)
        stranded = self.stranded()
        trains = self.trains(runs)
        tree_failures = self.predicted([p for p in predictions if p.at >= self.since])
        self.thresholds(latencies, trains, stranded, tree_failures)
        self.bar(predictions, shadow_start)
        return "\n".join(self.out)

    def merged(self) -> list[tuple[int, object, datetime]]:
        found = []
        for pull in self.client.updated_pulls(iso(self.since)):
            at = when(text_of(pull, "merged_at"))
            if at is not None and self.since <= at <= self.now and text_of(pull, "base", "ref") == "main":
                found.append((int_of(pull, "number"), pull, at))
        return sorted(found, key=lambda item: item[2])

    def throughput(self, merged: Sequence[tuple[int, object, datetime]], window: float) -> None:
        hours = Counter(at.replace(minute=0, second=0, microsecond=0) for _, _, at in merged)
        mean = len(merged) / (window / 3600) if window > 0 else 0.0
        self.emit(
            f"merged PRs: {len(merged)}; merges per hour mean {mean:.2f}, peak hour "
            f"{max(hours.values(), default=0)}"
        )

    def latency(self, merged: Sequence[tuple[int, object, datetime]]) -> list[tuple[datetime, float]]:
        """First green on the merged head -> merged, p50/p90 per arrival bucket; returns (merged
        time, latency) pairs for the daily threshold."""
        starts: list[tuple[datetime, datetime]] = []
        for _, pull, at in merged:
            runs = runs_of(self.client.workflow_runs(head_sha=text_of(pull, "head", "sha")))
            green = [r.updated for r in runs if r.event == "pull_request" and r.conclusion == "success"]
            if green:
                starts.append((min(green), at))
        rate = Counter(start.replace(minute=0, second=0, microsecond=0) for start, _ in starts)
        for name, low, high in BUCKETS:
            values = [
                (at - start).total_seconds()
                for start, at in starts
                if low <= rate[start.replace(minute=0, second=0, microsecond=0)] <= high
            ]
            self.emit(
                f"latency {name}: n={len(values)} p50 {minutes(percentile(values, 0.5))}, "
                f"p90 {minutes(percentile(values, 0.9))}"
            )
        return [(at, (at - start).total_seconds()) for start, at in starts]

    def merge_stage(
        self, merged: Sequence[tuple[int, object, datetime]], holds: Sequence[Hold], runs: Sequence[Run]
    ) -> None:
        branches = {str(pr): text_of(pull, "head", "ref") for pr, pull, _ in merged}
        count = 0
        for hold in holds:
            branch = TRAIN_BRANCH if hold.pr == merge_lock.TRAIN else branches.get(hold.pr)
            count += sum(1 for r in runs if r.branch == branch and hold.start <= r.created <= hold.end)
        per_pr = f"{count / len(merged):.2f}" if merged else "n/a"
        self.emit(f"merge-stage CI runs per merged PR: {per_pr} ({count} runs under the lock)")
        minutes_by: Counter[str] = Counter()
        for run in runs:
            if not run.conclusion:
                continue
            kind = run.branch if run.branch in (TRAIN_BRANCH, SHADOW_BRANCH, "main") else run.event
            for job in self.client.run_jobs(run.id):
                created, started, done = (
                    when(text_of(job, k)) for k in ("created_at", "started_at", "completed_at")
                )
                if started is not None and done is not None:
                    minutes_by[kind] += (done - started).total_seconds() / 60
                if created is not None and started is not None:
                    self.delays.append((started - created).total_seconds())
        total = sum(minutes_by.values())
        per = f"{total / len(merged):.1f}" if merged else "n/a"
        parts = ", ".join(f"{k} {v:.0f}" for k, v in sorted(minutes_by.items()))
        self.emit(f"CI runner-minutes per merged PR: {per} (total {total:.0f}: {parts or 'none'})")
        walls = [r.wall for r in runs if r.event == "pull_request" and r.conclusion]
        self.emit(
            f"PR CI wall time p90: {minutes(percentile(walls, 0.9))}; job queue delay p90: "
            f"{self.seconds(percentile(self.delays, 0.9))}"
        )

    @staticmethod
    def seconds(value: float | None) -> str:
        return "n/a" if value is None else f"{value:.0f} s"

    def lock_use(self, holds: Sequence[Hold], window: float) -> None:
        inside = [h for h in holds if h.end > self.since]
        held = sum((min(h.end, self.now) - max(h.start, self.since)).total_seconds() for h in inside)
        waits = [h.wait for h in inside if h.start >= self.since]
        share = f"{held / window:.0%}" if window > 0 else "n/a"
        self.emit(
            f"lock: {len(waits)} holds; wait p90 {minutes(percentile(waits, 0.9))}; held {share} of the time"
        )

    def stranded(self) -> list[int]:
        """Open PRs marked ready more than 2 h ago, unmerged and not ejected."""
        found = []
        for pull in self.client.pulls():
            labels = merge_ready.label_names(pull)
            if merge_ready.READY_LABEL not in labels or merge_ready.EJECTED_LABEL in labels:
                continue
            pr = int_of(pull, "number")
            marks = [
                when(text_of(c, "created_at"))
                for c in self.client.comments(pr)
                if merge_ready.has_marker(text_of(c, "body"))
            ]
            newest = max((m for m in marks if m is not None), default=None)
            if newest is not None and self.now - newest > STRANDED_AFTER:
                found.append(pr)
        self.emit("stranded PRs: " + (", ".join(f"#{pr}" for pr in found) or "none"))
        return found

    def trains(self, runs: Sequence[Run]) -> list[Train]:
        tagged = []
        for run in runs:
            if run.branch not in (TRAIN_BRANCH, SHADOW_BRANCH) or not SHA.fullmatch(run.sha):
                continue
            message = self.message(run.sha)
            base = TRAIN_HEADER.match(message)
            if base is not None:
                tagged.append((run, base[1], frozenset(int(pr) for pr, _ in BATCH_LINE.findall(message))))
        trains = trains_of(tagged)
        days = max((self.now - self.since) / DAY, 1e-9)
        for branch in (SHADOW_BRANCH, TRAIN_BRANCH):
            mine = [t for t in trains if t.branch == branch]
            settled = [t for t in mine if t.runs[0].conclusion in ("success", "failure")]
            red = sum(1 for t in settled if t.runs[0].conclusion == "failure")
            extra = sum(len(t.runs) - 1 for t in mine)
            rate = f"{red / len(settled):.0%}" if settled else "n/a"
            self.emit(
                f"{branch}: {len(mine)} trains, red-train rate {rate} ({red}/{len(settled)}), "
                f"bisection runs per day {extra / days:.1f}"
            )
        main_red = sum(
            1 for r in runs if r.branch == "main" and r.event == "push" and r.conclusion == "failure"
        )
        self.emit(f"main: {main_red} red ci.yml push runs")
        return trains

    def predicted(self, predictions: Sequence[Prediction]) -> int:
        """Ejections by reason, deferral rate, effective batch size and tree-check failures, from the
        shadow predictions in the window; returns the tree-check failures."""
        outcomes = Counter(o for p in predictions for o in p.outcomes.values())
        decided = sum(n for o, n in outcomes.items() if o != NO_RESULT)
        sizes = [sum(1 for o in p.outcomes.values() if o in ("merge", "eject:ci-red")) for p in predictions]
        ejections = ", ".join(f"{o[6:]} {n}" for o, n in sorted(outcomes.items()) if o.startswith("eject:"))
        deferral = f"{outcomes['defer'] / decided:.0%}" if decided else "n/a"
        mean = f"{sum(sizes) / len(sizes):.1f}" if sizes else "n/a"
        mismatches = sum(1 for p in predictions if p.tree_check == "mismatch")
        self.emit(
            f"shadow predictions: {len(predictions)}; ejections {ejections or 'none'}; deferral rate "
            f"{deferral}; effective batch size mean {mean}, p90 {'n/a' if not sizes else percentile(sizes, 0.9)}"
        )
        self.emit(f"tree-check failures: {mismatches}")
        return mismatches

    def thresholds(
        self,
        latencies: Sequence[tuple[datetime, float]],
        trains: Sequence[Train],
        stranded: Sequence[int],
        tree_failures: int,
    ) -> None:
        """The steward's daily thresholds; any one triggers the flip to `lock` and a P0."""
        triggers = []
        for day in sorted({at.date() for at, _ in latencies}):
            p90 = percentile([v for at, v in latencies if at.date() == day], 0.9)
            if p90 is not None and p90 / 60 > LATENCY_P90_MAX:
                triggers.append(f"p90 latency {minutes(p90)} on {day}")
        long = Counter(t.runs[0].created.date() for t in trains if len(t.runs) > TRAIN_RUNS_MAX)
        triggers += [
            f"{n} trains over {TRAIN_RUNS_MAX} CI runs on {day}"
            for day, n in sorted(long.items())
            if n > TRAINS_OVER_MAX
        ]
        triggers += [f"stranded #{pr}" for pr in stranded]
        triggers += [f"{tree_failures} tree-check failure(s)"] if tree_failures else []
        self.emit("daily thresholds: " + ("; ".join(triggers) + " -> flip to lock" if triggers else "ok"))

    def bar(self, predictions: Sequence[Prediction], shadow_start: datetime | None) -> None:
        """T-2991's evidence and load bar, over every prediction since shadow started."""
        verdicts: list[tuple[Prediction, int, str, str]] = []
        for p in predictions:
            for pr, outcome in p.outcomes.items():
                if outcome in SCORED and pr in p.heads:
                    verdict = score(outcome, p.heads[pr], p.at, self.later(pr))
                    verdicts.append((p, pr, outcome, verdict or UNSETTLED))
        scored = [v for v in verdicts if v[3] != UNSETTLED]
        agree = sum(1 for v in scored if v[3] == AGREE)
        trains = sorted({v[0].at for v in scored})
        span = trains[-1] - trains[0] if trains else timedelta(0)
        fresh = bool(trains) and self.now - trains[-1] <= BAR_FRESH
        rate = agree / len(scored) if scored else 0.0
        mismatches = sum(1 for p in predictions if p.tree_check == "mismatch")
        approx = sum(1 for v in scored if v[0].approximate)
        self.emit(
            f"shadow agreement: {agree}/{len(scored)} scored predictions agree ({rate:.0%}); "
            f"{len(verdicts) - len(scored)} unsettled; {approx} scored after a bisection (approximate)"
        )
        for p, pr, outcome, verdict in scored:
            if verdict == DISAGREE:
                self.emit(f"  disagree: #{pr} predicted {outcome} at {iso(p.at)} ({p.sha[:12]})")
        load = self.load(shadow_start)
        unmet = []
        if len(trains) < BAR_TRAINS or span < DAY or not fresh:
            unmet.append(f"{len(trains)} scored trains over {span}, newest fresh: {fresh}")
        if rate < BAR_AGREE:
            unmet.append(f"agreement {rate:.0%} < {BAR_AGREE:.0%}")
        if mismatches:
            unmet.append(f"{mismatches} tree-check mismatch(es)")
        unmet += load
        self.emit("T-2991 bar: " + ("met" if not unmet else "not met: " + "; ".join(unmet)))

    def load(self, shadow_start: datetime | None) -> list[str]:
        """The load bar: PR CI p90 wall time during shadow (its last 48 h at most) within 20% of the
        48 h before it, and job queue delay p90 under 1 min over the metrics window (job reads cost a
        call per run, so they are not repeated); returns what fails."""
        if shadow_start is None:
            self.emit("load bar: no shadow run yet")
            return ["no shadow run yet"]
        before = [
            r.wall
            for r in self.runs(shadow_start - LOAD_BEFORE, shadow_start)
            if r.event == "pull_request" and r.conclusion
        ]
        during = [
            r.wall
            for r in self.runs(max(shadow_start, self.now - LOAD_BEFORE), self.now)
            if r.event == "pull_request" and r.conclusion
        ]
        p_before, p_during = percentile(before, 0.9), percentile(during, 0.9)
        p_delay = percentile(self.delays, 0.9)
        self.emit(
            f"load bar: PR CI p90 {minutes(p_during)} during shadow vs {minutes(p_before)} before; "
            f"job queue delay p90 {self.seconds(p_delay)}"
        )
        failed = []
        if p_before is None or p_during is None or p_during > LOAD_WALL * p_before:
            failed.append("PR CI p90 wall time over 120% of before shadow (or unmeasured)")
        if p_delay is None or p_delay >= LOAD_QUEUE:
            failed.append("job queue delay p90 not under 1 min (or unmeasured)")
        return failed


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else "")
    parser.add_argument("--since", help="start of the metrics window (ISO date or instant; default 24 h ago)")
    args = parser.parse_args(argv)
    now = datetime.now(UTC)
    since = now - DAY
    if args.since:
        text = args.since if "T" in args.since else f"{args.since}T00:00:00Z"
        parsed = when(text.replace("Z", "+00:00"))
        if parsed is None:
            print(f"merge-train-stats: bad --since {args.since!r}", file=sys.stderr)
            return 2
        since = parsed
    client = gh_client.from_remote()
    if client is None:
        print("merge-train-stats: origin is not a github.com remote", file=sys.stderr)
        return 2
    try:
        print(Stats(client, GitHistory(), now, since).report())
    except (GhError, subprocess.CalledProcessError) as error:
        print(f"merge-train-stats: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
