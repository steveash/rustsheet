"""Gate A definitions of BINOM.DIST, NEGBINOM.DIST and POISSON.DIST (T-3553) against published values."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import discrete
from rustsheet_tools.accuracy.model import REGISTRY, DomainError, NoValueError

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(function: str, *args: float | bool) -> mpmath.mpf:
    return REGISTRY[function].evaluate([a if isinstance(a, bool) else mp(a) for a in args])


def near(got: mpmath.mpf, published: str, digits: int) -> bool:
    # Microsoft's examples show 7 significant digits
    return bool(mpmath.almosteq(got, mp(published), mp(10) ** -digits))


def test_binom_dist_matches_the_published_example_and_sums_to_the_cdf() -> None:
    assert value("BINOM.DIST", 6, 10, 0.5, False) == mp(210) / 1024  # Microsoft's example, 0.2050781
    total = mpmath.fsum(value("BINOM.DIST", k, 17, 0.3, False) for k in range(8))
    assert mpmath.almosteq(value("BINOM.DIST", 7, 17, 0.3, True), total, mp(10) ** -35)
    assert value("BINOM.DIST", 17, 17, 0.3, True) == 1
    assert value("BINOM.DIST", 0, 0, 1, False) == 1


def test_binom_dist_domain_follows_excel() -> None:
    for args in [(11, 10, 0.5, True), (0, 3, 1, False), (0, 2**31 - 1, 0.5, True), (1, 10, 1.5, False)]:
        with pytest.raises(DomainError):
            value("BINOM.DIST", *args)
    assert value("BINOM.DIST", 0, 3, 1, True) == 0  # the cdf at p = 1 stays a number


def test_negbinom_dist_matches_the_published_examples() -> None:
    assert near(value("NEGBINOM.DIST", 10, 5, 0.25, True), "0.3135141", 7)
    assert near(value("NEGBINOM.DIST", 10, 5, 0.25, False), "0.0550487", 7)
    for p in (0, 1):  # open at both ends, unlike BINOM.DIST (statistics.md, T-1048)
        with pytest.raises(DomainError):
            value("NEGBINOM.DIST", 2, 3, p, False)
    total = mpmath.fsum(value("NEGBINOM.DIST", f, 4, 0.6, False) for f in range(12))
    assert mpmath.almosteq(value("NEGBINOM.DIST", 11, 4, 0.6, True), total, mp(10) ** -35)


def test_poisson_dist_matches_the_published_examples_and_the_negative_mean_quirk() -> None:
    assert near(value("POISSON.DIST", 2, 5, True), "0.124652", 6)
    assert near(value("POISSON.DIST", 2, 5, False), "0.084224", 6)
    assert (
        value("POISSON.DIST", 0, -1, False) == mpmath.e
    )  # statistics.md, "POISSON at x 0 with a negative mean"
    with pytest.raises(DomainError):
        value("POISSON.DIST", 1, -1, True)
    with pytest.raises(NoValueError):  # EXP(1E+4) overflows a double: the sampler redraws
        value("POISSON.DIST", 0, -1e4, True)


def test_tail_maps_find_the_integer_quantile() -> None:
    n, p, m = mp(100), mp("0.5"), mp(30)
    k = discrete.integer_quantile(
        lambda k: discrete.binom_cdf(k, n, p), lambda k: discrete.binom_sf(k, n, p), mp("1e-6"), False, 100
    )
    assert discrete.binom_cdf(k, n, p) >= mp("1e-6") > discrete.binom_cdf(k - 1, n, p)
    x = discrete.integer_quantile(
        lambda x: discrete.poisson_cdf(x, m), lambda x: discrete.poisson_sf(x, m), mp("1e-200"), True, 1e15
    )
    assert discrete.poisson_sf(x, m) <= mp("1e-200") < discrete.poisson_sf(x - 1, m)
    for k in (0, 7, 99):  # the direct upper tails are the complements
        assert mpmath.almosteq(
            discrete.binom_cdf(mp(k), n, p) + discrete.binom_sf(mp(k), n, p), 1, mp(10) ** -35
        )
        assert mpmath.almosteq(
            discrete.poisson_cdf(mp(k), m) + discrete.poisson_sf(mp(k), m), 1, mp(10) ** -35
        )
        f, s = mp(k), mp(4)
        assert mpmath.almosteq(
            discrete.negbinom_cdf(f, s, p) + discrete.negbinom_sf(f, s, p), 1, mp(10) ** -35
        )
