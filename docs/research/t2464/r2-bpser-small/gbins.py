"""Which ln Gamma(a0) form does Excel's bpser use, as a function of a0? Unswapped w_bpser cells, sub-branch 1 < b0 < 8,
z = fl64(alx - fl(g + fl(GL(b0) - GL(a0+b0)))) (zforms.py best tree); per a0 bin, cells exact with g = GAMMALN(a0)
vs g = fl(gamln1(a0) - LN(a0)) vs g = fl(GAMMALN(fl(1 + a0)) - LN(a0)), on the cells where the g values differ.
usage: gbins.py [sets=runs,shapes] [sub=1<b0<8]"""
import sys
from collections import Counter

import lib
from lib import GAMMALN, LN, mpf, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
sub = "bpser:" + kw.get("sub", "1<b0<8")
rows = lib.load_probe(tuple(kw.get("sets", "runs,shapes").split(",")))
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], lib.V())
K = lib.Model(lib.bx.BEST, lib.V())
for q in res:  # make sure Excel's GAMMALN(fl(1 + a0)) is in the cache (engine model via the harness)
    if q[3]:
        lib.GAMMALN(sx("+", min(q[3][-1][1], q[3][-1][2]), 1.0))
lib.bx.fetch_missing()
c = Counter()
for r, q in zip(rows, res):
    if q[2] != "w_bpser" or not q[3]:
        continue
    fn, a, b, x = q[3][-1]
    if lib.sub_branch(fn, a, b) != sub:
        continue
    lo, hi = min(a, b), max(a, b)
    s = lib.Model.series_w3(a, b, x)
    alx = mpf(sx("*", a, LN(x)))
    hits = []
    gv = {"gl": GAMMALN(lo), "g1ln": sx("-", K.gamln1(lo), LN(lo)), "glp1": sx("-", GAMMALN(sx("+", lo, 1.0)), LN(lo))}
    for nm, g in gv.items():
        G = GAMMALN(hi) if hi >= 1 else sx("-", K.gamln1(hi), LN(hi))
        L = sx("+", g, sx("-", G, GAMMALN(sx("+", lo, hi))))
        hits.append(sx("*", lib.EXPX(lib.fl64(alx - mpf(L))), s) == r["excel"])
    bn = f"{int(lo * 10) / 10:.1f}"
    c[(bn, "n")] += 1
    if len(set(gv.values())) > 1:
        c[(bn, "differ")] += 1
        for nm, h in zip(gv, hits):
            c[(bn, nm)] += h
for bn in sorted({k[0] for k in c}):
    print(bn, {k[1]: v for k, v in sorted(c.items()) if k[0] == bn})
