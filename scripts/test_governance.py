from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_governance as gov  # noqa: E402 (import after sys.path setup)


class ProtectedPaths(unittest.TestCase):
    def test_frozen_milestone_lists_are_protected(self) -> None:
        # Decision 0043: a line leaves eval/milestones/m1-probes.txt only by a decision record.
        self.assertTrue(gov.is_protected("eval/milestones/m1-probes.txt"))

    def test_rule_paths_and_deviations_files_are_protected(self) -> None:
        for path in [
            "CLAUDE.md",
            "Makefile",
            "scripts/tasks.py",
            ".claude/skills/steward/SKILL.md",
            "eval/suites/probes/deviations",
        ]:
            self.assertTrue(gov.is_protected(path), path)

    def test_ordinary_paths_are_not(self) -> None:
        for path in ["eval/suites/probes/cases/a.case.json", "crates/rustsheet-eval/src/main.rs", "eval/milestones"]:
            self.assertFalse(gov.is_protected(path), path)


class NewProbeCaseTags(unittest.TestCase):
    """Decision 0057: an added probes case has one milestone tag, M2 or later."""

    TAGS = {
        "eval/suites/probes/cases/m1.case.json": ["M1", "probe"],
        "eval/suites/probes/cases/untagged.case.json": ["probe", "lookup"],
        "eval/suites/probes/cases/two.case.json": ["M2", "M9"],
        "eval/suites/probes/cases/m2.case.json": ["M2", "probe"],
        "eval/suites/probes/cases/m9.case.json": ["probe", "M9"],
        "eval/suites/probes/cases/catalog/sum.case.json": ["M0"],
        "eval/suites/probes/cases/old.case.json": ["M1"],
    }

    def problems(self, *lines: str) -> list[str]:
        return gov.probe_tag_problems(lines, lambda path: self.TAGS[path])

    def test_added_m0_m1_untagged_or_twice_tagged_cases_fail(self) -> None:
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/m1.case.json"),
            ["eval/suites/probes/cases/m1.case.json (milestone tags: M1)"],
        )
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/untagged.case.json"),
            ["eval/suites/probes/cases/untagged.case.json (milestone tags: none)"],
        )
        self.assertEqual(
            self.problems("A\teval/suites/probes/cases/two.case.json"),
            ["eval/suites/probes/cases/two.case.json (milestone tags: M2, M9)"],
        )

    def test_added_m2_or_later_and_catalog_cases_pass(self) -> None:
        self.assertEqual(
            self.problems(
                "A\teval/suites/probes/cases/m2.case.json",
                "A\teval/suites/probes/cases/m9.case.json",
                "A\teval/suites/probes/cases/catalog/sum.case.json",
            ),
            [],
        )

    def test_modified_cases_and_other_files_are_not_checked(self) -> None:
        self.assertEqual(
            self.problems(
                "M\teval/suites/probes/cases/old.case.json",
                "A\teval/suites/probes/cases/m1.oracle.json",
                "A\teval/suites/smoke/cases/m1.case.json",
            ),
            [],
        )


if __name__ == "__main__":
    unittest.main()
