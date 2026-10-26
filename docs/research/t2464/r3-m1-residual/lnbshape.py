"""Per-shape view of lnbfit's table: the j (ulps from lnb_base) with the most hits, and the j each listed variant
gives, with Excel GAMMALN(min) and the algdiv part.  Usage: lnbshape.py [set]"""
import sys

import algdiv_v as AV
import lnbfit as LF
import m1lib as M

VARS = {
    "d": dict(ar="d"),
    "x": dict(ar="x"),
    "e": dict(ar="e"),
    "e,gs=e": dict(ar="e", gsum="e"),
    "e,cr": dict(ar="e", aln="cr"),
    "d,lnb=e": dict(ar="d", lnb="e"),
}

if __name__ == "__main__":
    s = sys.argv[1] if len(sys.argv) > 1 else "a"
    tab = LF.table()
    print("shape            n   need(j:hits)            " + " ".join(f"{k:>7s}" for k in VARS))
    for (ss, sh, _), (L0, n, hits) in sorted(tab.items()):
        if ss != s:
            continue
        mx = max(hits.values())
        need = [j for j, h in hits.items() if h == mx]
        js = [M.ulp_steps(L0, float(AV.lnb(*sh, v))) for v in VARS.values()]
        lo = min(sh)
        print(f"{str(sh):18s} {n:3d} {str(need):10s} {mx:3d} at0={hits[0]:3d}  " + " ".join(f"{j:+7d}" for j in js)
              + f"   GL(min)={M.BE.GAMMALN(lo) if lo != 1 else 0:.17g} L0={L0:.17g}")
