"""Student t: T.DIST and T.INV (the definitions of SPEC-0029's first distribution and inverse).

Sources:
- statistics.md, "Student t sweep: df truncation, df limits, T.INV.2T above 1 (T-1043)": df < 1 is
  an error; the cumulative T.DIST and T.INV are `#NUM!` above df = 1E10 (the cut is on the
  truncated df), while the density stays finite above it; p outside (0, 1) is `#NUM!`.
- statistics.md, "Student-t quantile at a tiny probability (T-1531)": T.INV is finite for p down to
  1E-300 (the power-law tail), so p's extreme range reaches 1E-300.
- The math: the lower tail is I_y(df/2, 1/2)/2 with y = df/(df + x²) (Abramowitz and Stegun
  26.7.1, the regularized incomplete beta), the density Γ((df+1)/2)/(√(df·π) Γ(df/2)) ·
  (1 + x²/df)^(-(df+1)/2) (A&S 26.7.1); the quantile solves the lower tail.
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_bracket,
)

DF_SOURCE = "statistics.md#student-t-sweep-df-truncation-df-limits-tinv2t-above-1-t-1043"
TINY_P_SOURCE = "statistics.md#student-t-quantile-at-a-tiny-probability-t-1531"
DF_CUT = 1e10  # the cumulative functions' last valid (truncated) df


def lower_tail(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T ≤ x) for x ≤ 0: I_y(df/2, 1/2)/2, y = df/(df + x²); exact in the far tail since no
    complement is taken."""
    if x == 0:
        return mpmath.mpf(1) / 2
    y = df / (df + x * x)
    return mpmath.betainc(df / 2, mpmath.mpf(1) / 2, 0, y, regularized=True) / 2


def cdf(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T ≤ x)."""
    return lower_tail(x, df) if x <= 0 else 1 - lower_tail(-x, df)


def density(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The t density, through log-gamma so a huge df neither overflows nor cancels."""
    log_value = (
        mpmath.loggamma((df + 1) / 2)
        - mpmath.loggamma(df / 2)
        - mpmath.log(df * mpmath.pi) / 2
        - (df + 1) / 2 * mpmath.log1p(x * x / df)
    )
    return mpmath.exp(log_value)


def lower_quantile(p: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The x ≤ 0 with P(T ≤ x) = p, for 0 < p ≤ 1/2: solves ln P(T ≤ -e^s) = ln p in s (log
    scales keep a 1E-300 tail and a centre point equally well conditioned). NoValueError when the
    quantile is beyond the double range."""
    if p == mpmath.mpf(1) / 2:
        return mpmath.mpf(0)
    target = mpmath.log(p)

    def f(s: mpmath.mpf) -> mpmath.mpf:
        return target - mpmath.log(lower_tail(-mpmath.exp(s), df))

    # e^-60 ≈ 9E-27 is closer to 0 than any 15-digit p below 1/2 puts the quantile; e^710 is past
    # the double range, where the quantile would overflow.
    try:
        s = solve_bracket(f, mpmath.mpf(-60), mpmath.mpf(710))
    except NoValueError as err:
        msg = f"quantile outside the double range ({err})"
        raise NoValueError(msg) from err
    return -mpmath.exp(s)


def quantile(p: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The x with P(T ≤ x) = p (the t distribution is symmetric, so p > 1/2 is -x(1 - p), with
    1 - p exact at the working precision)."""
    return lower_quantile(p, df) if p <= mpmath.mpf(1) / 2 else -lower_quantile(1 - p, df)


def _tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    df = args[1]
    if isinstance(df, bool) or df < 1:
        msg = "no tail map below df 1"
        raise ValueError(msg)
    x = lower_quantile(q, mpmath.floor(df))
    return -x if upper else x


def _t_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df, cumulative = args
    assert not isinstance(x, bool) and not isinstance(df, bool)
    return cdf(x, df) if cumulative else density(x, df)


def _t_dist_domain(args: Sequence[MathArg]) -> str | None:
    df, cumulative = args[1], args[2]
    if cumulative and not isinstance(df, bool) and df > DF_CUT:
        return "cumulative df above 1E+10"
    return None


def _t_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, df = args
    assert not isinstance(p, bool) and not isinstance(df, bool)
    return quantile(p, df)


_DF = Param(
    "deg_freedom",
    "int",
    Interval(1, float("inf"), lo_open=False),
    typical=(1, 30),
    extreme=(1e3, DF_CUT),
    edges=(DF_CUT, DF_CUT + 1),
    source=DF_SOURCE,
)

T_DIST = register(
    Definition(
        function="T.DIST",
        params=(
            Param(
                "x",
                "real",
                Interval(),
                typical=(-6, 6),
                extreme=(1e2, 1e300),
                edges=(0.0,),
                source="A&S 26.7",
            ),
            _DF,
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_t_dist,
        domain=_t_dist_domain,
        tail=_tail_x,
        tail_param="x",
        sources=(DF_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)

T_INV = register(
    Definition(
        function="T.INV",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=TINY_P_SOURCE,
            ),
            Param(
                "deg_freedom",
                "int",
                Interval(1, DF_CUT, lo_open=False, hi_open=False),
                typical=(1, 30),
                extreme=(1e3, DF_CUT),
                source=DF_SOURCE,
            ),
        ),
        value=_t_inv,
        sources=(DF_SOURCE, TINY_P_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)
