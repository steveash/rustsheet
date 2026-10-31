"""RATE (the definition of SPEC-0029's first financial solver).

The math: RATE is the r > -1 at which the annuity's flows are worth nothing,
pv + pmt·(1 + r·type)·(1 - (1+r)^-nper)/r + fv·(1+r)^-nper = 0 (with pv + pmt·nper + fv at r = 0;
financial.md, "RATE is a secant on the FV residual", gives the same residual times (1+r)^nper).
The flows are pv (plus pmt when type is 1) at t = 0, pmt at each period and fv (plus pmt when type
is 0) at t = nper. With exactly one sign change in that sequence the root in (-1, ∞) is unique
(Descartes' rule of signs on x = 1 + r); with none there is no root (Excel's `#NUM!`, decision 0188
item 2); with several, Excel's root choice is a secant from the guess (financial.md, "IRR, RATE and
XIRR stop early"), which no math rule names, so such points are not drawn (SPEC-0029).

The solver draws the annuity as a whole: a rate, nper, pv, fv and type, then the pmt that makes
the drawn rate the root; the point's math is the root of the written (15-digit) arguments.
"""

from __future__ import annotations

import itertools
import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_bracket,
)

RATE_SOURCE = "financial.md#rate-is-a-secant-on-the-fv-residual"
MAX_BRACKET_DOUBLINGS = 24  # |ln(1 + r)| up to 2^24: far past any double rate


def sign_changes(flows: Sequence[mpmath.mpf]) -> int:
    """Sign changes in `flows`, zeros skipped."""
    signs = [f > 0 for f in flows if f != 0]
    return sum(1 for a, b in itertools.pairwise(signs) if a != b)


def annuity_flows(nper: int, pmt: mpmath.mpf, pv: mpmath.mpf, fv: mpmath.mpf, due: bool) -> list[mpmath.mpf]:
    """The flow sequence with each run of equal payments collapsed to one (sign changes only
    need the runs): pv (plus pmt when due) at t = 0, pmt at t = 1..nper-1, fv (plus pmt when not
    due) at t = nper."""
    first = pv + (pmt if due else 0)
    last = fv + (0 if due else pmt)
    return [first, pmt, last] if nper >= 2 else [first, last]


def present_value(
    u: mpmath.mpf, nper: int, pmt: mpmath.mpf, pv: mpmath.mpf, fv: mpmath.mpf, *, due: bool
) -> mpmath.mpf:
    """The annuity's value at r = e^u - 1 (u = ln(1 + r) keeps r near -1 and r near 0 well
    conditioned; expm1 avoids the cancellation of (1+r)^-n - 1)."""
    if u == 0:
        return pv + pmt * nper + fv
    r = mpmath.expm1(u)
    discount = mpmath.exp(-nper * u)
    factor = -mpmath.expm1(-nper * u) / r
    return pv + pmt * (1 + r * (1 if due else 0)) * factor + fv * discount


def rate(nper: int, pmt: mpmath.mpf, pv: mpmath.mpf, fv: mpmath.mpf, due: bool) -> mpmath.mpf:
    """The unique root r > -1; DomainError with no root, NoValueError with several."""
    changes = sign_changes(annuity_flows(nper, pmt, pv, fv, due))
    if changes == 0:
        msg = "flows of one sign: no rate"
        raise DomainError(msg)
    if changes > 1:
        msg = "several sign changes: no evidenced root choice"
        raise NoValueError(msg)

    def f(u: mpmath.mpf) -> mpmath.mpf:
        return present_value(u, nper, pmt, pv, fv, due=due)

    if f(mpmath.mpf(0)) == 0:  # pv + pmt·nper + fv = 0: the root is exactly 0, which no
        return mpmath.mpf(0)  # relative bracket test reaches
    lo, hi = mpmath.mpf(-1), mpmath.mpf(1)
    for _ in range(MAX_BRACKET_DOUBLINGS):
        if (f(lo) > 0) != (f(hi) > 0) or f(lo) == 0 or f(hi) == 0:
            return mpmath.expm1(solve_bracket(f, lo, hi))
        lo, hi = lo * 2, hi * 2
    msg = "no root bracket"
    raise NoValueError(msg)


def _rate(args: Sequence[MathArg]) -> mpmath.mpf:
    nper, pmt, pv, fv, kind = args
    assert not any(isinstance(a, bool) for a in args)
    return rate(int(nper), mpmath.mpf(pmt), mpmath.mpf(pv), mpmath.mpf(fv), kind != 0)


def payment(r: float, nper: int, pv: float, fv: float, due: bool) -> float:
    """The pmt that makes `r` the root (PMT's formula), clamped to the writable range."""
    with mpmath.workdps(30):
        x = mpmath.mpf(1) + r
        discount = x ** (-nper)
        value = -(pv + fv * discount) * r / ((1 + r * (1 if due else 0)) * (1 - discount))
        return float(max(min(value, MAX_LITERAL), -MAX_LITERAL))


def _log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return float(mpmath.exp(rng.uniform(float(mpmath.log(lo)), float(mpmath.log(hi)))))


def _draw(stratum: str, rng: random.Random) -> list[Arg]:
    """One annuity per stratum's shape: [nper, pmt, pv, fv, type]."""
    sign = 1.0 if rng.random() < 0.5 else -1.0
    nper = rng.randint(1, 360)
    r = _log_uniform(rng, 1e-3, 0.2)
    pv = sign * _log_uniform(rng, 1e2, 1e6)
    fv = 0.0 if rng.random() < 0.5 else -sign * abs(pv) * rng.random()
    due = rng.random() < 0.25
    same_sign = False
    if stratum == "tail":  # rates near 0, or large
        r = (
            rng.choice((-1.0, 1.0)) * _log_uniform(rng, 1e-12, 1e-4)
            if rng.random() < 0.5
            else rng.uniform(0.5, 5)
        )
    elif stratum == "edge":
        shape = rng.randrange(4)
        if shape == 0:
            nper = 1
        elif shape == 1:
            fv, due = 0.0, True
        elif shape == 2:
            r = -rng.uniform(1e-3, 0.05)
        else:
            same_sign = True
    elif stratum == "extreme":
        nper = round(_log_uniform(rng, 360, 1e4))
        pv = sign * _log_uniform(rng, 1e6, 1e15)
        fv = 0.0 if rng.random() < 0.5 else -sign * abs(pv) * rng.random()
    elif stratum == "log-uniform":
        nper = round(_log_uniform(rng, 1, 1e4))
        r = rng.choice((-1.0, 1.0)) * _log_uniform(rng, 1e-10, 0.9)
        pv = sign * _log_uniform(rng, 1, 1e12)
        fv = 0.0 if rng.random() < 0.5 else -sign * _log_uniform(rng, 1, 1e12)
        due = rng.random() < 0.5
    pmt = payment(r, nper, pv, fv, due)
    if same_sign:  # every flow the sign of pv: no rate (the point checks Excel's error)
        pmt, fv = sign * abs(pmt), sign * abs(fv)
    return [nper, pmt, pv, fv, 1 if due else 0]


_AMOUNT = Interval()

RATE = register(
    Definition(
        function="RATE",
        params=(
            Param("nper", "int", Interval(1, float("inf"), lo_open=False), source=RATE_SOURCE),
            Param("pmt", "real", _AMOUNT, source=RATE_SOURCE),
            Param("pv", "real", _AMOUNT, source=RATE_SOURCE),
            Param("fv", "real", _AMOUNT, source=RATE_SOURCE),
            Param("type", "int", Interval(0, 1, lo_open=False, hi_open=False), source=RATE_SOURCE),
        ),
        value=_rate,
        draw=_draw,
        solver=True,
        sources=(RATE_SOURCE, "decision 0188 item 2"),
    )
)
