#!/usr/bin/env python3
"""The merge train (SPEC-0027 section 4): batched merges of ready PRs under the merge lock.

  scripts/merge_train.py mode        # the mode on origin/main: lock, shadow or train
  scripts/merge_train.py --dry-run   # steps 2-4: collect, build, id checks; prints, writes nothing
  scripts/merge_train.py --train     # steps 1-7 for a green train, in `train` mode only

The mode is read from `scripts/merge_train.toml` on a freshly fetched `origin/main`, never from
the checkout, so a branch cannot switch its own flow. The read fails closed: a missing file, a
TOML error (a duplicate key included), or a missing, non-string or unknown `mode` reads as `lock`.
"""

from __future__ import annotations

import re
import signal
import subprocess
import sys
import tempfile
import time
import tomllib
import unicodedata
from collections.abc import Callable, Iterator, Sequence
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path

import ci_changes
import gh_client
import merge_lock
import merge_ready
from gh_client import SHA, Client, GhError
from merge_lock import GitError, git

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
SWITCH = "scripts/merge_train.toml"
LOCK, SHADOW, TRAIN = "lock", "shadow", "train"
MODES = (LOCK, SHADOW, TRAIN)
# The id checks of step 4; each takes `--root <tree>`. They run from `main`'s scripts (M0).
ID_CHECKS = (("tasks.py", "validate"), ("check_decisions.py",))
ID_CHECK_TIMEOUT = 300
BATCH_MAX = 5
# The committer of train merge commits, so a clone with no git identity can still build.
IDENTITY = ("-c", "user.name=merge-train", "-c", "user.email=merge-train@invalid")
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
        git("fetch", "-q", "--no-tags", REMOTE, "+refs/heads/main:refs/remotes/origin/main", root=root)
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


@contextmanager
def worktrees(root: Path, base: str) -> Iterator[tuple[Path, Path]]:
    """Two scratch worktrees at `base` outside the checkout: the train's and, read-only, M0's
    (its `scripts` run the id checks). Both are removed on the way out."""
    with tempfile.TemporaryDirectory(prefix="merge-train-") as scratch:
        trees = (Path(scratch) / "train", Path(scratch) / "m0")
        try:
            for tree in trees:
                git("worktree", "add", "-q", "--detach", str(tree), base, root=root)
            yield trees
        finally:
            for tree in trees:
                git("worktree", "remove", "--force", str(tree), root=root, check=False)


@dataclass
class Collected:
    """Step 2: the ready PRs oldest marker first, and what the merger would do about the rest."""

    ready: list[merge_ready.Verdict] = field(default_factory=list)
    unlabel: list[tuple[int, str]] = field(default_factory=list)
    keep: list[tuple[int, str]] = field(default_factory=list)


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
    return "\n".join(lines)


# The merge stage of a green train (SPEC-0027 section 4, steps 1 and 5-7; T-2994). The
# constants are section 4's.
TRAIN_BRANCH = "merge-train/current"
TRAIN_REF = f"refs/heads/{TRAIN_BRANCH}"
MAIN_REF = "refs/heads/main"
CI_WORKFLOW_FILE = Path(merge_ready.CI_WORKFLOW).name
TRAIN_EVENTS = ("push", "workflow_dispatch")
WINDOW = 40 * 60  # a merger run's polling window; the lock wait is capped at what is left of it
HOLD_BUDGET = 40 * 60  # counted from the lock's acquisition
CI_TIMEOUT = 10 * 60
MERGE_BUDGET = 150  # per PR a CI run can merge: the pre-squash wait, the merge call, the checks
CI_POLL = 60  # every CI poll is a stop checkpoint
DISPATCH_AFTER = 3 * 60  # no run by then: dispatch once; still none after as long again: stop
RENEW_EVERY = 5 * 60
PRE_SQUASH_POLL = 5
PRE_SQUASH_TIMEOUT = 2 * 60
TITLE_MAX = 100
ZWJ = "\u200d"
CLOSING_KEYWORD = re.compile(r"\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\b", re.IGNORECASE)
CO_AUTHOR = re.compile(r"Co-Authored-By: [^@#<>\n]+ <[^@<>\s]+@[^@<>\s]+>")
SESSION = re.compile(r"Claude-Session: https://claude\.ai/code/session_[A-Za-z0-9]{1,64}")


class Stop(Exception):
    """The train stops this hold with nothing more merged. `p0` when `main` differs from the
    tested train (step 7's mismatch response); the P0 and the revert PR are T-3055's."""

    def __init__(self, reason: str, *, p0: bool = False) -> None:
        super().__init__(reason)
        self.p0 = p0


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
    log: list[str] = field(default_factory=list)

    def checkpoint(self) -> None:
        """A stop checkpoint: when `main` moved since the last one, re-read the switch from it
        (failing closed) and stop unless it still says `train`. The stop label is T-3055's."""
        sha = remote_sha(MAIN_REF, self.root)
        if sha != self.main:
            self.main = sha
            mode = read_mode(self.root)
            if mode != TRAIN:
                raise Stop(f"the mode on main is now {mode}")

    def renew(self, prs: Sequence[int]) -> None:
        if self.lock.renew(prs) != 0:
            raise Stop("the merge lock was lost")
        self.renewed = self.clock()

    def left(self) -> float:
        return HOLD_BUDGET - (self.clock() - self.acquired)

    def push(self, train: Train, lease: str) -> str:
        """Step 5: push a record commit on the train's tip (same tree; its message lists the
        batch) to TRAIN_REF, leased on the sha read before the build. Returns the pushed sha."""
        lines = [f"merge train on main {train.base}", ""]
        lines += [f"#{m.pr} {m.head}" for m in train.members]
        tip = git(
            *IDENTITY, "commit-tree", tree_of(train.tip, self.root), "-p", train.tip, "-m", "\n".join(lines),
            root=self.root,
        ).stdout.strip()
        pushed = git("push", "-q", f"--force-with-lease={TRAIN_REF}:{lease}", REMOTE, f"{tip}:{TRAIN_REF}",
                     root=self.root, check=False)
        if pushed.returncode != 0:
            raise Stop(f"infrastructure: the push to {TRAIN_BRANCH} lost its lease under the lock")
        return tip

    def ci(self, tip: str, docs_only: bool | None) -> tuple[str, list[str], bool]:
        """The train CI's state on exactly `tip` (step 6), and whether a matching run exists."""
        runs = [
            r for r in self.client.workflow_runs(branch=TRAIN_BRANCH, head_sha=tip)
            if merge_ready.text_of(r, "head_sha") == tip and merge_ready.text_of(r, "head_branch") == TRAIN_BRANCH
        ]
        seen = any(
            merge_ready.text_of(r, "path") == merge_ready.CI_WORKFLOW
            and merge_ready.text_of(r, "event") in TRAIN_EVENTS for r in runs
        )
        state, jobs = merge_ready.ci_state(runs, self.client.check_runs(tip), docs_only, TRAIN_EVENTS)
        return state, jobs, seen

    def wait_ci(self, tip: str, docs_only: bool | None, prs: Sequence[int]) -> None:
        """Step 6: the budget gate, then the CI wait on `tip`; returns only when it is green."""
        need = CI_TIMEOUT + len(prs) * MERGE_BUDGET
        if self.left() < need:
            raise Stop(f"hold budget: {self.left():.0f} s left, a CI wait for {len(prs)} PR(s) needs {need} s")
        start = self.clock()
        dispatched: float | None = None
        while True:
            self.checkpoint()
            if self.clock() - self.renewed >= RENEW_EVERY:
                self.renew(prs)
            state, jobs, seen = self.ci(tip, docs_only)
            if state == merge_ready.GREEN:
                return
            if state == merge_ready.RED:
                raise Stop(f"train CI red ({'; '.join(jobs)}); red trains are T-2998's")
            now = self.clock()
            if not seen and dispatched is None and now - start >= DISPATCH_AFTER:
                self.client.dispatch(CI_WORKFLOW_FILE, TRAIN_BRANCH)
                dispatched = now
            elif not seen and dispatched is not None and now - dispatched >= DISPATCH_AFTER:
                raise Stop("infrastructure: no CI run on the train, even after a dispatch")
            if now - start >= CI_TIMEOUT:
                raise Stop(f"infrastructure: the train CI took over {CI_TIMEOUT // 60} min")
            self.sleep(CI_POLL)

    def pre_squash(self, member: Member, expected: str) -> dict[str, object]:
        """Step 7.2: wait until GitHub's test merge of the PR sits on `expected` (the current
        `main`), and require its tree to be the train's. Returns the PR read."""
        deadline = self.clock() + PRE_SQUASH_TIMEOUT
        while True:
            pull = self.client.pull(member.pr)
            merge = merge_ready.text_of(pull, "merge_commit_sha")
            if pull.get("mergeable") is not None and SHA.fullmatch(merge):
                commit = self.client.commit(merge)
                parents = merge_ready.field_of(commit, "parents")
                first = parents[0] if isinstance(parents, list) and parents else None
                if merge_ready.text_of(first, "sha") == expected:
                    if merge_ready.text_of(commit, "tree", "sha") != tree_of(member.merge, self.root):
                        raise Stop(f"main differs from the tested train at #{member.pr}: GitHub's test merge "
                                   "has another tree", p0=True)
                    return pull
                if remote_sha(MAIN_REF, self.root) != expected:
                    raise Stop(f"main moved without the lock before squashing #{member.pr}", p0=True)
            if self.clock() >= deadline:
                raise Stop(f"GitHub's test merge of #{member.pr} was not recomputed on {expected[:12]} "
                           f"within {PRE_SQUASH_TIMEOUT // 60} min")
            self.sleep(PRE_SQUASH_POLL)

    def squash(self, member: Member, pull: dict[str, object]) -> None:
        """Step 7.3: squash-merge the PR only at its tested head. A lost reply is read back."""
        title = squash_title(merge_ready.text_of(pull, "title"), member.pr)
        message = squash_trailers(self.client.pull_commits(member.pr))
        try:
            self.client.merge(member.pr, member.head, title=title, message=message)
        except GhError as error:
            if not merge_ready.field_of(self.client.pull(member.pr), "merged"):
                raise Stop(f"the merge of #{member.pr} was refused ({error})") from error

    def post_squash(self, member: Member, expected: str) -> str:
        """Step 7.4: `main` now has `expected` as its first parent and the train's tree."""
        git("fetch", "-q", "--no-tags", REMOTE, f"+{MAIN_REF}:refs/remotes/origin/main", root=self.root)
        main = git("rev-parse", "refs/remotes/origin/main", root=self.root).stdout.strip()
        parent = git("rev-parse", "--verify", "-q", f"{main}^1", root=self.root, check=False).stdout.strip()
        if parent != expected or tree_of(main, self.root) != tree_of(member.merge, self.root):
            raise Stop(f"main differs from the tested train after squashing #{member.pr} ({main})", p0=True)
        return main

    def merge_green(self, train: Train, tip: str) -> list[int]:
        """Step 7: squash the members in order, each between a pre- and a post-squash check. The
        PRs merged form a prefix of the train; the first failure stops the sequence."""
        prs = [m.pr for m in train.members]
        self.renew(prs)
        if remote_sha(MAIN_REF, self.root) != train.base:
            raise Stop("main moved without the lock before the first squash", p0=True)
        expected, merged = train.base, []
        batch = ", ".join(f"#{pr}" for pr in prs)
        for k, member in enumerate(train.members):
            self.checkpoint()
            self.renew(prs[k:])
            pull = self.pre_squash(member, expected)
            self.squash(member, pull)
            merged.append(member.pr)
            self.log.append(f"merged #{member.pr}")
            expected = self.post_squash(member, expected)
            self.main = expected  # our own squash is not a reason to re-read the switch
            self.client.remove_label(member.pr, merge_ready.READY_LABEL)
            self.client.comment(member.pr, f"merged by merge train {tip} (batch {batch})")
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
        for pr, reason in found.unlabel:
            self.unlabel(pr, reason)
        heads = [(v.pr, v.marker.head) for v in found.ready[:BATCH_MAX] if v.marker is not None]
        if not heads:
            self.log.append("no ready PR")
            return []
        lease = remote_sha(TRAIN_REF, self.root)
        with worktrees(self.root, base) as (train_tree, scripts_tree):
            train = build(train_tree, scripts_tree / "scripts", base, heads)
        for pr, reason in train.ejected:
            self.unlabel(pr, reason, eject=True)
        self.log += [f"deferred #{pr}: {reason}" for pr, reason in train.deferred]
        if not train.members:
            self.log.append("no PR left after the build")
            return []
        prs = [m.pr for m in train.members]
        tip = self.push(train, lease)
        self.log.append(f"pushed {tip} to {TRAIN_BRANCH}: " + ", ".join(f"#{pr}" for pr in prs))
        self.renew(prs)
        docs_only = not ci_changes.classify_range(base, tip, cwd=self.root)
        self.checkpoint()  # before the CI wait
        self.wait_ci(tip, docs_only, prs)
        return self.merge_green(train, tip)

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
            self.hold()
            return 0
        except Stop as stop:
            self.log.append(("P0: " if stop.p0 else "stopped: ") + str(stop))
            return 1
        finally:
            self.lock.release()


def raise_on_term(signum: int, _frame: object) -> None:
    """SIGTERM unwinds like an error, so the hold's `finally` releases the lock."""
    raise SystemExit(128 + signum)


def main(argv: list[str], root: Path = ROOT) -> int:
    if argv == ["mode"]:
        print(read_mode(root))
        return 0
    if argv not in (["--dry-run"], ["--train"]):
        print("usage: merge_train.py mode | --dry-run | --train", file=sys.stderr)
        return 2
    client = gh_client.from_remote(root, REMOTE)
    if client is None:
        print("merge_train: origin is not a github.com repository", file=sys.stderr)
        return 2
    try:
        if argv == ["--dry-run"]:
            print(dry_run(client, root))
            return 0
        signal.signal(signal.SIGTERM, raise_on_term)
        stage = Stage(client, root, LockOps(f"train-{int(time.time())}", root))
        code = stage.run()
        print("\n".join(stage.log))
        return code
    except (GhError, GitError) as error:
        print(f"merge_train: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
