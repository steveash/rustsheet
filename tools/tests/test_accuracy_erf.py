"""Gate A definitions of ERF, ERF.PRECISE, ERFC and ERFC.PRECISE (T-3784) against published values, the semantics
entries' Excel values and the two-argument cancellation cut."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.erf import (
    ERF,
    ERF_PRECISE,
    ERFC,
    ERFC_PRECISE,
    _draw_erf,
    _draw_one,
    cancels,
    erfc,
)
from rustsheet_tools.accuracy.model import Definition, Param
from rustsheet_tools.accuracy.sampler import STRATA
from rustsheet_tools.accuracy.tables import compute, draw_points, parse_formula

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def test_published_values() -> None:
    # Abramowitz and Stegun table 7.1: erf(0.5) = 0.5204998778, erf(1) = 0.8427007929, erf(2) = 0.9953222650
    assert mpmath.almosteq(ERF.evaluate([mp(0.5)]), mp("0.520499877813046537682746653892"), mp("1e-30"))
    assert mpmath.almosteq(ERF_PRECISE.evaluate([mp(1)]), mp("0.842700792949714869341220635083"), mp("1e-30"))
    assert mpmath.almosteq(ERFC.evaluate([mp(2)]), mp("0.00467773498104726583793074363275"), mp("1e-30"))
    assert ERF.evaluate([mp(0)]) == 0
    assert ERFC_PRECISE.evaluate([mp(0)]) == 1
    assert ERF.evaluate([mp(-0.5)]) == -ERF.evaluate([mp(0.5)])  # odd
    assert mpmath.almosteq(ERFC.evaluate([mp(-1)]), 2 - ERFC.evaluate([mp(1)]), mp("1e-38"))


def test_excel_values_of_the_semantics_entries_round_from_the_math() -> None:
    # statistics.md T-1040: ERFC(26.5) = 2.2109076642637346E-307, the last nonzero on the grid
    assert float(ERFC.evaluate([mp(26.5)])) == pytest.approx(2.2109076642637346e-307, rel=1e-15)
    # engineering.md: ERF(5.9) is 0.9999999999999999 and ERF(6) is 1
    assert float(ERF.evaluate([mp(5.9)])) == 0.9999999999999999
    assert float(ERF.evaluate([mp(6)])) == 1.0
    # statistics.md T-1778: tiny x keeps the leading term 2/sqrt(pi) (Excel is one ulp above it)
    tiny = mp(2) ** -40
    assert float(ERF_PRECISE.evaluate([tiny]) / tiny) == pytest.approx(1.1283791670955126, rel=1e-16)


def test_two_argument_erf_is_the_difference() -> None:
    a, b = mp(0.3), mp(1.7)
    assert ERF.evaluate([a, b]) == mpmath.erf(b) - mpmath.erf(a)
    assert ERF.evaluate([b, a]) == -ERF.evaluate([a, b])
    assert ERF.evaluate([a, a]) == 0


def test_the_cancellation_region_is_left_out() -> None:
    assert cancels(5.0, 6.0)  # both terms within 1.6E-12 of 1
    assert cancels(0.3, 0.3 + 1e-12)
    assert not cancels(0.3, 0.4)
    assert not cancels(0.0, 0.0)  # a true 0 with no terms to cancel
    rng = random.Random(11)
    for stratum in STRATA:
        for _ in range(60):
            point = _draw_erf(stratum, rng)
            assert len(point) in (1, 2)
            if len(point) == 2:
                assert not cancels(*point)


def test_draws_reach_the_flush_and_both_arities() -> None:
    rng = random.Random(3)
    tails = [_draw_one("tail", rng)[0] for _ in range(300)]
    assert any(abs(t) > 26.625 for t in tails)
    edges = [_draw_one("edge", rng)[0] for _ in range(300)]
    assert 0.0 in edges
    assert any(abs(e) == 0.5 for e in edges)
    points = draw_points(ERF, "s1-test")
    assert {len(p.args) for p in points} == {1, 2}


def test_an_optional_parameter_may_be_left_out_only_at_the_end() -> None:
    assert [p.name for p in ERF.params_for(1)] == ["lower_limit"]
    assert len(ERF.params_for(2)) == 2
    for count in (0, 3):
        with pytest.raises(ValueError, match="takes 1 to 2"):
            ERF.params_for(count)
    with pytest.raises(ValueError, match="optional parameter is followed"):
        Definition(
            function="BAD",
            params=(Param("a", "real", optional=True), Param("b", "real")),
            value=lambda _args: mp(0),
            sources=(),
        )


def test_tables_read_and_compute_one_and_two_argument_calls() -> None:
    assert parse_formula(ERF, "=ERF(0.5)") == [0.5]
    assert parse_formula(ERF, "=ERF(0.5,1)") == [0.5, 1.0]
    with pytest.raises(ValueError, match="takes 1 to 2"):
        parse_formula(ERF, "=ERF(0.5,1,2)")
    value = compute(ERF, [0.5, 1.0]).value
    assert value is not None
    assert mpmath.almosteq(mp(value), mpmath.erf(1) - mpmath.erf(0.5), mp("1e-38"))


def test_huge_arguments_take_the_asymptotic_series() -> None:
    # at the switch the series and mpmath's own erfc agree; past it mpmath's erfc overflows a float
    x = mp(10) ** 4
    with mpmath.workdps(60):
        assert mpmath.almosteq(erfc(x), mpmath.erfc(x), mp("1e-55"))
        assert mpmath.almosteq(erfc(mp(2e4)), mpmath.erfc(mp(2e4)), mp("1e-55"))
    for big in (mp(1e154), mp(1e300)):
        assert 0 < ERFC.evaluate([big]) < mp("1e-300")
        assert ERFC_PRECISE.evaluate([-big]) == 2  # 2 - erfc(big) at any finite precision
        assert ERF.evaluate([big]) == 1  # 1 - erfc is 1 at any finite precision
        assert ERF.evaluate([-big, big]) == 2
