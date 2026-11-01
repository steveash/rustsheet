"""Gate A definitions of T.INV.2T, TINV, F.DIST and F.INV (T-3358) against published values."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import fisher, student_t
from rustsheet_tools.accuracy.model import REGISTRY, DomainError, NoValueError

mp = mpmath.mpf


def close(value: mpmath.mpf, published: str, digits: int) -> bool:
    # within 10^-digits, relative above 1 and absolute below
    return bool(mpmath.almosteq(value, mp(published), mp(10) ** -digits, mp(10) ** -digits))


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def test_t_inv_2t_matches_the_published_example_and_is_symmetric_above_1() -> None:
    # Microsoft's T.INV.2T example: T.INV.2T(0.546449, 60) = 0.606533 (to the 6 digits shown).
    assert close(student_t.two_tailed_quantile(mp("0.546449"), mp(60)), "0.606533", 6)
    # T.INV.2T(p) = T.INV(1 - p/2): p = 1.5 at df 10 is T.INV(0.25, 10) = -0.6998120613124317
    # (statistics.md, Excel's own value, which equals the quantile to the last digit).
    assert close(student_t.two_tailed_quantile(mp("1.5"), mp(10)), "-0.6998120613124317", 14)
    assert student_t.two_tailed_quantile(mp(1), mp(5)) == 0
    # df 1 is the Cauchy law: T.INV.2T(p, 1) = cot(pi p / 2).
    p = mp("0.3")
    assert close(student_t.two_tailed_quantile(p, mp(1)), str(mpmath.cot(mpmath.pi * p / 2)), 30)


def test_tinv_is_the_same_definition_with_the_same_domain() -> None:
    t2, tinv = REGISTRY["T.INV.2T"], REGISTRY["TINV"]
    args = [mp("0.546449"), mp(60)]
    assert t2.evaluate(args) == tinv.evaluate(args)
    for bad in ([mp(0), mp(5)], [mp(2), mp(5)], [mp("0.5"), mp(0)]):
        with pytest.raises(DomainError):
            tinv.evaluate(bad)


def test_f_dist_matches_the_published_examples() -> None:
    # Microsoft's F.DIST example: F.DIST(15.2068649, 6, 4, TRUE) = 0.99 and FALSE = 0.0012238.
    x, d1, d2 = mp("15.2068649"), mp(6), mp(4)
    assert close(fisher.lower_tail(x, d1, d2), "0.99", 7)
    assert close(fisher.density(x, d1, d2), "0.0012238", 7)
    assert close(fisher.lower_tail(x, d1, d2) + fisher.upper_tail(x, d1, d2), "1", 35)
    # d1 = 2: P(F <= x) = 1 - (1 + 2x/d2)^(-d2/2) in closed form.
    x, d2 = mp("1.7"), mp(9)
    assert close(fisher.lower_tail(x, mp(2), d2), str(1 - (1 + 2 * x / d2) ** (-d2 / 2)), 35)
    assert fisher.density(mp(0), mp(2), mp(5)) == 1
    assert fisher.density(mp(0), mp(3), mp(5)) == 0
    with pytest.raises(NoValueError):
        fisher.density(mp(0), mp(1), mp(5))


def test_f_inv_matches_the_published_example_and_inverts_both_tails() -> None:
    # Microsoft's F.INV example: F.INV(0.01, 6, 4) = 0.10930991 (to the digits shown).
    assert close(fisher.quantile(mp("0.01"), mp(6), mp(4)), "0.10930991", 7)
    for p in (mp("1e-300"), mp("0.3"), mp("0.97"), 1 - mp("1e-15")):
        x = fisher.quantile(p, mp(5), mp(7))
        # The solved tail must match relatively: the lower one for p <= 1/2, the upper (1 - p) above.
        if p <= 0.5:
            assert abs(fisher.lower_tail(x, mp(5), mp(7)) / p - 1) < mp("1e-30")
        else:
            assert abs(fisher.upper_tail(x, mp(5), mp(7)) / (1 - p) - 1) < mp("1e-30")
    with pytest.raises(DomainError):
        REGISTRY["F.INV"].evaluate([mp("0.5"), mp(5), mp("1e10") + 1])
