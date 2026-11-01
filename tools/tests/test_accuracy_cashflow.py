"""Gate A definitions of IRR and XIRR (T-3586) and array-literal arguments, against published values."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.cashflow import IRR, XIRR, _irr_draw, _xirr_draw
from rustsheet_tools.accuracy.model import DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import Sampler, math_arg
from rustsheet_tools.accuracy.tables import compute, formula, parse_formula

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def irr(*flows: float) -> mpmath.mpf:
    return IRR.evaluate([tuple(mp(f) for f in flows)])


def test_irr_matches_microsofts_examples() -> None:
    # Microsoft's IRR examples; Excel's own value of the first is 0.08663094803652216 (financial.md, "XIRR (and IRR):
    # an empty guess argument is the default 0.1 for XIRR, guess 0 for IRR")
    assert mpmath.almosteq(irr(-70000, 12000, 15000, 18000, 21000, 26000), mp("0.086630948"), mp("1e-9"))
    assert mpmath.almosteq(irr(-70000, 12000, 15000, 18000, 21000), mp("-0.021244848"), mp("1e-9"))
    assert irr(-100, 50, 50) == 0  # the flows sum to 0: the root is exactly 0


def test_irr_domain_one_sign_and_several_sign_changes() -> None:
    with pytest.raises(DomainError):
        irr(100, 50, 20)
    with pytest.raises(NoValueError):  # -1, +3, -2 has two sign changes: no evidenced root choice
        irr(-1, 3, -2)


def test_xirr_matches_microsofts_example() -> None:
    # Microsoft's XIRR example: 2008-01-01, 2008-03-01, 2008-10-30, 2009-02-15, 2009-04-01
    values = tuple(mp(v) for v in (-10000, 2750, 4250, 3250, 2750))
    dates = tuple(mp(d) for d in (39448, 39508, 39751, 39859, 39904))
    # Excel's published 0.373362535 carries its bisection's early stop (financial.md, "IRR, RATE and XIRR stop early"):
    # the true root is 0.37336253352, 1.5E-9 below it
    assert mpmath.almosteq(XIRR.evaluate([values, dates]), mp("0.373362535"), mp("1e-8"))
    assert mpmath.almosteq(XIRR.evaluate([values, dates]), mp("0.3733625335188315"), mp("1e-15"))
    with pytest.raises(DomainError):
        XIRR.evaluate([values, dates[:4]])


def test_xirr_counts_sign_changes_in_time_order_and_refuses_fractional_dates() -> None:
    # listed out of order, the outlay on the earliest date: one sign change in time order, a unique root
    values = tuple(mp(v) for v in (300, -1000, 800))
    dates = tuple(mp(d) for d in (36700, 36526, 36900))
    ordered = XIRR.evaluate([(mp(-1000), mp(300), mp(800)), (mp(36526), mp(36700), mp(36900))])
    assert mpmath.almosteq(
        XIRR.evaluate([values, dates]), ordered, mp(10) ** -35
    )  # the base date only rescales
    with pytest.raises(NoValueError):
        XIRR.evaluate([values, (mp(36700), mp("36526.5"), mp(36900))])


def test_array_literals_round_trip_through_the_formula() -> None:
    args: list[literal.Arg] = [(-1000.5, 300.0, 2.5e-7), (36526, 36600, 37000)]
    text = formula("XIRR", args)
    assert text == "=XIRR({-1000.5,300,2.5E-07},{36526,36600,37000})"
    assert parse_formula(XIRR, text) == [(-1000.5, 300.0, 2.5e-7), (36526.0, 36600.0, 37000.0)]
    assert math_arg(XIRR.params[0], args[0]) == (mp(-1000.5), mp(300), mp(2.5e-7))


def test_check_domain_leaves_arrays_to_the_definition() -> None:
    # an array parameter has no interval: check_domain passes it on, and the definition's own rules apply
    IRR.check_domain([(mp(-1), mp(2))])
    with pytest.raises(DomainError):
        IRR.evaluate([(mp(1), mp(2))])


def test_draws_put_the_drawn_rate_at_the_root() -> None:
    rng = random.Random(7)
    sampler = Sampler(IRR, rng)
    for _ in range(10):
        (flows,) = _irr_draw("typical", rng)
        written = sampler.finish(IRR.params[0], flows)
        assert isinstance(written, tuple)
        assert all(literal.round15(float(f)) == f for f in written)
        value = compute(IRR, [written]).value
        assert value is not None
        assert 1e-2 * 0.99 < float(value) < 0.3 * 1.01
    for _ in range(10):
        values, dates = _xirr_draw("typical", rng)
        assert isinstance(dates, tuple)
        assert all(isinstance(d, int) for d in dates) and list(dates) == sorted(dates)
        assert compute(XIRR, [Sampler(XIRR, rng).finish(XIRR.params[0], values), dates]).value is not None


def test_each_stratum_draws_its_shape() -> None:
    rng = random.Random(11)
    one_sign = 0
    for _ in range(60):  # the edge stratum's third shape: every flow one sign, no rate
        (flows,) = _irr_draw("edge", rng)
        assert isinstance(flows, tuple)
        if all(f > 0 for f in flows) or all(f < 0 for f in flows):
            one_sign += 1
            with pytest.raises(DomainError):
                IRR.evaluate([tuple(mp(f) for f in flows)])
    assert one_sign > 0
    tails = [_irr_draw("tail", rng)[0] for _ in range(30)]
    assert all(isinstance(t, tuple) and len(t) >= 2 for t in tails)
    extreme = [_irr_draw("extreme", rng)[0] for _ in range(5)]
    assert all(isinstance(t, tuple) and 50 <= len(t) <= 200 for t in extreme)
    spans = []
    for _ in range(40):  # half of XIRR's extreme draws are two flows over 10 to 50 years (T-3353)
        values, dates = _xirr_draw("extreme", rng)
        assert isinstance(values, tuple) and isinstance(dates, tuple)
        if len(values) == 2:
            spans.append(int(dates[1]) - int(dates[0]))
    assert spans and all(3650 <= s <= 18250 for s in spans)
