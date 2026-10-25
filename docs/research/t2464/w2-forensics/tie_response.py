"""Response of Excel's relative error to the tie rounding of fl(1-x), read from runs where 1-x has one more bit than
x's grid allows (x in [0.25,0.5) or [0.125,0.25) etc.): c = (mean err at ties rounding 1-x up - mean at ties rounding
down)/ (2 * |rel rounding|), compared with model responses."""
import csv, collections, sys, struct, math
import numpy as np, mpmath
from common import *
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if "xruns" not in r["case"]: continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(r["case"][-1], i // 2048, r["cum"])].append(r)
M = mpmath.mpf
print("blk cum a b x  c_meas(+-se)  b  b-1  (1-x)(a+b-x*pdf/I)  [pdf/I]")
for k in sorted(g):
    rs = g[k]
    a, b = float(rs[0]["a"]), float(rs[0]["b"])
    d = []; e = []
    for r in rs:
        x = float(r["x"]); om = 1 - x
        rel = float((M(om) - (1 - M(x))) / (1 - M(x))) / U
        d.append(rel); e.append(float(r["rel"]))
    d = np.array(d); e = np.array(e)
    if d.std() == 0:
        continue
    up, dn, ex = e[d > 1e-9], e[d < -1e-9], e[np.abs(d) < 1e-9]
    mag = np.abs(d[np.abs(d) > 1e-9]).mean()
    c = (up.mean() - dn.mean()) / (2 * mag)
    se = math.sqrt(up.var() / len(up) + dn.var() / len(dn)) / (2 * mag)
    x = float(rs[len(rs) // 2]["x"])
    I = exact_cdf(x, a, b); p = exact_pdf(x, a, b)
    ratio = float(p / I)
    print(k, f"a={a} b={b} x={x:.4f}  c={c:+.2f}({se:.2f}) exact-mean {ex.mean():+.2f} up {up.mean():+.2f} dn {dn.mean():+.2f}  b={b} incbd={(1-x)*(a+b-x*ratio):.2f} x*pdf/I={x*ratio:.3f}")
