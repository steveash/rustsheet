"""Model M2 (frozen) for BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1, and its scores.

  tx = fl(fl(a-1) * LN(x)), ty = fl(fl(b-1) * ly), ly = Excel log1p(-x) (x <= 0.375) or LN(fl(1-x))   [as M1]
  lnB: min = 1 -> -LN(max); min < 1, max <= 8 -> fl(GL(min) + fl(GL(max) - GL(a+b))); max > 8 -> GL(min) +
       algdiv(min, max) with ALG = "toms" (TOMS 708 in doubles, Excel LN) or "exact" (accurate, rounded sum);
       GL = Excel's GAMMALN (measured, probe t2513-r3-m1-residual-gl, else the engine model)
  hi = fl(fl(tx + ty) - lnB), lo = (tx + ty - lnB) - hi exactly, density = fl(EXP(hi) * fl(1 + lo))   (every b)
Compared with M1 (r2-bratio Kernel.density + Excel log1p).
Usage: final_score.py old|probe1|a|hold [ALG=toms|exact]"""

import collections
import sys

import mpmath

import algdiv_v as AV
import m1lib as M
from lscan import rows_of
from m1score import region


def lnb_m2(a: float, b: float, alg: str = "toms") -> float:
    lo, hi = min(a, b), max(a, b)
    if lo == 1.0:
        return -M.BE.LN(hi)
    if hi <= 8.0 or lo > 1.0:
        return M.lnb_base(a, b)
    g = M.mpf(M.BE.GAMMALN(lo))
    if alg == "exact":
        al = mpmath.loggamma(M.mpf(hi)) - mpmath.loggamma(M.mpf(lo) + M.mpf(hi))
    else:
        al = AV.algdiv(lo, hi, dict(ar="d"))
    return float(M.fl53(M.fl64(g + al)))


def main(name, alg):
    rows = rows_of(name)
    M.ensure_gl(rows)
    by = collections.defaultdict(lambda: collections.defaultdict(list))
    cache = {}
    for r in rows:
        a, b = r["a"], r["b"]
        if (a, b) not in cache:
            cache[(a, b)] = (M.lnb_base(a, b), lnb_m2(a, b, alg))
        L1, L2 = cache[(a, b)]
        tx, ty = M.pieces(r["x"], a, b)
        m1 = M.combine(tx, ty, L1, a, b, "m1")
        m2 = M.combine(tx, ty, L2, a, b, "m2")
        for g in (r.get("blk", region(r)), "ALL"):
            by[g]["M1"].append((m1, r["excel"]))
            by[g]["M2"].append((m2, r["excel"]))
    for g in sorted(by):
        print(
            f"{name:6s} ALG={alg:5s} {g:14s} "
            + "  ".join(f"{k}: {M.fmt(M.score(*zip(*v)))}" for k, v in by[g].items())
        )


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "toms")
