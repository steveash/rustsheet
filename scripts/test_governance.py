from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_governance as gov  # noqa: E402


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


if __name__ == "__main__":
    unittest.main()
