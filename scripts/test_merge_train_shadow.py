"""Tests for merge_train.py's shadow mode (SPEC-0027 section 4, "Shadow"; T-2995), on test_merge_train's
GitHub simulator and stage fixture."""

from __future__ import annotations

import re
import sys
import unittest
from collections.abc import Callable, Sequence
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready  # noqa: E402 (the sys.path line above must run first)
import merge_train  # noqa: E402
from test_merge_train import TYPICAL, StageFixture, run, verdict  # noqa: E402


class Shadow(StageFixture):
    """SPEC-0027 section 4, "Shadow" (T-2995): PRs #1-#5 on a `shadow` switch, against the scripted
    CI oracle; nothing merges and nothing is labelled."""

    def setUp(self) -> None:
        super().setUp()
        for pr in (3, 4, 5):
            self.add_pr(pr, {f"f{pr}.txt": "x"})
        for pr, head in self.heads.items():
            self.verdicts[pr] = verdict(pr, head, f"2026-10-28T0{pr}:00:00Z")
        self.push_main({merge_train.SWITCH: 'mode = "shadow"\n'})
        self.gh.branch = merge_train.SHADOW_BRANCH
        self.script(lambda prs: False)
        self.stage = merge_train.ShadowStage(self.gh, self.clone, self.lock, self.clock, self.clock.sleep)

    def script(self, red: Callable[[set[int]], bool]) -> None:
        def oracle(sha: str) -> str:
            prs = {int(name[1:-4]) for name in self.gh.files(sha) if re.fullmatch(r"f\d+\.txt", name)}
            return "red" if red(prs) else "green"

        self.gh.oracle, self.gh.polls = oracle, TYPICAL

    def shadow_ref(self) -> str:
        ref = f"refs/heads/{merge_train.SHADOW_BRANCH}"
        return run("git", "for-each-ref", "--format=%(objectname)", ref, cwd=self.origin)

    def prediction(self) -> list[str]:
        return self.gh.git("log", "-1", "--format=%B", self.shadow_ref()).splitlines()

    def assert_no_pr_write(self) -> None:
        self.assertEqual((self.gh.merged, self.lock.calls, self.train_ref()), ([], [], ""))
        self.assertEqual([r for r in self.gh.writes if not r.path.endswith("/dispatches")], [])

    def test_a_green_shadow_train_predicts_merges_with_no_lock_and_no_pr_write(self) -> None:
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assert_no_pr_write()
        lines = self.prediction()
        self.assertEqual(lines[0], merge_train.PREDICTION_HEADER)
        self.assertEqual(lines[3:], [*(f"#{pr} merge" for pr in (1, 2, 3, 4, 5)),
                                     "unlabel: #1, #2, #3, #4, #5", "tree-check: match"])
        self.assertTrue(lines[2].startswith("shadow-input: "))
        self.assertEqual(len(self.gh.tested), 1)  # one green train, no bisection

    def test_a_repeated_round_pushes_nothing_and_a_moved_main_pushes_again(self) -> None:
        self.assertEqual(self.run_stage(), 0)
        tip = self.shadow_ref()
        self.stage = merge_train.ShadowStage(self.gh, self.clone, self.lock, self.clock, self.clock.sleep)
        self.assertEqual(self.run_stage(), 0)
        self.assertEqual(self.shadow_ref(), tip)
        self.assertIn("skipped: main and the ready set are unchanged", self.stage.log[-1])
        self.add_pr(6, {"f6.txt": "x"})  # sixth by marker time: not in the batch, so nothing changed
        self.stage = merge_train.ShadowStage(self.gh, self.clone, self.lock, self.clock, self.clock.sleep)
        self.assertEqual(self.run_stage(), 0)
        self.assertEqual(self.shadow_ref(), tip)
        self.push_main({"g.txt": "y"})
        self.stage = merge_train.ShadowStage(self.gh, self.clone, self.lock, self.clock, self.clock.sleep)
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertNotEqual(self.shadow_ref(), tip)

    def test_a_lost_lease_skips_the_round(self) -> None:
        collected = self.stage.collected

        def race(base: str, heads: Sequence[tuple[int, str]]) -> None:
            collected(base, heads)  # another shadow build pushes between the lease read and our push
            self.gh.git("update-ref", f"refs/heads/{merge_train.SHADOW_BRANCH}", self.m0)

        self.stage.collected = race  # type: ignore[method-assign]
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertIn("skipped: another shadow build won the race", self.stage.log[-1])
        self.assertEqual((self.shadow_ref(), self.gh.tested), (self.m0, {}))
        self.assert_no_pr_write()

    def test_the_only_pr_write_is_the_closed_pr_housekeeping(self) -> None:
        self.gh.closed = [7]
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        writes = [(r.method, r.path) for r in self.gh.writes]
        self.assertEqual(writes, [("DELETE", "repos/o/r/issues/7/labels/ready-to-merge")])

    def test_a_red_shadow_train_bisects_on_m0_and_predicts_the_culprit_at_p3(self) -> None:
        self.script(lambda prs: 3 in prs)
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assert_no_pr_write()
        self.assertEqual(self.prediction()[3:8], ["#1 merge", "#2 merge", "#3 eject:ci-red", "#4 merge", "#5 merge"])
        # the full train, the bisection (prefix 2 green, prefix 3 red... in at most two steps), the
        # confirmation on M0 alone, and the re-test of #4-#5 without the prefix
        self.assertLessEqual(len(self.gh.tested), 6)

    def test_ejections_deferrals_and_a_moved_main_are_predicted(self) -> None:
        self.add_pr(6, {"f1.txt": "conflict"})  # conflicts with #1 only: deferred
        self.verdicts[3] = verdict(3, self.heads[3], "2026-10-28T03:00:00Z",
                                   merge_ready.Failure("V2", "marker head moved", merge_ready.UNLABEL))
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        lines = self.prediction()
        self.assertIn("#3 eject:invalidated", lines)
        self.assertIn("#6 defer", lines)
        self.assertNotIn("#6", lines[-2])

    def test_another_mode_stops_shadow_at_once(self) -> None:
        self.flip()
        self.assertEqual(self.run_stage(), 0)
        self.assertEqual((self.shadow_ref(), self.gh.writes, self.lock.calls), ("", [], []))


if __name__ == "__main__":
    unittest.main()
