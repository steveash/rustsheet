"""IRR's fallback where both secant runs fail (T-2928): the candidates for its returned digits.

The fit set is the T-2927 sweep (probes-numeric/t2927-irr-fallback-sweep, flows where the fitted secant
`solvers.irr_secant_v` fails from guess 0.1 and its retry) and probes/irr-fallback-roots, `#NUM!` cells
included. The engine's fallback (`irr_fallback`, crates/rustsheet-engine/src/functions/financial/
financial_util.rs) is Newton on P(v) = sum c_i v^i from v = 0; the candidates vary its stop test,
returned iterate, start, residual test, summation order and variable. Results: docs/excel-semantics/
financial.md, "IRR's fallback when the secant fails", Fit.
"""

from __future__ import annotations

import math
from collections.abc import Callable

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.solvers import (
    IrrMethod,
    NoResultError,
    NpvV,
    _dnpv_v,
    _irr,
    _literal_irr,
    _npv_v,
    floats,
    irr_newton_r,
    irr_newton_x,
    irr_secant_v,
)
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

FALLBACK_CASES = ["probes-numeric/t2927-irr-fallback-sweep", "probes/irr-fallback-roots"]
FALLBACK_STEPS = 50  # the engine's IRR_FALLBACK_STEPS


def _npv_v_ascending(values: list[float], v: float) -> float:
    """P(v) = sum c_i v^i summed lowest power first, with the powers built by repeated products."""
    total, power = 0.0, 1.0
    for c in values:
        total += c * power
        power *= v
    return total


def _discount_from_zero(guess: float) -> float:
    """The fallback's start in the engine: v = 0 (r = +inf), whatever the guess."""
    del guess
    return 0.0


def _discount_from_one(guess: float) -> float:
    """A start at v = 1 (r = 0), whatever the guess."""
    del guess
    return 1.0


def _discount_from_guess(guess: float) -> float:
    """A start at the guess's discount factor 1/(1 + guess), as the secant's."""
    return X87.div(1.0, 1 + guess)


def irr_fallback_newton(
    npv: NpvV = _npv_v,
    tol: float = 1e-7,
    stop: str = "max",
    *,
    return_next: bool = True,
    start: Callable[[float], float] = _discount_from_zero,
    residual: float | None = None,
) -> Callable[..., float]:
    """IRR's fallback (financial.md "IRR's fallback when the secant fails", T-2727): Newton on
    P(v) = sum c_i v^i over the flows without their leading zeros, from `start(guess)` (v = 0 in
    the engine), the derivative by Horner. It stops when |next - v| < tol times max(|next|, 1)
    (`stop` "max", the engine), |next| ("rel") or 1 ("abs"), and |P(next)| < `residual` when one is
    given, and returns 1/next - 1 (`return_next` False: 1/v - 1). The root must be positive with
    no sign change of P on [-v, 0), as in the engine (T-2727's selection rule, verified)."""

    def call(flows: Arg, guess: Arg = 0.1) -> float:
        values = floats(flows)
        values = values[next((i for i, c in enumerate(values) if c != 0), len(values)) :]
        if not values:
            raise NoResultError("no flow")
        v = start(numeric_arg(guess))
        for _ in range(FALLBACK_STEPS):
            dp = _dnpv_v(values, v)
            if dp == 0:
                raise NoResultError("flat polynomial")
            nxt = v - npv(values, v) / dp
            if not math.isfinite(nxt):
                raise NoResultError("iterate is not finite")
            scale = {"max": max(abs(nxt), 1.0), "rel": abs(nxt), "abs": 1.0}[stop]
            if abs(nxt - v) < tol * scale and (residual is None or abs(npv(values, nxt)) < residual):
                root = nxt if return_next else v
                nearer_negative = (npv(values, -nxt) > 0) != (values[0] > 0)
                if root <= 0 or nearer_negative:
                    raise NoResultError("the root nearest v = 0 is not positive")
                return X87.div(1.0, root) - 1
            v = nxt
        raise NoResultError(f"no convergence in {FALLBACK_STEPS} steps")

    return call


def irr_fallback_newton_r(start_from_guess: bool, *, x: bool = False) -> Callable[..., float]:
    """Newton on NPV(r) (in x = 1 + r when `x`) to convergence, from the guess or from 0."""
    method = irr_newton_x if x else irr_newton_r

    def call(flows: Arg, guess: Arg = 0.1) -> float:
        return method(floats(flows), numeric_arg(guess) if start_from_guess else 0.0, None)

    return call


def irr_fallback_candidates() -> list[Candidate]:
    """The T-2928 candidates for the fallback's returned digits, each run as the fallback alone (no
    secant first). None beats the engine's 72 bit-exact of 212 cells (attempt 1; financial.md
    "IRR's fallback when the secant fails", Fit)."""
    newtons: dict[str, Callable[..., float]] = {
        "Newton in v from 0, fitted (the engine)": irr_fallback_newton(),
        "Newton in v from 0, returns v": irr_fallback_newton(return_next=False),
        "Newton in v from 0, |dv| < 1e-7": irr_fallback_newton(stop="abs"),
        "Newton in v from 0, |dv| < 1e-8": irr_fallback_newton(tol=1e-8, stop="abs"),
        "Newton in v from 0, |dv| < 1e-10": irr_fallback_newton(tol=1e-10, stop="abs"),
        "Newton in v from 0, |dv| < 1e-5": irr_fallback_newton(tol=1e-5, stop="abs"),
        "Newton in v from 0, |dv| < 1e-7 |v|": irr_fallback_newton(stop="rel"),
        "Newton in v from 0, |dv| < 1e-7 |v|, returns v": irr_fallback_newton(stop="rel", return_next=False),
        "Newton in v from 0, and |P| < 1e-7": irr_fallback_newton(residual=1e-7),
        "Newton in v from 0, P ascending": irr_fallback_newton(npv=_npv_v_ascending),
        "Newton in v from 1": irr_fallback_newton(start=_discount_from_one),
        "Newton in v from the guess": irr_fallback_newton(start=_discount_from_guess),
        "Newton in r from the guess": irr_fallback_newton_r(start_from_guess=True),
        "Newton in r from 0": irr_fallback_newton_r(start_from_guess=False),
        "Newton in x = 1 + r from the guess": irr_fallback_newton_r(start_from_guess=True, x=True),
    }
    out = [candidate(name, fn, "Newton") for name, fn in newtons.items()]
    secants: dict[str, IrrMethod] = {
        "secant in v from the guess, fitted, 1000 steps": irr_secant_v(max_steps=1000),
        "secant in v from the guess, by Horner, 1000 steps": irr_secant_v(_npv_v, max_steps=1000),
    }
    out += [candidate(name, _irr(method, None), "secant in v") for name, method in secants.items()]
    return out


FITS = [
    Fit(
        name="irr-fallback",
        description="IRR flows where both secant runs fail (T-2927 sweep, irr-fallback-roots): the fallback's digits",
        cases=FALLBACK_CASES,
        pattern=r"^=IRR\((.*)\)$",
        candidates=irr_fallback_candidates,
        where=_literal_irr,
        errors=True,
    ),
]
