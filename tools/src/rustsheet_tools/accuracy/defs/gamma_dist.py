"""The gamma and chi-squared distributions: GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT (Gate A slice 4p, T-3925).

The math, at the double of each written literal and the working precision, with y = x / beta (x / 2 for the
chi-squared functions, whose shape is df / 2; Abramowitz and Stegun 26.4.19):
- the cdf P(a, y) and CHISQ.DIST.RT's Q(a, y) are gamma.py's `lower_tail` and `upper_tail` (the series, continued
  fraction and Temme's expansion behind GAMMA.INV and the chi-squared quantiles; nothing is copied);
- the density y^(a-1) e^-y / (Gamma(a) beta), in log space (Abramowitz and Stegun 6.5.1, 26.1.32).

Sources:
- statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)":
  `CHISQ.DIST(x,k,TRUE)` = `GAMMA.DIST(x/2,k/2,1,TRUE)` = `GAMMA.DIST(x,k/2,2,TRUE)` and
  `GAMMA.DIST(x,a,b,TRUE)` = `GAMMA.DIST(x/b,a,1,TRUE)`: one P(a, y) and Q(a, y).
- statistics.md, "GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)": the density of a positive x is
  `#NUM!` where x / beta (in doubles) is subnormal; an x / beta that overflows is `#NUM!` for both forms at every
  probed shape, 3.7E-188 to 1E300 (probes/incomplete-gamma-cap E6, E18 at shape 1; probes/t4625-gamma-dist-ratio-overflow
  rows 1-8 at 0.01, 0.5, 2, 10, 1E5, 1E10 and 1E300, T-4625; Gate A tables GAMMADIST/s1 A182 at 3.7490322860808E-188,
  A118 and GAMMA.DIST/s1 A141). `_gamma_dist_domain` applies it in that shape range
  (`OVERFLOW_SHAPES`); outside it the point has no value. The same probe found an unexplained `#NUM!` at a finite
  x / beta of 1E308 at shapes 1.5 and 2 (B9, F9, B10, B11), not at 0.5, 1, 2.5, 2.9, 3, 10 or 1E5, and not at shape 2
  from 1E200 down (F10-I11). T-4635 explains it (statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"):
  above shape 1 a finite x / beta is `#NUM!` where y / shape * 4 overflows, and to shape 1.1 also where y^shape does,
  in the chi-squared functions too (`_huge_ratio_domain`; the unprobed edges of `HUGE_RATIO_*` have no value).
- statistics.md, "GAMMA.DIST at a huge shape (T-2435, T-3316)": rule 1, the density of a positive x is `#NUM!` where
  shape / x overflows a double; rule 2, from a shape of about 1E+55 a computed 0 is `#NUM!` at some points and not
  others, with no threshold (decision 0267 accepts those cells), so such a point has no math value.
- statistics.md, "Incomplete gamma: accurate kernel (T-0971)", the T-4308 finding: past the 2 pi shape overflow (a
  shape above about 2.9E307) the density at the mode is `#DIV/0!` (`GAMMA.DIST(a,a,1,FALSE)` at 5E307, 9E307 and
  9.99999999999999E307, a number at 2E307); there it is a domain rule, and off the mode, unprobed, no value.
- statistics.md, "Thin distribution functions: argument coercion, truncation and error edges (T-1049)", "Density at 0":
  `CHISQ.DIST(0,1,FALSE)` is `#NUM!` (infinite), the df 2 value is 0.5 and higher df give 0, as the math does;
  `CHISQ.DIST.RT(0,..)` is 1. GAMMA.DIST at x = 0 is 0 above shape 1 (`GAMMA.DIST(0,9,2,FALSE)`, Excel's GAMMA.xlsx
  E13); at shape 1 and below it is unprobed for GAMMA.DIST (no value), while CHISQ.DIST's df 2 (shape 1) is its math, 0.5.
- statistics.md, "GAMMA.INV and CHISQ.INV at a huge shape (T-2982, T-3070)": CHISQ.DIST and CHISQ.DIST.RT are
  `#NUM!` above 1E10 degrees of freedom; "Incomplete gamma: accurate kernel (T-0971)": CHISQ.DIST.RT truncates its
  df (`CHISQ.DIST.RT(8,3.34)` is Q(1.5, 4), Excel's CHISQ.xlsx G3). No entry shows CHISQ.DIST truncating, so an eval
  cell giving it a non-integer df gets no math value (`Param.truncates`).

What has no math value (the sampler redraws; the Excel cells stay in the eval suites):
- GAMMA.DIST's cdf at a positive x whose x / beta is subnormal in doubles, where the math is above 1E-300. Where the
  ratio underflows to 0, Excel gives 0 at every probed shape (statistics.md line "Where x / beta underflows to 0",
  T-2982: probes/t2982-gamma-inv-huge-shape A117-A122), against a math value up to about 1; the owner ruled to keep
  Excel's (decision 0300, T-4392), and the math is not cut to match. A nonzero subnormal ratio at shape 0.5 gives 0 in Excel
  (probes/t1942-ztest-numeric-edges A6:A7, T-1942); other shapes below 1 are unprobed. CHISQ.DIST's cdf at a positive x whose x / 2 is subnormal gets the same rule: it is
  `GAMMA.DIST(x,df/2,2,TRUE)` (KERNEL_SOURCE), and no cell probes it.
- GAMMA.DIST's density at x = 0 with shape 1: the math is 1 / beta, but only the chi-squared df-2 value (0.5) is
  evidenced, so GAMMA.DIST there gets no value.
- The chi-squared densities of a positive x whose x / 2 is subnormal or whose (df / 2) / x overflows: unprobed.
- Ill-conditioned points, where the one rounding of y that Excel's x / beta makes (none where the quotient is a
  double, as x / 2 always is above the subnormals) moves the value by more than `CONDITION_LIMIT` relative:
  kappa = y g(y) / P (or / Q) for a cdf, |a - 1 - y| for a density.
- Shapes from `HUGE_SHAPE_NUM` where the value is below the smallest normal double (rule 2 above), and huge-shape
  points Temme's expansion does not reach (gamma.py `TEMME_ETA_MAX`).
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT, NORMAL_MIN, ill_conditioned, is_real, real_arg
from rustsheet_tools.accuracy.defs.gamma import (
    CHI_SCALE,
    HUGE_SOURCE,
    chi_shape,
    deg_freedom_param,
    lower_tail,
    quantile,
    upper_tail,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

KERNEL_SOURCE = (
    "statistics.md#one-incomplete-gamma-kernel-under-erf-erfc-norm-chisq-gammadist-and-poisson-t-1778"
)
SUBNORMAL_SOURCE = "statistics.md#gammadists-density-is-num-at-a-subnormal-x--beta-t-2738"
HUGE_SHAPE_SOURCE = "statistics.md#gammadist-at-a-huge-shape-num-where-shapex-overflows-an-unexplained-num-from-about-1e56-t-2435-t-3316"
ZERO_SOURCE = "statistics.md#thin-distribution-functions-argument-coercion-truncation-and-error-edges-t-1049"
RT_TRUNC_SOURCE = "statistics.md#incomplete-gamma-accurate-kernel-t-0971"
MATH_SOURCE = "Abramowitz and Stegun 6.5.1, 26.1.32, 26.4.19"

TWO_PI = 2 * math.pi
FLOOR = mpmath.mpf("1e-300")  # the envelope's absolute floor (SPEC-0029)
# Rule 2 of HUGE_SHAPE_SOURCE: Excel's unexplained #NUM! for a computed 0 starts between shapes 1E+55 and 1E+56.
HUGE_SHAPE_NUM = 1e55
# The largest shape drawn: Temme's expansion (gamma.py) is summed at about log10(a) more digits, and the chi-squared
# functions stop at df 1E10 (shape 5E9).
MAX_DRAWN_DIST_SHAPE = 1e10
# Below e^this a value is 0 to the envelope's 1E-300 floor, and is recorded as 0: summing e^(-1E+67) at 60 and 80
# digits gives values that differ in every digit, and the generator would drop the point. A tail is at most its
# y g(y) times 1 / |a - y| from a distance of 1 from the mode, so it is below e^this where y g(y) is (and its
# complement is 1 to the working precision).
DEEP_TAIL_LOG = -2000
# T-4625: the shapes at which an overflowing x / beta is #NUM! in Excel's cells (probes/t4625-gamma-dist-ratio-overflow;
# the low end is GAMMADIST/s1 A182).
OVERFLOW_SHAPES = (3.7490322860808e-188, 1e300)
# T-4635 (statistics.md "GAMMA.DIST at a huge finite x / beta"): above shape 1 a finite y is #NUM! where y / a * 4
# overflows, and up to HUGE_RATIO_POWER_SHAPE_MAX also where y^a does. Shape 1.11 is a number where y^a overflows, so
# a shape strictly between the two has no value there; so has a shape below 1.01 where only y / a * 4 overflows (that
# branch is probed only where y^a overflows first).
HUGE_RATIO_POWER_SHAPE_MAX = 1.1
HUGE_RATIO_POWER_NUMBER_SHAPE = 1.11
HUGE_RATIO_QUOTIENT_SHAPE_MIN = 1.01
DENSITY_GUARD_DPS = 10  # digits added to the density's exponent, besides the log10(a) it cancels


def _floats(*args: MathArg) -> list[float]:
    return [float(real_arg(a)) for a in args]


def log_density_y(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """ln(y^a e^-y / Gamma(a)), y > 0: ln of y times the unit-scale density, at enough digits that the cancelling
    a ln y - y - ln Gamma(a) keeps the working precision."""
    with mpmath.workdps(mpmath.mp.dps + DENSITY_GUARD_DPS + max(0, int(mpmath.log10(a)))):
        return +(a * mpmath.log(y) - y - mpmath.loggamma(a))


def _rounded(y: mpmath.mpf) -> bool:
    """Whether y = x / scale is not a double, so Excel's division rounds it."""
    return bool(mpmath.mpf(float(y)) != y)


def _cdf(y: mpmath.mpf, a: mpmath.mpf, upper: bool) -> mpmath.mpf:
    """P(a, y) or Q(a, y), redrawn where a rounded y makes the point ill-conditioned."""
    if y > 0 and abs(y - a) > 1 and log_density_y(y, a) < DEEP_TAIL_LOG:
        # the far side of the mode from the asked tail is 1, the near side 0
        return mpmath.mpf(1 if (y < a) == upper else 0)
    value = upper_tail(y, a) if upper else lower_tail(y, a)
    if y > 0 and value > FLOOR and _rounded(y):
        ill_conditioned(float(mpmath.exp(log_density_y(y, a)) / value))
    return value


def _density(y: mpmath.mpf, a: mpmath.mpf, scale: mpmath.mpf) -> mpmath.mpf:
    """The density at x = y * scale: 0 at y = 0 above shape 1, 1 / scale at shape 1; NoValueError below 1 there."""
    if y == 0:
        if a > 1:
            return mpmath.mpf(0)
        if a == 1:
            return 1 / scale
        msg = "the density at 0 below shape 1 is unbounded (unprobed for GAMMA.DIST)"
        raise NoValueError(msg)
    log_value = log_density_y(y, a) - mpmath.log(y * scale)
    if log_value < DEEP_TAIL_LOG:
        return mpmath.mpf(0)
    value = mpmath.exp(log_value)
    if value > FLOOR and _rounded(y):
        ill_conditioned(float(abs(a - 1 - y)))
    return value


def _subnormal_ratio_cdf(fx: float, ratio: float, value: mpmath.mpf) -> None:
    """NoValueError for a cdf above the floor at a positive x whose ratio (x / beta, or x / 2) is subnormal or 0."""
    if fx > 0 and ratio < NORMAL_MIN and value > FLOOR:
        msg = "a subnormal x / scale: Excel's 0 where it underflows (T-2982; Excel kept, decision 0300), unprobed otherwise"
        raise NoValueError(msg)


def _huge_shape_zero(a: float, value: mpmath.mpf) -> None:
    if a >= HUGE_SHAPE_NUM and value < NORMAL_MIN:
        msg = "Excel's computed 0 from a shape of 1E+55 is #NUM! at some points (rule 2, T-3316)"
        raise NoValueError(msg)


def _gamma_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, a, beta = (real_arg(v) for v in args[:3])
    cumulative = args[3]
    fx, fa, fbeta = _floats(x, a, beta)
    y = x / beta
    if not cumulative and y == 0 and a == 1:
        msg = "GAMMA.DIST's density at 0 with shape 1 is unprobed (only CHISQ.DIST's df 2 is evidenced)"
        raise NoValueError(msg)
    value = _cdf(y, a, upper=False) if cumulative else _density(y, a, beta)
    if cumulative:
        _subnormal_ratio_cdf(fx, fx / fbeta, value)
    _huge_shape_zero(fa, value)
    return value


def _gamma_dist_domain(args: Sequence[MathArg]) -> str | None:
    x, a, beta, cumulative = args
    if any(not is_real(v) for v in (x, a, beta)):
        return None
    fx, fa, fbeta = _floats(x, a, beta)
    if fx <= 0 or fbeta <= 0:
        return None
    ratio = fx / fbeta
    if math.isinf(ratio):
        if not OVERFLOW_SHAPES[0] <= fa <= OVERFLOW_SHAPES[1]:
            msg = "an overflowing x / beta is probed #NUM! only at shapes 3.7E-188 to 1E300 (T-4625)"
            raise NoValueError(msg)
        return "x / beta overflows"
    huge = _huge_ratio_domain(fa, ratio)
    if huge is not None:
        return huge
    return None if cumulative else _density_domain(fx, fa, ratio)


def power_overflows(y: float, a: float) -> bool:
    """Whether the double y^a overflows (the engine's `powf`)."""
    try:
        return math.isinf(y**a)
    except OverflowError:
        return True


def _huge_ratio_domain(fa: float, y: float) -> str | None:
    """Excel's #NUM! at a huge finite y = x / beta (T-4635), shared by GAMMA.DIST and the chi-squared functions."""
    if fa <= 1:
        return None
    power = power_overflows(y, fa)
    quotient = math.isinf(y / fa * 4)
    if power and HUGE_RATIO_POWER_SHAPE_MAX < fa < HUGE_RATIO_POWER_NUMBER_SHAPE and not quotient:
        msg = "y^shape overflows between shapes 1.1 (#NUM!) and 1.11 (a number): unprobed (T-4635)"
        raise NoValueError(msg)
    if quotient and not power and fa < HUGE_RATIO_QUOTIENT_SHAPE_MIN:
        msg = "y / shape * 4 overflows below shape 1.01 where y^shape does not: unprobed (T-4635)"
        raise NoValueError(msg)
    if quotient or (power and fa <= HUGE_RATIO_POWER_SHAPE_MAX):
        return "a huge finite x / beta (T-4635)"
    return None


# The aimed draw of T-4635's investigation batch (`generate GAMMA.DIST --batch e-T-4635 --stratum aimed`): y within 10% of
# rule R's cut, x below 1E301 (Excel for the web refuses a workbook holding an x literal of 1E307 or more, T-4635's
# first fill), so beta is 1E-8 or smaller.
HUGE_RATIO_AIMED_SPREAD = 1.1
HUGE_RATIO_AIMED_BETA = (1e-100, 1e-8)


def _huge_ratio_cut(a: float) -> float:
    """The smallest y rule R makes #NUM! at shape `a` in (1, 4): y / a * 4 overflows, or (to 1.1) y^a does."""
    cut = sys.float_info.max / 4 * a
    if a <= HUGE_RATIO_POWER_SHAPE_MAX:
        cut = min(cut, sys.float_info.max ** (1 / a))
    return cut


def _huge_ratio_point(rng: random.Random) -> list[Arg]:
    """GAMMA.DIST near rule R's cut (T-4635), drawn after the rule froze: shapes 1.01 to 1.1 (the power branch) one
    time in four, 1.11 to 4 (the quotient) one in two, y log-uniform within 10% of the cut, either side; shapes 0.3
    to 1 and 4 to 10 (never #NUM!) one in eight each, y log-uniform from 1E300 to the largest double."""
    pick = rng.random()
    largest = sys.float_info.max
    if pick < 0.125:
        a, y = rng.uniform(0.3, 1.0), log_uniform(rng, 1e300, largest)
    elif pick < 0.25:
        a, y = rng.uniform(4.0, 10.0), log_uniform(rng, 1e300, largest)
    else:
        a = (
            rng.uniform(HUGE_RATIO_QUOTIENT_SHAPE_MIN, HUGE_RATIO_POWER_SHAPE_MAX)
            if pick < 0.5
            else rng.uniform(HUGE_RATIO_POWER_NUMBER_SHAPE, 4.0)
        )
        cut = _huge_ratio_cut(a)
        y = min(largest, log_uniform(rng, cut / HUGE_RATIO_AIMED_SPREAD, cut * HUGE_RATIO_AIMED_SPREAD))
    beta = log_uniform(rng, *HUGE_RATIO_AIMED_BETA)
    return [y * beta, a, beta, rng.random() < 0.5]


def _density_domain(fx: float, fa: float, ratio: float) -> str | None:
    """GAMMA.DIST's density-only error rules at a positive x (T-2738, T-3316 rule 1, T-4308)."""
    if ratio < NORMAL_MIN:
        return "the density at a subnormal x / beta"
    if math.isinf(fa / fx):
        return "the density where shape / x overflows"
    if math.isinf(TWO_PI * fa):
        if ratio == fa:
            return "the density at the mode where 2 pi shape overflows (#DIV/0!)"
        msg = "the density where 2 pi shape overflows is probed only at the mode (T-4308)"
        raise NoValueError(msg)
    return None


def _gamma_tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    a, beta = args[1], args[2]
    if not is_real(a) or not is_real(beta):
        msg = "no tail map for a logical shape or scale"
        raise ValueError(msg)
    return beta * quantile(q, a, upper=upper)


def _chisq_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df, cumulative = real_arg(args[0]), args[1], args[2]
    a = chi_shape(df)
    y = x / CHI_SCALE
    fx = float(x)
    if cumulative:
        value = _cdf(y, a, upper=False)
        _subnormal_ratio_cdf(fx, fx / 2, value)
        return value
    if fx > 0 and (fx / 2 < NORMAL_MIN or math.isinf(float(a) / fx)):
        msg = "a chi-squared density at a subnormal x / 2 or an overflowing (df / 2) / x is unprobed"
        raise NoValueError(msg)
    return _density(y, a, CHI_SCALE)


def _chisq_dist_domain(args: Sequence[MathArg]) -> str | None:
    x, df, cumulative = args
    if not is_real(x) or not is_real(df):
        return None
    if not cumulative and x == 0 and chi_shape(df) < 1:
        return "the density at 0 with one degree of freedom is infinite"
    return _chisq_huge_domain(x, df)


def _chisq_huge_domain(x: MathArg, df: MathArg) -> str | None:
    """T-4635's rule at y = x / 2 (probes/t4635-chisq-huge-x: df 3 from x = 1.4E308)."""
    if not is_real(x) or not is_real(df) or float(real_arg(x)) <= 0:
        return None
    return _huge_ratio_domain(float(chi_shape(df)), float(real_arg(x)) / 2)


def _chisq_dist_rt_domain(args: Sequence[MathArg]) -> str | None:
    return _chisq_huge_domain(args[0], args[1])


def _chisq_dist_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df = args
    return _cdf(real_arg(x) / CHI_SCALE, chi_shape(df), upper=True)


def _chi_tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    df = args[1]
    if not is_real(df):
        msg = "no tail map for a logical df"
        raise ValueError(msg)
    return CHI_SCALE * quantile(q, chi_shape(df), upper=upper)


def _x(typical: tuple[float, float], source: str) -> Param:
    return Param(
        "x",
        "real",
        Interval(0, float("inf"), lo_open=False),
        typical=typical,
        extreme=(1e-300, 1e300),
        source=source,
    )


def _cumulative() -> Param:
    return Param("cumulative", "bool", source="catalog")


GAMMA_DIST = register(
    Definition(
        function="GAMMA.DIST",
        params=(
            _x((0.01, 30), KERNEL_SOURCE),
            Param(
                "alpha",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 50),
                extreme=(1e-3, MAX_DRAWN_DIST_SHAPE),
                edges=(1.0,),
                source=HUGE_SHAPE_SOURCE,
                draw_hi=MAX_DRAWN_DIST_SHAPE,
            ),
            Param(
                "beta",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 10),
                extreme=(1e-100, 1e100),
                source=SUBNORMAL_SOURCE,
            ),
            _cumulative(),
        ),
        value=_gamma_dist,
        domain=_gamma_dist_domain,
        aimed=_huge_ratio_point,
        tail=_gamma_tail_x,
        tail_param="x",
        sources=(
            KERNEL_SOURCE,
            SUBNORMAL_SOURCE,
            HUGE_SHAPE_SOURCE,
            ZERO_SOURCE,
            RT_TRUNC_SOURCE,
            MATH_SOURCE,
        ),
    )
)

CHISQ_DIST = register(
    Definition(
        function="CHISQ.DIST",
        params=(_x((0.01, 100), ZERO_SOURCE), deg_freedom_param(DF_CUT), _cumulative()),
        value=_chisq_dist,
        domain=_chisq_dist_domain,
        tail=_chi_tail_x,
        tail_param="x",
        sources=(KERNEL_SOURCE, ZERO_SOURCE, HUGE_SOURCE, MATH_SOURCE),
    )
)

CHISQ_DIST_RT = register(
    Definition(
        function="CHISQ.DIST.RT",
        params=(_x((0.01, 100), ZERO_SOURCE), deg_freedom_param(DF_CUT, RT_TRUNC_SOURCE)),
        value=_chisq_dist_rt,
        domain=_chisq_dist_rt_domain,
        tail=_chi_tail_x,
        tail_param="x",
        sources=(KERNEL_SOURCE, ZERO_SOURCE, HUGE_SOURCE, RT_TRUNC_SOURCE, MATH_SOURCE),
    )
)
