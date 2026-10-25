"""lnGamma(min) candidates inside lnB for min < 1 (algdiv branch max > 8 and GL branch max <= 8), per shape: the j
(ulps from lnb_base) each gives against the needed j.  Usage: lgam.py [set] [mode=m2]"""
import collections
import sys

import mpmath

import algdiv_v as AV
import lnbfit as LF
import m1lib as M
from lscan import scan, rows_of

mpf, R53 = M.mpf, M.fl53
TS = M.BE.Kernel(M.BE.replace(M.BE.BEST, gamln="toms"))  # TOMS gamln/gamln1 with Excel LN


def lg_variants(v):
    return {"GL": mpf(M.BE.GAMMALN(v)), "toms": mpf(TS.gamln(v)),
            "g1-LN": mpf(R53(mpf(TS.gamln1(v)) - mpf(M.BE.LN(v)))),
            "cr": mpf(float(mpmath.loggamma(v)))}


def lnb_with(a, b, lg, alg=dict(ar="d")):
    lo, hi = min(a, b), max(a, b)
    if hi > 8:
        return float(R53(lg + AV.algdiv(lo, hi, alg)))
    return float(R53(lg + mpf(R53(mpf(M.BE.GAMMALN(hi)) - mpf(M.BE.GAMMALN(a + b))))))


if __name__ == "__main__":
    s = sys.argv[1] if len(sys.argv) > 1 else "a"
    rows = [r for r in rows_of(s) if min(r["a"], r["b"]) < 1 and ("--gl" not in sys.argv or max(r["a"], r["b"]) <= 8)]
    res = scan(rows, sys.argv[2] if len(sys.argv) > 2 else "m2", 12)
    tot = collections.Counter()
    for sh, (n, hits, L0) in sorted(res.items()):
        lo = min(sh)
        mx = max(hits.values())
        need = [j for j, h in hits.items() if h == mx]
        out = []
        for k, lg in lg_variants(lo).items():
            j = M.ulp_steps(L0, lnb_with(*sh, lg))
            tot[k] += hits.get(j, 0)
            out.append(f"{k}:{j:+d}")
        tot["best"] += mx
        tot["n"] += n
        print(f"{str(sh):16s} n={n:3d} need {need} ({mx}) " + " ".join(out))
    print(dict(tot))
