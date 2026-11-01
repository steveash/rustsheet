"""SIN, COS, TAN, COT, SEC and CSC (Gate A slice 4a, T-3668).

The math: each function of the double of the written literal, at the working precision (SEC, CSC and COT are 1 over
COS, SIN and TAN).

Sources:
- numbers.md, "Trig of a huge angle":
  - all six are `#NUM!` from |x| = 2^27 on, fractions included, so the domain is the open interval (-2^27, 2^27);
  - Excel reduces the argument by multiples of π₆₄/2, π rounded to a 64-bit significand (the x87 `FLDPI`
    constant), so near a multiple of π/2 Excel can be far from the math relative to a tiny result (`SIN(PI())` is
    1.22514845490862E-16 against the true 1.2246467991473532E-16).
  That shift is the inaccuracy of Excel's constant, not a semantic rule, so the definition stays the true function:
  such points pass on Excel's side of the envelope, where the engine reproduces the π₆₄ model bit for bit
  (T-1563). The tail stratum draws near multiples of π/2 on purpose.
- gen-m5 (excel-graph): `COT(0)` and `CSC(0)` are `#DIV/0!` (Calc!A38, A289; A113, A125) and `SEC(0)` is 1 (A175):
  0 is a domain error of COT and CSC only. No other double is a pole: π/2 and π are irrational.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

HUGE_SOURCE = "numbers.md#trig-of-a-huge-angle"
POLE_SOURCE = "gen-m5 COT(0), CSC(0) #DIV/0! (excel-graph)"
CUT = 2.0**27  # |x| at and above it is #NUM!


def _zero_pole(args: Sequence[MathArg]) -> str | None:
    """COT and CSC at 0: Excel's `#DIV/0!`."""
    (x,) = scalars(args)
    return "a pole at 0" if x == 0 else None


def _edge(sign: float, rng: random.Random) -> float:
    shape = rng.randrange(5)
    if shape == 0:  # just inside the cut
        return sign * (CUT - rng.choice((1.0, 0.5, 0.25)))
    if shape == 1:  # at or past the cut: #NUM!
        return sign * rng.choice((CUT, CUT + 0.25, 1e9, 1e300))
    if shape == 2:
        return 0.0
    if shape == 3:  # tiny arguments
        return sign * log_uniform(rng, 1e-300, 1e-8)
    return sign * rng.choice((math.pi / 4, math.pi / 2, math.pi, 1.0))


def _draw(stratum: str, rng: random.Random) -> list[Arg]:
    sign = -1.0 if rng.random() < 0.5 else 1.0
    if stratum == "tail":  # near k·π/2: where Excel's π₆₄ reduction shows
        k = round(log_uniform(rng, 1, 1e6))
        d = 0.0 if rng.random() < 0.2 else rng.choice((-1.0, 1.0)) * 10 ** -rng.uniform(1, 12)
        x = k * math.pi / 2 + d
    elif stratum == "edge":
        return [_edge(sign, rng)]
    elif stratum == "extreme":
        x = log_uniform(rng, 1e3, CUT * 0.999)
    elif stratum == "log-uniform":
        x = log_uniform(rng, 1e-300, CUT * 0.999)
    else:
        return [rng.uniform(-10, 10)]
    return [sign * x]


def _definition(
    function: str, kernel: Callable[[mpmath.mpf], mpmath.mpf], pole_at_zero: bool = False
) -> Definition:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = scalars(args)
        return kernel(x)

    return register(
        Definition(
            function=function,
            params=(Param("number", "real", Interval(-CUT, CUT), source=HUGE_SOURCE),),
            value=value,
            domain=_zero_pole if pole_at_zero else None,
            draw=_draw,
            sources=(HUGE_SOURCE, POLE_SOURCE) if pole_at_zero else (HUGE_SOURCE,),
        )
    )


SIN = _definition("SIN", mpmath.sin)
COS = _definition("COS", mpmath.cos)
TAN = _definition("TAN", mpmath.tan)
COT = _definition("COT", mpmath.cot, pole_at_zero=True)
SEC = _definition("SEC", mpmath.sec)
CSC = _definition("CSC", mpmath.csc, pole_at_zero=True)
