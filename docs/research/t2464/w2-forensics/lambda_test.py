"""Hypothesis H-lambda: the power term x^a y^b is formed TOMS-brcomp style from lambda = a - (a+b)x (or
(a+b)y - b with y = fl(1-x)), so Excel's relative error carries -(x0/x - y0/y)*dlambda, x0 = a/(a+b), y0 = b/(a+b).
Regress Excel's error on the predicted term for each lambda form, per block (all w2f blocks with a, b > 1)."""
import csv, collections
import numpy as np, mpmath
from common import *
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if r["case"] == "t2464-w2f-unit-shape": continue
    i = int(r["cell"].split("!A")[1]) - 1
    size = 2048 if "xruns" in r["case"] else 1024
    g[(r["case"][10:], i // size, r["cum"])].append(r)
M = mpmath.mpf
def forms(x, a, b):
    X, A, B = M(x), M(a), M(b)
    y = 1 - x
    lam_true = A - (A + B) * X
    s = A + B
    out = {}
    ab = fl53(fl64(s))  # a+b stored
    out["a-(a+b)x"] = M(x87("-", a, x87("*", ab, x))) - lam_true
    out["(a+b)y-b"] = M(x87("-", x87("*", ab, y), b)) - lam_true
    out["a-(a+b)x ext"] = M(fl53(A - fl64(M(ab) * X))) - lam_true  # product kept in extended
    out["(a+b)y-b ext"] = M(fl53(fl64(M(ab) * M(y)) - B)) - lam_true
    sl = (A / s) / X - (B / s) / (1 - X)
    return {k: float(-sl * v) / U for k, v in out.items()}
for k in sorted(g):
    rs = g[k]
    a, b = float(rs[0]["a"]), float(rs[0]["b"])
    if a <= 1 or b <= 1: continue
    rs = [r for r in rs if float(r["x"]) <= a / (a + b) or True]
    y = np.array([float(r["rel"]) for r in rs])
    F = collections.defaultdict(list)
    for r in rs:
        for kk, v in forms(float(r["x"]), float(r["a"]), float(r["b"])).items(): F[kk].append(v)
    out = []
    for kk, v in F.items():
        v = np.array(v)
        if v.std() < 1e-9: out.append(f"{kk}: const"); continue
        c = np.polyfit(v, y, 1)[0]; rr = np.corrcoef(v, y)[0, 1]
        out.append(f"{kk}: slope {c:+.2f} r {rr:+.2f} sd {v.std():.2f}")
    print(f"{k[0]:8s} {k[1]} {'cdf' if k[2]=='1' else 'pdf'} a={a:.4g} b={b:.4g} sd(excel) {y.std():5.2f} | " + " | ".join(out))
