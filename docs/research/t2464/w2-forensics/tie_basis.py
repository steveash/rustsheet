"""Tie responses of Excel vs basis responses for the pdf and cdf blocks (x runs with 1-x losing bits)."""
import csv, collections, math
import numpy as np, mpmath
from common import *
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if "xruns" not in r["case"]: continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(r["case"][-1], i // 2048, r["cum"])].append(r)
M = mpmath.mpf
res = {}
for k in sorted(g):
    rs = g[k]; a, b = float(rs[0]["a"]), float(rs[0]["b"])
    xs = np.array([float(r["x"]) for r in rs]); e = np.array([float(r["rel"]) for r in rs])
    d = np.array([float((M(1 - x) - (1 - M(x))) / (1 - M(x))) / U for x in xs])
    if d.std() == 0: continue
    # restrict to the direct side of the mean
    keep = xs <= a / (a + b)
    up, dn = (d > 1e-9) & keep, (d < -1e-9) & keep
    if up.sum() < 20: continue
    c = (e[up].mean() - e[dn].mean()) / (2 * np.abs(d[up | dn]).mean())
    se = math.sqrt(e[up].var() / up.sum() + e[dn].var() / dn.sum()) / (2 * np.abs(d[up | dn]).mean())
    x = float(np.median(xs[keep]))
    I = exact_cdf(x, a, b); p = exact_pdf(x, a, b); r_ = float(p / I)
    if k[2] == "0":
        print(f"pdf {k[:2]} a={a} b={b} x={x:.4f} c={c:+.2f}({se:.2f})  b-1={b-1:.2f} -(a-1)(1-x)/x={-(a-1)*(1-x)/x:.2f}")
    else:
        print(f"cdf {k[:2]} a={a} b={b} x={x:.4f} c={c:+.2f}({se:.2f})  b={b:.2f} -(1-x)p/I={-(1-x)*r_:.2f} -a(1-x)/x={-a*(1-x)/x:.2f} x*p/I={x*r_:.2f} (1-x)={1-x:.3f}")
