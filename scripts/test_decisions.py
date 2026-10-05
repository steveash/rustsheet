from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_decisions as dec  # noqa: E402


class DecisionRecords(unittest.TestCase):
    def test_duplicate_numbers_are_reported(self) -> None:
        # 2026-10-05: T-0032 and T-0033 both took 0021 in parallel PRs.
        names = ["0020-a.md", "0021-sonnet-queue-buffer.md", "0021-no-per-pr-status-log.md", "README.md"]
        (line,) = dec.problems(names)
        self.assertIn("0021 used twice", line)

    def test_bad_names_are_reported(self) -> None:
        for bad in ["0025_foo.md", "25-foo.md", "00250-x.md", "0025-Foo.md"]:
            self.assertEqual(len(dec.problems(["0001-a.md", bad])), 1, bad)

    def test_unique_well_named_records_pass(self) -> None:
        self.assertEqual(dec.problems(["0001-a.md", "0002-b-c.md", "README.md"]), [])

    def test_repository_records_pass(self) -> None:
        self.assertEqual(dec.main(), 0)


if __name__ == "__main__":
    unittest.main()
