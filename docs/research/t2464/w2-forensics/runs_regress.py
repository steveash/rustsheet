"""Regress Excel's relative error (units 2^-53) along the x runs on candidate intermediate rounding errors.
Base regressors (all in units of 2^-53, absolute errors of log-domain quantities = relative errors of the result):
  dLx   = LN(x) - ln x                     (Excel's LN)
  d1x   = (fl(1-x) - (1-x))/(1-x)          (relative error of the stored 1-x)
  dL1   = LN(fl(1-x)) - ln(fl(1-x))
  rA1   = fl((a-1)*LN x) - (a-1)*LN x      rA = same with a
  rB1   = fl((b-1)*LN(1-x)) - ...          rB = same with b
  rS1   = fl(rA1-term + rB1-term) rounding  rS = same with a, b
Coefficient on dLx is the multiplier of ln x in the exponent (a or a-1), on d1x and dL1 that of ln(1-x).
"""
import csv, collections, math, sys
import numpy as np, mpmath
from common import *

rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
case = sys.argv[1] if len(sys.argv) > 1 else "t2464-w2f-xruns-1"
blocks = collections.defaultdict(list)
for r in rows:
    if r["case"] != case: continue
    i = int(r["cell"].split("!A")[1]) - 1
    blocks[(i // 2048, r["cum"])].append(r)
xs = sorted({float(r["x"]) for r in rows if r["case"] == case})
oms = [1 - x for x in xs]
LN = dict(zip(xs, xl("LN", xs))); LN1 = dict(zip(oms, xl("LN", oms)))
M = mpmath.mpf
def rnd(v):  # rounding error of storing exact mpf v as a double, absolute
    return float(M(fl53(fl64(v))) - v)
for key in sorted(blocks):
    rs = blocks[key]
    a, b = float(rs[0]["a"]), float(rs[0]["b"])
    y = np.array([float(r["rel"]) for r in rs])
    C = collections.defaultdict(list)
    for r in rs:
        x = float(r["x"]); X = M(x); om = 1 - x
        Lx, L1 = LN[x], LN1[om]
        C["dLx"].append(float(M(Lx) - mpmath.log(X)) / U)
        C["d1x"].append(float((M(om) - (1 - X)) / (1 - X)) / U)
        C["dL1"].append(float(M(L1) - mpmath.log(M(om))) / U)
        for tag, ma, mb in (("1", a - 1, b - 1), ("", a, b)):
            pa = M(ma) * M(Lx); pb = M(mb) * M(L1)
            C["rA" + tag].append(rnd(pa) / U)
            C["rB" + tag].append(rnd(pb) / U)
            C["rS" + tag].append(rnd(M(fl53(fl64(pa))) + M(fl53(fl64(pb)))) / U)
    C = {k: np.array(v) for k, v in C.items()}
    coef, s0, s1 = ols(y, C)
    print(f"block {key[0]} {'cdf' if key[1]=='1' else 'pdf'} a={a} b={b} x0={float(rs[0]['x']):.6g} n={len(y)} sd {s0:.2f} -> {s1:.2f}")
    print("   " + "  ".join(f"{k}={c:+.2f}({s:.2f})" for k, (c, s) in coef.items()))
    # simple single correlations
    print("   corr " + "  ".join(f"{k}={np.corrcoef(y, v)[0,1]:+.2f}" for k, v in C.items() if v.std() > 0))
