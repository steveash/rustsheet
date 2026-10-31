"""The fit lab's hold-out split and accept rule (decision 0188 R4.1, T-3360)."""

from __future__ import annotations

import math
import random

from rustsheet_tools.fitlab import holdout
from rustsheet_tools.fitlab.cli import holdout_table
from rustsheet_tools.fitlab.fit import Fit, FitResult, Sides
from rustsheet_tools.fitlab.holdout import REFERENCE, is_holdout, split, verdict
from rustsheet_tools.fitlab.score import Score, candidate, merged, score, within_fifteen_digits
from rustsheet_tools.fitlab.sweep import Cell, Sweep


def cell(case: str, address: str, x: float = 1.0, expected: float = 1.0) -> Cell:
    return Cell(case, "Calc", address, f"=F({x})", (x,), expected)


def grid(n: int) -> list[Cell]:
    return [cell(f"probes-numeric/c{i % 7}", f"A{i}", float(i), float(i)) for i in range(1, n + 1)]


def test_the_split_is_a_pure_function_of_case_and_cell() -> None:
    cells = grid(2000)
    first = {c.case + c.key for c in cells if is_holdout(c)}
    shuffled = cells[:]
    random.Random(7).shuffle(shuffled)
    train, test = split(shuffled)
    assert {c.case + c.key for c in test} == first
    assert len(train) + len(test) == len(cells)
    # A cell's side does not depend on the rest of the sweep, its formula or its values.
    for c in cells[:50]:
        alone = split([cell(c.case, c.address, 99.0, -3.0)])
        assert (len(alone[1]) == 1) == is_holdout(c)


def test_the_split_holds_out_about_thirty_percent() -> None:
    _, test = split(grid(20000))
    assert 0.28 < len(test) / 20000 < 0.32


def test_the_split_is_pinned() -> None:
    # A change of salt, hash or ratio moves cells between sides; it must be a deliberate change.
    sides = [is_holdout(cell("probes-numeric/t1048-tinv", f"A{i}")) for i in range(1, 21)]
    assert sides == PINNED
    assert holdout.HOLDOUT_SALT == "rustsheet-fitlab-holdout/v1"
    assert holdout.HOLDOUT_PERMILLE == 300


# Calc!A1..A20 of probes-numeric/t1048-tinv; True is held out.
PINNED = [True, False, False, False, True, False, False, False, True, False]
PINNED += [False, True, False, True, True, True, False, False, True, True]


def side(exact: int, ulp: int, total: int) -> Score:
    return Score("c", "c", total, exact=exact, ulp=ulp, close=ulp, digits15=ulp)


def test_a_candidate_that_wins_on_train_but_not_on_test_is_refused() -> None:
    reference = side(40, 60, 100)
    overfit = verdict(side(230, 230, 233), side(40, 55, 100), reference, 0.05)
    assert not overfit.accepted
    assert any("below train" in r for r in overfit.reasons)
    assert any("does not beat correctly rounded" in r for r in overfit.reasons)


def test_a_candidate_that_generalizes_and_beats_the_reference_is_accepted() -> None:
    reference = side(40, 60, 100)
    good = verdict(side(160, 200, 233), side(70, 90, 100), reference, 0.05)
    assert good.accepted, good.reasons
    assert good.text() == "accept"


def test_the_margin_and_the_reference_tie_break() -> None:
    reference = side(50, 60, 100)
    # Exactly at the margin passes; a tie in exact counts is broken by the within-4-ulp count.
    assert verdict(side(55, 60, 100), side(50, 61, 100), reference, 0.05).accepted
    assert not verdict(side(55, 60, 100), side(50, 60, 100), reference, 0.05).accepted
    assert not verdict(side(56, 60, 100), side(50, 61, 100), reference, 0.05).accepted
    # A test rate above the train rate is never a reason to refuse.
    assert verdict(side(10, 10, 100), side(60, 60, 100), reference, 0.0).accepted


def test_without_a_reference_or_test_cells_nothing_is_accepted() -> None:
    no_ref = verdict(side(50, 50, 100), side(50, 50, 100), None, 0.05)
    assert not no_ref.accepted and "no 'correctly rounded' candidate" in no_ref.reasons[0]
    no_test = verdict(side(50, 50, 100), side(0, 0, 0), side(0, 0, 0), 0.05)
    assert not no_test.accepted and "no hold-out cells" in no_test.reasons


def test_merging_the_sides_gives_the_score_of_the_whole_sweep() -> None:
    cells = [cell("s/c", f"A{i}", float(i), float(i) + (i % 3) * 1e-14 * i) for i in range(1, 400)]
    cand = candidate("id", lambda x: x)
    whole = score(cells, cand, keep=5)
    train, test = split(cells)
    both = merged(score(train, cand, keep=5), score(test, cand, keep=5), keep=5)
    counts = ("total", "exact", "ulp", "close", "digits15", "errors", "incomparable", "largest")
    assert [getattr(both, k) for k in counts] == [getattr(whole, k) for k in counts]
    assert both.exact_keys == whole.exact_keys
    assert sorted(r.ulps or 0 for r in both.results) == sorted(r.ulps or 0 for r in whole.results)


def test_within_fifteen_digits_matches_the_harness() -> None:
    # closeness.rs: half a unit in the 15th significant digit, boundary included.
    assert within_fifteen_digits(1.0, 1.0 + 4.8e-15)
    assert not within_fifteen_digits(1.0, 1.0 + 5.4e-15)
    assert within_fifteen_digits(123.0, 123.0 + 4e-13)
    assert not within_fifteen_digits(123.0, 123.0 + 6e-13)
    assert within_fifteen_digits(0.0, -0.0) and not within_fifteen_digits(0.0, 1e-300)
    assert not within_fifteen_digits(math.inf, math.inf)
    assert not within_fifteen_digits(1.0, math.nan)


def test_the_report_shows_both_sides_and_the_verdicts() -> None:
    cells = grid(300)
    exact = candidate(REFERENCE, lambda x: x)
    off = candidate("off by 1e-12", lambda x: x * (1 + 1e-12))
    fit = Fit("t", "test", ["probes-numeric/c0"], r"^=F\((.*)\)$", lambda: [exact, off])
    train, test = split(cells)
    result = FitResult(fit, Sweep(cells, [], 0, 0), [], train_cells=len(train), test_cells=len(test))
    reference = score(test, exact)
    for c in (exact, off):
        tr, te = score(train, c), score(test, c)
        result.scores.append(merged(tr, te))
        result.sides[c.name] = Sides(
            tr, te, None if c.name == REFERENCE else verdict(tr, te, reference, fit.margin)
        )
    text = holdout_table(result, 10)
    assert f"{len(test)} of 300 cells held out" in text
    assert "test exact rate at most 5.0% below train" in text
    rows = text.splitlines()[2:]
    assert rows[0].split()[1:3] == ["correctly", "rounded"] and rows[0].endswith("reference")
    assert "refuse: does not beat correctly rounded" in rows[1]
    # 1e-12 relative is visible at 15 digits; the reference is exact everywhere.
    assert result.sides["off by 1e-12"].to_json()["test"]["within_15_digits"] == 0
    assert result.sides[REFERENCE].to_json()["test"] == {
        "cells": len(test),
        "exact": len(test),
        "within_4_ulp": len(test),
        "within_15_digits": len(test),
    }
