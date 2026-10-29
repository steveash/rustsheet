"""Tests for merge_train.py's stop label, revert PR, task filing and branch deletion (SPEC-0027 section 4;
T-3055), on test_merge_train's GitHub simulator and stage fixture."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready  # noqa: E402 (the sys.path line above must run first)
import merge_train  # noqa: E402
from gh_client import GhError  # noqa: E402
from test_merge_train import StageFixture  # noqa: E402


class StopAndRevert(StageFixture):
    """T-3055: the stop label at every checkpoint, `make merge-mode`, the revert path and step 7.5."""

    def add_stop(self, pr: int = 50) -> None:
        self.flips[pr] = []
        self.gh.stop.add(pr)

    def test_a_valid_stop_label_stops_at_each_checkpoint(self) -> None:
        def at_acquisition() -> None:
            self.lock.on_wait.append(self.add_stop)

        def before_the_ci_wait() -> None:
            self.lock.on_renew.append(self.add_stop)  # the renewal right after the push

        def on_a_ci_poll() -> None:
            self.gh.ci = ["pending", "green"]
            self.gh.hooks[("GET", "actions/runs")] = self.add_stop

        def before_a_squash() -> None:
            self.gh.hooks[("POST", "issues/1/comments")] = self.add_stop

        for arm, pushed, merged in ((at_acquisition, False, []), (before_the_ci_wait, True, []),
                                    (on_a_ci_poll, True, []), (before_a_squash, True, [1])):
            with self.subTest(arm.__name__):
                self.setUp()
                arm()
                self.assertEqual(self.run_stage(), 1)
                self.assertIn(f"stopped: a flip to lock is pending: #50 carries {merge_ready.STOP_LABEL}",
                              self.stage.log)
                self.assertEqual((self.train_ref() != "", self.gh.merged), (pushed, merged))
                self.assertEqual(self.gh.stop, {50})  # an honoured label stays
                self.assertEqual(self.lock.calls[-1], ("release", None))
                if not pushed:
                    self.assertEqual(self.gh.writes, [])

    def test_a_stop_label_on_a_pr_that_is_not_a_flip_is_removed_and_ignored(self) -> None:
        self.gh.stop = {51}
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual((self.gh.merged, self.gh.stop), ([1, 2], set()))
        writes = [(r.method, r.path.removeprefix("repos/o/r/"), r.body) for r in self.gh.writes if "/51/" in r.path]
        self.assertEqual(writes, [
            ("DELETE", f"issues/51/labels/{merge_ready.STOP_LABEL}", None),
            ("POST", "issues/51/comments", {"body": "merge train: stop label removed: not a valid flip PR "
                                                    f"(`make merge-flip-check` fails at {'f' * 40})"}),
        ])

    def test_a_flip_check_that_cannot_run_git_honours_the_label(self) -> None:
        self.gh.stop = {52}
        self.flips[52] = ["git: fetch failed"]
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual((self.gh.writes, self.gh.stop), ([], {52}))

    def test_merge_mode_names_a_pending_stop_and_writes_nothing(self) -> None:
        for flips, line in (({50: []}, "train (stop pending: #50)"), ({}, "train")):
            with self.subTest(line=line):
                self.flips = flips
                self.gh.stop = {50}
                with mock.patch.object(merge_train.gh_client, "from_remote", return_value=self.gh):
                    self.assertEqual(merge_train.mode_line(self.clone), line)
                self.assertEqual((self.gh.writes, self.gh.stop), ([], {50}))
        self.flip()
        self.assertEqual(merge_train.mode_line(self.clone), merge_train.LOCK)

    def test_a_mismatch_after_a_squash_opens_a_revert_pr_and_files_a_p0(self) -> None:
        self.gh.bad_squash = {1}
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.merged, [1])  # #2 is not squashed past the mismatch
        squash = self.gh.squashes[1]
        self.assertIn(f"P0: main differs from the tested train after squashing #1 ({squash})", self.stage.log)
        revert, tasks = self.gh.opened
        branch = f"{merge_train.REVERT_PREFIX}{squash}"
        self.assertEqual((revert["title"], revert["head"], revert["base"]),
                         ("revert: merge train mismatch at #1", branch, "main"))
        tested = self.gh.git("rev-parse", f"{self.train_ref()}~2^{{tree}}")  # T_1: the record commit is on T_2
        self.assertIn(f"- actual tree: {self.gh.git('rev-parse', f'{squash}^{{tree}}')}", str(revert["body"]))
        self.assertIn(f"- squash: {squash}", str(revert["body"]))
        self.assertIn(f"- expected tree: {tested}", str(revert["body"]))
        # The revert branch undoes exactly the squash on the current main.
        self.assertEqual(self.gh.git("rev-parse", f"refs/heads/{branch}^"), self.gh.main())
        self.assertEqual(self.gh.git("rev-parse", f"refs/heads/{branch}^{{tree}}"),
                         self.gh.git("rev-parse", f"{self.m0}^{{tree}}"))
        task = self.filed()
        self.assertIn("priority: P0", task)
        self.assertIn("- revert PR: #101;", task)
        self.assertEqual(tasks["title"], self.tasks_title())
        # The revert PR is never merged by the merger, and the lock is released last.
        self.assertFalse([r for r in self.gh.writes if r.path.endswith("/merge") and "/101/" in r.path])
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_task_that_cannot_be_filed_is_logged_and_the_lock_released(self) -> None:
        self.gh.bad_tree = True  # a P0 before the squash
        failed = merge_train.GitError("tasks.py new: exit 3")
        with mock.patch.object(merge_train.Stage, "write_task", side_effect=failed):
            self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.gh.opened, [])
        self.assertEqual(self.stage.log[-2:], [
            "the tasks were not filed: tasks.py new: exit 3",
            "unfiled P0: merge train: main differs from the tested train at #1",
        ])
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_an_error_mid_hold_still_files_the_tasks_held(self) -> None:
        def fail() -> None:
            self.stage.tasks.append(("P1", "merge train: possible flaky test or interaction at #1", "- x"))
            raise GhError("boom")

        self.gh.hooks[("GET", "actions/runs")] = fail
        with self.assertRaises(GhError):
            self.run_stage()
        self.assertEqual([o["title"] for o in self.gh.opened], [self.tasks_title()])
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_refused_branch_deletion_is_logged_and_the_train_goes_on(self) -> None:
        self.gh.deny_delete = True
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])
        self.assertIn("the branch of #2 was not deleted: gh api DELETE: HTTP 403 Access to this GitHub API path "
                      "is not permitted", self.stage.log)


if __name__ == "__main__":
    unittest.main()
