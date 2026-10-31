"""IRR's secant in d = 1 - v = r/(1 + r) (T-3120), and the `irr` fit that scores it beside the secant in v.

The secant in v of kernels/solvers.py (`irr_secant_v`, T-1533) matched 189 of the 263 fit cells; run on
d = 1 - v, from d0 = g/(1 + g), stepped from the newer iterate, with NPV by worksheet NPV's running product,
leading zero flows dropped and a zero-step stop, it matches 262 (T-3120), and all 263 with that NPV's
operators rounded twice as Excel's are (T-3205). The state d is what puts Excel's results on
the 2^-53 grid in v (financial.md "IRR is a secant in d = 1 - v").
"""

from __future__ import annotations

import math
import operator
import sys

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.solvers import (
    FIT_CASES,
    IrrMethod,
    NoResultError,
    NpvV,
    _irr,
    _literal_irr,
    _npv_v_over_x,
    irr_candidates,
    irr_secant_v,
)
from rustsheet_tools.fitlab.score import Candidate, candidate


def _flush(x: float) -> float:
    """A subnormal is 0, as the engine's `flush_subnormal` makes it."""
    return 0.0 if abs(x) < sys.float_info.min else x


def _npv_v_running(values: list[float], v: float, twice: bool = True) -> float:
    """NPV(v) as the sum of c_i / d_i with d_i = d_(i-1) * x a running product, x = 1/v: worksheet NPV's
    arithmetic (the engine's `discounted_sum`, T-2671). `twice`: every `*`, `/` and `+` is Excel's
    twice-rounded operator (T-3205; plain doubles leave the fit set's Calc!A214 and gen-m9's A395 one
    secant step early). Discounts and quotients below the smallest normal double are flushed to 0 as in
    the engine."""
    mul, div, add = (X87.mul, X87.div, X87.add) if twice else (operator.mul, operator.truediv, operator.add)
    x = X87.div(1.0, v)
    total = 0.0
    discount = 1.0
    for i, c in enumerate(values):
        if i:
            discount = _flush(mul(discount, x))
        if discount == 0 or not math.isfinite(discount):
            raise NoResultError("discount out of range")
        total = add(total, _flush(div(c, discount)))
    return total


def npv_v_running_plain(values: list[float], v: float) -> float:
    """`_npv_v_running` in plain doubles: T-3120's shipped NPV (262/263)."""
    return _npv_v_running(values, v, twice=False)


def irr_secant_d(
    npv: NpvV = _npv_v_running,
    *,
    start: str = "g/(1+g)",
    newer: bool = True,
    strip: bool = True,
    stall: float | None = 1e-7,
    stall_always: bool = False,
) -> IrrMethod:
    """The secant of `irr_secant_v` run on d = 1 - v = r/(1 + r) (T-3120): d0 = `start` (g/(1+g), or
    1 - 1/(1+g)), d1 = d0 -+ 0.001, NPV evaluated at v = 1 - d, so from v = 0.5 down every iterate's v
    sits on the 2^-53 grid Excel's results sit on. `newer`: the step is d1 - f1 (d1 - d0)/(f1 - f0)
    (else anchored on d0). `strip`: leading zero flows dropped. `stall`: a zero step (d2 = d1) also
    stops when |NPV| < stall * max|c_i| (None: never); `stall_always`: a zero step stops whatever the NPV
    (financial.md's "costs 6 fit cells"). Same stop tests, retry and 50-step cap."""

    def run(values: list[float], g: float) -> float:
        div = X87.div
        largest = max(abs(c) for c in values)
        d0 = div(g, 1 + g) if start == "g/(1+g)" else 1.0 - div(1.0, 1 + g)
        f0 = npv(values, 1.0 - d0)
        d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
        f1 = npv(values, 1.0 - d1)
        for _ in range(50):
            if f1 == f0:
                raise NoResultError("flat secant")
            d2 = d1 - div(f1 * (d1 - d0), f1 - f0) if newer else d0 - div(f0 * (d1 - d0), f1 - f0)
            if not math.isfinite(d2) or d2 >= 1:
                raise NoResultError("iterate is not a discount factor")
            f2 = npv(values, 1.0 - d2)
            stalled = d2 == d1 and (stall_always or (stall is not None and abs(f2) < stall * largest))
            if abs(d2 - d1) < 1e-7 and (abs(f2) < 1e-7 or stalled):
                return div(1.0, 1.0 - d2) - 1
            d0, f0, d1, f1 = d1, f1, d2, f2
        raise NoResultError("no convergence in 50 steps")

    def method(values: list[float], g: float, k: int | None) -> float:  # noqa: ARG001 - IrrMethod's step count
        if strip:
            values = values[next((i for i, c in enumerate(values) if c != 0), 0) :]
        try:
            return run(values, g)
        except (NoResultError, ZeroDivisionError, OverflowError):
            if g == 0.1:
                raise
            return run(values, 0.1)

    return method


def irr_grid_result(method: IrrMethod) -> IrrMethod:
    """`method`'s result with v = 1/(1 + r) rounded to the nearest multiple of 2^-53 below v = 0.5
    (T-3120's candidate A: Excel's results sit on that grid)."""

    def rounded(values: list[float], g: float, k: int | None) -> float:
        v = X87.div(1.0, 1 + method(values, g, k))
        if v < 0.5:
            v = round(v * 2.0**53) / 2.0**53
        return X87.div(1.0, v) - 1

    return rounded


def secant_d_candidates() -> list[Candidate]:
    methods: dict[str, IrrMethod] = {
        "secant in v, result on the 2^-53 grid (candidate A)": irr_grid_result(irr_secant_v()),
        "secant in d, shipped (T-3205)": irr_secant_d(),
        "secant in d, NPV in plain doubles (T-3120)": irr_secant_d(npv_v_running_plain),
        "secant in d, d0 = 1 - 1/(1+g)": irr_secant_d(start="1-v"),
        "secant in d, step anchored on d0 (candidate B)": irr_secant_d(newer=False),
        "secant in d, NPV as c/x^i": irr_secant_d(_npv_v_over_x),
        "secant in d, leading zeros kept": irr_secant_d(strip=False),
        "secant in d, no zero-step stop": irr_secant_d(stall=None),
        "secant in d, unconditional zero-step stop": irr_secant_d(stall_always=True),
    }
    return [candidate(name, _irr(method, None), name.split(",")[0]) for name, method in methods.items()]


FITS = [
    Fit(
        name="irr",
        description="IRR fit set: the secant in d (T-3120), the fitted secant in v (T-1533) and their variants, Newton candidates",
        cases=FIT_CASES,
        pattern=r"^=IRR\((.*)\)$",
        candidates=lambda: secant_d_candidates() + irr_candidates(),
        where=_literal_irr,
    ),
]
