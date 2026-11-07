"""Gate A definitions of GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT (T-3925) against Microsoft's published examples,
the semantics entries' values, the evidenced error domain and the excluded regions."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.gamma_dist import CHISQ_DIST, CHISQ_DIST_RT, GAMMA_DIST
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float | bool) -> float:
    return float(definition.evaluate([a if isinstance(a, bool) else mp(a) for a in args]))


def test_microsoft_examples() -> None:
    # Microsoft's GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT pages
    assert value(GAMMA_DIST, 10.00001131, 9, 2, False) == pytest.approx(0.032639, abs=1e-6)
    assert value(GAMMA_DIST, 10.00001131, 9, 2, True) == pytest.approx(0.068094, abs=1e-6)
    assert value(CHISQ_DIST, 0.5, 1, True) == pytest.approx(0.52049988, abs=1e-8)
    assert value(CHISQ_DIST, 2, 3, False) == pytest.approx(0.20755375, abs=1e-8)
    assert value(CHISQ_DIST_RT, 18.307, 10) == pytest.approx(0.0500006, abs=1e-7)


def test_semantics_values() -> None:
    # statistics.md "Incomplete gamma: accurate kernel (T-0971)": Excel is correctly rounded at Q(2.5, 5) and at
    # CHISQ.DIST.RT(8,3.34) = Q(1.5, 4), its df truncated
    assert value(CHISQ_DIST_RT, 10, 5) == 0.07523524614651218
    assert value(CHISQ_DIST_RT, 8, 3.34) == 0.04601170568923137
    # statistics.md T-1891: Excel's GAMMA.DIST(1E+10,1E+10,1,TRUE) from Temme's expansion
    assert value(GAMMA_DIST, 1e10, 1e10, 1, True) == pytest.approx(0.5000013298076014, rel=1e-15)
    # T-1778: CHISQ.DIST(x,k,TRUE) = GAMMA.DIST(x,k/2,2,TRUE); the densities agree at scale 2
    for x, k in ((3.7, 5), (0.01, 1), (80.0, 60)):
        for cumulative in (True, False):
            assert mpmath.almosteq(
                CHISQ_DIST.evaluate([mp(x), mp(k), cumulative]),
                GAMMA_DIST.evaluate([mp(x), mp(k) / 2, mp(2), cumulative]),
                mp("1e-35"),
            )
    # T-2753: the exact density at x = 1E-300, shape 1 is 1 (Excel's 1.0000000000000235 is its LN's error)
    assert value(GAMMA_DIST, 1e-300, 1, 1, False) == 1.0
    # Density at 0 (T-1049): 0 above shape 1, the df-2 chi-squared 0.5
    assert value(GAMMA_DIST, 0, 9, 2, False) == 0.0
    assert value(CHISQ_DIST, 0, 2, False) == 0.5
    assert value(CHISQ_DIST, 0, 3, False) == 0.0
    assert value(CHISQ_DIST_RT, 0, 7) == 1.0


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (GAMMA_DIST, (-1.0, 1.0, 1.0, True)),
        (GAMMA_DIST, (1.0, 0.0, 1.0, True)),
        (GAMMA_DIST, (1.0, 1.0, 0.0, False)),
        # T-2738: the density at a subnormal x / beta, either side of the evidenced cut
        (GAMMA_DIST, (4.4e-308, 1.0, 2.0, False)),
        (GAMMA_DIST, (1e-10, 1.0, 1e300, False)),
        # T-3316 rule 1: shape / x overflows
        (GAMMA_DIST, (5.5626e-300, 1e9, 1.0, False)),
        # incomplete-gamma-cap E6: x / beta overflows, the cdf too
        (GAMMA_DIST, (1e300, 1.0, 1e-10, True)),
        # T-4308: the density at the mode past the 2 pi shape overflow is #DIV/0!
        (GAMMA_DIST, (5e307, 5e307, 1.0, False)),
        (CHISQ_DIST, (0.0, 1.0, False)),
        (CHISQ_DIST, (1.0, 0.0, True)),
        (CHISQ_DIST, (1.0, 10000000001.0, True)),
        (CHISQ_DIST_RT, (1.0, 10000000001.0)),
        (CHISQ_DIST_RT, (-1.0, 3.0)),
        # T-4635: a huge finite x / beta, y / shape * 4 overflowing (shape 1.5 at 7E307) and y^shape (1.1 at 1E282)
        (GAMMA_DIST, (7e299, 1.5, 1e-8, True)),
        (GAMMA_DIST, (1e274, 1.1, 1e-8, False)),
        (CHISQ_DIST, (1.4e308, 3.0, True)),
        (CHISQ_DIST_RT, (1.4e308, 3.0)),
    ],
)
def test_error_domain(definition: Definition, args: tuple[float | bool, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([a if isinstance(a, bool) else mp(a) for a in args])


def test_evidenced_numbers_next_to_the_cuts() -> None:
    # T-2738: 4.5E-308 / 2 is normal; T-3316: 5.5628E-300 at shape 1E+9 is 0
    assert value(GAMMA_DIST, 4.5e-308, 1, 2, False) == pytest.approx(0.5, rel=1e-15)
    assert value(GAMMA_DIST, 5.5628e-300, 1e9, 1, False) == 0.0
    # a subnormal x / beta's cdf below the floor keeps its value; above it, there is no math value (Excel reads 0)
    assert value(GAMMA_DIST, 3e-308, 1, 2, True) < 1e-300
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(1e-300), mp(1e-17), mp(1e100), True])
    # CHISQ.DIST's cdf gets the same rule at a subnormal x / 2 (T-4395); a normal x / 2 keeps its value
    with pytest.raises(NoValueError):
        CHISQ_DIST.evaluate([mp(4e-308), mp(1), True])
    assert value(CHISQ_DIST, 4.5e-308, 1, True) > 1e-300


def test_huge_finite_ratio_numbers() -> None:
    # T-4635: numbers just below rule R's cuts, at shape 1 and below, and at df 2 and 4
    assert value(GAMMA_DIST, 5e299, 1.5, 1e-8, True) == 1.0
    assert value(GAMMA_DIST, 1e272, 1.1, 1e-8, True) == 1.0
    assert value(GAMMA_DIST, 1e298, 1.25, 1e-8, True) == 1.0
    assert value(GAMMA_DIST, 1.79e300, 0.7, 1e-8, True) == 1.0
    assert value(GAMMA_DIST, 1.79e300, 4, 1e-8, True) == 1.0
    assert value(CHISQ_DIST_RT, 1.3e308, 3) == 0.0
    assert value(CHISQ_DIST_RT, 1.79e308, 4) == 0.0


def test_no_value_regions() -> None:
    # T-4635: y^shape overflowing between shapes 1.1 and 1.11, and y / shape * 4 alone below 1.01, are unprobed
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(1e298), mp(1.105), mp(1e-8), True])
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(5e299), mp(1.001), mp(1e-8), True])
    # rule 2 of T-3316: a computed 0 from shape 1E+55 can be #NUM!
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(2e-10), mp(1e70), mp(1), True])
    # off the mode past the 2 pi shape overflow is unprobed
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(4e307), mp(5e307), mp(1), False])
    # the density at 0 below shape 1 is unbounded
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(0), mp(0.5), mp(1), False])
    # a rounded x / beta at a sharp huge-shape cdf is ill-conditioned; an exact one is not
    with pytest.raises(NoValueError):
        GAMMA_DIST.evaluate([mp(1e300), mp(1e300), mp(3), True])
    assert value(GAMMA_DIST, 1e300, 1e300, 1, True) == pytest.approx(0.5, rel=1e-9)


def test_deep_tails_are_zero_or_one() -> None:
    assert value(CHISQ_DIST_RT, 2.67618389548329e67, 58) == 0.0
    assert value(CHISQ_DIST, 2.67618389548329e67, 58, True) == 1.0
    assert value(CHISQ_DIST, 1.96698157885739e65, 24, False) == 0.0
    assert value(CHISQ_DIST_RT, 1, 1e10) == 1.0
