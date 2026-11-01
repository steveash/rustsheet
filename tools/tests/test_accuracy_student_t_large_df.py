"""The Student t tail and quantile at df above 5E6 (T-3425): no non-convergence, against independent references."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import student_t

mp = mpmath.mpf
HALF = mp(1) / 2


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(60):
        yield


def relative(value: mpmath.mpf, reference: mpmath.mpf) -> mpmath.mpf:
    return abs(value - reference) / abs(reference)


@pytest.mark.parametrize("df", [1e7, 1e10])
@pytest.mark.parametrize("x", [-0.5, -3, -30])
def test_the_tail_matches_mpmaths_hypergeometric_beta_at_200_digits(df: float, x: float) -> None:
    # mpmath's betainc (a hypergeometric series with its 1 - y transformation) converges here at any precision;
    # the continued fraction (|x| >= 2) and the betainc branch (|x| < 2) must agree with it to 1e-40
    d, t = mp(df), mp(x)
    with mpmath.workdps(200):
        reference = mpmath.betainc(d / 2, HALF, 0, d / (d + t * t), regularized=True) / 2
    assert relative(student_t.lower_tail(t, d), reference) < mp("1e-40")


def test_the_tail_in_the_stalled_band_matches_the_complementary_series() -> None:
    # |x| = 100 at df 1E7: mpmath's betainc on I_y(df/2, 1/2) stops there (too many terms). The complement
    # 1 - I_(1-y)(1/2, df/2) converges in about x²/2 terms and cancels about 2,180 digits, so it runs at 2,260.
    d, t = mp(1e7), mp(-100)
    value = student_t.lower_tail(t, d)
    with mpmath.workdps(2260):
        z = t * t / (d + t * t)
        reference = (1 - mpmath.betainc(HALF, d / 2, 0, z, regularized=True)) / 2
    assert relative(value, reference) < mp("1e-40")


@pytest.mark.parametrize("df", [5395690, 1e10])
def test_the_tail_converges_across_the_root_search_bracket(df: float) -> None:
    # T.INV's root search evaluates the tail from x = -e^-60 to -e^710 (`lower_quantile`); before T-3425 mpmath raised
    # NoConvergence from about |x| = 90 up to about √df. Each value agrees at 60 and 80 digits (the table's check).
    d = mp(df)
    for s in (-60, -3, 0, 1, 4, 4.5, 5, 6, 8, 10, 20, 100, 700):
        t = -mpmath.exp(s)
        low = student_t.lower_tail(t, d)
        with mpmath.workdps(80):
            high = student_t.lower_tail(t, d)
        assert 0 <= low <= HALF
        assert low == 0 or relative(low, high) < mp("1e-45")


def test_the_dropped_t_inv_points_now_have_a_quantile() -> None:
    # two of the 11 points T-3357's trial batch dropped (T-3425's Context)
    for p, df in (("0.00103896513665999", 5395690), ("5.13945494250381E-214", 3344838607)):
        x = student_t.quantile(mp(p), mp(df))
        assert relative(student_t.lower_tail(x, mp(df)), mp(p)) < mp("1e-40")


def test_the_density_agrees_at_60_and_80_digits_at_a_huge_df() -> None:
    # T-3425: at df 8.3E+129 the log-gamma difference cancelled past 60 digits
    x, d = mp("-4.8121275954683E+66"), mp("8.30732917352682E+129")
    low = student_t.density(x, d)
    with mpmath.workdps(80):
        high = student_t.density(x, d)
    assert relative(low, high) < mp("1e-45")
