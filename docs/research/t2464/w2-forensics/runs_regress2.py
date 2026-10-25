"""Leaner regression per x-run block (direct side only): Excel's rel err on
 d1x (rel rounding of fl(1-x)), dLx (Excel LN(x) abs error), dL1 (Excel LN(fl(1-x)) abs error vs ln(fl(1-x))),
 rA = rounding of fl(a*LN x), rB = rounding of fl(b*LN(1-x)) [and a-1, b-1 variants for the pdf]."""
import csv, collections, sys
import numpy as np, mpmath
from common import *
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if "xruns" not in r["case"]: continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(r["case"][-1], i // 2048, r["cum"])].append(r)
xs = sorted({float(r["x"]) for r in rows if "xruns" in r["case"]})
LN = dict(zip(xs, xl("LN", xs))); oms = [1 - x for x in xs]; LN1 = dict(zip(oms, xl("LN", oms)))
M = mpmath.mpf
def rnd(v): return float(M(fl53(fl64(v))) - v) / U
for k in sorted(g):
    rs = [r for r in g[k] if float(r["x"]) <= float(r["a"]) / (float(r["a"]) + float(r["b"]))]
    if len(rs) < 100: continue
    a, b = float(rs[0]["a"]), float(rs[0]["b"])
    pdf = k[2] == "0"
    ma, mb = (a - 1, b - 1) if pdf else (a, b)
    y = np.array([float(r["rel"]) for r in rs]); C = collections.defaultdict(list)
    for r in rs:
        x = float(r["x"]); om = 1 - x
        C["d1x"].append(float((M(om) - (1 - M(x))) / (1 - M(x))) / U)
        C["dLx"].append(float(M(LN[x]) - mpmath.log(M(x))) / U)
        C["dL1"].append(float(M(LN1[om]) - mpmath.log(M(om))) / U)
        C["rA"].append(rnd(M(ma) * M(LN[x])))
        C["rB"].append(rnd(M(mb) * M(LN1[om])))
    C = {kk: np.array(v) for kk, v in C.items() if np.std(v) > 0}
    coef, s0, s1 = ols(y, C)
    print(f"{'pdf' if pdf else 'cdf'} {k[:2]} a={a} b={b} n={len(y)} mult=({ma:g},{mb:g}) sd {s0:.2f}->{s1:.2f} | " +
          " ".join(f"{kk}={c:+.2f}({s:.2f})" for kk, (c, s) in coef.items()))
