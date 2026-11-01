"""The gamma and chi-squared quantiles: GAMMA.INV, CHISQ.INV and CHISQ.INV.RT.

Sources:
- statistics.md, "GAMMA.INV and CHISQ.INV at a huge shape (T-2982, T-3070)": CHISQ.INV and the cdfs are `#NUM!` above
  1E10 degrees of freedom, while CHISQ.INV.RT takes any df (evidenced to about 1E16); the shape is df/2 and the scale 2.
- statistics.md, "Chi-squared quantile at df 2 and gamma quantile at alpha 1 are closed forms (T-2860)":
  `GAMMA.INV(1E-300,1,1)` = 1E-300 shows p reaches 1E-300.
- The math: P(X ≤ x) = P(a, x/θ), the regularized lower incomplete gamma, and its complement Q(a, x/θ) taken
  directly (Abramowitz and Stegun 6.5.1, 26.4.19 for the chi-squared law); the quantile solves the smaller tail in log
  scales, so a 1E-300 tail is as well conditioned as the centre.
"""

from __future__ import annotations

from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.student_t import DF_CUT
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_log_bracket,
)

HUGE_SOURCE = "statistics.md#gammainv-and-chisqinv-at-a-huge-shape-the-true-quantile-num-or-a-wilson-hilferty-closed-form-t-2982-t-3070"
CLOSED_SOURCE = (
    "statistics.md#chi-squared-quantile-at-df-2-and-gamma-quantile-at-alpha-1-are-closed-forms-t-2860"
)
CHI_SCALE = mpmath.mpf(2)
TABLE_DF = 2e7  # the s1 batches' largest drawn df (shape 1E7), see CHISQ_INV_RT


MAX_TERMS = 10**7  # far above the about 20 sqrt(a) terms a 1E7 shape needs at 50 digits
# The largest shape the series and continued fraction are summed at: about 20 sqrt(a) terms each, so a 1E9 shape
# (probes-numeric's huge-shape cells) would take minutes per tail evaluation. Above it there is no value here; the
# huge-shape branch (a uniform-asymptotic expansion) is T-3578's.
SHAPE_CAP = mpmath.mpf(10) ** 8


def _log_prefix(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """ln(y^a e^-y / Γ(a)), the factor both expansions share."""
    return a * mpmath.log(y) - y - mpmath.loggamma(a)


def _series(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """P(a, y) by its power series, Σ y^n / (a (a+1) ... (a+n)) (A&S 6.5.29), for y < a + 1."""
    eps = mpmath.mpf(2) ** (-mpmath.mp.prec + 4)  # a few ulps: delta - 1 is rounding noise at the last bit
    term = total = 1 / a
    n = a
    for _ in range(MAX_TERMS):
        n += 1
        term *= y / n
        total += term
        if abs(term) < abs(total) * eps:
            return total * mpmath.exp(_log_prefix(y, a))
    msg = "the incomplete gamma series did not converge"
    raise NoValueError(msg)


def _continued_fraction(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """Q(a, y) by its continued fraction (A&S 6.5.31, modified Lentz), for y ≥ a + 1."""
    eps = mpmath.mpf(2) ** (-mpmath.mp.prec + 4)  # a few ulps: delta - 1 is rounding noise at the last bit
    tiny = mpmath.mpf(10) ** -(mpmath.mp.dps * 4)
    b = y + 1 - a
    c = 1 / tiny
    d = 1 / b
    h = d
    for i in range(1, MAX_TERMS):
        an = -i * (i - a)
        b += 2
        d = an * d + b
        d = tiny if d == 0 else d
        c = b + an / c
        c = tiny if c == 0 else c
        d = 1 / d
        delta = d * c
        h *= delta
        if abs(delta - 1) < eps:
            return h * mpmath.exp(_log_prefix(y, a))
    msg = "the incomplete gamma continued fraction did not converge"
    raise NoValueError(msg)


def _check_shape(a: mpmath.mpf) -> None:
    if a > SHAPE_CAP:
        msg = f"shape {mpmath.nstr(a, 3)} is above the summed expansions' cap of 1E8 (T-3578)"
        raise NoValueError(msg)


def lower_tail(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """P(a, y), y ≥ 0 (unit scale). mpmath's `gammainc` does not converge from a shape of about 1E5, so the classic
    series and continued fraction are summed here at the working precision; each side's complement is taken only
    where the other tail is below about 1/2, so no digits cancel. NoValueError above SHAPE_CAP."""
    _check_shape(a)
    if y == 0:
        return mpmath.mpf(0)
    return _series(y, a) if y < a + 1 else 1 - _continued_fraction(y, a)


def upper_tail(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """Q(a, y), y ≥ 0, from the continued fraction past a + 1 (no 1 - P cancellation in the upper tail).
    NoValueError above SHAPE_CAP."""
    _check_shape(a)
    if y == 0:
        return mpmath.mpf(1)
    return 1 - _series(y, a) if y < a + 1 else _continued_fraction(y, a)


def quantile(p: mpmath.mpf, a: mpmath.mpf, upper: bool) -> mpmath.mpf:
    """The unit-scale y with P(a, y) = p (or Q(a, y) = p when `upper`), 0 < p < 1: solves the smaller tail
    ln tail(e^s) = ln q in s. NoValueError when the root is beyond the double range."""
    lower = (p <= mpmath.mpf(1) / 2) != upper
    q = p if (lower != upper) else 1 - p
    target = mpmath.log(q)

    def f(s: mpmath.mpf) -> mpmath.mpf:
        y = mpmath.exp(s)
        # the lower tail grows with s and the upper falls: opposite signs at the bracket ends
        return (mpmath.log(lower_tail(y, a)) - target) if lower else (target - mpmath.log(upper_tail(y, a)))

    return mpmath.exp(solve_log_bracket(f, -1500, 710))


def _gamma_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, alpha, beta = args
    assert (
        not isinstance(p, (bool, tuple))
        and not isinstance(alpha, (bool, tuple))
        and not isinstance(beta, (bool, tuple))
    )
    # The quantile of probability 0 is the support's lower end.
    return mpmath.mpf(0) if p == 0 else beta * quantile(p, alpha, upper=False)


def _chisq(upper: bool) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        p, df = args
        assert not isinstance(p, (bool, tuple)) and not isinstance(df, (bool, tuple))
        if p == (1 if upper else 0):
            return mpmath.mpf(0)
        return CHI_SCALE * quantile(p, mpmath.floor(df) / 2, upper=upper)

    return value


def _probability(source: str, lo_open: bool, hi_open: bool) -> Param:
    return Param(
        "probability",
        "real",
        Interval(0, 1, lo_open=lo_open, hi_open=hi_open),
        typical=(0.001, 0.999),
        extreme=(1e-300, 1e-15),
        source=source,
    )


def _df(hi: float) -> Param:
    """df from 1 to `hi` (CHISQ.INV's 1E10, CHISQ.INV.RT's none); the extreme draws stop at TABLE_DF."""
    return Param(
        "deg_freedom",
        "int",
        Interval(1, hi, lo_open=False, hi_open=False),
        typical=(1, 60),
        extreme=(1e3, TABLE_DF),
        edges=(2.0,),
        source=HUGE_SOURCE,
    )


GAMMA_INV = register(
    Definition(
        function="GAMMA.INV",
        params=(
            _probability(CLOSED_SOURCE, lo_open=False, hi_open=True),
            Param(
                "alpha",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 50),
                extreme=(1e-3, 1e7),
                edges=(1.0,),
                source=HUGE_SOURCE,
            ),
            Param(
                "beta",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 10),
                extreme=(1e-100, 1e100),
                source="Abramowitz and Stegun 6.5.1",
            ),
        ),
        value=_gamma_inv,
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 6.5.1"),
    )
)

CHISQ_INV = register(
    Definition(
        function="CHISQ.INV",
        params=(_probability(CLOSED_SOURCE, lo_open=False, hi_open=True), _df(DF_CUT)),
        value=_chisq(upper=False),
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 26.4.19"),
    )
)

# CHISQ.INV.RT takes any df (statistics.md, the huge-shape entry). The s1 batches of all three stop at a shape of
# 1E7 (df 2E7), where the series and continued fraction are still quick; the huge-shape branch from 1E9 is T-3578's.
CHISQ_INV_RT = register(
    Definition(
        function="CHISQ.INV.RT",
        params=(_probability(CLOSED_SOURCE, lo_open=True, hi_open=False), _df(float("inf"))),
        value=_chisq(upper=True),
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 26.4.19"),
    )
)
