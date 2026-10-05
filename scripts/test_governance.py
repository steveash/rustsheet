from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_governance as gov  # noqa: E402


class DecisionNumbers(unittest.TestCase):
    def test_duplicate_numbers_are_reported(self) -> None:
        # 2026-10-05: T-0032 and T-0033 both took 0021 in parallel PRs.
        names = ["0020-a.md", "0021-sonnet-queue-buffer.md", "0021-no-per-pr-status-log.md", "README.md"]
        self.assertEqual(gov.duplicate_decision_numbers(names), ["0021"])

    def test_unique_numbers_pass(self) -> None:
        self.assertEqual(gov.duplicate_decision_numbers(["0001-a.md", "0002-b.md", "README.md"]), [])


if __name__ == "__main__":
    unittest.main()
