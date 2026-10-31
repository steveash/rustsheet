#!/usr/bin/env python3
"""The merge train (SPEC-0027 section 4): batched merges of ready PRs under the merge lock.

  scripts/merge_train.py mode        # the mode on origin/main: lock, shadow or train (stop pending: #n)
  scripts/merge_train.py --dry-run   # steps 2-4: collect, build, id checks; prints, writes nothing
  scripts/merge_train.py --train     # steps 1-9 and section 6 (red trains), in `train` mode only
  scripts/merge_train.py --shadow    # steps 2-6 and section 6 on merge-train/shadow, in `shadow` mode only
  ... --train --step, --shadow --step  # each write is a JSON action the session performs (decision 0169)

The mode is read from `scripts/merge_train.toml` on a freshly fetched `origin/main`, never from
the checkout, so a branch cannot switch its own flow. The read fails closed: a missing file, a
TOML error (a duplicate key included), or a missing, non-string or unknown `mode` reads as `lock`.
"""

from __future__ import annotations

import re
import signal
import subprocess
import sys
import time
import tomllib
import unicodedata
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from pathlib import Path

import ci_changes
import gh_client
import merge_flip
import merge_lock
import merge_ready
import merge_step
from gh_client import SHA, Client, GhError
from merge_flip import LOCK, SWITCH
from merge_lock import GitError, git
from merge_train_filing import (
    ID_CHECK_TIMEOUT, IDENTITY, MAIN_REF, REMOTE, CutOff, Landed, Stop, fetch_main, file_tasks, respond, worktrees,
)

ROOT = Path(__file__).resolve().parent.parent
SHADOW, TRAIN = "shadow", "train"
MODES = (LOCK, SHADOW, TRAIN)
# The id checks of step 4; each takes `--root <tree>`. They run from `main`'s scripts (M0).
ID_CHECKS = (("tasks.py", "validate"), ("check_decisions.py",))
BATCH_MAX = 5
# Ids that an id-check failure line names; only these, never its file names, reach a reason.
COLLIDING_ID = re.compile(r"\b(?:duplicate id (T-\d{4})|decision number (\d{4}))\b")


def parse_mode(text: str | None) -> str:
    """The mode a switch file's text sets; LOCK for anything that is not exactly a valid mode."""
    if text is None:
        return LOCK
    try:
        mode = tomllib.loads(text).get("mode")
    except tomllib.TOMLDecodeError:
        return LOCK
    return mode if isinstance(mode, str) and mode in MODES else LOCK


def read_mode(root: Path = ROOT) -> str:
    """The mode on a freshly fetched origin/main; LOCK when the fetch or the read fails."""
    try:
        fetch_main(root)
        shown = git("show", f"refs/remotes/origin/main:{SWITCH}", root=root, check=False)
    except GitError:
        return LOCK
    return parse_mode(shown.stdout if shown.returncode == 0 else None)


def id_check_failures(scripts: Path, tree: Path) -> list[str]:
    """The FAIL lines of M0's id checks on the train worktree `tree` (SPEC-0027 section 4 step 4).

    `scripts` is M0's scripts directory, in a worktree outside `tree`. `python3 -I` with the
    working directory there imports no module from `tree`, so a PR cannot change how its own tree
    is judged (decision 0066). A check that cannot run counts as one failure.
    """
    failures = []
    for script, *args in ID_CHECKS:
        argv = [sys.executable, "-I", str(scripts / script), *args, "--root", str(tree)]
        try:
            run = subprocess.run(
                argv, cwd=scripts, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                timeout=ID_CHECK_TIMEOUT, check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            failures.append(f"{script}: {error.__class__.__name__}")
            continue
        lines = [line for line in run.stdout.splitlines() if line.startswith("FAIL")]
        if run.returncode != 0:
            failures += lines or [f"{script}: exit {run.returncode}"]
    return failures


@dataclass(frozen=True)
class Member:
    """PR `pr` at head `head`, merged into the train as merge commit `merge`."""

    pr: int
    head: str
    merge: str


@dataclass
class Train:
    """A built train on `base` (M0): its members in order, and the PRs left out with the reason."""

    base: str
    members: list[Member] = field(default_factory=list)
    deferred: list[tuple[int, str]] = field(default_factory=list)
    ejected: list[tuple[int, str]] = field(default_factory=list)

    @property
    def tip(self) -> str:
        return self.members[-1].merge if self.members else self.base


def independent(worktree: Path, base: str, head: str, members: Sequence[Member]) -> Member | None:
    """The first member whose head shares history with `head` beyond `base`, or None.

    Every merge base (`--all`, so criss-cross histories count) of a member's head and `head` must
    be an ancestor of `base`; otherwise squashing them one after the other would not give the
    train's trees (SPEC-0027 section 5)."""
    for member in members:
        bases = git("merge-base", "--all", member.head, head, root=worktree, check=False).stdout.split()
        for merge_base in bases:
            if git("merge-base", "--is-ancestor", merge_base, base, root=worktree, check=False).returncode != 0:
                return member
    return None


def id_collision(failures: Sequence[str]) -> str:
    ids = sorted({name for line in failures for pair in COLLIDING_ID.findall(line) for name in pair if name})
    named = ", ".join(ids) or "a task id or decision number"
    return f"id collision with {named}; renumber per decision 0121 (2)"


def build(worktree: Path, scripts: Path, base: str, heads: Sequence[tuple[int, str]]) -> Train:
    """Steps 3 and 4 of SPEC-0027 section 4 in the scratch worktree `worktree`, checked out at
    `base`: merge each (PR, head) in order, deferring dependent or intra-batch conflicting PRs
    and ejecting those that conflict with `base` or collide on an id. `scripts` is M0's scripts
    directory (outside `worktree`). The heads must already be fetched."""
    train = Train(base)
    git("checkout", "-q", "--detach", base, root=worktree)
    if id_check_failures(scripts, worktree):
        raise GitError("the id checks fail on main itself")
    for pr, head in heads:
        if len(train.members) == BATCH_MAX:
            break
        other = independent(worktree, base, head, train.members)
        if other is not None:
            train.deferred.append((pr, f"shares history with #{other.pr} beyond main"))
            continue
        merged = git(*IDENTITY, "merge", "-q", "--no-ff", "--no-edit", head, root=worktree, check=False)
        if merged.returncode != 0:
            git("merge", "--abort", root=worktree, check=False)
            git("checkout", "-q", "-f", "--detach", train.tip, root=worktree)
            alone = git("merge-tree", "--write-tree", base, head, root=worktree, check=False)
            if alone.returncode == 0:
                train.deferred.append((pr, "conflicts with an earlier PR of this train"))
            else:
                train.ejected.append((pr, "conflict with main"))
            continue
        failures = id_check_failures(scripts, worktree)
        if failures:
            # The tip passed the checks, so this PR adds the later copy.
            git("checkout", "-q", "-f", "--detach", train.tip, root=worktree)
            train.ejected.append((pr, id_collision(failures)))
            continue
        merge = git("rev-parse", "HEAD", root=worktree).stdout.strip()
        train.members.append(Member(pr, head, merge))
    return train


@dataclass
class Collected:
    """Step 2: the ready PRs oldest marker first, and what the merger would do about the rest."""

    ready: list[merge_ready.Verdict] = field(default_factory=list)
    unlabel: list[tuple[int, str]] = field(default_factory=list)
    keep: list[tuple[int, str]] = field(default_factory=list)
    # Verdict-record gaps under `verdict_records = "warn"` (decision 0178): reported only.
    warnings: list[tuple[int, str]] = field(default_factory=list)

    def warning_lines(self) -> list[str]:
        return [f"verdict records #{pr} (warn): {warning}" for pr, warning in self.warnings]


def collect(client: Client, repo: merge_ready.Repo) -> Collected:
    """Runs V1-V6 on every open PR labelled ready-to-merge (SPEC-0027 section 4, step 2)."""
    found = Collected()
    for pull in client.pulls():
        if merge_ready.READY_LABEL not in merge_ready.label_names(pull):
            continue
        pr = gh_client.number(merge_ready.int_of(pull, "number"))
        verdict = merge_ready.verify(client, pr, repo)
        actions = {f.action for f in verdict.failures}
        reasons = "; ".join(f.reason for f in verdict.failures)
        found.warnings += [(pr, warning) for warning in verdict.warnings]
        if merge_ready.UNLABEL in actions:
            found.unlabel.append((pr, reasons))
        elif merge_ready.KEEP in actions:
            found.keep.append((pr, reasons))
        elif verdict.ready:
            if merge_ready.text_of(client.pull(pr), "mergeable_state") == "dirty":
                found.unlabel.append((pr, "conflict with main"))
            else:
                found.ready.append(verdict)
    found.ready.sort(key=lambda v: (v.updated_at, v.comment_id or 0))
    return found


def dry_run(client: Client, root: Path = ROOT) -> str:
    """Steps 2-4 against a fresh origin/main in scratch worktrees; returns the report. No writes:
    no push, label, comment or lock (the write policy of SPEC-0027 section 4)."""
    repo = merge_ready.Repo(root)
    repo.fetch_base()
    base = git("rev-parse", f"refs/merge-ready/{merge_ready.BASE_BRANCH}", root=root).stdout.strip()
    found = collect(client, repo)
    heads = [(v.pr, v.marker.head) for v in found.ready[:BATCH_MAX] if v.marker is not None]
    with worktrees(root, base) as (train_tree, scripts_tree):
        train = build(train_tree, scripts_tree / "scripts", base, heads)
    lines = [f"dry run on main {base}: {len(heads)} candidate(s); nothing was written"]
    lines.append("batch: " + (", ".join(f"#{m.pr} ({m.head[:12]})" for m in train.members) or "none"))
    lines += [f"deferred #{pr}: {reason}" for pr, reason in train.deferred]
    lines += [f"would eject #{pr}: {reason}" for pr, reason in train.ejected]
    lines += [f"would unlabel #{pr}: {reason}" for pr, reason in found.unlabel]
    lines += [f"skipped, label kept #{pr}: {reason}" for pr, reason in found.keep]
    lines += [f"waiting for a later batch #{v.pr}" for v in found.ready[BATCH_MAX:]]
    lines += found.warning_lines()
    return "\n".join(lines)


# The merge stage (SPEC-0027 section 4, steps 1 and 5-9; T-2994) and red trains (section 6; T-2998). The
# constants are section 4's.
TRAIN_BRANCH = "merge-train/current"
TRAIN_REF = f"refs/heads/{TRAIN_BRANCH}"
CI_WORKFLOW_FILE = Path(merge_ready.CI_WORKFLOW).name
TRAIN_EVENTS = ("push", "workflow_dispatch")
WINDOW = 40 * 60  # a merger run's polling window; the lock wait is capped at what is left of it
HOLD_BUDGET = 40 * 60  # counted from the lock's acquisition
CI_TIMEOUT = 10 * 60
MERGE_BUDGET = 150  # per PR a CI run can merge: the pre-squash check, the merge call, the post-squash check
CI_POLL = 60  # every CI poll is a stop checkpoint
DISPATCH_AFTER = 3 * 60  # no run by then: dispatch once; still none after as long again: stop
RENEW_EVERY = 5 * 60
TITLE_MAX = 100
ZWJ = "\u200d"
CLOSING_KEYWORD = re.compile(r"\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\b", re.IGNORECASE)
CO_AUTHOR = re.compile(r"Co-Authored-By: [^@#<>\n]+ <[^@<>\s]+@[^@<>\s]+>")
SESSION = re.compile(r"Claude-Session: https://claude\.ai/code/session_[A-Za-z0-9]{1,64}")



# The outcome of one train CI wait (step 6). INFRA is a run that is not the PRs' fault: no run, a
# run over CI_TIMEOUT, or a run or job that was cancelled, timed out or never started (section 6).
INFRA = "infrastructure"
INFRA_CONCLUSIONS = ("cancelled", "timed_out", "startup_failure", "missing")


@dataclass(frozen=True)
class Outcome:
    """GREEN, RED or INFRA for one tested tip, the jobs that were not green, and the run ids."""

    state: str
    jobs: tuple[str, ...] = ()
    runs: tuple[int, ...] = ()


# The search state a cut-off records on TRAIN_REF (section 6, "Resume is required"). The commit
# says `[skip ci]`, so recording it starts no run.
STATE_HEADER = "merge train state [skip ci]"
STATE_LINE = re.compile(r"(bisect-base|known-red|known-green-prefix): (.*)")
KNOWN_RED = re.compile(r"#([1-9]\d{0,8}) ([0-9a-f]{40})")


@dataclass(frozen=True)
class Resume:
    """A recorded cut-off: the PRs `known_red` (number, head) are a known-red train on `base`."""

    base: str
    known_red: tuple[tuple[int, str], ...]

    def render(self) -> str:
        red = " ".join(f"#{pr} {head}" for pr, head in self.known_red)
        return f"{STATE_HEADER}\n\nbisect-base: {self.base}\nknown-red: {red}\nknown-green-prefix: 0"

    @staticmethod
    def parse(message: str) -> Resume | None:
        """The state in a TRAIN_REF commit message, or None unless it is exactly a state record.
        Anyone who can push the branch can write one, so it only ever saves runs: a culprit is
        ejected only after its own confirmation run is red, and only tested prefixes merge."""
        lines = message.strip().splitlines()
        if len(lines) != 5 or lines[0] != STATE_HEADER or lines[1]:
            return None
        fields = {}
        for line in lines[2:]:
            matched = STATE_LINE.fullmatch(line)
            if matched is None or matched[1] in fields:
                return None
            fields[matched[1]] = matched[2]
        red = fields.get("known-red", "")
        pairs = KNOWN_RED.findall(red)
        if (
            not SHA.fullmatch(fields.get("bisect-base", ""))
            or fields.get("known-green-prefix") != "0"
            or not pairs
            or " ".join(f"#{pr} {head}" for pr, head in pairs) != red
        ):
            return None
        return Resume(fields["bisect-base"], tuple((int(pr), head) for pr, head in pairs))

    def matches(self, train: Train) -> tuple[int | None, str]:
        """How many leading members of `train` are known red, or None and the restart reason."""
        if train.base != self.base:
            return None, "main moved since the cut-off"
        k = len(self.known_red)
        prefix = [(m.pr, m.head) for m in train.members[:k]]
        if [pr for pr, _ in prefix] != [pr for pr, _ in self.known_red]:
            return None, "the batch no longer starts with the known-red PRs"
        for (pr, head), (_, known) in zip(prefix, self.known_red, strict=True):
            if head != known:
                return None, f"#{pr}'s head changed since the cut-off"
        return k, ""


def squash_title(title: str, pr: int) -> str:
    """The squash commit title: control, format and line or paragraph separators stripped, at most TITLE_MAX
    characters, `@`, `#` and closing keywords neutralised by a zero-width joiner, then ` (#pr)`."""
    kept = (c for c in title if unicodedata.category(c)[0] != "C" and unicodedata.category(c) not in ("Zl", "Zp"))
    clean = "".join(kept)[:TITLE_MAX].strip()
    clean = CLOSING_KEYWORD.sub(lambda m: m[0][0] + ZWJ + m[0][1:], clean)
    clean = clean.replace("@", "@" + ZWJ).replace("#", "#" + ZWJ)
    return f"{clean or 'merge train'} (#{pr})"


def squash_trailers(commits: Sequence[object]) -> str:
    """The squash commit body: the well-formed Co-Authored-By and Claude-Session trailers of the
    PR's commits, de-duplicated. No other text from the PR enters the commit."""
    found: dict[re.Pattern[str], list[str]] = {CO_AUTHOR: [], SESSION: []}
    for commit in commits:
        for line in merge_ready.text_of(commit, "commit", "message").splitlines():
            for pattern, kept in found.items():
                if pattern.fullmatch(line.strip()) and line.strip() not in kept:
                    kept.append(line.strip())
    return "\n".join(found[CO_AUTHOR] + found[SESSION])


def remote_sha(ref: str, root: Path) -> str:
    """`ref`'s sha on origin, or "" when it does not exist."""
    out = git("ls-remote", REMOTE, ref, root=root).stdout.split()
    return out[0] if out else ""


def tree_of(commit: str, root: Path) -> str:
    return git("rev-parse", f"{commit}^{{tree}}", root=root).stdout.strip()


def stop_pending(client: Client, root: Path, on_invalid: Callable[[int, str], None] | None = None) -> int | None:
    """The first open PR (lowest number) labelled `merge-train-stop` that passes `make
    merge-flip-check`, or None. `on_invalid(pr, head)` is called for each labelled PR before it
    that fails the check. A check that could not run git honours the label: stopping is safe."""
    labelled = sorted(
        gh_client.number(merge_ready.int_of(pull, "number"))
        for pull in client.pulls()
        if merge_ready.STOP_LABEL in merge_ready.label_names(pull)
    )
    for pr in labelled:
        head, problems = merge_flip.pr_problems(client, pr, root)
        if not problems or any(problem.startswith("git: ") for problem in problems):
            return pr
        if on_invalid is not None:
            on_invalid(pr, head)
    return None


def mode_line(root: Path) -> str:
    """`make merge-mode`'s line: the mode, and in `train` mode the stop label a train would honour.
    It only reads: an invalid stop label is left for the train to remove."""
    mode = read_mode(root)
    client = gh_client.from_remote(root, REMOTE) if mode == TRAIN else None
    if client is None:
        return mode
    try:
        pending = stop_pending(client, root)
    except GhError as error:
        print(f"merge_train: the stop labels were not read: {error}", file=sys.stderr)
        return mode
    return f"{mode} (stop pending: #{pending})" if pending is not None else mode


class LockOps:
    """The merge lock calls of a train run, as `scripts/merge_lock.py` exit codes."""

    def __init__(self, holder: str, root: Path) -> None:
        self.holder, self.root = holder, root

    def wait(self, timeout_minutes: float) -> int:
        return merge_lock.wait(self.holder, merge_lock.TRAIN, prs=(), timeout_minutes=timeout_minutes, root=self.root)

    def renew(self, prs: Sequence[int]) -> int:
        return merge_lock.acquire(
            self.holder, merge_lock.TRAIN, root=self.root, prs=[str(pr) for pr in prs], renew=True
        )

    def release(self) -> int:
        return merge_lock.release(self.holder, self.root)


@dataclass
class Stage:
    """One train hold. `clock` and `sleep` are injectable so tests run the waits instantly."""

    client: Client
    root: Path
    lock: LockOps
    clock: Callable[[], float] = time.monotonic
    sleep: Callable[[float], None] = time.sleep
    acquired: float = 0.0
    renewed: float = 0.0
    main: str = ""
    lease: str = ""
    branch: str = TRAIN_BRANCH
    log: list[str] = field(default_factory=list)
    # The tasks this hold asks for, as (priority, title, context); `file_tasks` files them.
    tasks: list[tuple[str, str, str]] = field(default_factory=list)

    mode = TRAIN

    @property
    def ref(self) -> str:
        return f"refs/heads/{self.branch}"

    def checkpoint(self) -> None:
        """A stop checkpoint: stop on a valid stop label; and when `main` moved since the last
        one, re-read the switch from it (failing closed) and stop unless it still says this
        stage's mode."""
        self.stop_label()
        sha = remote_sha(MAIN_REF, self.root)
        if sha != self.main:
            self.main = sha
            mode = read_mode(self.root)
            if mode != self.mode:
                raise Stop(f"the mode on main is now {mode}")

    def stop_label(self) -> None:
        """Honours a `merge-train-stop` label on a flip PR, and removes it from any other PR."""
        pending = stop_pending(self.client, self.root, self.remove_stop)
        if pending is not None:
            raise Stop(f"a flip to lock is pending: #{pending} carries {merge_ready.STOP_LABEL}")

    def remove_stop(self, pr: int, head: str) -> None:
        self.client.remove_label(pr, merge_ready.STOP_LABEL)
        self.client.comment(pr, f"merge train: stop label removed: not a valid flip PR "
                                f"(`make merge-flip-check` fails at {head})")
        self.log.append(f"removed {merge_ready.STOP_LABEL} from #{pr}: not a valid flip PR")

    def renew(self, prs: Sequence[int]) -> None:
        if self.lock.renew(prs) != 0:
            raise Stop("the merge lock was lost")
        self.renewed = self.clock()

    def left(self) -> float:
        return HOLD_BUDGET - (self.clock() - self.acquired)

    def push(self, train: Train, notes: Sequence[str] = ()) -> str:
        """Step 5: push a record commit on the train's tip (same tree; its message lists the
        batch, then `notes`) to the train branch, leased on the last sha this hold read or pushed there.
        Returns the pushed sha."""
        lines = [f"merge train on main {train.base}", ""]
        lines += [f"#{m.pr} {m.head}" for m in train.members]
        lines += ["", *notes] if notes else []
        return self.write_train_ref(tree_of(train.tip, self.root), train.tip, "\n".join(lines))

    def write_train_ref(self, tree: str, parent: str, message: str) -> str:
        tip = git(*IDENTITY, "commit-tree", tree, "-p", parent, "-m", message, root=self.root).stdout.strip()
        pushed = git("push", "-q", f"--force-with-lease={self.ref}:{self.lease}", REMOTE, f"{tip}:{self.ref}",
                     root=self.root, check=False)
        if pushed.returncode != 0:
            self.lost_lease()
        self.lease = tip
        return tip

    def lost_lease(self) -> None:
        raise Stop(f"infrastructure: the push to {self.branch} lost its lease under the lock")

    def record(self, state: Resume) -> None:
        """A cut-off's search state, as a `[skip ci]` commit on the last train tip."""
        git("fetch", "-q", "--no-tags", REMOTE, f"+{self.ref}:refs/{self.branch}", root=self.root)
        self.write_train_ref(tree_of(self.lease, self.root), self.lease, state.render())
        self.log.append("recorded known-red " + ", ".join(f"#{pr}" for pr, _ in state.known_red)
                        + f" on {state.base[:12]}")

    def recorded(self) -> Resume | None:
        """The search state of the last cut-off, read through the commits API, or None."""
        if not self.lease:
            return None
        return Resume.parse(merge_ready.text_of(self.client.commit(self.lease), "message"))

    def ci(self, tip: str, docs_only: bool | None) -> tuple[str, list[str], list[int]]:
        """The train CI's state on exactly `tip` (step 6), and the ids of the matching runs."""
        runs = [
            r for r in self.client.workflow_runs(branch=self.branch, head_sha=tip)
            if merge_ready.text_of(r, "head_sha") == tip and merge_ready.text_of(r, "head_branch") == self.branch
        ]
        ids = sorted(
            merge_ready.int_of(r, "id") for r in runs
            if merge_ready.text_of(r, "path") == merge_ready.CI_WORKFLOW
            and merge_ready.text_of(r, "event") in TRAIN_EVENTS and merge_ready.int_of(r, "id") > 0
        )
        state, jobs = merge_ready.ci_state(runs, self.client.check_runs(tip), docs_only, TRAIN_EVENTS)
        return state, jobs, ids

    def gate(self, n: int) -> None:
        """Step 6's budget gate for a CI wait whose green result can merge `n` PRs."""
        need = CI_TIMEOUT + n * MERGE_BUDGET
        if self.left() < need:
            raise CutOff(f"hold budget: {self.left():.0f} s left, a CI wait for {n} PR(s) needs {need} s")

    def wait_ci(self, tip: str, docs_only: bool | None, prs: Sequence[int]) -> Outcome:
        """Step 6's CI wait on `tip`, holding the lock for `prs`."""
        start = self.clock()
        dispatched: float | None = None
        while True:
            self.checkpoint()
            if self.clock() - self.renewed >= RENEW_EVERY:
                self.renew(prs)
            state, jobs, ids = self.ci(tip, docs_only)
            if state == merge_ready.GREEN:
                return Outcome(state, (), tuple(ids))
            if state == merge_ready.RED:
                infra = bool(jobs) and all(job.endswith(INFRA_CONCLUSIONS) for job in jobs)
                return Outcome(INFRA if infra else state, tuple(jobs), tuple(ids))
            now = self.clock()
            if not ids and dispatched is None and now - start >= DISPATCH_AFTER:
                self.client.dispatch(CI_WORKFLOW_FILE, self.branch)
                dispatched = now
            elif not ids and dispatched is not None and now - dispatched >= DISPATCH_AFTER:
                return Outcome(INFRA, ("no CI run on the train, even after a dispatch",))
            if now - start >= CI_TIMEOUT:
                return Outcome(INFRA, (f"the train CI took over {CI_TIMEOUT // 60} min",), tuple(ids))
            self.sleep(CI_POLL)

    def test(self, train: Train, n: int, prs: Sequence[int], notes: Sequence[str] = ()) -> tuple[Outcome, str]:
        """Steps 5 and 6 for `train`, whose green result can merge `n` PRs: the gate, the push and
        the CI wait, with one retry of an infrastructure failure on a fresh record commit (section
        6). A second one is a CutOff. Returns the outcome and the tested tip."""
        for attempt in (1, 2):
            self.gate(n)
            tip = self.push(train, notes if attempt == 1 else (*notes, "retry after an infrastructure failure"))
            names = ", ".join(f"#{m.pr}" for m in train.members)
            self.log.append(f"pushed {tip} to {self.branch}: {names}")
            self.renew(prs)
            docs_only = not ci_changes.classify_range(train.base, tip, cwd=self.root)
            self.checkpoint()  # before the CI wait
            outcome = self.wait_ci(tip, docs_only, prs)
            jobs = f" ({'; '.join(outcome.jobs)})" if outcome.jobs else ""
            self.log.append(f"train CI {outcome.state} on {names}{jobs}")
            if outcome.state != INFRA:
                return outcome, tip
        raise CutOff(f"infrastructure: {'; '.join(outcome.jobs)}, twice")

    def pre_squash(self, member: Member, expected: str) -> dict[str, object]:
        """Step 7.2: the PR's merge onto `expected` (the current `main`) must have the train's tree.
        GitHub's own test merge is checked when it sits on `expected`. GitHub recomputes it only when
        the PR changes, not when `main` moves, so a stale one (or one still computing) is replaced by
        the same merge done locally (decision 0200). Returns the PR read."""
        pull = self.client.pull(member.pr)
        want = tree_of(member.merge, self.root)
        merge = merge_ready.text_of(pull, "merge_commit_sha")
        if pull.get("mergeable") is not None and SHA.fullmatch(merge):
            commit = self.client.commit(merge)
            parents = merge_ready.field_of(commit, "parents")
            first = parents[0] if isinstance(parents, list) and parents else None
            if merge_ready.text_of(first, "sha") == expected:
                if merge_ready.text_of(commit, "tree", "sha") != want:
                    raise Stop(f"main differs from the tested train at #{member.pr}: GitHub's test merge "
                               "has another tree", p0=True, pr=member.pr)
                return pull
        if remote_sha(MAIN_REF, self.root) != expected:
            raise Stop(f"main moved without the lock before squashing #{member.pr}", p0=True, pr=member.pr)
        local = git("merge-tree", "--write-tree", expected, member.head, root=self.root, check=False)
        if local.returncode == 1:
            raise Stop(f"#{member.pr} does not merge cleanly onto main {expected[:12]}")
        if local.returncode != 0:
            raise Stop(f"git merge-tree failed for #{member.pr} (exit {local.returncode}): {local.stderr.strip()[:200]}")
        if local.stdout.split()[0] != want:
            raise Stop(f"main differs from the tested train at #{member.pr}: its merge onto "
                       f"{expected[:12]} has another tree", p0=True, pr=member.pr)
        self.log.append(f"#{member.pr}: GitHub's test merge is stale; its merge onto {expected[:12]} has the "
                        "train's tree")
        return pull

    def squash(self, member: Member, pull: dict[str, object]) -> str:
        """Step 7.3: squash-merge the PR only at its tested head. A lost reply is read back.
        Returns the squash commit's sha as GitHub reports it ("" when it does not)."""
        title = squash_title(merge_ready.text_of(pull, "title"), member.pr)
        message = squash_trailers(self.client.pull_commits(member.pr))
        try:
            reply = self.client.merge(member.pr, member.head, title=title, message=message)
        except GhError as error:
            after = self.client.pull(member.pr)
            if not merge_ready.field_of(after, "merged"):
                raise Stop(f"the merge of #{member.pr} was refused ({error})") from error
            return merge_ready.text_of(after, "merge_commit_sha")
        return merge_ready.text_of(reply, "sha")

    def post_squash(self, member: Member, expected: str, squashed: str) -> str:
        """Step 7.4: our squash `squashed` (`main` itself when GitHub did not report it) has
        `expected` as its first parent and the train's tree, and `main` is that squash. A squash
        with another parent or tree is `Landed`; a foreign commit on top of a good one is not
        ours to revert."""
        main = fetch_main(self.root)
        sha = squashed if SHA.fullmatch(squashed) else main
        if git("merge-base", "--is-ancestor", sha, main, root=self.root, check=False).returncode != 0:
            raise Stop(f"main ({main}) does not hold the squash of #{member.pr} ({sha})", p0=True, pr=member.pr)
        parent = git("rev-parse", "--verify", "-q", f"{sha}^1", root=self.root, check=False).stdout.strip()
        tree, want = tree_of(sha, self.root), tree_of(member.merge, self.root)
        if parent != expected or tree != want:
            raise Landed(member.pr, sha, parent, expected, tree, want)
        if main != sha:
            raise Stop(f"main moved without the lock after squashing #{member.pr}", p0=True, pr=member.pr)
        return sha

    def delete_branch(self, pr: int, pull: dict[str, object]) -> None:
        """Step 7.5: the merged PR's branch goes when it is in this repository. A failure (the
        session proxy answers 403, T-1938) is logged, never a stop: the merge has landed."""
        repo = self.client.base.removeprefix("repos/").lower()
        if merge_ready.text_of(pull, "head", "repo", "full_name").lower() != repo:
            return
        try:
            self.client.delete_branch(merge_ready.text_of(pull, "head", "ref"))
        except GhError as error:
            self.log.append(f"the branch of #{pr} was not deleted: {error}")

    def merge_green(self, train: Train, tip: str) -> list[int]:
        """Step 7: squash the members in order, each between a pre- and a post-squash check. The
        PRs merged form a prefix of the train; the first failure stops the sequence."""
        prs = [m.pr for m in train.members]
        self.renew(prs)
        if remote_sha(MAIN_REF, self.root) != train.base:
            raise Stop("main moved without the lock before the first squash", p0=True, pr=prs[0])
        expected, merged = train.base, []
        batch = ", ".join(f"#{pr}" for pr in prs)
        for k, member in enumerate(train.members):
            self.checkpoint()
            self.renew(prs[k:])
            pull = self.pre_squash(member, expected)
            squashed = self.squash(member, pull)
            merged.append(member.pr)
            self.log.append(f"merged #{member.pr}")
            expected = self.post_squash(member, expected, squashed)
            self.main = expected  # our own squash is not a reason to re-read the switch
            self.client.remove_label(member.pr, merge_ready.READY_LABEL)
            self.client.comment(member.pr, f"merged by merge train {tip} (batch {batch})")
            self.delete_branch(member.pr, pull)
        return merged

    def unlabel(self, pr: int, reason: str, *, eject: bool = False) -> None:
        """Step 2's and step 3's side effects in `train` mode: the PR leaves the ready set."""
        self.client.remove_label(pr, merge_ready.READY_LABEL)
        if eject:
            self.client.add_label(pr, merge_ready.EJECTED_LABEL)
        self.client.comment(pr, f"merge train: {merge_ready.READY_LABEL} removed: {reason}")
        self.log.append(f"{'ejected' if eject else 'unlabelled'} #{pr}: {reason}")

    def hold(self) -> list[int]:
        """Steps 2-7 under a held lock. Returns the PRs merged."""
        self.acquired = self.renewed = self.clock()
        self.checkpoint()  # right after acquisition: a pending flip pushes nothing
        repo = merge_ready.Repo(self.root)
        repo.fetch_base()
        base = git("rev-parse", f"refs/merge-ready/{merge_ready.BASE_BRANCH}", root=self.root).stdout.strip()
        found = collect(self.client, repo)
        self.log += found.warning_lines()
        for pr, reason in found.unlabel:
            self.unlabel(pr, reason)
        heads = [(v.pr, v.marker.head) for v in found.ready[:BATCH_MAX] if v.marker is not None]
        if not heads:
            self.log.append("no ready PR")
            return []
        self.lease = remote_sha(self.ref, self.root)
        self.collected(base, heads)
        state = self.recorded()
        merged: list[int] = []
        while heads:
            train = self.build_on(base, heads)
            if not train.members:
                self.log.append("no PR left after the build")
                break
            known_red, notes = None, []
            if state is not None:
                known_red, reason = state.matches(train)
                if known_red is None:
                    notes = [f"restart: {reason}"]
                    self.log.append(f"restarting the search: {reason}")
                else:
                    self.log.append(f"resuming the search: the first {known_red} PR(s) are known red")
                state = None
            done, base, heads = self.resolve(train, known_red, notes)
            merged += done
        return merged

    def collected(self, base: str, heads: Sequence[tuple[int, str]]) -> None:
        """A hook between step 2 and the first build; shadow mode skips a repeated round there."""

    def build_on(self, base: str, heads: Sequence[tuple[int, str]]) -> Train:
        """Steps 3 and 4 on `base` in scratch worktrees, with step 3's ejections made."""
        with worktrees(self.root, base) as (train_tree, scripts_tree):
            train = build(train_tree, scripts_tree / "scripts", base, heads)
        for pr, reason in train.ejected:
            self.unlabel(pr, reason, eject=True)
        self.log += [f"deferred #{pr}: {reason}" for pr, reason in train.deferred]
        return train

    def resolve(
        self, train: Train, hi: int | None, notes: Sequence[str]
    ) -> tuple[list[int], str, list[tuple[int, str]]]:
        """One train to its end (SPEC-0027 section 6). A green train merges whole. A red one is
        searched for its first red prefix `P_1..P_hi` (`hi` given when a resume knows it), the
        green prefix `P_1..P_lo` merges, and `P_lo+1` gets its confirmation run on the new `main`.
        Returns the PRs merged, the new `main` and the members left to re-test as a new train."""
        members, prs = train.members, [m.pr for m in train.members]
        lo, green_tip, red_runs = 0, "", tuple[int, ...]()
        try:
            if hi is None:
                outcome, tip = self.test(train, len(members), prs, notes)
                if outcome.state == merge_ready.GREEN:
                    return self.merge_green(train, tip), self.main, []
                hi, red_runs = len(members), outcome.runs
            while hi - lo > 1:
                mid = (lo + hi) // 2
                prefix = Train(train.base, members[:mid])
                outcome, tip = self.test(prefix, mid, prs, [f"bisect: prefix {mid} of {len(members)}"])
                if outcome.state == merge_ready.GREEN:
                    lo, green_tip = mid, tip
                else:
                    hi, red_runs = mid, outcome.runs
        except CutOff:
            if hi is not None:
                base = self.merge_prefix(train, lo, green_tip)
                self.record(Resume(base, tuple((m.pr, m.head) for m in members[lo:hi])))
            raise
        base = self.merge_prefix(train, lo, green_tip)
        merged = prs[:lo]
        culprit = members[lo]
        rest = [(m.pr, m.head) for m in members[lo + 1:]]
        alone = self.build_on(base, [(culprit.pr, culprit.head)])
        if not alone.members:
            return merged, base, rest
        try:
            outcome, tip = self.test(alone, 1, prs[lo:], [f"confirm: #{culprit.pr}"])
        except CutOff:
            self.record(Resume(base, ((culprit.pr, culprit.head),)))
            raise
        links = ", ".join(self.run_link(r) for r in (*red_runs, *outcome.runs)) or "none"
        if outcome.state == merge_ready.GREEN:
            self.tasks.append(("P1", f"merge train: possible flaky test or interaction at #{culprit.pr}",
                               f"#{culprit.pr}'s train prefix was red on {train.base}, then green alone on "
                               f"{base}; runs: {links}"))
            self.log.append(f"P1: #{culprit.pr}'s red train did not reproduce; not ejected")
            return merged + self.merge_green(alone, tip), self.main, rest
        self.eject_red(culprit, outcome.jobs, links)
        return merged, base, rest

    def merge_prefix(self, train: Train, k: int, tip: str) -> str:
        """Step 7 for the tested prefix `P_1..P_k` (its CI tested `tip`); returns `main` after."""
        if k == 0:
            return train.base
        self.merge_green(Train(train.base, train.members[:k]), tip)
        return self.main

    def run_link(self, run_id: int) -> str:
        return f"https://github.com/{self.client.base.removeprefix('repos/')}/actions/runs/{run_id}"

    def eject_red(self, member: Member, jobs: Sequence[str], links: str) -> None:
        """Section 6's ejection of a confirmed culprit: the labels and a comment that names the
        failing jobs and runs and ends with the `merge-train-ejected v1` block."""
        self.client.remove_label(member.pr, merge_ready.READY_LABEL)
        self.client.add_label(member.pr, merge_ready.EJECTED_LABEL)
        failing = "; ".join(jobs).replace("@", "@" + ZWJ)
        self.client.comment(member.pr, (
            f"merge train: {merge_ready.READY_LABEL} removed: train CI red, confirmed on this PR alone on main\n\n"
            f"failing jobs: {failing}\nruns: {links}\n\n"
            f"<!-- merge-train-ejected v1 head: {member.head} reason: ci-red -->"
        ))
        self.log.append(f"ejected #{member.pr}: ci-red")

    def run(self, window: float = WINDOW) -> int:
        """Step 1, the hold and step 9. 0 when the hold ended normally, 1 when it stopped."""
        start = self.clock()
        mode = read_mode(self.root)
        if mode != TRAIN:
            self.log.append(f"the mode on main is {mode}: the merger stops")
            return 0
        if self.lock.wait(timeout_minutes=max(window - (self.clock() - start), 0) / 60) != 0:
            self.log.append("the merge lock was not acquired in this window")
            return 0
        try:
            code = 0
            try:
                self.hold()
            except Stop as stop:
                self.log.append(("P0: " if stop.p0 else "stopped: ") + str(stop))
                if stop.p0:
                    respond(self.root, self.client, self.log, self.tasks, self.lease, stop)
                code = 1
            except (GhError, GitError):
                file_tasks(self.root, self.client, self.log, self.tasks)  # a P1 found before the error is still filed
                raise
            file_tasks(self.root, self.client, self.log, self.tasks)
            return code
        finally:
            self.lock.release()


def raise_on_term(signum: int, _frame: object) -> None:
    """SIGTERM unwinds like an error, so the hold's `finally` releases the lock."""
    raise SystemExit(128 + signum)


def main(argv: list[str], root: Path = ROOT) -> int:
    if argv == ["mode"]:
        print(mode_line(root))
        return 0
    step = argv[1:] == ["--step"] and argv[0] in ("--train", "--shadow")
    if argv not in (["--dry-run"], ["--train"], ["--shadow"]) and not step:
        print("usage: merge_train.py mode | --dry-run | (--train | --shadow) [--step]", file=sys.stderr)
        return 2
    real = gh_client.from_remote(root, REMOTE)
    if real is None:
        print("merge_train: origin is not a github.com repository", file=sys.stderr)
        return 2
    stage: Stage | None = None
    try:
        if argv == ["--dry-run"]:
            print(dry_run(real, root))
            return 0
        signal.signal(signal.SIGTERM, raise_on_term)
        # Every write of a --step run is an action the session performs (decision 0169).
        client: Client = merge_step.StepClient(real, merge_step.AckFile(root)) if step else real
        # Imported here: merge_train_shadow subclasses this module's Stage.
        from merge_train_shadow import ShadowStage

        kind = ShadowStage if argv[0] == "--shadow" else Stage
        stage = kind(client, root, LockOps(f"train-{int(time.time())}", root))
        code = stage.run()
        print("\n".join(stage.log))
        return code
    except merge_step.StepError as error:
        # The lock was released on the way out; nothing after the failed action ran.
        print("\n".join(stage.log if stage is not None else []))
        print(f"merge_train: step mode aborted: {error}", file=sys.stderr)
        return 2
    except (GhError, GitError) as error:
        print(f"merge_train: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    # Run the imported module, not `__main__`, so merge_train_shadow subclasses the same `Stage`.
    import merge_train

    sys.exit(merge_train.main(sys.argv[1:]))
