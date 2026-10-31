from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_fitted as fit  # noqa: E402 (import after sys.path setup)

TASKS = {"T-2587", "T-3067"}


class FittedTags(unittest.TestCase):
    def test_well_formed_tags_are_counted_per_file(self) -> None:
        files = {
            "a.rs": "/// Why.\n/// FITTED(T-2587, 8 cells, relative gap 5e-16..7e-16)\nconst A: f64 = 6e-16;\n",
            "b.rs": "    //! FITTED(T-3067, 1 cell, range unknown)\n    /// FITTED(T-3067, 40 cells, x (0, 1])\n",
            "c.rs": "fn f() {}\n",
        }
        counts, problems = fit.scan(files, TASKS)
        self.assertEqual(problems, [])
        self.assertEqual(counts, {"a.rs": 1, "b.rs": 2})

    def test_malformed_tags_are_rejected(self) -> None:
        bad = [
            "/// FITTED(T-2587, 8, x 1..2)",  # no `cells`
            "/// FITTED(T-2587, many cells, x 1..2)",  # count not a number
            "/// FITTED(T-2587, 8 cells, )",  # empty range
            "/// FITTED(T-2587, 8 cells)",  # no range
            "/// FITTED(2587, 8 cells, x 1..2)",  # no `T-`
            "/// FITTED(T-2587, 8 cells, x 1..2) and more",  # text after the tag
            "const X: f64 = 1.0; // FITTED(T-2587, 8 cells, x 1..2)",  # not a comment line
        ]
        for line in bad:
            counts, problems = fit.scan({"a.rs": line}, TASKS)
            self.assertEqual(counts, {}, line)
            self.assertEqual(len(problems), 1, line)
            self.assertIn("malformed", problems[0], line)

    def test_unknown_task_is_rejected(self) -> None:
        counts, (problem,) = fit.scan({"a.rs": "/// FITTED(T-9999, 3 cells, x 1..2)"}, TASKS)
        self.assertEqual(counts, {})
        self.assertIn("T-9999", problem)

    def test_count_may_only_fall(self) -> None:
        fails, notes = fit.compare({"a.rs": 2, "b.rs": 1}, {"a.rs": 1, "c.rs": 3})
        self.assertEqual(len(fails), 2)  # a.rs rose, b.rs is new
        self.assertEqual(len(notes), 1)  # c.rs fell to 0
        self.assertEqual(fit.compare({"a.rs": 1}, {"a.rs": 1}), ([], []))

    def test_repository_tags_match_the_lock(self) -> None:
        self.assertEqual(fit.main([]), 0)


if __name__ == "__main__":
    unittest.main()
