"""The discrete distributions: BINOM.DIST, NEGBINOM.DIST and POISSON.DIST.

Sources:
- statistics.md, "BINOM.DIST is #NUM! from 2^31 - 1 trials (T-2464, T-2645)": the accepted trial count is n < 2^31 - 1,
  whatever the other arguments; NEGBINOM.DIST has no such limit.
- statistics.md, "BINOM.DIST's pmf at k = 0 is #NUM! when p = 1 (T-2435)": `BINOM.DIST(0,n,1,FALSE)` is `#NUM!` from
  one trial on, while the cdf there is 0.
- statistics.md, "POISSON at x 0 with a negative mean": at x 0 a negative mean gives EXP(-mean), cumulative or not;
  at x 1 or more a negative mean is `#NUM!`.
- statistics.md, "Legacy distribution names sweep, second family (T-1048)": NEGBINOM's probability is `#NUM!` at
  p = 1 and at p = 0 (an open interval), where BINOM.DIST's is closed.
- Microsoft's documentation: number_s > trials, a BINOM.DIST probability outside [0, 1], number_f < 0, number_s < 1
  (NEGBINOM) and x < 0 are `#NUM!`.
- The math: the binomial pmf C(n, k) p^k (1-p)^(n-k) and cdf I_{1-p}(n-k, k+1); the negative binomial pmf
  C(f+s-1, f) p^s (1-p)^f and cdf I_p(s, f+1) (Abramowitz and Stegun 26.5.24, 26.5.26, the regularized incomplete
  beta); the Poisson pmf e^-m m^x / x! and cdf Q(x+1, m), the regularized upper incomplete gamma (A&S 26.4.21).
"""

from __future__ import annotations

from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs.annuity import scalars
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)

TRIALS_SOURCE = "statistics.md#binomdist-is-num-from-231---1-trials-t-2464-t-2645"
P1_SOURCE = "statistics.md#binomdists-pmf-at-k--0-is-num-when-p--1-t-2435"
NEGATIVE_MEAN_SOURCE = "statistics.md#poisson-at-x-0-with-a-negative-mean"
TRIALS_LIMIT = 2**31 - 2  # the largest accepted (truncated) trial count
COUNT_DRAW_HI = 1e12  # the largest count drawn (a cap on the math, not an Excel limit)
_PROBABILITY = Param(
    "probability_s",
    "real",
    Interval(0, 1, lo_open=False, hi_open=False),
    typical=(0.01, 0.99),
    extreme=(1e-300, 1e-6),
    source="Microsoft documentation",
)
NEGBINOM_P_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
# NEGBINOM.DIST's probability is open at both ends: `NEGBINOM.DIST(21,4,0,TRUE)` and `NEGBINOM.DIST(20,1,1,FALSE)`
# are #NUM! in Excel (accuracy/NEGBINOM.DIST/s1 A117 and A119; also A127, A137 at p = 0, A129, A139 at p = 1).
_NEGBINOM_PROBABILITY = Param(
    "probability_s",
    "real",
    Interval(0, 1),
    typical=(0.01, 0.99),
    extreme=(1e-300, 1e-6),
    source=NEGBINOM_P_SOURCE,
)
_CUMULATIVE = Param("cumulative", "bool", source="catalog")


def _reals(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The numeric arguments (every argument but the trailing cumulative flag)."""
    return scalars(args[:-1])


def binom_pmf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K = k) for K ~ Binomial(n, p), 0 ≤ k ≤ n."""
    if p in (0, 1):
        return mpmath.mpf(1 if k == (n if p == 1 else 0) else 0)
    return mpmath.binomial(n, k) * p**k * (1 - p) ** (n - k)


def binom_cdf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K ≤ k): I_{1-p}(n-k, k+1), and 1 at k = n."""
    if k >= n or p == 0:
        return mpmath.mpf(1)
    if p == 1:
        return mpmath.mpf(0)
    return mpmath.betainc(n - k, k + 1, 0, 1 - p, regularized=True)


def negbinom_pmf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F = f) for the number of failures F before the s-th success, 0 < p < 1."""
    return mpmath.binomial(f + s - 1, f) * p**s * (1 - p) ** f


def negbinom_cdf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F ≤ f): I_p(s, f+1)."""
    return mpmath.betainc(s, f + 1, 0, p, regularized=True)


def poisson_pmf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X = x) for X ~ Poisson(m), m ≥ 0."""
    if m == 0:
        return mpmath.mpf(1 if x == 0 else 0)
    return mpmath.exp(x * mpmath.log(m) - m - mpmath.loggamma(x + 1))


def poisson_cdf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X ≤ x): Q(x+1, m)."""
    if m == 0:
        return mpmath.mpf(1)
    return mpmath.gammainc(x + 1, m, mpmath.inf, regularized=True)


def binom_sf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K > k): I_p(k+1, n-k), taken directly so a tiny upper tail keeps its digits."""
    if k >= n or p == 0:
        return mpmath.mpf(0)
    if p == 1:
        return mpmath.mpf(1)
    return mpmath.betainc(k + 1, n - k, 0, p, regularized=True)


def negbinom_sf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F > f): I_{1-p}(f+1, s)."""
    return mpmath.betainc(f + 1, s, 0, 1 - p, regularized=True)


def poisson_sf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X > x): P(x+1, m), the regularized lower incomplete gamma."""
    if m == 0:
        return mpmath.mpf(0)
    return mpmath.gammainc(x + 1, 0, m, regularized=True)


Tail = Callable[[mpmath.mpf], mpmath.mpf]


def integer_quantile(cdf: Tail, sf: Tail, q: mpmath.mpf, upper: bool, hi: float) -> mpmath.mpf:
    """The smallest integer k in [0, hi] with cdf(k) ≥ q (lower tail) or sf(k) ≤ q (upper tail): a doubling search
    from 1 brackets it, so no tail is evaluated far beyond the answer, then bisection over the integers. ValueError
    when no such k is at most `hi`."""

    def reached(k: mpmath.mpf) -> bool:
        return bool(sf(k) <= q) if upper else bool(cdf(k) >= q)

    lo, top = mpmath.mpf(0), mpmath.mpf(1)
    while not reached(top):
        lo, top = top + 1, top * 2
        if lo > hi:
            msg = "tail quantile beyond the count range"
            raise ValueError(msg)
    top = min(top, mpmath.mpf(int(hi)))
    if reached(lo):
        return lo
    while top - lo > 1:
        mid = mpmath.floor((lo + top) / 2)
        if reached(mid):
            top = mid
        else:
            lo = mid
    return top


def _binom_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, n, p = _reals(args)
    return integer_quantile(lambda k: binom_cdf(k, n, p), lambda k: binom_sf(k, n, p), q, upper, float(n))


def _negbinom_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, s, p = _reals(args)
    return integer_quantile(
        lambda f: negbinom_cdf(f, s, p), lambda f: negbinom_sf(f, s, p), q, upper, COUNT_DRAW_HI
    )


def _poisson_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, m = _reals(args)
    if m < 0:
        msg = "no tail map at a negative mean"
        raise ValueError(msg)
    return integer_quantile(lambda x: poisson_cdf(x, m), lambda x: poisson_sf(x, m), q, upper, COUNT_DRAW_HI)


def _binom_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    k, n, p = _reals(args)
    return binom_cdf(k, n, p) if args[-1] else binom_pmf(k, n, p)


def _binom_domain(args: Sequence[MathArg]) -> str | None:
    k, n, p = _reals(args)
    if k > n:
        return "number_s above trials"
    if p == 1 and k == 0 and n >= 1 and not args[-1]:
        return "pmf at k = 0 with p = 1 (Excel's LN(1 - p) closed form)"
    return None


def _negbinom_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    f, s, p = _reals(args)
    return negbinom_cdf(f, s, p) if args[-1] else negbinom_pmf(f, s, p)


def _poisson_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, m = _reals(args)
    if m < 0:
        # Excel's quirk: x = 0 with a negative mean is EXP(-mean), whatever the cumulative flag.
        value = mpmath.exp(-m)
        if value > mpmath.mpf("1.7976931348623157e308"):
            msg = "EXP(-mean) overflows"
            raise NoValueError(msg)
        return value
    return poisson_cdf(x, m) if args[-1] else poisson_pmf(x, m)


def _poisson_domain(args: Sequence[MathArg]) -> str | None:
    x, m = _reals(args)
    if m < 0 and x > 0:
        return "negative mean above x = 0"
    return None


def _count(
    name: str,
    lo: float,
    typical: tuple[float, float],
    extreme: tuple[float, float],
    source: str,
) -> Param:
    """A count with no Excel upper limit: Excel accepts huge counts (`NEGBINOM.DIST(2^31,1,1-1E-12,FALSE)` is 0,
    statistics.md "BINOM.DIST is #NUM! from 2^31 - 1 trials"), so the domain is unbounded above and only the
    draws are capped."""
    return Param(
        name,
        "int",
        Interval(lo, float("inf"), lo_open=False),
        typical=typical,
        extreme=extreme,
        source=source,
        draw_hi=COUNT_DRAW_HI,
    )


BINOM_DIST = register(
    Definition(
        function="BINOM.DIST",
        params=(
            # number_s and trials are drawn apart, so their ranges keep number_s ≤ trials in the typical and extreme
            # strata (number_s > trials is `#NUM!`); the tail stratum sets number_s from trials' own quantile.
            Param(
                "number_s",
                "int",
                Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False),
                typical=(0, 20),
                extreme=(1e2, 1e6),
                source="Microsoft documentation",
            ),
            Param(
                "trials",
                "int",
                Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False),
                typical=(20, 80),
                extreme=(1e6, TRIALS_LIMIT),
                edges=(TRIALS_LIMIT,),
                source=TRIALS_SOURCE,
            ),
            _PROBABILITY,
            _CUMULATIVE,
        ),
        value=_binom_dist,
        domain=_binom_domain,
        tail=_binom_tail,
        tail_param="number_s",
        sources=(TRIALS_SOURCE, P1_SOURCE, "Abramowitz and Stegun 26.5.24"),
    )
)

NEGBINOM_DIST = register(
    Definition(
        function="NEGBINOM.DIST",
        params=(
            _count("number_f", 0, (0, 30), (1e3, 1e12), "Microsoft documentation"),
            _count("number_s", 1, (1, 30), (1e3, 1e12), "Microsoft documentation"),
            _NEGBINOM_PROBABILITY,
            _CUMULATIVE,
        ),
        value=_negbinom_dist,
        tail=_negbinom_tail,
        tail_param="number_f",
        sources=(TRIALS_SOURCE, NEGBINOM_P_SOURCE, "Abramowitz and Stegun 26.5.24, 26.5.26"),
    )
)

POISSON_DIST = register(
    Definition(
        function="POISSON.DIST",
        params=(
            _count("x", 0, (0, 40), (1e3, 1e12), "Microsoft documentation"),
            Param(
                "mean",
                "real",
                Interval(),
                typical=(0.1, 40),
                extreme=(1e3, 1e12),
                edges=(0.0,),
                source=NEGATIVE_MEAN_SOURCE,
                # `POISSON.DIST(2,-2.09435621420062E+249,TRUE)` alone keeps the excel-graph oracle from opening its
                # workbook (createSession times out; -1.1E+32 is answered), so the mean's magnitude is capped.
                draw_hi=1e12,
            ),
            _CUMULATIVE,
        ),
        value=_poisson_dist,
        domain=_poisson_domain,
        tail=_poisson_tail,
        tail_param="x",
        sources=(NEGATIVE_MEAN_SOURCE, "Abramowitz and Stegun 26.4.21"),
    )
)
