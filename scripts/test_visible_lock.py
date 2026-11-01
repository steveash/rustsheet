"""Tests for visible_lock.py, the visible-parity lock as the merge tooling reads it (decision 0219)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import visible_lock  # noqa: E402 (the sys.path line above must run first)

# Lock lines both parsers must agree on; rustsheet-eval's visible_parity tests read it too.
CORPUS = Path(__file__).resolve().parent.parent / "crates/rustsheet-eval/tests/data/visible-lock-lines.txt"


def on_main(case: str) -> bool:
    """Every case is on main except `p/pr-*`, which the PR adds."""
    return not case.startswith("p/pr-")


class VisibleLock(unittest.TestCase):
    def test_the_shared_corpus(self) -> None:
        entries = [line.split("\t", 1) for line in CORPUS.read_text(encoding="utf-8").splitlines()]
        entries = [e for e in entries if not e[0].startswith("#")]
        self.assertTrue(any(v == "ok" for v, _ in entries) and any(v == "bad" for v, _ in entries))
        for verdict, line in entries:
            with self.subTest(line=line):
                self.assertIn(verdict, ("ok", "bad"))
                self.assertTrue(visible_lock.is_entry(line))
                self.assertEqual(visible_lock.parse_line(line) is not None, verdict == "ok")
                # A malformed line is a rise for the carry, never read as a count.
                if verdict == "bad":
                    self.assertEqual(visible_lock.visible_rises(line, "", on_main), [line])

    def test_a_non_ascii_or_signed_count_is_malformed(self) -> None:
        for count in ("²", "+3", "-1", "٣"):
            with self.subTest(count=count):
                self.assertIsNone(visible_lock.parse_line(f"p/a\t{count}"))
                self.assertEqual(visible_lock.visible_counts(f"p/a\t{count}\n"), {})

    def test_three_way(self) -> None:
        self.assertEqual(visible_lock.three_way(3, 3, 1), 1)  # the PR left it: main's (lowered) count
        self.assertEqual(visible_lock.three_way(3, 3, 0), 0)  # main dropped the line: it stays dropped
        self.assertEqual(visible_lock.three_way(3, 5, 3), 5)  # main left it: the PR's
        self.assertEqual(visible_lock.three_way(0, 4, 0), 4)  # a case the PR adds
        self.assertEqual(visible_lock.three_way(3, 5, 1), 1)  # both changed it: the lower
        self.assertEqual(visible_lock.three_way(3, 2, 4), 2)

    def test_merge_visible_is_three_way_per_case(self) -> None:
        base = "# old\np/a\t3\np/b\t2\np/z\t1\n"
        pr = "# old\np/a\t3\np/b\t2\np/pr-x\t4\np/z\t1\n"
        main = "# new\np/a\t1\np/b\t2\np/m\t5\n"
        merged = visible_lock.merge_visible(base, pr, main)
        self.assertEqual(merged, "# new\np/a\t1\np/b\t2\np/m\t5\np/pr-x\t4\n")
        self.assertEqual(visible_lock.visible_rises(merged, main, on_main), [])

    def test_visible_rises_holds_mains_cases_to_mains_lines(self) -> None:
        main = "# h\np/a\t1\np/b\t2\n"
        self.assertEqual(visible_lock.visible_rises("# x\np/a\t1\np/b\t2\n", main, on_main), [])
        self.assertEqual(visible_lock.visible_rises("p/b\t1\n", main, on_main), [])  # lower is fine
        # main lowered a from 3 to 1 while the PR kept 3: the PR's 3 is a rise (#2660 B1).
        self.assertEqual(visible_lock.visible_rises("p/a\t3\np/b\t2\n", main, on_main), ["p/a\t3"])
        self.assertEqual(
            visible_lock.visible_rises("p/b\t3\np/c\t1\np/d\tx\np/b\t1\n", main, on_main),
            sorted(["p/b\t3", "p/c\t1", "p/d\tx", "p/b\t1"]),
        )
        # A case the PR adds may have any line: main has none for it to hide a regression behind.
        self.assertEqual(visible_lock.visible_rises("p/pr-x\t9\n", main, on_main), [])
        self.assertEqual(visible_lock.visible_rises("p/pr-x\t9\np/pr-x\t9\n", main, on_main), ["p/pr-x\t9"])


if __name__ == "__main__":
    unittest.main()
