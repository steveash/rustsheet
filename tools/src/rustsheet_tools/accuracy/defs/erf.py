"""ERF, ERF.PRECISE, ERFC and ERFC.PRECISE (Gate A slice 4n, T-3784).

The math: mpmath `erf` and `erfc` of the double of each written literal at the working precision;
the two-argument `ERF(a, b)` is erf(b) - erf(a). Every real is in the domain: no Excel error is
evidenced for a number argument (the engine has none either).

Sources:
- statistics.md, "Normal CDF and ERFC sweep (T-1040)": on its grid (x step 0.125) ERFC and ERFC.PRECISE are
  bit-identical and within 1e-15 of the correctly rounded erfc wherever the result is a normal double (off the grid
  the rounding of x*x in Excel's EXP(-fl(x*x)) shows: `ERFC.PRECISE(26.5432)` is 4.05E-14 off); from x = 26.625 on
  they are 0, where the true
  value is subnormal (the last nonzero is `ERFC(26.5)` = 2.2109076642637346E-307). The tail stratum draws that
  region on purpose: Gate A's absolute floor 1e-300 (decision 0188 R2; numbers.md "Subnormal results become 0")
  puts a flushed 0 and a kept subnormal inside the envelope alike, so those points are drawn, not left out.
- statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)":
  - ERF is odd, ERF(d) = fl(1 - ERFC(d)) and ERFC(-d) = fl(2 - ERFC(d)) for d >= 0.5; the small-x kernel runs up to
    x = 0.5 itself and the ERFC kernel starts at 0.5 + 2^-53, so the edge stratum draws 0.5 and its neighbours.
  - Tiny x keeps the leading term: `ERF.PRECISE(2^-k)*2^k` = 1.1283791670955128 for k from 27 to 79.
  - Two-argument `ERF(a,b)` is the exact difference of the two rounded single values (research probe
    `kmyst-erf-twoarg`; eval cells in probes-numeric `t1916-erf-wrappers`).
- engineering.md: `ERF(5.9)` is 0.9999999999999999 and `ERF(6)` is 1; numeric text such as `ERF("0.5")` is
  accepted. A table writes number literals only, so the text forms stay in the eval suites.

The two-argument cancellation region (decision 0188 R3). Because Excel subtracts two rounded values, its error
relative to erf(b) - erf(a) is up to about 2^-52 * max(|erf a|, |erf b|) / |erf(b) - erf(a)|: where the two values
nearly cancel (b a few ulps above a, or both deep in the same tail), Excel can be more than 1e-4 off the math: 602
of the 915 two-argument cells of probes-numeric `t1916-erf-wrappers` are, up to a relative 3.28 with the opposite
sign (`ERF(0.5000000000000002,0.5000000000000003)` is -2.220446049250313E-16 against 9.756E-17). A milder deep-tail
pair such as `ERF(5,6)` is only 9.3E-7 off but still inside the excluded region below. Those are R3 candidates, and their ruling is the owner's (T-3794). The table leaves the region out,
so the gate does not depend on that ruling: a two-argument point is redrawn while |erf(b) - erf(a)| < `CANCEL` *
max(|erf a|, |erf b|), which bounds Excel's subtraction error near 2.2E-7 relative, far inside R3's 1e-4. The
region's Excel cells stay in `t1916-erf-wrappers`, so no behaviour goes unchecked; they are just not Gate A
reference points.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

SWEEP_SOURCE = "statistics.md#normal-cdf-and-erfc-sweep-t-1040"
KERNEL_SOURCE = (
    "statistics.md#one-incomplete-gamma-kernel-under-erf-erfc-norm-chisq-gammadist-and-poisson-t-1778"
)
TEXT_SOURCE = "engineering.md (ERF(5.9), ERF(6); numeric text accepted)"
FLUSH = 26.625  # ERFC is 0 from here on (the true value is subnormal)
SPLIT = 0.5  # the small-x kernel takes x <= 0.5, the ERFC kernel x > 0.5
CANCEL = 1e-9  # a two-argument point whose difference is below this share of its terms is redrawn
TWO_ARG_SHARE = 0.5  # the share of ERF's points that pass upper_limit
CANCEL_REDRAWS = 50
ASYMPTOTIC = mpmath.mpf(10) ** 4  # erfc takes the asymptotic series from here on
SERIES_TERMS = 200  # a cap only: from 1E+4 at dps 80 the series needs under 20 terms


def _signed(rng: random.Random, x: float) -> float:
    return -x if rng.random() < 0.5 else x


def _edge(rng: random.Random) -> float:
    """A point at an evidenced break: 0, the kernel split, tiny x, the flush, or the ERF(6) step."""
    shape = rng.randrange(6)
    if shape == 0:
        return 0.0
    if shape == 1:  # the kernel split and its neighbours
        return _signed(rng, rng.choice((SPLIT, math.nextafter(SPLIT, 0.0), math.nextafter(SPLIT, 1.0))))
    if shape == 2:  # tiny x: the leading term 2x/sqrt(pi)
        return _signed(rng, rng.choice((2.0**-27, 2.0**-53, 2.0**-79, log_uniform(rng, 1e-300, 1e-8))))
    if shape == 3:  # around the subnormal flush of ERFC
        return _signed(rng, rng.choice((26.5, 26.5432, FLUSH, rng.uniform(26.0, 27.5))))
    if shape == 4:  # where ERF reaches 1
        return _signed(rng, rng.choice((5.9, 6.0, rng.uniform(5.5, 6.5))))
    return _signed(rng, rng.choice((1.0, 2.0, 4.0, 8.0)))


def _one(stratum: str, rng: random.Random) -> float:
    """One argument of `stratum`."""
    if stratum == "tail":  # erf near ±1, erfc deep in its tail up to the flush
        return _signed(rng, rng.uniform(2.0, 27.5))
    if stratum == "edge":
        return _edge(rng)
    if stratum == "extreme":  # erf is ±1, erfc 0 or 2
        return _signed(rng, log_uniform(rng, 27.5, 1e300))
    if stratum == "log-uniform":
        return _signed(rng, log_uniform(rng, 1e-300, 1e300))
    return rng.uniform(-3.0, 3.0)


def cancels(a: float, b: float) -> bool:
    """Whether erf(b) - erf(a) is below `CANCEL` of its larger term (the excluded region)."""
    with mpmath.workdps(30):
        ea, eb = _erf(mpmath.mpf(a)), _erf(mpmath.mpf(b))
        return bool(abs(eb - ea) < CANCEL * max(abs(ea), abs(eb)))


def _draw_one(stratum: str, rng: random.Random) -> list[Arg]:
    return [_one(stratum, rng)]


def _draw_erf(stratum: str, rng: random.Random) -> list[Arg]:
    """A one-argument point, or (half the time) a two-argument one outside the cancellation region; the upper
    limit comes from the same stratum, or near the lower one to reach just short of the cancellation cut."""
    if rng.random() >= TWO_ARG_SHARE:
        return _draw_one(stratum, rng)
    for _ in range(CANCEL_REDRAWS):
        a = _one(stratum, rng)
        b = (
            _one(stratum, rng)
            if rng.random() < 0.7
            else a + _signed(rng, max(abs(a), 1.0) * 10 ** -rng.uniform(1, 8))
        )
        a, b = (
            float(f"{a:.15g}"),
            float(f"{b:.15g}"),
        )  # the literals as written, so the cut sees what Excel gets
        if not cancels(a, b):
            return [a, b]
    return [_one(stratum, rng)]


def erfc(x: mpmath.mpf) -> mpmath.mpf:
    """erfc at the working precision. mpmath's own `erfc` overflows a float in its series check from about
    x = 1E+154, so from `ASYMPTOTIC` on the value is the asymptotic series exp(-x^2)/(x sqrt(pi)) *
    sum (-1)^k (2k-1)!! / (2x^2)^k (A&S 7.1.23), summed until a term is below the working precision; a negative
    x is 2 - erfc(-x)."""
    if x < 0:
        return 2 - erfc(-x)
    if x < ASYMPTOTIC:
        return mpmath.erfc(x)
    step = 1 / (2 * x * x)
    term, total, k = mpmath.mpf(1), mpmath.mpf(1), 1
    while abs(term) > mpmath.eps and k <= SERIES_TERMS:
        term *= -(2 * k - 1) * step
        total += term
        k += 1
    return mpmath.exp(-x * x) / (x * mpmath.sqrt(mpmath.pi)) * total


def _erf(x: mpmath.mpf) -> mpmath.mpf:
    """erf at the working precision, as 1 - erfc where mpmath's own erf would meet the same overflow."""
    return mpmath.erf(x) if abs(x) < ASYMPTOTIC else mpmath.sign(x) * (1 - erfc(abs(x)))


def _definition(
    function: str,
    kernel: Callable[[mpmath.mpf], mpmath.mpf],
    sources: tuple[str, ...],
) -> Definition:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (x,) = scalars(args)
        return kernel(x)

    return register(
        Definition(
            function=function,
            params=(Param("x", "real", source=sources[0]),),
            value=value,
            draw=_draw_one,
            sources=sources,
        )
    )


def _erf_value(args: Sequence[MathArg]) -> mpmath.mpf:
    values = scalars(args)
    if len(values) == 1:
        return _erf(values[0])
    lower, upper = values
    return _erf(upper) - _erf(lower)


ERF = register(
    Definition(
        function="ERF",
        params=(
            Param("lower_limit", "real", source=KERNEL_SOURCE),
            Param("upper_limit", "real", source=KERNEL_SOURCE, optional=True),
        ),
        value=_erf_value,
        draw=_draw_erf,
        sources=(KERNEL_SOURCE, SWEEP_SOURCE, TEXT_SOURCE),
    )
)
ERF_PRECISE = _definition("ERF.PRECISE", _erf, (KERNEL_SOURCE, SWEEP_SOURCE))
ERFC = _definition("ERFC", erfc, (SWEEP_SOURCE, KERNEL_SOURCE))
ERFC_PRECISE = _definition("ERFC.PRECISE", erfc, (SWEEP_SOURCE, KERNEL_SOURCE))
