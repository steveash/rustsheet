"""The beta distribution: BETA.DIST and BETA.INV on the standard interval [0, 1].

Sources:
- statistics.md, "BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)": the density is `#NUM!` at x = 0 when
  alpha ≤ 1 and at x = 1 when beta ≤ 1; the cdf is unaffected.
- statistics.md, "BETA.INV solves Excel's own CDF to the crossing (T-1745)" and "BETAINV is #NUM! where its quantile
  underflows (T-2435)": BETA.INV takes p in (0, 1) and is `#NUM!` where the quantile underflows (the sampler redraws).
- The math: P(X ≤ x) = I_x(a, b) and its complement I_{1-x}(b, a) taken directly (Abramowitz and Stegun 26.5.1,
  26.5.2); the density x^(a-1) (1-x)^(b-1) / B(a, b) in log space. The quantile solves the smaller tail in log
  scale, of x for the lower tail and of 1 - x for the upper, so a 1E-300 tail is as well conditioned as the centre.

The optional [A, B] bounds are a linear map of x (`A + (B - A) x`); the s1 batches stay on [0, 1] (T-3529).
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
    solve_log_bracket,
)

DENSITY_SOURCE = "statistics.md#betadists-density-is-num-at-an-x-of-0-or-1-t-2435"
INV_SOURCE = "statistics.md#betainv-solves-excels-own-cdf-to-the-crossing-t-1745"
UNDERFLOW_SOURCE = (
    "statistics.md#betainv-is-num-where-its-quantile-underflows-but-05-at-a-tiny-alpha-and-a-high-p-t-2435"
)


def _betainc(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b); NoValueError where mpmath's series does not converge (a shape near 1E7), so the sampler redraws."""
    try:
        return mpmath.betainc(a, b, 0, x, regularized=True)
    except mpmath.libmp.NoConvergence as err:
        msg = f"the incomplete beta did not converge ({err})"
        raise NoValueError(msg) from err


def lower_tail(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b)."""
    return _betainc(a, b, x)


def upper_tail(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """1 - I_x(a, b) as I_{1-x}(b, a), exact at the working precision."""
    return _betainc(b, a, 1 - x)


def density(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """The beta density; at an end it is 0 where the exponent is positive (the ≤ 1 cases are domain errors)."""
    if x in (0, 1):
        return mpmath.mpf(0)
    log_value = (a - 1) * mpmath.log(x) + (b - 1) * mpmath.log1p(-x) - mpmath.log(mpmath.beta(a, b))
    return mpmath.exp(log_value)


def quantile(p: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """The x with I_x(a, b) = p, 0 < p < 1. NoValueError when x underflows or rounds to 1 (no double strictly
    inside (0, 1) to record)."""
    if p <= mpmath.mpf(1) / 2:
        target = mpmath.log(p)
        s = solve_log_bracket(lambda s: mpmath.log(lower_tail(mpmath.exp(s), a, b)) - target, -1500, 0)
        return mpmath.exp(s)
    target = mpmath.log(1 - p)
    # 1 - x = e^s: the upper tail I_{e^s}(b, a) grows with s
    s = solve_log_bracket(lambda s: mpmath.log(_betainc(b, a, mpmath.exp(s))) - target, -1500, 0)
    x = 1 - mpmath.exp(s)
    if x >= 1 - mpmath.mpf(2) ** -54:
        msg = "the quantile rounds to 1"
        raise NoValueError(msg)
    return x


def _beta_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, a, b, cumulative = args
    assert not isinstance(x, (bool, tuple))
    assert not isinstance(a, (bool, tuple))
    assert not isinstance(b, (bool, tuple))
    return lower_tail(x, a, b) if cumulative else density(x, a, b)


def _beta_dist_domain(args: Sequence[MathArg]) -> str | None:
    x, a, b, cumulative = args
    if (
        cumulative
        or isinstance(x, (bool, tuple))
        or isinstance(a, (bool, tuple))
        or isinstance(b, (bool, tuple))
    ):
        return None
    if x == 0 and a <= 1:
        return "density at x = 0 with alpha <= 1"
    if x == 1 and b <= 1:
        return "density at x = 1 with beta <= 1"
    return None


def _tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    a, b = args[1], args[2]
    if isinstance(a, (bool, tuple)) or isinstance(b, (bool, tuple)):
        msg = "no tail map for a logical shape"
        raise ValueError(msg)
    return quantile(1 - q if upper else q, a, b)


def _beta_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, a, b = args
    assert (
        not isinstance(p, (bool, tuple))
        and not isinstance(a, (bool, tuple))
        and not isinstance(b, (bool, tuple))
    )
    return quantile(p, a, b)


def _shape(name: str) -> Param:
    return Param(
        name,
        "real",
        Interval(0, float("inf")),
        typical=(0.1, 20),
        extreme=(1e-3, 1e6),
        edges=(1.0,),
        source="Abramowitz and Stegun 26.5",
    )


BETA_DIST = register(
    Definition(
        function="BETA.DIST",
        params=(
            Param(
                "x",
                "real",
                Interval(0, 1, lo_open=False, hi_open=False),
                typical=(0.01, 0.99),
                extreme=(1e-300, 1e-15),
                source=DENSITY_SOURCE,
            ),
            _shape("alpha"),
            _shape("beta"),
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_beta_dist,
        domain=_beta_dist_domain,
        tail=_tail_x,
        tail_param="x",
        sources=(DENSITY_SOURCE, "Abramowitz and Stegun 26.5.1, 26.5.2"),
    )
)

BETA_INV = register(
    Definition(
        function="BETA.INV",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=INV_SOURCE,
            ),
            _shape("alpha"),
            _shape("beta"),
        ),
        value=_beta_inv,
        sources=(INV_SOURCE, UNDERFLOW_SOURCE, "Abramowitz and Stegun 26.5.1"),
    )
)
