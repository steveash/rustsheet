"""Gate A definition of NPER (T-3535) against published values and Excel's domain rules."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.annuity import NPER, NPER_R3_RATE, _nper_draw
from rustsheet_tools.accuracy.model import DomainError

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(*args: float) -> mpmath.mpf:
    return NPER.evaluate([mp(a) for a in args])


def test_nper_matches_microsofts_examples() -> None:
    # Microsoft's NPER examples, shown to 7 decimals
    assert mpmath.almosteq(value(0.01, -100, -1000, 10000, 1), mp("59.6738657"), mp("1e-9"))
    assert mpmath.almosteq(value(0.01, -100, -1000, 10000, 0), mp("60.0821229"), mp("1e-9"))
    assert mpmath.almosteq(value(0.01, -100, -1000, 0, 0), mp("-9.57859404"), mp("1e-9"))
    assert value(0, -100, 1000, 0, 0) == 10  # rate 0: -(pv + fv)/pmt


def test_nper_domain_follows_excel() -> None:
    # financial.md, "Annuity and DB argument domains at their limits"
    for args in [
        (1e-300, -113, 3037, 368, 1),  # 1 + rate is 1: #DIV/0!
        (-1, -31, 1, 0, 0),  # LN(0): #NUM!
        (1e-300, 0, 100, 200, 0),  # a non-positive ratio: #NUM!
        (0, 0, 100, 0, 0),  # pmt 0 at rate 0
    ]:
        with pytest.raises(DomainError):
            value(*args)


def test_nper_tiny_positive_rate_keeps_the_math() -> None:
    # 1E-15 is inside R3's region: 1 + rate is not 1, and the count is the continuous limit's to first order
    got = value(1e-15, -100, 1000, 0, 0)
    assert NPER_R3_RATE > 1e-15
    assert mpmath.almosteq(got, mp(10), mp("1e-12"))


def test_nper_draw_is_well_conditioned_at_a_short_horizon() -> None:
    # The draw sets pmt so the drawn count is the root. NPER is ill-conditioned once (1+r)^-n is tiny (the ratio's
    # denominator cancels), so the drawn count comes back only where the horizon is short.
    rng = random.Random(4)
    checked = 0
    for _ in range(200):
        r, pmt, pv, fv, kind = _nper_draw("typical", rng)
        if r * 360 > 1:
            continue
        got = value(r, pmt, pv, fv, kind)
        assert abs(got - mpmath.nint(got)) < mp("1e-6"), (r, pmt, pv, fv, kind, got)
        checked += 1
    assert checked > 10
