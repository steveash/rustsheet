"""s14: which stored double is Excel's big term a function of?  The m1 residual (Excel - model, model without the
power product's rounding, i.e. with c*LN(x) unrounded) is grouped by the value of each candidate stored double;
a term that is a function of that double leaves only the smooth drift inside the groups.  Printed: mean
within-group sd and the number of groups."""
import collections
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex
src, minmax = sys.argv[1], float(sys.argv[2])
for blk, rs, m, n in P.runs(src, minmax):
    keys = collections.defaultdict(list)
    res = []
    for r in rs:
        x = r["x"]
        s, e = C.pieces(x, m, n)
        side1 = s["mx"] <= n
        c, sg = (s["a"], 1) if side1 else (s["b"], -1)
        p = LDB(c) * s["Lx"]
        rho = sg * float((ex(S(p)) - ex(p)) * P.U)
        res.append(P.D_units(r) - float((M.lnF_model(x, m, n, "xl") - mpmath.log(r["exact"])) * P.U) + rho)
        for k in ("Lx", "Lmx", "Lr1", "Lr2", "Lden", "mx", "r1", "r2", "den"):
            keys[k].append(float(s[k]))
        keys["x"].append(x)
    res = np.array(res)
    out = []
    for k, vs in keys.items():
        grp = collections.defaultdict(list)
        for v, rr in zip(vs, res):
            grp[v].append(rr)
        sds = [np.std(v) for v in grp.values() if len(v) > 1]
        out.append((np.mean(sds) if sds else np.nan, k, len(grp)))
    out.sort(key=lambda t: (np.isnan(t[0]), t[0]))
    print(f"{blk:20s} sd {res.std():6.1f} | " + " | ".join(f"{k} {v:.1f}/{ng}" for v, k, ng in out[:6]))
