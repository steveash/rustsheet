"""Inverse regularized incomplete beta: BETA.INV and BETAINV (T-1745, rank 5 in
docs/research/numeric-kernel-map.md).

One fit, `beta-inv`, over the T-1044/T-1047 sweeps (probes-numeric/m9-beta-inv, m9-legacy-betainv).

The T-1745 probe (probes-numeric/t1745-beta-inv-roundtrip) gives BETA.DIST at Excel's quantile x
and at its two neighbouring doubles. On 155 of the 188 numeric sweep points x is a crossing point
of Excel's own CDF: BETA.DIST(x) = p on 79, and p lies between BETA.DIST at x and at a neighbour on
76 (36 with x at or just above the crossing, 40 with x the last double below it, so not a strict
one-sided rule). Of the other 33, Excel's CDF is flat across the three doubles at 22 (tiny x), and
at 11 x stops a few CDF ulps short of the crossing, which suggests a tolerance stop.
So Excel solves its own CDF to the last bit, and a quantile can only be as exact as the forward
kernel (T-1744) it inverts.

Candidates: bisection on [0, 1] to adjacent doubles, returning the midpoint step's last value, the
lower end or the upper end, on each T-1744 forward candidate family (the correctly rounded I_x,
the best combination, Cephes incbet as published, Numerical Recipes betai).

Result (T-1745 Outcome; docs/excel-semantics/statistics.md, "BETA.INV solves Excel's own CDF to the
crossing"): the midpoint of the last bisection bracket on the correctly rounded I_x is bit-exact on
87 of the 212 cells (202 with numeric arguments), against 34 for the engine (statrs
`inverse_cdf`); no candidate is Excel's, since none has Excel's forward kernel.
"""

from __future__ import annotations

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.beta import (
    Ibeta,
    best_combination,
    cephes_incbet,
    gamma_ratio_libm,
    ibeta_reference,
    libm_pow,
    nr_betai,
)
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

CASES = ["probes-numeric/m9-beta-inv", "probes-numeric/m9-legacy-betainv"]
PATTERN = r"^=(?:BETA\.INV|BETAINV)\(([^,()]+),([^,()]+),([^,()]+)\)$"
_MAX_BISECTIONS = 1100  # directive 5: halving [0, 1] reaches adjacent doubles within ~1075 steps


def bisect(cdf: Ibeta, p: float, a: float, b: float, end: str) -> float:
    """The double x in (0, 1) where cdf(x, a, b) crosses p, by bisection to adjacent doubles.
    `end` picks the result: "lo" (last x with cdf < p), "hi" (first with cdf >= p) or "mid"."""
    if not 0.0 < p < 1.0:
        msg = f"p = {p} is outside (0, 1)"
        raise ValueError(msg)
    lo, hi = 0.0, 1.0
    for _ in range(_MAX_BISECTIONS):
        mid = 0.5 * (lo + hi)
        if mid in {lo, hi}:
            break
        if cdf(mid, a, b) < p:
            lo = mid
        else:
            hi = mid
    return {"lo": lo, "hi": hi}.get(end, 0.5 * (lo + hi))


def _cephes_published(x: float, a: float, b: float) -> float:
    return cephes_incbet(x, a, b, libm_pow, gamma_ratio_libm)


FORWARD: dict[str, Ibeta] = {
    "correctly rounded I_x": ibeta_reference,
    "T-1744 best combination": best_combination,
    "Cephes incbet as published": _cephes_published,
    "NR betai": nr_betai,
}


def beta_inv_candidates() -> list[Candidate]:
    out: list[Candidate] = []
    for fname, cdf in FORWARD.items():
        for end in ("hi", "lo", "mid"):

            def solve(*args: Arg, f: Ibeta = cdf, e: str = end) -> float:
                p, a, b = (numeric_arg(v) for v in args)
                return bisect(f, p, a, b, e)

            out.append(candidate(f"bisection to the crossing ({end}) on {fname}", solve, fname))
    return out


FITS = [
    Fit(
        name="beta-inv",
        description="BETA.INV/BETAINV (T-1044, T-1047): bisection to the crossing on the T-1744 candidates",
        cases=CASES,
        pattern=PATTERN,
        candidates=beta_inv_candidates,
    ),
]
