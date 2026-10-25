"""For min(a, b) < 1 < 8 < max shapes with a unique pinned lnB (M2 exponent): the interval of algdiv values alg with
fl(GL(min) + alg) = the pinned lnB, relative to the exact ln(Gamma(b)/Gamma(a+b)) and to TOMS algdiv in doubles, in
units of ulp(alg).  Usage: algneed.py [set]"""
import sys

import mpmath

import algdiv_v as AV
import lnbfit as LF
import m1lib as M

mpmath.mp.prec = 200


def ulp(v):
    return mpmath.mpf(2) ** (mpmath.floor(mpmath.log(abs(v), 2)) - 52)


def rows(sets=("a", "probe1", "old"), minhits=1.0):
    tab = LF.table()
    out = []
    for (s, sh, _), (L0, n, hits) in sorted(tab.items()):
        lo, hi = min(sh), max(sh)
        if s not in sets or lo >= 1.0:
            continue
        mx = max(hits.values())
        need = [j for j, h in hits.items() if h == mx]
        if len(need) != 1 or mx < minhits * n:
            continue
        Ln = M.nb(L0, need[0])
        g = M.mpf(M.BE.GAMMALN(lo))
        uL = ulp(Ln)
        a_lo, a_hi = M.mpf(Ln) - uL / 2 - g, M.mpf(Ln) + uL / 2 - g
        ex = mpmath.loggamma(M.mpf(hi)) - mpmath.loggamma(M.mpf(lo) + M.mpf(hi))
        tm = AV.algdiv(lo, hi, dict(ar="d"))
        ua = ulp(ex)
        out.append((s, sh, n, float((a_lo - ex) / ua), float((a_hi - ex) / ua), float((tm - ex) / ua)))
    return out


if __name__ == "__main__":
    for s, sh, n, lo, hi, tm in rows(tuple(sys.argv[1:]) or ("a", "probe1")):
        print(f"{s:6s} {str(sh):18s} n={n:4d} need alg-exact in [{lo:+.2f}, {hi:+.2f}] ulp(alg)   TOMS-d {tm:+.2f}")
