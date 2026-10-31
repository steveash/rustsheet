"""The gamma family: GAMMA.DIST, GAMMA.INV, CHISQ.DIST.RT, CHISQ.INV.RT, POISSON.DIST, GAMMALN, GAMMA.

Every one is the regularized incomplete gamma P(a, x) (the lower incomplete Γ over Γ(a)), its
complement Q = 1 - P, or Γ itself (Abramowitz and Stegun 6.5.1, 6.5.3; 26.4.2 for chi-squared,
26.4.21 for the Poisson cdf as Q(k + 1, λ)). The quantiles solve P (lower half) or Q (upper half)
for ln x, so a 1E-300 tail and a centre point are equally well conditioned, and 1 - p is exact at
the working precision.

Domains are the math's:
- deg_freedom below 1 is outside (statistics.md, "Incomplete beta: the integrated model ...
  (T-2464)": "CHISQ.DIST.RT truncates its df to an integer (2b < 1 is `#NUM!`)");
- the right-tail inverse allows p = 1 and not p = 0 (statistics.md, "Legacy distribution names
  sweep (T-1047)": CHIINV(1, ..) = 0);
- the df cut above 1E10 is evidenced only for the legacy CHIDIST (same section), so it is not
  encoded; the extreme df range stops at 1E10;
- Excel-only error regions inside the math's domain are not encoded (the GAMMA.DIST density at a
  subnormal x/beta, T-2738; the cdf above a shape of about 1E76, T-2435; GAMMA.INV's `#NUM!` branch
  at a huge shape, T-2982): there the math has a value and Excel's error is matched through the
  Excel half of Gate A's envelope.

First used by T-3509's held-out probe (docs/research/t3509/README.md), whose math sidecars these
definitions write.
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)

CHI_SOURCE = "statistics.md#legacy-distribution-names-sweep-t-1047 (CHIINV p = 0 and p = 1; CHIDIST df 1E10)"
GAMMA_SOURCE = "Abramowitz and Stegun 6.5.1, 6.5.3 (regularized incomplete gamma)"
DF_EXTREME = 1e10  # the largest df drawn (the legacy CHIDIST's last valid df)
MAX_TERMS = 10_000_000  # series or fraction steps before a point is given up (the time limit comes first)
GUARD_DPS = 20  # extra digits for the expansions and the one complement
MAX_NEWTON = 2000  # safeguarded Newton steps (each failed one at least bisects the bracket)
MAX_DOUBLINGS = 64  # an upper-tail bracket doubles from a + 1 at most this often (past any double)

_HALF = mpmath.mpf(1) / 2


def _prefactor(a: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """x^a·e^-x/Γ(a), in log space (a huge shape neither overflows nor cancels)."""
    return mpmath.exp(a * mpmath.log(x) - x - mpmath.loggamma(a))


def _series(a: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """P(a, x) by its power series (A&S 6.5.29): every term is positive, so the sum is accurate
    to the working precision; used below x = a + 1, where the term ratio x/(a + k) is below 1."""
    eps = mpmath.eps
    term = total = 1 / a
    ap = a
    for _ in range(MAX_TERMS):
        ap += 1
        term *= x / ap
        total += term
        if term < total * eps:
            return _prefactor(a, x) * total
    msg = "the incomplete gamma series did not converge"
    raise NoValueError(msg)


def _fraction(a: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """Q(a, x) by Legendre's continued fraction (A&S 6.5.31) in the modified Lentz form; used
    from x = a + 1, where it converges in a few times √a steps."""
    eps, tiny = mpmath.eps, mpmath.mpf(10) ** (-mpmath.mp.dps - 50)
    b = x + 1 - a
    c, d = 1 / tiny, 1 / b
    h = d
    for i in range(1, MAX_TERMS):
        an = -i * (i - a)
        b += 2
        d = an * d + b
        d = tiny if abs(d) < tiny else d
        c = b + an / c
        c = tiny if abs(c) < tiny else c
        d = 1 / d
        step = d * c
        h *= step
        if abs(step - 1) < eps:
            return _prefactor(a, x) * h
    msg = "the incomplete gamma continued fraction did not converge"
    raise NoValueError(msg)


def lower(a: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """P(a, x). mpmath's `gammainc` gives up (hypergeometric maxterms) or runs for minutes at a
    shape of 1E6 and above, so the two classical expansions are summed here instead, with guard
    digits; the complement is taken only where it is at least about 1/2 (the median is below a),
    so it never cancels."""
    if x == 0:
        return mpmath.mpf(0)
    with mpmath.extradps(GUARD_DPS):
        value = _series(a, x) if x < a + 1 else 1 - _fraction(a, x)
    return +value


def upper(a: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """Q(a, x) = 1 - P(a, x), with no cancellation in the far upper tail (see `lower`)."""
    if x == 0:
        return mpmath.mpf(1)
    with mpmath.extradps(GUARD_DPS):
        value = 1 - _series(a, x) if x < a + 1 else _fraction(a, x)
    return +value


def density(x: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """The gamma density at scale 1, in log space so a huge shape neither overflows nor cancels."""
    if x == 0:
        if a > 1:
            return mpmath.mpf(0)
        if a == 1:
            return mpmath.mpf(1)
        msg = "the density has a pole at x = 0 below shape 1"
        raise NoValueError(msg)
    return mpmath.exp((a - 1) * mpmath.log(x) - x - mpmath.loggamma(a))


def _lead_root(p: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """(p·Γ(a + 1))^(1/a): P(a, x) ≤ x^a/Γ(a + 1), so P at this x is at most p."""
    return mpmath.exp((mpmath.log(p) + mpmath.loggamma(a + 1)) / a)


def _start(p: mpmath.mpf, a: mpmath.mpf, upper_tail: bool) -> mpmath.mpf | None:
    """The Wilson-Hilferty approximation of the quantile (A&S 26.4.17 at df = 2a), or None where
    it is not positive (a small shape deep in the lower tail)."""
    z = mpmath.sqrt(2) * mpmath.erfinv(2 * p - 1)  # the normal quantile of p
    z = -z if upper_tail else z
    c = 1 / (9 * a)
    base = 1 - c + z * mpmath.sqrt(c)
    return a * base**3 if base > 0 else None


def quantile(p: mpmath.mpf, a: mpmath.mpf, *, upper_tail: bool) -> mpmath.mpf:
    """The x at scale 1 with P(a, x) = p (or Q(a, x) = p when `upper_tail`), for 0 < p ≤ 1/2.

    Newton's method on s = ln x for ln P(a, e^s) = ln p (ln Q), whose slope is x·density/P, kept
    inside a bracket that every step narrows, with a bisection whenever a step leaves it (so it
    always converges). It starts from the Wilson-Hilferty approximation when that is inside the
    bracket. A bracket-only solver (`solve_bracket`) is too slow at a shape of 1E6 and above,
    where one P costs about 1E5 series terms.

    Brackets (the median is in (a - 1/3, a), Chen and Rubin 1986, so P(a, a) > 1/2):
    - lower: [lead root of p, a]: the lead root has P ≤ p ≤ 1/2 < P(a, a);
    - upper: [lead root of 1/2, a + 1 doubled until Q < p]: the lead root of 1/2 has Q ≥ 1/2."""
    target = mpmath.log(p)
    if upper_tail:
        lo = _lead_root(_HALF, a)
        hi = a + 1
        for _ in range(MAX_DOUBLINGS):
            if upper(a, hi) < p:
                break
            hi *= 2
        else:
            msg = "upper-tail quantile beyond the double range"
            raise NoValueError(msg)
    else:
        lo, hi = _lead_root(p, a), a
    if lo == 0:  # the lead root underflowed mpmath's exponent range: far below any double
        return mpmath.mpf(0)
    tail = upper if upper_tail else lower

    def f(s: mpmath.mpf) -> tuple[mpmath.mpf, mpmath.mpf]:
        """(ln T(e^s) - ln p, its derivative in s), T = P or Q; increasing in s for P."""
        x = mpmath.exp(s)
        t = tail(a, x)
        slope = _prefactor(a, x) / t
        return mpmath.log(t) - target, -slope if upper_tail else slope

    s_lo, s_hi = mpmath.log(lo), mpmath.log(hi)
    value, _ = f(s_lo)
    if not upper_tail and value >= 0:
        # At a tiny x, P(a, x) = x^a/Γ(a + 1)·(1 - a·x/(a + 1) + ...): the lead root is the
        # quantile to the working precision, and P there rounds to p.
        return lo
    guess = _start(p, a, upper_tail)
    s = mpmath.log(guess) if guess is not None and lo < guess < hi else (s_lo + s_hi) / 2
    tolerance = 16 * mpmath.eps * max(abs(s_lo), abs(s_hi), 1)
    for _ in range(MAX_NEWTON):
        value, slope = f(s)
        if value == 0:
            return mpmath.exp(s)
        if (value < 0) != upper_tail:  # below the root
            s_lo = s
        else:
            s_hi = s
        step = value / slope
        if abs(step) <= tolerance:  # converged: the next step is below the working precision
            return mpmath.exp(s - step)
        nxt = s - step
        if not s_lo < nxt < s_hi:
            nxt = (s_lo + s_hi) / 2
        if s_hi - s_lo <= tolerance:
            return mpmath.exp(nxt)
        s = nxt
    msg = "the quantile's Newton steps did not converge"
    raise NoValueError(msg)


def central_quantile(p: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """The x at scale 1 with P(a, x) = p, for 0 < p < 1 (1 - p is exact at the working precision)."""
    return quantile(p, a, upper_tail=False) if p <= _HALF else quantile(1 - p, a, upper_tail=True)


def _real(args: Sequence[MathArg], i: int) -> mpmath.mpf:
    v = args[i]
    if isinstance(v, bool):
        msg = "a logical where a number is expected"
        raise ValueError(msg)
    return v


def _gamma_tail(q: mpmath.mpf, upper_tail: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    a, b = _real(args, 1), _real(args, 2)
    if a <= 0 or b <= 0:
        msg = "no tail map outside the domain"
        raise ValueError(msg)
    return b * quantile(q, a, upper_tail=upper_tail)


def _chisq_tail(q: mpmath.mpf, upper_tail: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    df = _real(args, 1)
    if df < 1:
        msg = "no tail map below df 1"
        raise ValueError(msg)
    return 2 * quantile(q, mpmath.floor(df) / 2, upper_tail=upper_tail)


def _gamma_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, a, b, cumulative = _real(args, 0), _real(args, 1), _real(args, 2), args[3]
    if cumulative:
        return lower(a, x / b)
    return density(x / b, a) / b


def _gamma_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, a, b = _real(args, 0), _real(args, 1), _real(args, 2)
    return b * central_quantile(p, a)


def _chisq_dist_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df = _real(args, 0), _real(args, 1)
    return upper(df / 2, x / 2)


def _chisq_inv_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    p, df = _real(args, 0), _real(args, 1)
    if p == 1:
        return mpmath.mpf(0)
    # p is the upper tail: solve Q for p up to 1/2 and P for 1 - p above (1 - 1E-300 is not
    # representable even at the working precision, so the complement is never taken of a tiny p).
    a = df / 2
    return 2 * (quantile(p, a, upper_tail=True) if p <= _HALF else quantile(1 - p, a, upper_tail=False))


def _poisson_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    k, mean, cumulative = _real(args, 0), _real(args, 1), args[2]
    if mean == 0:  # every mass at 0
        return mpmath.mpf(1) if cumulative or k == 0 else mpmath.mpf(0)
    if cumulative:
        return upper(k + 1, mean)
    return mpmath.exp(k * mpmath.log(mean) - mean - mpmath.loggamma(k + 1))


def _gammaln(args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.loggamma(_real(args, 0))


def _gamma(args: Sequence[MathArg]) -> mpmath.mpf:
    x = _real(args, 0)
    if x <= 0 and x == mpmath.floor(x):
        msg = "pole at a non-positive integer"
        raise DomainError(msg)
    return mpmath.gamma(x)


_POSITIVE = Interval(0, float("inf"))
_NON_NEGATIVE = Interval(0, float("inf"), lo_open=False)
_SHAPE = Param("alpha", "real", _POSITIVE, typical=(0.1, 50), extreme=(1e-3, 1e7), source=GAMMA_SOURCE)
_SCALE = Param("beta", "real", _POSITIVE, typical=(0.1, 10), extreme=(1e-6, 1e6), source=GAMMA_SOURCE)
_DF = Param(
    "deg_freedom",
    "int",
    Interval(1, float("inf"), lo_open=False),
    typical=(1, 100),
    extreme=(1e3, DF_EXTREME),
    source=CHI_SOURCE,
)
_TAIL_P = Param(
    "probability",
    "real",
    Interval(0, 1),
    typical=(0.001, 0.999),
    extreme=(1e-300, 1e-15),
    source=GAMMA_SOURCE,
)
_CUMULATIVE = Param("cumulative", "bool", source="catalog")

GAMMA_DIST = register(
    Definition(
        function="GAMMA.DIST",
        params=(
            Param(
                "x", "real", _NON_NEGATIVE, typical=(0.01, 100), extreme=(1e-300, 1e8), source=GAMMA_SOURCE
            ),
            _SHAPE,
            _SCALE,
            _CUMULATIVE,
        ),
        value=_gamma_dist,
        tail=_gamma_tail,
        tail_param="x",
        sources=(GAMMA_SOURCE,),
    )
)

GAMMA_INV = register(
    Definition(
        function="GAMMA.INV",
        params=(_TAIL_P, _SHAPE, _SCALE),
        value=_gamma_inv,
        sources=(GAMMA_SOURCE,),
    )
)

CHISQ_DIST_RT = register(
    Definition(
        function="CHISQ.DIST.RT",
        params=(
            Param(
                "x", "real", _NON_NEGATIVE, typical=(0.01, 100), extreme=(1e-300, 1e8), source=GAMMA_SOURCE
            ),
            _DF,
        ),
        value=_chisq_dist_rt,
        tail=_chisq_tail,
        tail_param="x",
        sources=(GAMMA_SOURCE, "Abramowitz and Stegun 26.4.2", CHI_SOURCE),
    )
)

CHISQ_INV_RT = register(
    Definition(
        function="CHISQ.INV.RT",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1, hi_open=False),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=CHI_SOURCE,
            ),
            _DF,
        ),
        value=_chisq_inv_rt,
        sources=(GAMMA_SOURCE, "Abramowitz and Stegun 26.4.2", CHI_SOURCE),
    )
)

POISSON_DIST = register(
    Definition(
        function="POISSON.DIST",
        params=(
            Param("x", "int", _NON_NEGATIVE, typical=(0, 50), extreme=(1e3, 1e7), source="A&S 26.4.21"),
            Param("mean", "real", _NON_NEGATIVE, typical=(0.1, 50), extreme=(1e3, 1e7), source="A&S 26.4.21"),
            _CUMULATIVE,
        ),
        value=_poisson_dist,
        sources=("Abramowitz and Stegun 26.4.21 (the cdf is Q(k + 1, mean))",),
    )
)

GAMMALN = register(
    Definition(
        function="GAMMALN",
        params=(
            Param("x", "real", _POSITIVE, typical=(0.1, 100), extreme=(1e-300, 1e300), source="A&S 6.1"),
        ),
        value=_gammaln,
        sources=("Abramowitz and Stegun 6.1 (ln Γ)",),
    )
)

GAMMA = register(
    Definition(
        function="GAMMA",
        params=(Param("x", "real", Interval(), typical=(-10, 30), extreme=(1e-300, 171), source="A&S 6.1"),),
        value=_gamma,
        sources=("Abramowitz and Stegun 6.1 (Γ; poles at the non-positive integers)",),
    )
)
