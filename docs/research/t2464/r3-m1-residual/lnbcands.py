"""Named algdiv / lnB candidates on lnbfit's M2 tables (rows hit, shapes at their best j), per set; min = 1 shapes
use -LN(max) in every candidate, so the counts below are the algdiv shapes plus those.  Usage: lnbcands.py"""
import mpmath

import algdiv_v as AV
import lnbfit as LF
import m1lib as M

mpmath.mp.prec = 200


def with_a1(fn):
    return lambda a, b: -M.BE.LN(max(a, b)) if min(a, b) == 1.0 else fn(a, b)


def accurate(a, b):
    lo, hi = min(a, b), max(a, b)
    s = M.mpf(M.BE.GAMMALN(lo)) + mpmath.loggamma(M.mpf(hi)) - mpmath.loggamma(M.mpf(lo) + M.mpf(hi))
    return float(M.fl53(M.fl64(s)))


CANDS = {"TOMS doubles (frozen)": M.lnb_base, "accurate algdiv": accurate}
for nm, v in (("x87 stored ops", dict(ar="x")), ("extended, toms alnrel", dict(ar="e")),
              ("extended, cr alnrel", dict(ar="e", aln="cr")), ("extended, series alnrel", dict(ar="e", aln="ser")),
              ("extended, LN(1+q)", dict(ar="e", aln="ln")), ("extended, log b extended", dict(ar="e", lnb="e")),
              ("extended, sum extended", dict(ar="e", gsum="e")), ("doubles, series alnrel", dict(ar="d", aln="ser")),
              ("doubles, cr alnrel", dict(ar="d", aln="cr"))):
    CANDS[nm] = (lambda vv: (lambda a, b: float(AV.lnb(a, b, vv))))(v)

if __name__ == "__main__":
    tab = LF.table()
    nsh = {s: sum(1 for k in tab if k[0] == s) for s in ("a", "old", "probe1")}
    best = {s: sum(max(v[2].values()) for k, v in tab.items() if k[0] == s) for s in nsh}
    print("per-shape best (rows):", best, "shapes:", nsh)
    for nm, fn in CANDS.items():
        e = LF.evaluate(tab, with_a1(fn))
        print(f"{nm:28s} " + "  ".join(f"{s}: {e[s][0]} rows, {e[s][2]} shapes" for s in ("a", "old", "probe1")))
