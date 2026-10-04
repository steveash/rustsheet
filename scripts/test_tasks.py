"""Unit tests for scripts/tasks.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasks  # noqa: E402


def task(tid: str, milestone: str) -> dict[str, object]:
    return {"id": tid, "milestone": milestone}


class NextId(unittest.TestCase):
    def test_takes_next_after_highest_in_the_milestones_block(self) -> None:
        ts = [task("T-0004", "M0"), task("T-0023", "M0"), task("T-0300", "M0"), task("T-0110", "M1")]
        taken = {str(t["id"]) for t in ts}
        self.assertEqual(tasks.next_id("M0", ts, taken), "T-0024")
        self.assertEqual(tasks.next_id("M1", ts, taken), "T-0111")

    def test_never_reuses_a_gap(self) -> None:
        ts = [task("T-0002", "M0"), task("T-0009", "M0")]
        self.assertEqual(tasks.next_id("M0", ts, {"T-0002", "T-0009"}), "T-0010")

    def test_ids_on_other_branches_count_as_taken(self) -> None:
        ts = [task("T-0012", "M0")]
        # T-0013 and T-0014 exist only on another session's pushed branch.
        self.assertEqual(tasks.next_id("M0", ts, {"T-0012", "T-0013", "T-0014"}), "T-0015")

    def test_new_milestone_or_full_block_goes_above_everything(self) -> None:
        ts = [task("T-0099", "M0"), task("T-0932", "M9")]
        taken = {"T-0099", "T-0932"}
        self.assertEqual(tasks.next_id("M0", ts, taken), "T-0933")
        self.assertEqual(tasks.next_id("M5", ts, taken), "T-0933")


class GitTaskIds(unittest.TestCase):
    def test_reads_task_files_on_every_remote_branch(self) -> None:
        outputs = {
            ("for-each-ref",): "refs/remotes/origin/main\nrefs/remotes/origin/agent/x\n",
            ("ls-tree", "refs/remotes/origin/main"): "docs/tasks/M0/T-0010-a.md\ndocs/tasks/BOARD.md\n",
            ("ls-tree", "refs/remotes/origin/agent/x"): "docs/tasks/M0/T-0013-b.md\n",
        }

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            key = ("for-each-ref",) if args[0] == "for-each-ref" else ("ls-tree", args[3])
            return mock.Mock(stdout=outputs[key])

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            self.assertEqual(tasks.git_task_ids(fetch=False), {"T-0010", "T-0013"})

    def test_without_git_falls_back_to_nothing(self) -> None:
        with mock.patch.object(tasks.subprocess, "run", side_effect=OSError("no git")):
            self.assertEqual(tasks.git_task_ids(fetch=False), set())


class Slug(unittest.TestCase):
    def test_slug(self) -> None:
        self.assertEqual(tasks.slug("Parser: bound `nesting` depth!"), "parser-bound-nesting-depth")


if __name__ == "__main__":
    unittest.main()
