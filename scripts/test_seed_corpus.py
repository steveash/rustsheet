"""Tests for fuzz/seed_corpus.py's formula seed selection (T-3163, decision 0171; stdlib unittest,
run by `make hygiene`)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "fuzz"))
import seed_corpus  # noqa: E402 (the sys.path line above must run first)


class Shape(unittest.TestCase):
    def test_numbers_and_text_are_blanked_references_and_names_kept(self) -> None:
        cases = {
            'SUM(A1,1.5e3,"a""b")': 'SUM(A1,0,"")',
            "$A$1+B$22*1.+.5": "$A$1+B$22*0+0",
            "LOG10(100)-ATAN2(1E-5,2)": "LOG10(0)-ATAN2(0,0)",
            "Sheet2!R1C1&T_1": "Sheet2!R1C1&T_1",
        }
        for formula, shape in cases.items():
            with self.subTest(formula=formula):
                self.assertEqual(seed_corpus.shape(formula), shape)


class Seeds(unittest.TestCase):
    def test_one_seed_per_shape(self) -> None:
        seeds = seed_corpus.formula_seeds({"SIN(1)", "SIN(2.5)", 'LEN("x")', 'LEN("yy")', "A1"})
        self.assertEqual(
            seeds, sorted(seeds, key=lambda f: seed_corpus.hashlib.sha256(f.encode()).hexdigest())
        )
        self.assertEqual({seed_corpus.shape(f) for f in seeds}, {"SIN(0)", 'LEN("")', "A1"})
        self.assertIn("SIN(1)", seeds)  # the first of its shape in sorted order

    def test_the_cap_keeps_a_deterministic_sample(self) -> None:
        formulas = {f"F{n}(A1)" for n in range(50)}  # 50 shapes: F0( ... F49( are names, not numbers
        with mock.patch.object(seed_corpus, "MAX_FORMULA_SEEDS", 7):
            first = seed_corpus.formula_seeds(formulas)
            again = seed_corpus.formula_seeds(set(sorted(formulas, reverse=True)))
        self.assertEqual(len(first), 7)
        self.assertEqual(first, again)


if __name__ == "__main__":
    unittest.main()
