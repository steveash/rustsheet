"""Gate A definitions of GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, BETA.DIST and BETA.INV (T-3529) against published values."""

from __future__ import annotations

from collections.abc import Iterator

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import beta, gamma
from rustsheet_tools.accuracy.model import REGISTRY, DomainError

mp = mpmath.mpf


def close(value: mpmath.mpf, published: str, digits: int) -> bool:
    # within 10^-digits, relative above 1 and absolute below
    return bool(mpmath.almosteq(value, mp(published), mp(10) ** -digits, mp(10) ** -digits))


@pytest.fixture(autouse=True)
def precision() -> Iterator[None]:
    with mpmath.workdps(50):
        yield


def test_gamma_and_chi_squared_quantiles_match_the_published_examples() -> None:
    # Microsoft's examples, to the digits shown: GAMMA.INV(0.068094,9,2) = 10.0000112,
    # CHISQ.INV(0.93,1) = 5.318520074, CHISQ.INV(0.6,2) = 1.832581464, CHISQ.INV.RT(0.050001,10) = 18.30697346.
    assert close(REGISTRY["GAMMA.INV"].evaluate([mp("0.068094"), mp(9), mp(2)]), "10.0000112", 6)
    assert close(REGISTRY["CHISQ.INV"].evaluate([mp("0.6"), mp(2)]), "1.832581464", 9)
    assert close(REGISTRY["CHISQ.INV.RT"].evaluate([mp("0.050001"), mp(10)]), "18.30697346", 8)
    # CHISQ.INV at df 2 is the exponential quantile -2 ln(1 - p) (statistics.md, T-2860).
    p = mp("0.3")
    assert close(REGISTRY["CHISQ.INV"].evaluate([p, mp(2)]), str(-2 * mpmath.log1p(-p)), 40)
    # df truncates: 2.9 is 2.
    assert REGISTRY["CHISQ.INV"].evaluate([p, mp("2.9")]) == REGISTRY["CHISQ.INV"].evaluate([p, mp(2)])


def test_incomplete_gamma_tails_agree_with_mpmath_where_it_converges() -> None:
    for a, y in [(mp("2.5"), mp(3)), (mp(40), mp(25)), (mp(40), mp(70)), (mp("0.3"), mp("1e-5"))]:
        assert close(gamma.lower_tail(y, a), str(mpmath.gammainc(a, 0, y, regularized=True)), 45)
        assert close(gamma.upper_tail(y, a), str(mpmath.gammainc(a, y, mpmath.inf, regularized=True)), 45)


def test_gamma_quantile_at_a_large_shape_round_trips() -> None:
    # mpmath's gammainc does not converge here; the series and continued fraction do.
    a, p = mp(100000), mp("1e-50")
    y = gamma.quantile(p, a, upper=False)
    assert close(gamma.lower_tail(y, a) / p, "1", 30)
    y = gamma.quantile(p, a, upper=True)
    assert close(gamma.upper_tail(y, a) / p, "1", 30)


def test_beta_matches_the_published_examples() -> None:
    # Microsoft's BETA.DIST(2,8,10,TRUE,1,3) = 0.6854706 and BETA.INV(0.685470581,8,10,1,3) = 2: on [0, 1] that is
    # x = 0.5; the density BETA.DIST(2,8,10,FALSE,1,3) = 1.4837646 is the [0, 1] density at 0.5 over B - A = 2.
    assert close(REGISTRY["BETA.DIST"].evaluate([mp("0.5"), mp(8), mp(10), True]), "0.6854706", 7)
    assert close(REGISTRY["BETA.DIST"].evaluate([mp("0.5"), mp(8), mp(10), False]) / 2, "1.4837646", 7)
    assert close(REGISTRY["BETA.INV"].evaluate([mp("0.685470581"), mp(8), mp(10)]), "0.5", 9)
    # the upper-tail branch: I_x(2, 1) = x^2
    assert close(beta.quantile(mp("0.81"), mp(2), mp(1)), "0.9", 40)


def test_beta_density_is_a_domain_error_at_an_end_with_a_shape_up_to_1() -> None:
    dist = REGISTRY["BETA.DIST"]
    with pytest.raises(DomainError):
        dist.evaluate([mp(0), mp(1), mp(2), False])
    with pytest.raises(DomainError):
        dist.evaluate([mp(1), mp(2), mp("0.5"), False])
    assert dist.evaluate([mp(0), mp("1.5"), mp(2), False]) == 0
    assert dist.evaluate([mp(0), mp("0.5"), mp(2), True]) == 0
