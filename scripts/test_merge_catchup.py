"""Tests for merge_catchup.py (`make catch-up`, decision 0207) over a temporary origin."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_catchup  # noqa: E402 (the sys.path line above must run first)
import merge_ready  # noqa: E402
import visible_lock  # noqa: E402
from merge_catchup import Refused  # noqa: E402

BASELINE = "eval/baseline/probes.txt"


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


class TempOrigin(unittest.TestCase):
    """A bare origin with a clone on a fresh `main` (shared with test_merge_renumber.py)."""

    def make_origin(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        self.origin, self.clone = base / "origin.git", base / "clone"
        run("git", "init", "-q", "--bare", "-b", "main", str(self.origin), cwd=base)
        run("git", "clone", "-q", str(self.origin), str(self.clone), cwd=base)
        for key, value in (("user.name", "t"), ("user.email", "t@invalid")):
            self.git("config", key, value)
        self.git("checkout", "-q", "-b", "main")

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.clone)

    def write(self, path: str, text: str) -> None:
        (self.clone / path).parent.mkdir(parents=True, exist_ok=True)
        (self.clone / path).write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD")


class CatchUp(TempOrigin):
    """An origin whose `main` and PR branch `agent/x` both edit files after a common commit."""

    def setUp(self) -> None:
        self.make_origin()
        self.write(BASELINE, "F\tprobes/a\tA1\n")
        self.write("eval/baseline/poi.txt", "F\tpoi/a\tA1\n")
        self.write("eval/suites/probes/suite.toml", 'name = "probes"\ngate = true\n')
        self.write("eval/suites/poi/suite.toml", 'name = "poi"\ngate = true\n[fetch]\nurl = "x"\n')
        self.write("code.txt", "one\n")
        self.base = self.commit("base")
        self.git("push", "-q", "origin", "main")
        self.recorded: list[Path] = []
        self.listed: list[str] = []

    def branch(self, files: dict[str, str]) -> str:
        self.git("checkout", "-q", "-B", "agent/x", self.base)
        for path, text in files.items():
            self.write(path, text)
        head = self.commit("pr")
        self.git("push", "-q", "-f", "origin", "agent/x")
        return head

    def move_main(self, files: dict[str, str]) -> None:
        self.git("checkout", "-q", "main")
        for path, text in files.items():
            self.write(path, text)
        self.commit("main moves")
        self.git("push", "-q", "origin", "main")

    def record(self, code: int, text: str = "F\tprobes/main\tM1\n", path: str = BASELINE):  # type: ignore[no-untyped-def]
        def fake(root: Path, new_cases: Path) -> int:
            self.recorded.append(root)
            self.listed.append(new_cases.read_text())
            (root / path).write_text(text)
            return code

        return fake

    def remote(self, ref: str) -> str:
        return run("git", "rev-parse", ref, cwd=self.origin)

    def test_a_baseline_only_conflict_is_resolved_re_recorded_and_pushed(self) -> None:
        before = self.branch({BASELINE: "F\tprobes/pr\tP1\n"})
        self.move_main({BASELINE: "F\tprobes/main\tM1\n"})
        start = self.git("rev-parse", "HEAD")
        head = merge_catchup.catch_up(self.clone, "agent/x", self.record(0))
        self.assertEqual(self.remote("agent/x"), head)
        self.assertEqual(self.git("rev-parse", "HEAD"), start)  # the checkout is put back
        self.assertEqual(self.recorded, [self.clone])
        self.assertEqual(run("git", "show", f"{head}:{BASELINE}", cwd=self.origin), "F\tprobes/main\tM1")
        # One merge commit of main into the PR's old head, which the carry rule accepts.
        self.assertEqual(run("git", "rev-parse", f"{head}^1", cwd=self.origin), before)
        self.assertEqual(run("git", "rev-parse", f"{head}^2", cwd=self.origin), self.remote("main"))
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(before, head))

    def test_a_conflict_outside_the_baselines_is_refused_and_nothing_pushed(self) -> None:
        before = self.branch({"code.txt": "pr\n", BASELINE: "F\tprobes/pr\tP1\n"})
        self.move_main({"code.txt": "main\n", BASELINE: "F\tprobes/main\tM1\n"})
        with self.assertRaisesRegex(Refused, "outside the eval-gated baseline lists: code.txt"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0))
        self.assertEqual(self.remote("agent/x"), before)
        self.assertEqual(self.recorded, [])
        self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")

    def test_a_refused_re_record_pushes_nothing(self) -> None:
        before = self.branch({BASELINE: "F\tprobes/pr\tP1\n"})
        self.move_main({BASELINE: "F\tprobes/main\tM1\n"})
        with self.assertRaisesRegex(Refused, "eval-baseline refused"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(1))
        self.assertEqual(self.remote("agent/x"), before)

    def test_a_clean_merge_is_re_recorded_and_pushed_too(self) -> None:
        before = self.branch({"pr.txt": "x\n"})
        self.move_main({"other.txt": "y\n"})
        head = merge_catchup.catch_up(self.clone, "agent/x", self.record(0, "F\tprobes/a\tA1\n"))
        self.assertEqual(self.remote("agent/x"), head)
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(before, head))

    def test_a_conflict_in_a_fetched_suites_list_is_refused(self) -> None:
        before = self.branch({"eval/baseline/poi.txt": "F\tpoi/pr\tP1\n"})
        self.move_main({"eval/baseline/poi.txt": "F\tpoi/main\tM1\n"})
        with self.assertRaisesRegex(Refused, "outside the eval-gated baseline lists: eval/baseline/poi.txt"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0))
        self.assertEqual(self.remote("agent/x"), before)

    def test_a_re_record_that_changes_another_file_is_refused(self) -> None:
        before = self.branch({BASELINE: "F\tprobes/pr\tP1\n"})
        self.move_main({BASELINE: "F\tprobes/main\tM1\n"})
        with self.assertRaisesRegex(Refused, "does not accept: eval/baseline/poi.txt"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0, "F\tpoi/x\tX1\n", "eval/baseline/poi.txt"))
        self.assertEqual(self.remote("agent/x"), before)

    def test_a_visible_parity_conflict_is_merged_three_way(self) -> None:
        # Decision 0219: the PR adds its case's line next to lines main lowered (a) and dropped (z).
        # main's side alone would drop the PR's line; the higher side would undo main's lowering
        # (#2660 B1). Three-way keeps the PR's line, main's lowering and main's drop.
        lock = visible_lock.VISIBLE_PARITY
        self.write(lock, "# h\nprobes/a\t3\nprobes/z\t1\n")
        self.base = self.commit("lock")
        self.git("push", "-q", "origin", "main")
        before = self.branch({lock: "# h\nprobes/a\t3\nprobes/m-pr\t2\nprobes/z\t1\n"})
        self.move_main({lock: "# h\nprobes/a\t1\nprobes/m-main\t5\n"})
        head = merge_catchup.catch_up(self.clone, "agent/x", self.record(0, "F\tprobes/a\tA1\n"))
        self.assertEqual(
            run("git", "show", f"{head}:{lock}", cwd=self.origin),
            "# h\nprobes/a\t1\nprobes/m-main\t5\nprobes/m-pr\t2",
        )
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(before, head))

    def test_a_retired_json_lock_conflict_is_resolved_by_deleting_it(self) -> None:
        # Decision 0219: a PR from before the per-case lock modified the json lock main deleted.
        json = visible_lock.RETIRED_LOCK
        self.write(json, '{"visibly_off_cells": 1}\n')
        self.base = self.commit("json lock")
        self.git("push", "-q", "origin", "main")
        before = self.branch({json: '{"visibly_off_cells": 2}\n', "eval/suites/probes/cases/pr.case.json": "{}\n"})
        self.git("checkout", "-q", "main")
        self.git("rm", "-q", json)
        self.write(visible_lock.VISIBLE_PARITY, "# h\nprobes/a\t1\n")
        self.commit("main retires the json lock")
        self.git("push", "-q", "origin", "main")
        lock = "# h\nprobes/a\t1\nprobes/pr.case.json\t7\n"  # the re-record gives the PR's case its line
        head = merge_catchup.catch_up(self.clone, "agent/x", self.record(0, lock, visible_lock.VISIBLE_PARITY))
        self.assertEqual(self.listed, ["eval/suites/probes/cases/pr.case.json\n"])
        files = run("git", "ls-tree", "-r", "--name-only", head, "eval/baseline", cwd=self.origin).split()
        self.assertNotIn(json, files)
        self.assertIsNone(merge_ready.Repo(self.clone).carry_problem(before, head))

    def test_a_line_above_mains_for_a_case_on_main_is_refused(self) -> None:
        # Decision 0219: a PR's sanctioned raise of one of main's cases is merged by hand.
        lock = visible_lock.VISIBLE_PARITY
        self.write("eval/suites/probes/cases/a", "{}\n")
        self.write(lock, "# h\nprobes/a\t3\n")
        self.base = self.commit("lock")
        self.git("push", "-q", "origin", "main")
        before = self.branch({lock: "# h\nprobes/a\t5\n"})
        self.move_main({"other.txt": "main moved\n"})
        with self.assertRaisesRegex(Refused, "a line above main's \\(probes/a\t5\\)"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0, "F\tprobes/a\tA1\n"))
        self.assertEqual(self.remote("agent/x"), before)

    def test_the_re_record_lists_only_the_case_files_the_pr_adds(self) -> None:
        # Decision 0219: after the merge the PR's cases are in the tree's baseline, so only this
        # list lets the re-record write their visible-parity lines, and never main's.
        self.branch({BASELINE: "F\tprobes/pr\tP1\n", "eval/suites/probes/cases/pr.case.json": "{}\n"})
        self.move_main({BASELINE: "F\tprobes/main\tM1\n", "eval/suites/probes/cases/main.case.json": "{}\n"})
        merge_catchup.catch_up(self.clone, "agent/x", self.record(0))
        self.assertEqual(self.listed, ["eval/suites/probes/cases/pr.case.json\n"])

    def test_the_checkout_is_returned_to_its_branch(self) -> None:
        self.branch({BASELINE: "F\tprobes/pr\tP1\n"})
        self.move_main({BASELINE: "F\tprobes/main\tM1\n"})
        self.git("checkout", "-q", "main")
        merge_catchup.catch_up(self.clone, "agent/x", self.record(0))
        self.assertEqual(self.git("symbolic-ref", "--short", "HEAD"), "main")

    def test_a_branch_that_already_has_main_is_refused(self) -> None:
        self.branch({"pr.txt": "x\n"})
        with self.assertRaisesRegex(Refused, "already contains main"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0))

    def test_local_changes_are_refused(self) -> None:
        self.write("code.txt", "dirty\n")
        with self.assertRaisesRegex(Refused, "local changes"):
            merge_catchup.catch_up(self.clone, "agent/x", self.record(0))

    def test_checked_only(self) -> None:
        lists = {"eval/baseline/a.txt", "eval/baseline/b.txt"}
        self.assertTrue(merge_catchup.checked_only(["eval/baseline/a.txt", "eval/baseline/b.txt"], lists))
        self.assertFalse(merge_catchup.checked_only(["eval/baseline/a.txt", "eval/baseline/c.txt"], lists))
        self.assertFalse(merge_catchup.checked_only([], lists))


if __name__ == "__main__":
    unittest.main()
