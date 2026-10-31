"""Tests for scripts/merge_train.py (SPEC-0027 section 4 and its Test plan), against temporary
remotes and worktrees (stdlib unittest; run by `make hygiene`)."""

from __future__ import annotations

import random
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import unittest
from collections.abc import Callable, Sequence
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready  # noqa: E402 (the sys.path line above must run first)
import merge_train  # noqa: E402
import merge_train_filing  # noqa: E402
from gh_client import Client, FakeGh, GhError, Reply, Request  # noqa: E402

SCRIPTS = Path(__file__).resolve().parent
TASK = """---
id: {tid}
title: "t"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---
"""


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


class Mode(unittest.TestCase):
    def test_the_read_fails_closed(self) -> None:
        self.assertEqual(merge_train.parse_mode('mode = "train"\n'), merge_train.TRAIN)
        self.assertEqual(merge_train.parse_mode('mode = "shadow"\n'), merge_train.SHADOW)
        for text in (
            None,  # missing file
            "mode = ",  # malformed
            'mode = "train"\nmode = "train"\n',  # duplicate key
            'mode = "fast"\n',  # unknown value
            "mode = 1\n",
            '[x]\nmode = "train"\n',
            "",
        ):
            with self.subTest(text=text):
                self.assertEqual(merge_train.parse_mode(text), merge_train.LOCK)

    def test_the_mode_comes_from_origin_main_not_the_checkout(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            origin, clone = base / "origin.git", base / "clone"
            run("git", "init", "-q", "--bare", "-b", "main", str(origin), cwd=base)
            run("git", "clone", "-q", str(origin), str(clone), cwd=base)
            for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
                run("git", "config", key, value, cwd=clone)
            run("git", "checkout", "-q", "-b", "main", cwd=clone)
            (clone / "scripts").mkdir()
            (clone / "README").write_text("x")
            run("git", "add", "-A", cwd=clone)
            run("git", "commit", "-q", "-m", "no switch", cwd=clone)
            run("git", "push", "-q", "origin", "main", cwd=clone)
            # The checkout says train; origin/main has no switch file.
            (clone / merge_train.SWITCH).write_text('mode = "train"\n')
            self.assertEqual(merge_train.read_mode(clone), merge_train.LOCK)
            run("git", "add", "-A", cwd=clone)
            run("git", "commit", "-q", "-m", "switch", cwd=clone)
            run("git", "push", "-q", "origin", "main", cwd=clone)
            self.assertEqual(merge_train.read_mode(clone), merge_train.TRAIN)
            self.assertEqual(merge_train.read_mode(base / "nowhere"), merge_train.LOCK)

    def test_the_switch_file_names_a_mode(self) -> None:
        # Not pinned to one mode: a flip PR (decision 0157) changes only merge_train.toml.
        text = (SCRIPTS / "merge_train.toml").read_text()
        self.assertRegex(text, r'(?m)^mode = "(lock|shadow|train)"$')
        self.assertIn(merge_train.parse_mode(text), (merge_train.LOCK, merge_train.SHADOW, merge_train.TRAIN))


class IdChecks(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        # M0's scripts, and a train tree in a git repository of its own.
        self.scripts = base / "m0" / "scripts"
        self.scripts.mkdir(parents=True)
        for name in ("tasks.py", "check_decisions.py"):
            shutil.copy(SCRIPTS / name, self.scripts / name)
        self.tree = base / "train"
        for path, text in (
            ("docs/tasks/M2/T-0001-a.md", TASK.format(tid="T-0001")),
            ("docs/decisions/README.md", "x"),
            ("docs/decisions/0001-a.md", "x"),
        ):
            (self.tree / path).parent.mkdir(parents=True, exist_ok=True)
            (self.tree / path).write_text(text)
        run("git", "init", "-q", str(self.tree), cwd=base)
        run("git", "add", "-A", cwd=self.tree)

    def add(self, path: str, text: str) -> None:
        (self.tree / path).parent.mkdir(parents=True, exist_ok=True)
        (self.tree / path).write_text(text)
        run("git", "add", "-A", cwd=self.tree)

    def test_a_clean_tree_passes_and_collisions_fail(self) -> None:
        self.assertEqual(merge_train.id_check_failures(self.scripts, self.tree), [])
        self.add("docs/tasks/M2/T-0001-b.md", TASK.format(tid="T-0001"))
        self.add("docs/decisions/0001-b.md", "x")
        failures = merge_train.id_check_failures(self.scripts, self.tree)
        self.assertTrue(any("duplicate id T-0001" in f for f in failures), failures)
        self.assertTrue(any("decision number 0001 used twice" in f for f in failures), failures)

    def test_a_planted_module_in_the_train_tree_is_not_imported(self) -> None:
        planted = self.tree / "imported"
        hostile = f"from pathlib import Path\nPath({str(planted)!r}).write_text('x')\n"
        for name in ("tasks.py", "check_decisions.py", "re.py", "pathlib.py", "sitecustomize.py"):
            self.add(f"scripts/{name}", hostile)
            self.add(name, hostile)
        self.assertEqual(merge_train.id_check_failures(self.scripts, self.tree), [])
        self.assertFalse(planted.exists())

    def test_root_is_refused_outside_validate(self) -> None:
        tasks = subprocess.run(
            [sys.executable, "-I", str(self.scripts / "tasks.py"), "list", "--root", str(self.tree)],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(tasks.returncode, 2)
        decisions = subprocess.run(
            [sys.executable, "-I", str(self.scripts / "check_decisions.py"), "--root", str(self.tree / "none")],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(decisions.returncode, 2)


class Build(unittest.TestCase):
    """Steps 3-4 on a scratch repository whose `main` is M0; PRs are branches of it."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name) / "repo"
        run("git", "init", "-q", "-b", "main", str(self.repo), cwd=Path(self.tmp.name))
        for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
            self.git("config", key, value)
        self.write("docs/decisions/README.md", "x")
        self.write("docs/decisions/0001-a.md", "x")
        self.write("shared.txt", "one\ntwo\nthree\n")
        self.m0 = self.commit("M0")

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.repo)

    def write(self, path: str, text: str) -> None:
        (self.repo / path).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / path).write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")

    def pr(self, start: str, files: dict[str, str | None]) -> str:
        """A PR head: from `start`, write each file (None deletes it)."""
        self.git("checkout", "-q", "-f", "--detach", start)
        for path, text in files.items():
            if text is None:
                (self.repo / path).unlink()
            else:
                self.write(path, text)
        return self.commit(f"pr {sorted(files)}")

    def build(self, *heads: str, base: str | None = None) -> merge_train.Train:
        return merge_train.build(self.repo, SCRIPTS, base or self.m0, list(enumerate(heads, 1)))

    def test_independent_prs_merge_in_order_up_to_the_cap(self) -> None:
        heads = [self.pr(self.m0, {f"f{k}.txt": "x"}) for k in range(merge_train.BATCH_MAX + 1)]
        train = self.build(*heads)
        self.assertEqual([m.pr for m in train.members], list(range(1, merge_train.BATCH_MAX + 1)))
        self.assertEqual((train.deferred, train.ejected), ([], []))
        self.assertEqual(self.git("rev-parse", f"{train.tip}^1"), train.members[-2].merge)

    def test_stacked_and_intra_batch_conflicts_defer_and_main_conflicts_eject(self) -> None:
        first = self.pr(self.m0, {"a.txt": "a", "shared.txt": "one\nTWO\nthree\n"})
        stacked = self.pr(first, {"b.txt": "b"})
        rival = self.pr(self.m0, {"shared.txt": "one\n2\nthree\n"})
        self.git("checkout", "-q", "-f", "--detach", self.m0)
        self.write("shared.txt", "one\nzwei\nthree\n")
        main = self.commit("main moves")
        train = self.build(first, stacked, rival, base=main)
        self.assertEqual([m.pr for m in train.members], [])
        # The stacked PR carries the first one's edit, so it conflicts with main too.
        self.assertEqual([pr for pr, _ in train.ejected], [1, 2, 3])
        train = self.build(first, stacked, rival)
        self.assertEqual([m.pr for m in train.members], [1])
        self.assertEqual([pr for pr, _ in train.deferred], [2, 3])

    def test_a_criss_cross_history_defers(self) -> None:
        x = self.pr(self.m0, {"x.txt": "x"})
        y = self.pr(self.m0, {"y.txt": "y"})
        self.git("checkout", "-q", "-f", "--detach", x)
        self.git("merge", "-q", "--no-edit", y)
        one = self.pr(self.git("rev-parse", "HEAD"), {"one.txt": "1"})
        self.git("checkout", "-q", "-f", "--detach", y)
        self.git("merge", "-q", "--no-edit", x)
        two = self.pr(self.git("rev-parse", "HEAD"), {"two.txt": "2"})
        self.assertEqual(len(self.git("merge-base", "--all", one, two).split()), 2)
        train = self.build(one, two)
        self.assertEqual(([m.pr for m in train.members], [pr for pr, _ in train.deferred]), ([1], [2]))

    def test_id_collisions_eject_the_later_copy(self) -> None:
        task = TASK.format(tid="T-0005")
        first = self.pr(self.m0, {"docs/tasks/M2/T-0005-a.md": task})
        later = self.pr(self.m0, {"docs/tasks/M2/T-0005-b.md": task})
        with_main = self.pr(self.m0, {"docs/decisions/0001-b.md": "x"})
        train = self.build(first, later, with_main)
        self.assertEqual([m.pr for m in train.members], [1])
        self.assertEqual(
            train.ejected,
            [
                (2, "id collision with T-0005; renumber per decision 0121 (2)"),
                (3, "id collision with 0001; renumber per decision 0121 (2)"),
            ],
        )

    def test_squashing_the_members_in_order_gives_each_train_tree(self) -> None:
        """SPEC-0027 section 5: for independent PRs, each squash onto the previous one has the
        train's tree, with renames, deletions and merges of `main` into a PR included."""
        rng = random.Random(2987)
        for trial in range(3):
            with self.subTest(trial=trial):
                self.git("checkout", "-q", "-f", "--detach", self.m0)
                history = [self.m0]
                for i in range(3):
                    self.write(f"main{trial}-{i}.txt", f"{i}\n")
                    history.append(self.commit(f"main {i}"))
                base = history[-1]
                heads = []
                for k in range(merge_train.BATCH_MAX):
                    own = f"pr{trial}-{k}"
                    head = self.pr(rng.choice(history[:-1]), {f"{own}.txt": "a\n", f"{own}-gone.txt": "b\n"})
                    if rng.random() < 0.5:  # a later merge of main into the PR
                        self.git("merge", "-q", "--no-edit", rng.choice(history))
                    self.git("mv", f"{own}.txt", f"{own}-renamed.txt")
                    heads.append(self.pr(self.commit("rename"), {f"{own}-gone.txt": None}))
                train = self.build(*heads, base=base)
                self.assertEqual(len(train.members), merge_train.BATCH_MAX)
                tip = set(self.git("ls-tree", "-r", "--name-only", train.tip).split())
                self.assertIn(f"pr{trial}-0-renamed.txt", tip)
                self.assertNotIn(f"pr{trial}-0-gone.txt", tip)
                squash = base
                for member in train.members:
                    tree = self.git("merge-tree", "--write-tree", squash, member.head)
                    squash = self.git("commit-tree", tree, "-p", squash, "-m", "squash")
                    self.assertEqual(tree, self.git("rev-parse", f"{member.merge}^{{tree}}"))


def verdict(pr: int, head: str, at: str, *failures: merge_ready.Failure) -> merge_ready.Verdict:
    mark = merge_ready.Marker(head, None, None, {}, "session-1")
    return merge_ready.Verdict(pr, mark, pr, at, failures)


class DryRun(unittest.TestCase):
    """Collect and the dry run against a temporary origin whose refs/pull/<n>/head are PR heads;
    V1-V6 themselves are scripts/test_merge_ready.py's."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        self.origin, self.clone = base / "origin.git", base / "clone"
        run("git", "init", "-q", "--bare", "-b", "main", str(self.origin), cwd=base)
        run("git", "clone", "-q", str(self.origin), str(self.clone), cwd=base)
        for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
            self.git("config", key, value)
        self.git("checkout", "-q", "-b", "main")
        (self.clone / "scripts").mkdir()
        for name in ("tasks.py", "check_decisions.py"):
            shutil.copy(SCRIPTS / name, self.clone / "scripts" / name)
        for path in ("docs/decisions/README.md", "docs/decisions/0001-a.md"):
            (self.clone / path).parent.mkdir(parents=True, exist_ok=True)
            (self.clone / path).write_text("x")
        self.base = self.commit("main")
        self.git("push", "-q", "origin", "main")
        self.heads = {}
        for pr in (1, 2, 3, 4):
            self.git("checkout", "-q", "-f", "--detach", self.base)
            (self.clone / f"f{pr}.txt").write_text("x")
            self.heads[pr] = self.commit(f"pr {pr}")
            self.git("push", "-q", "origin", f"HEAD:refs/pull/{pr}/head")
        self.git("checkout", "-q", "-f", "main")
        # PR 2's marker is the oldest; PR 3 has red CI, PR 4 pending CI, PR 5 no label.
        self.verdicts = {
            1: verdict(1, self.heads[1], "2026-10-28T11:00:00Z"),
            2: verdict(2, self.heads[2], "2026-10-28T10:00:00Z"),
            3: verdict(3, self.heads[3], "2026-10-28T09:00:00Z", merge_ready.Failure("V3", "CI red", merge_ready.UNLABEL)),
            4: verdict(4, self.heads[4], "2026-10-28T09:00:00Z", merge_ready.Failure("V3", "CI pending", merge_ready.KEEP)),
        }
        label = [{"name": merge_ready.READY_LABEL}]
        pulls = [{"number": pr, "labels": label} for pr in (1, 2, 3, 4)] + [{"number": 5, "labels": []}]
        replies = {("GET", "repos/o/r/pulls?state=open&per_page=100&page=1"): Reply(pulls)}
        for pr in (1, 2):
            replies[("GET", f"repos/o/r/pulls/{pr}")] = Reply({"mergeable_state": "clean"})
        self.fake = FakeGh("o", "r", replies)

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.clone)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD")

    def dry_run(self) -> str:
        with mock.patch.object(merge_train.merge_ready, "verify", side_effect=lambda c, pr, repo: self.verdicts[pr]):
            for pr in self.heads:  # verify fetches each head; the mock does not
                self.git("fetch", "-q", "origin", f"+refs/pull/{pr}/head:refs/merge-ready/pull/{pr}")
            return merge_train.dry_run(self.fake, self.clone)

    def test_a_dry_run_reports_the_batch_and_writes_nothing(self) -> None:
        refs = run("git", "for-each-ref", cwd=self.origin)
        report = self.dry_run()
        self.assertIn(f"batch: #2 ({self.heads[2][:12]}), #1 ({self.heads[1][:12]})", report)
        self.assertIn("would unlabel #3: CI red", report)
        self.assertIn("skipped, label kept #4: CI pending", report)
        self.assertNotIn("#5", report)
        self.assertEqual({r.method for r in self.fake.requests}, {"GET"})
        self.assertEqual(run("git", "for-each-ref", cwd=self.origin), refs)
        self.assertEqual(self.git("worktree", "list", "--porcelain").count("worktree "), 1)

    def test_a_conflicting_pr_is_unlabelled(self) -> None:
        self.fake.replies[("GET", "repos/o/r/pulls/1")] = Reply({"mergeable_state": "dirty"})
        report = self.dry_run()
        self.assertIn("would unlabel #1: conflict with main", report)
        self.assertIn("batch: #2", report)


class SquashText(unittest.TestCase):
    def test_titles_are_sanitized(self) -> None:
        zwj = merge_train.ZWJ
        title = merge_train.squash_title("T-1: fixes #12 for @bob\x07\u202e\u2028" + "x" * 200, 7)
        self.assertTrue(title.startswith(f"T-1: f{zwj}ixes #{zwj}12 for @{zwj}bob"))
        self.assertTrue(title.endswith(" (#7)"))
        self.assertNotIn("\x07", title)
        self.assertNotIn("\u202e", title)
        self.assertNotIn("\u2028", title)
        self.assertEqual(merge_train.squash_title("Closed: resolve it", 1), f"C{zwj}losed: r{zwj}esolve it (#1)")

    def test_only_well_formed_trailers_are_kept_once(self) -> None:
        good = "Co-Authored-By: Claude <noreply@anthropic.com>"
        session = "Claude-Session: https://claude.ai/code/session_01Abc"
        commits = [
            {"commit": {"message": f"T-1: a\n\nfree text @someone #3\n{good}\n{session}"}},
            {"commit": {"message": f"b\n\n{good}\nCo-Authored-By: @evil <x@y.z>\nCo-Authored-By: a#b <x@y.z>\n"
                                   "Claude-Session: https://evil.example/session_1"}},
        ]
        self.assertEqual(merge_train.squash_trailers(commits), f"{good}\n{session}")


class FakeLock(merge_train.LockOps):
    """The merge lock as a script: `renew` answers from `renewals` (then 0)."""

    def __init__(self, renewals: list[int] | None = None) -> None:
        super().__init__("train-test", Path("."))
        self.renewals = renewals or []
        self.calls: list[tuple[str, object]] = []
        self.on_wait: list[Callable[[], None]] = []
        self.on_renew: list[Callable[[], None]] = []

    def wait(self, timeout_minutes: float) -> int:
        self.calls.append(("wait", timeout_minutes))
        for hook in self.on_wait:
            hook()
        return 0

    def renew(self, prs: Sequence[int]) -> int:
        self.calls.append(("renew", list(prs)))
        if self.on_renew:
            self.on_renew.pop(0)()
        return self.renewals.pop(0) if self.renewals else 0

    def release(self) -> int:
        self.calls.append(("release", None))
        return 0


class Clock:
    def __init__(self) -> None:
        self.now = 1000.0

    def __call__(self) -> float:
        return self.now

    def sleep(self, seconds: float) -> None:
        self.now += seconds


class GitHub(Client):
    """GitHub over a temporary bare `origin`: test merges, squash merges onto `main`, labels,
    comments, and train CI from `ci` (a list of states, one per poll; the last repeats)."""

    def __init__(self, origin: Path, heads: dict[int, str]) -> None:
        super().__init__("o", "r")
        self.origin, self.heads = origin, heads
        self.merged: list[int] = []
        self.writes: list[Request] = []
        self.ci = ["green"]
        self.hooks: dict[tuple[str, str], Callable[[], None]] = {}
        self.after: dict[tuple[str, str], Callable[[], None]] = {}
        self.bad_tree = False
        self.lose_reply = False
        self.mergeable: object = True
        self.refuse: set[int] = set()
        self.unlabelled: set[int] = set()
        self.closed: list[int] = []  # closed PRs still labelled ready-to-merge
        self.stop: set[int] = set()  # open PRs labelled merge-train-stop
        self.squashes: dict[int, str] = {}
        self.bad_squash: set[int] = set()  # PRs whose squash gets the tree of a merge with #2 instead
        self.opened: list[dict[str, object]] = []  # the bodies of the PRs the stage opened
        self.deleted: list[str] = []
        self.deny_delete = False  # the session proxy refuses branch deletion (T-1938)
        self.branch = merge_train.TRAIN_BRANCH  # the branch whose push CI runs report
        # The scripted CI oracle (T-2998): when set, each tested sha is pending for `polls` polls,
        # then concludes `oracle(sha)` ("green", "red" or "cancelled"), judged once per sha.
        self.oracle: Callable[[str], str] | None = None
        self.polls = 0
        self.tested: dict[str, list[object]] = {}

    def git(self, *args: str) -> str:
        return run("git", *merge_train_filing.IDENTITY, *args, cwd=self.origin)  # CI has no git identity

    def main(self) -> str:
        return self.git("rev-parse", "refs/heads/main")

    def test_merge(self, pr: int) -> str:
        main, head = self.main(), self.heads[pr]
        if self.bad_tree:  # a test merge on the right parent whose tree is not the train's
            tree = self.git("rev-parse", f"{main}^{{tree}}")
        else:
            tree = self.git("merge-tree", "--write-tree", main, head)
        return self.git("commit-tree", tree, "-p", main, "-p", head, "-m", "test merge")

    def files(self, sha: str) -> set[str]:
        return set(self.git("ls-tree", "--name-only", sha).split())

    def scripted(self, sha: str) -> str:
        assert self.oracle is not None
        polls = self.tested.setdefault(sha, [])
        polls.append(None)
        if len(polls) <= self.polls:
            return "pending"
        if not isinstance(polls[0], str):
            polls[0] = self.oracle(sha)
        return str(polls[0])

    def green_trees(self) -> set[str]:
        return {self.git("rev-parse", f"{sha}^{{tree}}") for sha, polls in self.tested.items() if polls[0] == "green"}

    def runs(self, sha: str) -> object:
        if self.oracle is not None:
            state = self.scripted(sha)
        else:
            state = self.ci.pop(0) if len(self.ci) > 1 else self.ci[0]
        if state == "none":
            return {"workflow_runs": []}
        done = state in ("green", "red", "cancelled")
        run_ = {
            "id": 9 + len(self.tested), "path": merge_ready.CI_WORKFLOW, "event": "push", "head_sha": sha,
            "head_branch": self.branch, "status": "completed" if done else "in_progress",
            "conclusion": {"green": "success", "red": "failure", "cancelled": "cancelled"}.get(state),
            "check_suite_id": 5,
        }
        return {"workflow_runs": [run_]}

    def checks(self) -> object:
        jobs = (*merge_ready.REQUIRED_JOBS, *merge_ready.HEAVY_JOBS)
        return {"check_runs": [
            {"id": k + 1, "name": name, "status": "completed", "conclusion": "success",
             "app": {"id": merge_ready.ACTIONS_APP_ID}, "check_suite": {"id": 5}}
            for k, name in enumerate(jobs)
        ]}

    def send(self, request: Request) -> object:
        try:
            return self.answer(request)
        finally:
            after = self.after.pop((request.method, request.path.removeprefix("repos/o/r/")), None)
            if after is not None:
                after()

    def answer(self, request: Request) -> object:
        """One branch per endpoint the stage calls."""
        hook = self.hooks.pop((request.method, request.path.removeprefix("repos/o/r/")), None)
        if hook is not None:
            hook()
        path = request.path.removeprefix("repos/o/r/")
        if request.method != "GET":
            self.writes.append(request)
        if path == "pulls" and request.method == "POST":
            assert isinstance(request.body, dict)
            self.opened.append(request.body)
            return {"number": 100 + len(self.opened)}
        if path == "pulls":
            label = [{"name": merge_ready.READY_LABEL}]
            gone = {*self.merged, *self.unlabelled}
            stop = [{"number": pr, "labels": [{"name": merge_ready.STOP_LABEL}]} for pr in sorted(self.stop)]
            return [{"number": pr, "labels": label} for pr in self.heads if pr not in gone] + stop
        if m := re.fullmatch(r"pulls/(\d+)", path):
            pr = int(m[1])
            if pr in self.merged:
                merge_commit: str | None = self.squashes[pr]
            else:
                merge_commit = self.test_merge(pr) if self.mergeable is not None else None
            return {
                "number": pr, "title": f"T-000{pr}: change {pr}", "mergeable_state": "clean",
                "mergeable": self.mergeable, "merged": pr in self.merged, "merge_commit_sha": merge_commit,
                "head": {"ref": f"agent/harness/T-000{pr}-x", "sha": self.heads[pr], "repo": {"full_name": "o/r"}},
            }
        if re.fullmatch(r"pulls/(\d+)/commits", path):
            return [{"commit": {"message": "x\n\nCo-Authored-By: A <a@b.c>"}}]
        if m := re.fullmatch(r"git/commits/([0-9a-f]{40})", path):
            parents = self.git("rev-parse", f"{m[1]}^@").split()
            tree = self.git("rev-parse", f"{m[1]}^{{tree}}")
            message = self.git("log", "-1", "--format=%B", m[1])
            return {"tree": {"sha": tree}, "parents": [{"sha": p} for p in parents], "message": message}
        if m := re.fullmatch(rf"issues/(\d+)/labels/{merge_ready.READY_LABEL}", path):
            self.unlabelled.add(int(m[1]))
            return {}
        if m := re.fullmatch(rf"issues/(\d+)/labels/{merge_ready.STOP_LABEL}", path):
            self.stop.discard(int(m[1]))
            return {}
        if path.startswith("git/refs/heads/") and request.method == "DELETE":
            if self.deny_delete:
                raise GhError("gh api DELETE: HTTP 403 Access to this GitHub API path is not permitted")
            self.deleted.append(path.removeprefix("git/refs/heads/"))
            return None
        if m := re.fullmatch(r"pulls/(\d+)/merge", path):
            pr = int(m[1])
            assert request.body is not None
            if pr in self.refuse or request.body["sha"] != self.heads[pr]:
                raise GhError("HTTP 409: head moved")
            main = self.main()
            tree = self.git("merge-tree", "--write-tree", main, self.heads[2 if pr in self.bad_squash else pr])
            message = f"{request.body['commit_title']}\n\n{request.body['commit_message']}"
            squash = self.git("commit-tree", tree, "-p", main, "-m", message)
            self.git("update-ref", "refs/heads/main", squash, main)
            self.merged.append(pr)
            self.squashes[pr] = squash
            if self.lose_reply:
                raise GhError("gh api PUT: connection reset")
            return {"sha": squash, "merged": True}
        if path == "actions/runs":
            query = dict(request.query)
            return self.runs(query["head_sha"])
        if path.endswith("/check-runs"):
            return self.checks()
        if path == "issues" and dict(request.query) == {"state": "closed", "labels": merge_ready.READY_LABEL,
                                                          "per_page": "100", "page": "1"}:
            closed = [pr for pr in self.closed if pr not in self.unlabelled]
            return [{"number": pr, "pull_request": {}} for pr in closed] + [{"number": 99}]  # 99: an issue
        if path.startswith(("issues/", "actions/workflows/")):
            return {}
        raise GhError(f"sim: no endpoint for {request.method} {path}")


class StageFixture(unittest.TestCase):
    """A temporary origin with PRs #1 and #2, the GitHub simulator and a scripted lock."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        self.origin, self.clone = base / "origin.git", base / "clone"
        run("git", "init", "-q", "--bare", "-b", "main", str(self.origin), cwd=base)
        run("git", "clone", "-q", str(self.origin), str(self.clone), cwd=base)
        for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
            self.git("config", key, value)
        self.git("checkout", "-q", "-b", "main")
        for name in ("tasks.py", "check_decisions.py"):
            self.write(f"scripts/{name}", (SCRIPTS / name).read_text())
        for path in ("docs/decisions/README.md", "docs/decisions/0001-a.md"):
            self.write(path, "x")
        self.write(merge_train.SWITCH, 'mode = "train"\n')
        self.m0 = self.commit("main")
        self.git("push", "-q", "origin", "main")
        self.heads = {}
        for pr in (1, 2):
            self.git("checkout", "-q", "-f", "--detach", self.m0)
            self.write(f"f{pr}.txt", "x")
            self.heads[pr] = self.commit(f"pr {pr}")
            self.git("push", "-q", "origin", f"HEAD:refs/pull/{pr}/head")
        self.git("checkout", "-q", "-f", "main")
        self.gh = GitHub(self.origin, self.heads)
        self.lock = FakeLock()
        self.clock = Clock()
        self.stage = merge_train.Stage(self.gh, self.clone, self.lock, self.clock, self.clock.sleep)
        at = {1: "2026-10-28T10:00:00Z", 2: "2026-10-28T11:00:00Z"}
        self.verdicts = {pr: verdict(pr, head, at[pr]) for pr, head in self.heads.items()}

        def verify(_client: object, pr: int, repo: merge_ready.Repo) -> merge_ready.Verdict:
            repo.fetch_head(pr)
            return self.verdicts[pr]

        patch = mock.patch.object(merge_train.merge_ready, "verify", side_effect=verify)
        patch.start()
        self.addCleanup(patch.stop)
        # `make merge-flip-check` per PR (scripts/test_merge_flip.py tests the rule itself).
        self.flips: dict[int, list[str]] = {}

        def flip_check(_client: object, pr: int, _root: Path) -> tuple[str, list[str]]:
            return "f" * 40, self.flips.get(pr, ["not a flip PR"])

        patch = mock.patch.object(merge_train.merge_flip, "pr_problems", side_effect=flip_check)
        patch.start()
        self.addCleanup(patch.stop)

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.clone)

    def write(self, path: str, text: str) -> None:
        (self.clone / path).parent.mkdir(parents=True, exist_ok=True)
        (self.clone / path).write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD")

    def push_main(self, files: dict[str, str]) -> None:
        """Someone else's commit on origin's main (a flip to lock, or a merge without the lock)."""
        self.git("fetch", "-q", "origin", "main")
        self.git("checkout", "-q", "-f", "--detach", "origin/main")
        for path, text in files.items():
            self.write(path, text)
        self.commit("foreign")
        self.git("push", "-q", "origin", "HEAD:refs/heads/main")

    def flip(self) -> None:
        self.push_main({merge_train.SWITCH: 'mode = "lock"\n'})

    def train_ref(self) -> str:
        return run("git", "for-each-ref", "--format=%(objectname)", merge_train.TRAIN_REF, cwd=self.origin)

    def tasks_title(self) -> str:
        [line] = [line for line in self.stage.log if line.startswith("filed ")]
        return "tasks: merge train " + line.removeprefix("filed ").split(" in #")[0]

    def filed(self) -> str:
        """The one task file on the task-only PR's branch, after checking it with main's tasks.py."""
        [opened] = [o for o in self.gh.opened if str(o["head"]).startswith(merge_train_filing.TASKS_PREFIX)]
        branch = f"refs/heads/{opened['head']}"
        [path] = self.gh.git("diff", "--name-only", f"{branch}^", branch).split()
        self.assertRegex(path, r"^docs/tasks/M2/T-\d{4}-[a-z0-9-]+\.md$")
        with tempfile.TemporaryDirectory() as tmp:
            tree = Path(tmp) / "t"
            self.gh.git("worktree", "add", "-q", "--detach", str(tree), branch)
            check = subprocess.run([sys.executable, "-I", str(tree / "scripts/tasks.py"), "validate", "--root",
                                    str(tree)], capture_output=True, text=True, check=False)
            self.gh.git("worktree", "remove", "--force", str(tree))
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)
        return self.gh.git("show", f"{branch}:{path}")

    def run_stage(self) -> int:
        return self.stage.run()

    def add_pr(self, pr: int, files: dict[str, str], *failures: merge_ready.Failure) -> None:
        self.git("checkout", "-q", "-f", "--detach", self.m0)
        for path, text in files.items():
            self.write(path, text)
        self.heads[pr] = self.commit(f"pr {pr}")
        self.git("push", "-q", "origin", f"HEAD:refs/pull/{pr}/head")
        self.git("checkout", "-q", "-f", "main")
        self.verdicts[pr] = verdict(pr, self.heads[pr], f"2026-10-28T0{pr}:00:00Z", *failures)


class MergeStage(StageFixture):
    """Steps 1 and 5-7."""

    def test_a_green_train_merges_in_order_and_main_ends_with_the_tested_tree(self) -> None:
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])
        tip = self.train_ref()
        self.assertEqual(self.gh.main(), self.gh.git("rev-parse", "main"))
        self.assertEqual(self.gh.git("rev-parse", "main^{tree}"), self.gh.git("rev-parse", f"{tip}^{{tree}}"))
        self.assertIn("#1 " + self.heads[1], self.gh.git("log", "-1", "--format=%B", tip))
        message = self.gh.git("log", "-1", "--format=%B", "main")
        self.assertEqual(message, "T-0002: change 2 (#2)\n\nCo-Authored-By: A <a@b.c>")
        self.assertEqual(self.lock.calls[-1], ("release", None))
        self.assertIn(("renew", [2]), self.lock.calls)
        removed = [r.path for r in self.gh.writes if r.method == "DELETE" and "/issues/" in r.path]
        self.assertEqual(removed, [f"repos/o/r/issues/{pr}/labels/ready-to-merge" for pr in (1, 2)])
        self.assertEqual(self.gh.deleted, [f"agent/harness/T-000{pr}-x" for pr in (1, 2)])
        self.assertEqual(self.gh.opened, [])  # nothing to file

    def test_lock_mode_stops_at_once_with_no_lock_and_no_write(self) -> None:
        self.push_main({merge_train.SWITCH: "mode = \n"})  # malformed reads as lock
        self.assertEqual(self.run_stage(), 0)
        self.assertEqual((self.lock.calls, self.gh.writes, self.train_ref()), ([], [], ""))

    def test_a_flip_right_after_acquisition_pushes_nothing(self) -> None:
        self.lock.on_wait.append(self.flip)
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual((self.gh.writes, self.train_ref(), self.gh.merged), ([], "", []))
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_flip_during_the_ci_wait_merges_nothing(self) -> None:
        self.gh.ci = ["pending", "pending", "green"]
        self.gh.hooks[("GET", "actions/runs")] = self.flip
        self.assertEqual(self.run_stage(), 1)
        self.assertIn("stopped: the mode on main is now lock", self.stage.log)
        self.assertEqual(self.gh.merged, [])
        self.assertNotEqual(self.train_ref(), "")

    def test_a_flip_between_squashes_stops_after_the_first(self) -> None:
        self.gh.hooks[("POST", "issues/1/comments")] = self.flip
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [1])
        self.assertEqual(self.stage.log[-1], "stopped: the mode on main is now lock")

    def test_main_moving_between_squashes_stops_with_a_p0(self) -> None:
        self.gh.hooks[("POST", "issues/1/comments")] = lambda: self.push_main({"other.txt": "x"})
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [1])
        self.assertIn("P0: main moved without the lock before squashing #2", self.stage.log)

    def test_main_moving_during_a_squash_fails_the_post_squash_check(self) -> None:
        # Our squash is the tested tree, so the foreign commit on top is not ours to revert.
        self.gh.after[("PUT", "pulls/1/merge")] = lambda: self.push_main({"other.txt": "x"})
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [1])
        self.assertIn("P0: main moved without the lock after squashing #1", self.stage.log)
        self.assertEqual([o["title"] for o in self.gh.opened], [self.tasks_title()])

    def test_main_moved_before_the_first_squash(self) -> None:
        self.gh.hooks[("GET", "actions/runs")] = lambda: self.push_main({"other.txt": "x"})
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [])
        self.assertIn("P0: main moved without the lock before the first squash", self.stage.log)

    def test_a_merge_ref_that_is_never_recomputed_stops(self) -> None:
        self.gh.mergeable = None
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [])
        self.assertIn("was not recomputed", self.stage.log[-1])

    def test_a_lost_lock_merges_nothing_more(self) -> None:
        # Renewals: after the push, before step 7, before squash 1, before squash 2.
        for renewals, merged in (([4], []), ([0, 4], []), ([0, 0, 0, 4], [1])):
            with self.subTest(renewals=renewals):
                self.setUp()
                self.lock.renewals = list(renewals)
                self.assertEqual(self.run_stage(), 1)
                self.assertEqual(self.gh.merged, merged)
                self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_refused_merge_leaves_a_partly_merged_batch(self) -> None:
        self.gh.refuse = {2}
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [1])
        self.assertIn("refused", self.stage.log[-1])

    def test_red_and_missing_ci_merge_nothing(self) -> None:
        self.gh.ci = ["red"]
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual((self.gh.merged, sorted(self.gh.unlabelled)), ([], [1, 2]))
        self.assertEqual(self.stage.log[-1], "ejected #2: ci-red")
        # An infrastructure failure is retried once on a fresh commit, then stops ejecting nobody.
        for ci, dispatches, words in ((["none"], 2, "even after a dispatch, twice"),
                                      (["pending"], 0, "took over 10 min, twice")):
            with self.subTest(ci=ci):
                self.setUp()
                self.gh.ci = ci
                self.assertEqual(self.run_stage(), 1)
                self.assertEqual((self.gh.merged, self.gh.unlabelled), ([], set()))
                self.assertEqual(self.stage.log[-1], f"stopped: infrastructure: no CI run on the train, {words}"
                                 if ci == ["none"] else f"stopped: infrastructure: the train CI {words}")
                sent = [r for r in self.gh.writes if r.path.endswith("/dispatches")]
                self.assertEqual(len(sent), dispatches)

    def test_the_hold_budget_gates_the_ci_wait_and_caps_the_lock_wait(self) -> None:
        with mock.patch.object(merge_train, "HOLD_BUDGET", merge_train.CI_TIMEOUT + merge_train.MERGE_BUDGET):
            self.assertEqual(self.run_stage(), 1)  # two PRs need two merge budgets
        self.assertIn("hold budget", self.stage.log[-1])
        self.assertEqual(self.gh.merged, [])
        self.assertEqual(self.lock.calls[0], ("wait", merge_train.WINDOW / 60))

    def test_an_error_releases_the_lock(self) -> None:
        def fail() -> None:
            raise GhError("boom")

        self.gh.hooks[("GET", "actions/runs")] = fail
        with self.assertRaises(GhError):
            self.run_stage()
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_train_mode_unlabels_failed_prs_and_ejects_with_a_label(self) -> None:
        self.add_pr(3, {"docs/decisions/0001-b.md": "x"})  # collides with main's 0001
        self.add_pr(4, {"f4.txt": "x"}, merge_ready.Failure("V3", "CI red", merge_ready.UNLABEL))
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])
        writes = [
            (r.method, r.path.removeprefix("repos/o/r/"), r.body)
            for r in self.gh.writes
            if "/3/" in r.path or "/4/" in r.path
        ]
        label = merge_ready.READY_LABEL
        collision = "id collision with 0001; renumber per decision 0121 (2)"
        self.assertEqual(writes, [
            ("DELETE", f"issues/4/labels/{label}", None),
            ("POST", "issues/4/comments", {"body": f"merge train: {label} removed: CI red"}),
            ("DELETE", f"issues/3/labels/{label}", None),
            ("POST", "issues/3/labels", {"labels": [merge_ready.EJECTED_LABEL]}),
            ("POST", "issues/3/comments", {"body": f"merge train: {label} removed: {collision}"}),
        ])

    def test_a_push_that_loses_its_lease_merges_nothing(self) -> None:
        build = merge_train.build

        def build_then_race(*args: object) -> merge_train.Train:
            train = build(*args)  # type: ignore[arg-type]  # the stage's own arguments, passed through
            run("git", "update-ref", merge_train.TRAIN_REF, self.m0, cwd=self.origin)
            return train

        with mock.patch.object(merge_train, "build", side_effect=build_then_race):
            self.assertEqual(self.run_stage(), 1)
        self.assertIn("lost its lease", self.stage.log[-1])
        self.assertEqual((self.gh.merged, self.train_ref()), ([], self.m0))

    def test_a_test_merge_with_another_tree_stops_with_a_p0(self) -> None:
        self.gh.bad_tree = True
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [])
        self.assertIn("P0: main differs from the tested train at #1: GitHub's test merge has another tree",
                      self.stage.log)
        # Before the squash nothing of ours landed: a P0 only, no revert PR.
        [opened] = self.gh.opened
        self.assertEqual(opened["title"], self.tasks_title())
        task = self.filed()
        self.assertIn("priority: P0", task)
        self.assertIn('title: "merge train: main differs from the tested train at #1"', task)
        self.assertIn("no revert PR", task)
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_lost_merge_reply_is_read_back(self) -> None:
        self.gh.lose_reply = True
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])

    def test_the_ci_wait_renews_the_lock(self) -> None:
        polls = merge_train.RENEW_EVERY // merge_train.CI_POLL
        self.gh.ci = ["pending"] * (polls + 1) + ["green"]
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        renewals = [c for c in self.lock.calls if c == ("renew", [1, 2])]
        self.assertEqual(len(renewals), 1 + 1 + 2)  # after the push, one in the wait, two in step 7

    def test_sigterm_releases_the_lock(self) -> None:
        self.gh.hooks[("GET", "actions/runs")] = lambda: merge_train.raise_on_term(signal.SIGTERM, None)
        with self.assertRaises(SystemExit) as raised:
            self.run_stage()
        self.assertEqual(raised.exception.code, 128 + signal.SIGTERM)
        self.assertEqual(self.lock.calls[-1], ("release", None))


if __name__ == "__main__":
    unittest.main()
