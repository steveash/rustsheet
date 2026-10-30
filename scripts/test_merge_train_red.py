"""Tests for merge_train.py's red trains (SPEC-0027 section 6; T-2998), on test_merge_train's GitHub
simulator and stage fixture."""

from __future__ import annotations

import re
import sys
import unittest
from collections.abc import Callable
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready  # noqa: E402 (the sys.path line above must run first)
import merge_train  # noqa: E402
import merge_train_filing  # noqa: E402
from test_merge_train import StageFixture, verdict  # noqa: E402

TYPICAL, TIMEOUT = 5, 10  # CI polls before a result: about 5 min, and the 10-min timeout


class RedTrain(StageFixture):
    """SPEC-0027 section 6 against a scripted CI oracle: PRs #1..#5 (PR k adds `f<k>.txt`), whose
    trains are red when `red` holds for the set of PRs in the tested tree."""

    def setUp(self) -> None:
        super().setUp()
        for pr in (3, 4, 5):
            self.add_pr(pr, {f"f{pr}.txt": "x"})
        for pr, head in self.heads.items():
            self.verdicts[pr] = verdict(pr, head, f"2026-10-28T0{pr}:00:00Z")
        self.gates: list[int] = []
        gate = self.stage.gate

        def counted(n: int) -> None:
            self.gates.append(n)
            gate(n)

        self.stage.gate = counted  # type: ignore[method-assign]  # records each gate's n

    def script(self, red: Callable[[set[int]], bool], polls: int = TYPICAL) -> None:
        def oracle(sha: str) -> str:
            prs = {int(name[1:-4]) for name in self.gh.files(sha) if re.fullmatch(r"f\d+\.txt", name)}
            return "red" if red(prs) else "green"

        self.gh.oracle, self.gh.polls = oracle, polls

    def runs(self) -> int:
        return len(self.gh.tested)

    def holds(self, count: int) -> None:
        """`count` merger runs; every hold but the last is cut off."""
        for k in range(count):
            code = self.run_stage()
            self.assert_tested_tree_on_main()
            if k < count - 1:
                self.assertEqual(code, 1, self.stage.log)
                self.assertTrue(self.stage.log[-1].startswith("stopped: hold budget"), self.stage.log)
        self.assertEqual(code, 0, self.stage.log)

    def assert_tested_tree_on_main(self) -> None:
        main = self.gh.git("rev-parse", "main^{tree}")
        self.assertIn(main, {*self.gh.green_trees(), self.gh.git("rev-parse", f"{self.m0}^{{tree}}")})

    def assert_outcome(self, culprits: set[int]) -> None:
        """Only culprits are ejected (labelled, with the v1 block), and every innocent PR merged."""
        self.assertEqual(sorted(self.gh.merged), sorted(set(self.heads) - culprits))
        self.assertEqual(self.gh.unlabelled - set(self.gh.merged), culprits)
        ejected = [r for r in self.gh.writes if r.body == {"labels": [merge_ready.EJECTED_LABEL]}]
        self.assertEqual(sorted(int(r.path.split("/")[-2]) for r in ejected), sorted(culprits))
        for pr in culprits:
            body = next(str(r.body) for r in self.gh.writes if r.path.endswith(f"issues/{pr}/comments"))
            self.assertIn(f"<!-- merge-train-ejected v1 head: {self.heads[pr]} reason: ci-red -->", body)
            self.assertIn("/actions/runs/", body)

    def test_one_culprit_at_each_position(self) -> None:
        for culprit in (1, 2, 3, 4, 5):
            with self.subTest(culprit=culprit):
                self.setUp()
                self.script(lambda prs, c=culprit: c in prs)
                self.holds(1)
                self.assert_outcome({culprit})
                rest = 1 if culprit < 5 else 0  # the PRs after the culprit re-test as one train
                self.assertLessEqual(self.runs(), 5 + rest)
                self.assertEqual(len(self.stage.tasks), 0)

    def test_the_culprit_at_p5_takes_exactly_five_runs_in_one_hold(self) -> None:
        self.script(lambda prs: 5 in prs)
        self.holds(1)
        self.assert_outcome({5})
        self.assertEqual(self.gh.merged, [1, 2, 3, 4])
        self.assertEqual(self.runs(), 5)
        self.assertEqual(self.gates, [5, 2, 3, 4, 1])  # n = the PRs each run can merge

    def test_a_cut_off_at_the_timeout_merges_the_green_prefix_and_resumes(self) -> None:
        for culprit in (4, 5):
            with self.subTest(culprit=culprit):
                self.setUp()
                self.script(lambda prs, c=culprit: c in prs, polls=TIMEOUT)
                self.assertEqual(self.run_stage(), 1)
                self.assertEqual(self.gh.merged, [1, 2, 3])  # merged before the release
                self.assertEqual(self.lock.calls[-1], ("release", None))
                state = self.gh.git("log", "-1", "--format=%B", merge_train.TRAIN_REF)
                self.assertEqual(state, merge_train.Resume(
                    self.gh.main(), ((4, self.heads[4]), (5, self.heads[5]))).render())
                self.holds(1)
                self.assertIn("resuming the search: the first 2 PR(s) are known red", self.stage.log)
                self.assert_outcome({culprit})
                # Full train, mid 2, mid 3; then mid 1 of [#4, #5] and the confirmation; for #4
                # also #5 as the rest. No third rebuild of the full train.
                self.assertEqual(self.runs(), 5 + (culprit == 4))

    def test_a_culprit_early_in_a_train_at_the_timeout_is_confirmed_in_the_next_hold(self) -> None:
        for culprit in (1, 2):
            with self.subTest(culprit=culprit):
                self.setUp()
                self.script(lambda prs, c=culprit: c in prs, polls=TIMEOUT)
                self.assertEqual(self.run_stage(), 1)  # the confirmation gate fails
                self.assertEqual(self.gh.merged, list(range(1, culprit)))
                state = self.gh.git("log", "-1", "--format=%B", merge_train.TRAIN_REF)
                self.assertIn(f"known-red: #{culprit} {self.heads[culprit]}", state)
                before = self.runs()
                self.holds(1)
                self.assertIn("resuming the search: the first 1 PR(s) are known red", self.stage.log)
                self.assertEqual(self.runs(), before + 2)  # the confirmation, then the rest
                self.assertLessEqual(before + 1, 5)
                self.assert_outcome({culprit})

    def test_a_moved_main_or_head_restarts_the_search_with_the_reason_recorded(self) -> None:
        def move_head() -> None:
            self.git("checkout", "-q", "-f", "--detach", self.heads[5])
            self.write("f5.txt", "y")
            self.heads[5] = self.gh.heads[5] = self.commit("pr 5 again")
            self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/5/head")
            self.git("checkout", "-q", "-f", "main")
            self.verdicts[5] = verdict(5, self.heads[5], "2026-10-28T05:00:00Z")

        for change, reason in ((lambda: self.push_main({"other.txt": "x"}), "main moved since the cut-off"),
                               (move_head, "#5's head changed since the cut-off")):
            with self.subTest(reason=reason):
                self.setUp()
                self.script(lambda prs: 4 in prs, polls=TIMEOUT)
                self.assertEqual(self.run_stage(), 1)
                change()
                self.script(lambda prs: 4 in prs)
                self.holds(1)
                self.assertIn(f"restarting the search: {reason}", self.stage.log)
                messages = [self.gh.git("log", "-1", "--format=%B", sha) for sha in self.gh.tested]
                self.assertTrue(any(f"restart: {reason}" in m for m in messages), messages)
                self.assert_outcome({4})

    def test_two_culprits(self) -> None:
        self.script(lambda prs: bool(prs & {2, 4}))
        self.holds(2)  # the hold budget cuts the second search off; the next hold resumes it
        self.assert_outcome({2, 4})

    def test_an_interaction_ejects_the_later_pr(self) -> None:
        self.script(lambda prs: {2, 4} <= prs)
        self.holds(1)
        self.assert_outcome({4})

    def test_a_failure_that_does_not_reproduce_ejects_nobody_and_asks_for_a_p1(self) -> None:
        self.script(lambda prs: self.runs() == 1)  # only the first run is red
        self.holds(1)
        self.assert_outcome(set())
        title = "merge train: possible flaky test or interaction at #5"
        self.assertEqual([(p, t) for p, t, _ in self.stage.tasks], [("P1", title)])
        self.assertIn("#5's train prefix was red", self.stage.tasks[0][2])
        [opened] = self.gh.opened
        self.assertTrue(opened["head"].startswith(merge_train_filing.TASKS_PREFIX), opened)
        self.assertIn(f"(P1): {title}", opened["body"])

    def test_an_infrastructure_failure_is_retried_once_and_ejects_nobody(self) -> None:
        def oracle(cancelled: int) -> Callable[[str], str]:
            return lambda _sha: "cancelled" if self.runs() <= cancelled else "green"

        self.gh.polls = TYPICAL
        self.gh.oracle = oracle(1)
        self.holds(1)
        self.assert_outcome(set())
        self.assertEqual(self.runs(), 2)
        self.setUp()
        self.gh.oracle = oracle(2)
        self.assertEqual(self.run_stage(), 1)
        self.assertEqual(self.stage.log[-1], "stopped: infrastructure: the ci.yml run concluded cancelled, twice")
        self.assertEqual((self.gh.merged, self.gh.unlabelled), ([], set()))

    def test_infrastructure_conclusions_are_read_from_ci_state(self) -> None:
        """The INFRA rule matches the ends of `ci_state`'s reasons; pin each one it relies on."""
        def jobs(run_conclusion: str, job: str | None) -> list[str]:
            run_ = {"id": 1, "path": merge_ready.CI_WORKFLOW, "event": "push", "status": "completed",
                    "conclusion": run_conclusion, "check_suite_id": 5}
            names = (*merge_ready.REQUIRED_JOBS, *merge_ready.HEAVY_JOBS)
            checks = [{"id": k + 1, "name": name, "status": "completed", "conclusion": job or "success",
                       "app": {"id": merge_ready.ACTIONS_APP_ID}, "check_suite": {"id": 5}}
                      for k, name in enumerate(names) if job is not None or k > 0]
            state, reasons = merge_ready.ci_state([run_], checks, False, merge_train.TRAIN_EVENTS)
            self.assertEqual(state, merge_ready.RED)
            return reasons

        for reasons in (jobs("cancelled", "success"), jobs("timed_out", "success"),
                        jobs("startup_failure", "success"), jobs("success", "timed_out"),
                        jobs("success", "cancelled"), jobs("success", None)):
            self.assertTrue(all(r.endswith(merge_train.INFRA_CONCLUSIONS) for r in reasons), reasons)
        self.assertFalse(jobs("failure", "success")[0].endswith(merge_train.INFRA_CONCLUSIONS))

    def test_a_forged_or_malformed_state_record_is_ignored(self) -> None:
        good = merge_train.Resume("a" * 40, ((4, "b" * 40),))
        self.assertEqual(merge_train.Resume.parse(good.render()), good)
        for text in (good.render().replace("#4", "#x"), good.render() + "\nextra: 1",
                     good.render().replace("known-green-prefix: 0", "known-green-prefix: 2"),
                     good.render().replace(merge_train.STATE_HEADER, "merge train state")):
            with self.subTest(text=text):
                self.assertIsNone(merge_train.Resume.parse(text))


if __name__ == "__main__":
    unittest.main()
