"""Tests for visible_lock.py, the visible-parity lock as the merge tooling reads it (decision 0217)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import visible_lock  # noqa: E402 (the sys.path line above must run first)


class VisibleLock(unittest.TestCase):
    def test_visible_rises(self) -> None:
        parents = ["# h\np/a\t3\np/b\t1\n", "p/a\t1\np/c\t2\n"]
        self.assertEqual(visible_lock.visible_rises("# x\np/a\t3\np/b\t1\np/c\t2\n", parents), [])
        self.assertEqual(visible_lock.visible_rises("p/a\t2\np/c\t1\n", parents), [])  # lower is fine
        self.assertEqual(
            visible_lock.visible_rises("p/a\t4\np/b\t1\np/d\t1\np/c\tx\np/b\t1\n", parents),
            sorted(["p/a\t4", "p/d\t1", "p/c\tx", "p/b\t1"]),
        )
        self.assertEqual(visible_lock.visible_rises("p/a\t1\n", []), ["p/a\t1"])

    def test_merge_visible_keeps_each_cases_higher_count(self) -> None:
        merged = visible_lock.merge_visible("# old\np/a\t3\np/p\t2\n", "# new\np/a\t1\np/m\t5\np/z\t0\n")
        self.assertEqual(merged, "# new\np/a\t3\np/m\t5\np/p\t2\n")
        self.assertEqual(visible_lock.visible_rises(merged, ["p/a\t3\np/p\t2\n", "p/a\t1\np/m\t5\n"]), [])


if __name__ == "__main__":
    unittest.main()
