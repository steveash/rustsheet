"""BINOM.DIST(0, n, p, TRUE) = EXP(fl(n * l)), l = Excel's log1p(-p) (p <= 0.375) or LN(fl(1-p)) (w5, r2-log1p):
the visibly-off standing cells of the BINOM k = 0 shortcut (not the kernel) and how many the model fixes."""
import math

import implib as lib
import bratio_excel as bx
import excel_log1p as el


def main() -> None:
    fails, _ = lib.failing_cells()
    eng = {(c["case"], c["key"]): c for c in fails}
    t = dict(n=0, vis=0, fixed=0, broke=0, exact=0)
    for r in lib.all_cells():
        if r["fn"] not in ("BINOM.DIST", "BINOMDIST") or "/t2464-" in r["case"]:
            continue
        a = r["args"]
        if len(a) < 4 or math.trunc(a[0]) != 0 or not (0 < a[2] < 1) or math.trunc(a[1]) < 1:
            continue
        n, p = float(math.trunc(a[1])), a[2]
        l = el.log1p_series(-p) if p <= 0.375 else bx.LN(1.0 - p)
        m = bx.EXP(n * l)
        e = r["excel"]
        c = eng.get((r["case"], r["key"]))
        g = c["engine"] if c else e
        gv = isinstance(g, float) and lib.within15(e, g)
        t["n"] += 1
        t["vis"] += not gv
        t["fixed"] += (not gv) and lib.within15(e, m)
        t["broke"] += gv and not lib.within15(e, m)
        t["exact"] += m == e
    print("BINOM.DIST(0,n,p) (pmf and cdf), standing pre-T2464 cells:", t)


if __name__ == "__main__":
    main()
