"""a/b sweep forensics: regress Excel's relative error (units 2^-53) on rounding errors of a-dependent intermediates.
 ds   = fl(a+b) - (a+b)            (absolute, units 2^-53), with predicted coefficient psi(a+b) in ln pdf if
        Gamma(a+b) is evaluated at the stored sum
 eGa, eGb, eGs = Excel GAMMALN error at a, b, fl(a+b)  (abs; ln pdf coefficient -1, -1, +1 for 1/B)
 rP   = rounding of fl(a*LN x) or fl(b*LN(1-x)) (the swept shape's product), rP1 the (shape-1) product
 rL   = rounding of fl(fl(a+b)*x)  (lambda = a - (a+b)x)."""
import csv, collections
import numpy as np, mpmath
from common import *
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if r["case"] != "t2464-w2f-absweep": continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(i // 1024, r["cum"])].append(r)
M = mpmath.mpf
allv = set(); xs = set()
for r in rows:
    if r["case"] != "t2464-w2f-absweep": continue
    a, b = float(r["a"]), float(r["b"]); allv |= {a, b, a + b}; xs |= {float(r["x"]), 1 - float(r["x"])}
GL = dict(zip(sorted(allv), xl("GAMMALN", sorted(allv)))); LN = dict(zip(sorted(xs), xl("LN", sorted(xs))))
def rnd(v): return float(M(fl53(fl64(v))) - v) / U
for k in sorted(g):
    rs = g[k]; swept = "a" if k[0] < 4 else "b"
    y = np.array([float(r["rel"]) for r in rs]); C = collections.defaultdict(list)
    for r in rs:
        x, a, b = float(r["x"]), float(r["a"]), float(r["b"]); s = a + b
        C["ds"].append(float(M(s) - (M(a) + M(b))) / U)
        C["eGa"].append(float(M(GL[a]) - mpmath.loggamma(M(a))) / U)
        C["eGb"].append(float(M(GL[b]) - mpmath.loggamma(M(b))) / U)
        C["eGs"].append(float(M(GL[s]) - mpmath.loggamma(M(s))) / U)
        if swept == "a":
            C["rP"].append(rnd(M(a) * M(LN[x]))); C["rP1"].append(rnd(M(a - 1) * M(LN[x])))
        else:
            C["rP"].append(rnd(M(b) * M(LN[1 - x]))); C["rP1"].append(rnd(M(b - 1) * M(LN[1 - x])))
        C["rL"].append(rnd(M(s) * M(x)))
    C = {kk: np.array(v) for kk, v in C.items() if np.std(v) > 1e-12}
    coef, s0, s1 = ols(y, C)
    a, b = float(rs[0]["a"]), float(rs[0]["b"]); s = a + b
    psi = float(mpmath.digamma(s))
    print(f"{'pdf' if k[1]=='0' else 'cdf'} sweep {swept} a~{a:.3g} b~{b:.3g} x={rs[0]['x']} psi(s)={psi:.2f} sd {s0:.2f}->{s1:.2f} | " +
          " ".join(f"{kk}={c:+.2f}({e:.2f})" for kk, (c, e) in coef.items()))
