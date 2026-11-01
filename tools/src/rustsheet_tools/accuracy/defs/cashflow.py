"""IRR and XIRR: the rates of a series of flows (Gate A slice 3f, T-3586).

The math: both are the r > -1 at which the flows are worth nothing, sum_i c_i·(1 + r)^(-t_i) = 0, with t_i = i for
IRR (periodic flows) and t_i = (d_i - d_0)/365 for XIRR (dated flows, Excel's Actual/365). In u = ln(1 + r) the sum
is an exponential sum f(u) = sum_i c_i·e^(-t_i·u). For time-ordered flows it has at most as many real roots as the
flows have sign changes (Descartes' rule for exponential sums):
- with exactly one sign change the root is unique, and it is the rate;
- with none there is no rate (Excel's `#NUM!`);
- with several, Excel's root is a secant (IRR) or bisection (XIRR) path from the guess. No math rule names that root
  (financial.md, "IRR, RATE and XIRR stop early"), so such points are not drawn (SPEC-0029).

Each draw builds the series around a drawn rate: the later flows first, then the first flow that makes the drawn rate
the root. The point's math is the root of the written (15-digit) flows.

Sources:
- financial.md, "IRR, RATE and XIRR stop early, and the guess and flow scaling change the result": the multi-root
  case, and Excel's guess (IRR 0.1, XIRR 0.1) that the table leaves at its default.
- financial.md, "XIRR is a bracketing bisection, not Newton" and "XIRR reads empty cells as 0 and accepts earlier
  dates": the table writes increasing integer dates, so the order and truncation rules do not apply.
"""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import MAX_BRACKET_DOUBLINGS, sign_changes
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_bracket,
)
from rustsheet_tools.accuracy.sampler import log_uniform

STOP_EARLY_SOURCE = (
    "financial.md#irr-rate-and-xirr-stop-early-and-the-guess-and-flow-scaling-change-the-result"
)
XIRR_SOURCE = "financial.md#xirr-is-a-bracketing-bisection-not-newton"
DAYS_PER_YEAR = 365
FIRST_DATE = (36526, 47000)  # 2000-01-01 to 2028-09-04: integer date serials


def flow_rate(flows: Sequence[mpmath.mpf], times: Sequence[mpmath.mpf]) -> mpmath.mpf:
    """The unique r > -1 with sum c_i (1 + r)^(-t_i) = 0, `times` increasing from 0. DomainError with flows of one sign,
    NoValueError with several sign changes (no evidenced root choice) or no bracket."""
    changes = sign_changes(flows)
    if changes == 0:
        msg = "flows of one sign: no rate"
        raise DomainError(msg)
    if changes > 1:
        msg = "several sign changes: no evidenced root choice"
        raise NoValueError(msg)

    def f(u: mpmath.mpf) -> mpmath.mpf:
        return mpmath.fsum(c * mpmath.exp(-t * u) for c, t in zip(flows, times, strict=True))

    if f(mpmath.mpf(0)) == 0:  # the flows sum to 0: the root is exactly 0
        return mpmath.mpf(0)
    lo, hi = mpmath.mpf(-1), mpmath.mpf(1)
    for _ in range(MAX_BRACKET_DOUBLINGS):
        if (f(lo) > 0) != (f(hi) > 0):
            return mpmath.expm1(solve_bracket(f, lo, hi))
        lo, hi = lo * 2, hi * 2
    msg = "no root bracket"
    raise NoValueError(msg)


def _array(a: MathArg) -> tuple[mpmath.mpf, ...]:
    assert isinstance(a, tuple)
    return a


def _irr(args: Sequence[MathArg]) -> mpmath.mpf:
    flows = _array(args[0])
    return flow_rate(flows, [mpmath.mpf(i) for i in range(len(flows))])


def _xirr(args: Sequence[MathArg]) -> mpmath.mpf:
    flows, dates = _array(args[0]), _array(args[1])
    if len(flows) != len(dates):
        msg = "values and dates differ in size"
        raise DomainError(msg)
    return flow_rate(flows, [(d - dates[0]) / DAYS_PER_YEAR for d in dates])


def _series(stratum: str, rng: random.Random) -> tuple[float, int, float, bool]:
    """(rate, number of flows, flow scale, all one sign) for a stratum."""
    r = log_uniform(rng, 1e-2, 0.3)
    n = rng.randint(2, 20)
    scale = log_uniform(rng, 1e2, 1e6)
    one_sign = False
    if stratum == "tail":  # near 0 and near -10% (T-3347), or large
        shape = rng.randrange(3)
        if shape == 0:
            r = rng.choice((-1.0, 1.0)) * log_uniform(rng, 1e-12, 1e-4)
        elif shape == 1:
            r = -0.1 + rng.choice((-1.0, 1.0)) * log_uniform(rng, 1e-8, 1e-2)
        else:
            r = rng.uniform(1, 5)
    elif stratum == "edge":
        shape = rng.randrange(3)
        if shape == 0:
            n = 2
        elif shape == 1:
            r = -rng.uniform(0.01, 0.5)
        else:
            one_sign = True
    elif stratum == "extreme":
        n = rng.randint(50, 200)
        scale = log_uniform(rng, 1e9, 1e14)
    elif stratum == "log-uniform":
        r = rng.choice((-1.0, 1.0)) * log_uniform(rng, 1e-10, 0.9)
        n = round(log_uniform(rng, 2, 100))
        scale = log_uniform(rng, 1, 1e12)
    return r, n, scale, one_sign


def _flows(
    r: float, times: Sequence[float], scale: float, one_sign: bool, rng: random.Random
) -> tuple[float, ...]:
    """Later flows of one sign at `scale`, then the first flow that makes `r` the root; the series' sign is flipped at
    random. With `one_sign` every flow keeps one sign (no rate: the point checks Excel's error)."""
    later = [scale * log_uniform(rng, 0.1, 10) for _ in times[1:]]
    with mpmath.workdps(30):
        x = mpmath.mpf(1) + r
        first = -float(mpmath.fsum(c * x ** (-t) for c, t in zip(later, times[1:], strict=True)))
    if one_sign:
        first = -first
    sign = 1.0 if rng.random() < 0.5 else -1.0
    return tuple(sign * c for c in (first, *later))


def _irr_draw(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, scale, one_sign = _series(stratum, rng)
    return [_flows(r, [float(i) for i in range(n)], scale, one_sign, rng)]


def _xirr_draw(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, scale, one_sign = _series(stratum, rng)
    if stratum == "extreme" and rng.random() < 0.5:  # two flows over a long span (T-3353)
        n = 2
        gaps = [rng.randint(3650, 18250)]
    else:
        gaps = [rng.randint(1, 400) for _ in range(n - 1)]
    dates = [rng.randint(*FIRST_DATE)]
    for g in gaps:
        dates.append(dates[-1] + g)
    times = [(d - dates[0]) / DAYS_PER_YEAR for d in dates]
    return [_flows(r, times, scale, one_sign, rng), tuple(dates)]


IRR = register(
    Definition(
        function="IRR",
        params=(Param("values", "array", source=STOP_EARLY_SOURCE),),
        value=_irr,
        draw=_irr_draw,
        solver=True,
        sources=(STOP_EARLY_SOURCE, "decision 0188 item 2"),
    )
)

XIRR = register(
    Definition(
        function="XIRR",
        params=(
            Param("values", "array", source=STOP_EARLY_SOURCE),
            Param("dates", "array", source=XIRR_SOURCE),
        ),
        value=_xirr,
        draw=_xirr_draw,
        solver=True,
        sources=(STOP_EARLY_SOURCE, XIRR_SOURCE, "decision 0188 item 2"),
    )
)
