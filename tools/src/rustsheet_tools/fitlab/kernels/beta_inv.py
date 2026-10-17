"""Inverse regularized incomplete beta: BETA.INV/BETAINV, and through T-1744's F and t mappings
F.INV, F.INV.RT/FINV, T.INV and T.INV.2T/TINV (T-1745, rank 5 in docs/research/numeric-kernel-map.md).

The T-1745 probe (probes-numeric/t1745-beta-inv-roundtrip) gives BETA.DIST at Excel's quantile x
and at its two neighbouring doubles. On 155 of its 188 rows x is a crossing point of Excel's own
CDF: BETA.DIST(x) = p on 79, and p lies between BETA.DIST at x and at a neighbour on 76 (36 with
x at or just above the crossing, 40 with x the last double below it, so not a strict one-sided
rule). Of the other 33, Excel's CDF is flat across the three doubles at 22 (tiny x), and at 11 x
misses the crossing by 1 to 188 ulps of p on both sides. So Excel solves its own CDF, usually to
the last bit, and a quantile can only be as exact as the forward kernel (T-1744) it inverts.

Candidates: bisection to adjacent doubles, returning the bracket's lower end, upper end or "mid"
(for adjacent doubles that is the even-significand end), on each T-1744 forward candidate family
(the correctly rounded I_x, the best combination, Cephes incbet as published, Numerical Recipes
betai). F and t brackets grow by doubling from 1.

Result (T-1745 Outcome; docs/excel-semantics/statistics.md, "BETA.INV solves Excel's own CDF to
the crossing"): the best BETA.INV candidate is bit-exact on 87 of 212 cells against 34 for the
engine (statrs `inverse_cdf`); F.INV 8 vs 2, F.INV.RT 11 vs 9, T.INV 29 vs 17, T.INV.2T 25 vs 9.
No candidate is Excel's, since none has Excel's forward kernel.
"""

from __future__ import annotations

from collections.abc import Callable

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.beta import (
    Ibeta,
    best_combination,
    cephes_incbet,
    f_cdf,
    f_cdf_rt,
    gamma_ratio_libm,
    ibeta_reference,
    libm_pow,
    nr_betai,
    t_cdf,
    t_two_tail,
)
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

CASES = ["probes-numeric/m9-beta-inv", "probes-numeric/m9-legacy-betainv"]
PATTERN = r"^=(?:BETA\.INV|BETAINV)\(([^,()]+),([^,()]+),([^,()]+)\)$"
_F_INV_CASES = ["probes-numeric/m9-f-inv", "probes-numeric/t1049-finv"]
_F_INV_PATTERN = r"^=F\.INV\(([^,()]+),([^,()]+),([^,()]+)\)$"
_F_INV_RT_CASES = [
    "probes-numeric/m9-f-inv-rt",
    "probes-numeric/m9-legacy-finv",
    "probes-numeric/t1049-finvrt",
]
_F_INV_RT_PATTERN = r"^=(?:F\.INV\.RT|FINV)\(([^,()]+),([^,()]+),([^,()]+)\)$"
_T_INV_CASES = ["probes-numeric/m9-t-inv"]
_T_INV_PATTERN = r"^=T\.INV\(([^,()]+),([^,()]+)\)$"
_T_INV_2T_CASES = ["probes-numeric/m9-t-inv-2t", "probes-numeric/t1048-tinv"]
_T_INV_2T_PATTERN = r"^=(?:T\.INV\.2T|TINV)\(([^,()]+),([^,()]+)\)$"
# directive 5: halving a bracket of at most 2^1024 reaches adjacent doubles within ~2100 steps,
# and doubling the bracket from 1 overflows after 1024.
_MAX_BISECTIONS = 2200
_MAX_DOUBLINGS = 1100
ENDS = ("hi", "lo", "mid")

Cdf = Callable[[float], float]


def _check_p(p: float) -> None:
    if not 0.0 < p < 1.0:
        msg = f"p = {p} is outside (0, 1)"
        raise ValueError(msg)


def crossing(g: Cdf, p: float, lo: float, hi: float, end: str, *, increasing: bool = True) -> float:
    """The double where the monotone g crosses p in [lo, hi], by bisection to adjacent doubles.
    `end` picks the result: "lo" (the bracket's lower end), "hi" (its upper end) or "mid"."""
    if end not in ENDS:
        msg = f"end must be one of {ENDS}, not {end!r}"
        raise ValueError(msg)
    for _ in range(_MAX_BISECTIONS):
        mid = 0.5 * (lo + hi)
        if mid in {lo, hi}:
            break
        value = g(mid)
        if (value < p) if increasing else (value > p):
            lo = mid
        else:
            hi = mid
    return {"lo": lo, "hi": hi, "mid": 0.5 * (lo + hi)}[end]


def _upper_bracket(g: Cdf, p: float, *, increasing: bool) -> float:
    """The first power of two hi >= 1 whose g is past p (g(hi) >= p, or <= p when decreasing)."""
    hi = 1.0
    for _ in range(_MAX_DOUBLINGS):
        value = g(hi)
        if (value >= p) if increasing else (value <= p):
            return hi
        hi *= 2.0
    msg = f"no bracket for p = {p}"
    raise ValueError(msg)


def bisect(cdf: Ibeta, p: float, a: float, b: float, end: str) -> float:
    """BETA.INV: where I_x(a, b) crosses p on [0, 1]."""
    _check_p(p)
    return crossing(lambda x: cdf(x, a, b), p, 0.0, 1.0, end)


def f_inv(cdf: Ibeta, p: float, d1: float, d2: float, end: str, *, right_tail: bool) -> float:
    """F.INV (or F.INV.RT/FINV): where F.DIST (or F.DIST.RT) crosses p on [0, inf)."""
    _check_p(p)
    increasing = not right_tail

    def g(x: float) -> float:
        return f_cdf_rt(cdf, x, d1, d2) if right_tail else f_cdf(cdf, x, d1, d2)

    return crossing(g, p, 0.0, _upper_bracket(g, p, increasing=increasing), end, increasing=increasing)


def t_inv(cdf: Ibeta, p: float, n: float, end: str, *, two_tailed: bool) -> float:
    """T.INV: where T.DIST crosses p; T.INV.2T/TINV: the t > 0 where P(|T| > t) = p."""
    _check_p(p)
    if two_tailed:

        def tail(t: float) -> float:
            return t_two_tail(cdf, t, n)

        return crossing(tail, p, 0.0, _upper_bracket(tail, p, increasing=False), end, increasing=False)

    def g(t: float) -> float:
        return t_cdf(cdf, t, n)

    if p == 0.5:
        return 0.0
    if p > 0.5:
        return crossing(g, p, 0.0, _upper_bracket(g, p, increasing=True), end)

    def mirrored(t: float) -> float:
        return g(-t)

    lower = -_upper_bracket(mirrored, p, increasing=False)
    return crossing(g, p, lower, 0.0, end)


def _cephes_published(x: float, a: float, b: float) -> float:
    return cephes_incbet(x, a, b, libm_pow, gamma_ratio_libm)


FORWARD: dict[str, Ibeta] = {
    "correctly rounded I_x": ibeta_reference,
    "T-1744 best combination": best_combination,
    "Cephes incbet as published": _cephes_published,
    "NR betai": nr_betai,
}

Solver = Callable[[Ibeta, tuple[float, ...], str], float]


def _candidates(solver: Solver) -> list[Candidate]:
    out: list[Candidate] = []
    for fname, cdf in FORWARD.items():
        for end in ENDS:

            def solve(*args: Arg, f: Ibeta = cdf, e: str = end) -> float:
                return solver(f, tuple(numeric_arg(v) for v in args), e)

            out.append(candidate(f"bisection to the crossing ({end}) on {fname}", solve, fname))
    return out


def beta_inv_candidates() -> list[Candidate]:
    return _candidates(lambda f, a, e: bisect(f, a[0], a[1], a[2], e))


def f_inv_candidates() -> list[Candidate]:
    return _candidates(lambda f, a, e: f_inv(f, a[0], a[1], a[2], e, right_tail=False))


def f_inv_rt_candidates() -> list[Candidate]:
    return _candidates(lambda f, a, e: f_inv(f, a[0], a[1], a[2], e, right_tail=True))


def t_inv_candidates() -> list[Candidate]:
    return _candidates(lambda f, a, e: t_inv(f, a[0], a[1], e, two_tailed=False))


def t_inv_2t_candidates() -> list[Candidate]:
    return _candidates(lambda f, a, e: t_inv(f, a[0], a[1], e, two_tailed=True))


def _fit(
    name: str, description: str, cases: list[str], pattern: str, cands: Callable[[], list[Candidate]]
) -> Fit:
    return Fit(name=name, description=description, cases=cases, pattern=pattern, candidates=cands)


FITS = [
    _fit(
        "beta-inv",
        "BETA.INV/BETAINV (T-1044, T-1047): bisection to the crossing on the T-1744 candidates",
        CASES,
        PATTERN,
        beta_inv_candidates,
    ),
    _fit(
        "f-inv",
        "F.INV (T-1044, T-1049): bisection on F.DIST through each I_x",
        _F_INV_CASES,
        _F_INV_PATTERN,
        f_inv_candidates,
    ),
    _fit(
        "f-inv-rt",
        "F.INV.RT/FINV (T-1044, T-1047, T-1049): bisection on F.DIST.RT",
        _F_INV_RT_CASES,
        _F_INV_RT_PATTERN,
        f_inv_rt_candidates,
    ),
    _fit(
        "t-inv",
        "T.INV (T-1043): bisection on T.DIST through each I_x",
        _T_INV_CASES,
        _T_INV_PATTERN,
        t_inv_candidates,
    ),
    _fit(
        "t-inv-2t",
        "T.INV.2T/TINV (T-1043, T-1048): bisection on the two-tailed t probability",
        _T_INV_2T_CASES,
        _T_INV_2T_PATTERN,
        t_inv_2t_candidates,
    ),
]
