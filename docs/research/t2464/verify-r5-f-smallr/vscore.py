"""verify-r5-f-smallr: own scorer.  Tallies bit-exact / within 4 ulp / within half a unit of the 15th significant
digit (of Excel's value), against Excel, for the worker's frozen variants (imported after reading final_model.py) and
for an own correctly rounded density (mpmath).  Usage: vscore.py CASE_ID [CASE_ID ...] | --all"""
from __future__ import annotations

import collections
import math
import sys
from pathlib import Path

import mpmath

import vparse as V

mpmath.mp.prec = 200
W = Path(__file__).resolve().parent.parent / "r5-f-smallr"
VARS = ("CR", "msd", "xs1", "dd", "xs")


def cr(x: float, m: float, n: float) -> float:
    X, M, N = mpmath.mpf(x), mpmath.mpf(m), mpmath.mpf(n)
    a, b = M / 2, N / 2
    lf = a * mpmath.log(M / N) + (a - 1) * mpmath.log(X) - (a + b) * mpmath.log1p(M * X / N) - mpmath.log(mpmath.beta(a, b))
    return float(mpmath.exp(lf))


def side_r(x: float, m: float, n: float):
    """(side, r) with mx = fl(m x) (exact product rounded once), r = fl(mx/n) or fl(n/mx)."""
    mx = float(mpmath.mpf(m) * mpmath.mpf(x))
    return (1, float(mpmath.mpf(mx) / n)) if mx <= n else (2, float(mpmath.mpf(n) / mx))


def grade(v: float, e: float):
    if not (math.isfinite(v) and math.isfinite(e)):
        return (v == e, v == e, v == e)
    d = abs(v - e)
    ex = v == e
    w4 = d <= 4 * math.ulp(e)
    w15 = e == 0 and v == 0 or (e != 0 and d <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14))
    return ex, w4, w15


class Tally:
    def __init__(self):
        self.n = self.ex = self.w4 = self.w15 = 0

    def add(self, g):
        self.n += 1
        self.ex += g[0]
        self.w4 += g[1]
        self.w15 += g[2]

    def __str__(self):
        return f"{self.ex}/{self.w4}/{self.w15} of {self.n}"


_FM = None


def model():
    global _FM
    if _FM is None:
        sys.path.insert(0, str(W))
        import final_model  # the worker's frozen variants (checked against RESULTS.md section 2)

        _FM = final_model
    return _FM


def value(var, x, m, n):
    if var == "CR":
        return cr(x, m, n)
    return model().value(x, m, n, var)


def score(ids, variants=VARS, smallr_only=False):
    res = collections.defaultdict(Tally)
    for cid in ids:
        rows, _ = V.rows(cid)
        for cell, x, m, n, e in rows:
            if not (x > 0 and m >= 1 and n >= 1 and e > 0):
                continue
            side, r = side_r(x, m, n)
            sub = "r<=0.375" if r <= 0.375 else "r>0.375"
            if smallr_only and sub != "r<=0.375":
                continue
            for var in variants:
                try:
                    v = value(var, x, m, n)
                except Exception:  # model outside its domain: counted as a miss
                    v = math.nan
                g = grade(v, e)
                res[(var, sub)].add(g)
                res[(var, "all")].add(g)
                res[(var, cid, sub)].add(g)
    return res


if __name__ == "__main__":
    ids = V.all_case_ids() if sys.argv[1:] == ["--all"] else sys.argv[1:]
    res = score(ids)
    for k in sorted(res, key=lambda k: (len(k), k)):
        print(" | ".join(map(str, k)), "  ", res[k])
        sys.stdout.flush()
