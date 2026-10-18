"""ASIN over probes/asin-argument-sweep and probes/asin-negative-band-sweep (T-1259, T-1605, T-1797).

Excel's ASIN is not odd: ± pairs differ in the last digit, and the negative band -0.95..-0.80 at a
0.001 step fails the symmetric formulations at every argument whose last significand bit is 1.
Rounding y = 1 - x drops that bit, so the winner forms the factor 1 + x as 2 - y:
`atan(x / sqrt(y * (2 - y)))`, with the square root rounded twice like Excel's SQRT. Results:
docs/excel-semantics/numbers.md, "ASIN's formulation", and T-1797's Outcome.
"""

from __future__ import annotations

import math
from collections.abc import Callable

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

Kernel = Callable[[float], float]
_IDENTITY_BOUND = 2.0**-28  # the engine's tiny-argument cut before T-1797 (T-1605)


def _quotient(x: float, radicand: float, sqrt: Kernel) -> float:
    # At ±1 the radicand is 0 and the quotient ±inf, whose atan is ±π/2.
    root = sqrt(radicand)
    return math.copysign(math.inf, x) if root == 0 else x / root


def atan_product(x: float) -> float:
    """atan(x / sqrt((1-x)(1+x))) in doubles (T-1259)."""
    return math.atan(_quotient(x, (1 - x) * (1 + x), math.sqrt))


def atan_product_identity(x: float) -> float:
    """`atan_product` with x itself below 2^-28: the engine from T-1605 to T-1797."""
    return x if abs(x) < _IDENTITY_BOUND else atan_product(x)


def atan_square(x: float) -> float:
    """atan(x / sqrt(1 - x*x))."""
    return math.atan(_quotient(x, 1 - x * x, math.sqrt))


def atan2_product(x: float) -> float:
    """atan2(x, sqrt((1-x)(1+x)))."""
    return math.atan2(x, math.sqrt((1 - x) * (1 + x)))


def half_angle(x: float) -> float:
    """2 atan(x / (1 + sqrt((1-x)(1+x))))."""
    return 2 * math.atan(x / (1 + math.sqrt((1 - x) * (1 + x))))


def half_pi_minus_acos(x: float) -> float:
    """pi/2 - libm acos(x)."""
    return math.pi / 2 - math.acos(x)


def two_minus_y(sqrt: Kernel) -> Kernel:
    """atan(x / sqrt(y (2 - y))) with y = 1 - x, under the given square root."""

    def f(x: float) -> float:
        y = 1 - x
        return math.atan(_quotient(x, y * (2 - y), sqrt))

    return f


def _on_arg(kernel: Kernel) -> Callable[[Arg], float]:
    return lambda a: kernel(numeric_arg(a))


KERNELS: dict[str, Kernel] = {
    "libm asin": math.asin,
    "atan(x/sqrt((1-x)(1+x)))": atan_product,
    "atan(x/sqrt((1-x)(1+x))), x below 2^-28": atan_product_identity,
    "atan(x/sqrt(1-x*x))": atan_square,
    "atan2(x, sqrt((1-x)(1+x)))": atan2_product,
    "2 atan(x/(1+sqrt((1-x)(1+x))))": half_angle,
    "pi/2 - acos(x)": half_pi_minus_acos,
    "atan(x/sqrt(y(2-y))), y=1-x": two_minus_y(math.sqrt),
    "atan(x/sqrt(y(2-y))), y=1-x, sqrt rounded twice": two_minus_y(X87.sqrt),
}


def asin_candidates() -> list[Candidate]:
    return [candidate(name, _on_arg(kernel)) for name, kernel in KERNELS.items()]


FITS = [
    Fit(
        name="asin",
        description="ASIN over asin-argument-sweep and asin-negative-band-sweep (T-1797)",
        cases=["probes/asin-argument-sweep", "probes/asin-negative-band-sweep"],
        pattern=r"^=ASIN\((.*)\)$",
        candidates=asin_candidates,
    ),
]
