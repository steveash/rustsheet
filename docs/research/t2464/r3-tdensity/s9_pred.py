"""In-sample a1/a2 runs: linearised prediction test.  For a candidate base b(t) (a double built from fl(t t) by x87
ops) and exponent w, the model f = K b^w (everything else exact) predicts the relative error
w * (b - b_exact)/b_exact with b_exact the same base at the exact t.  Reports sd of (Excel err - prediction - mean)
per block for each candidate; floor (final rounding only) is about 0.3-0.5.  usage: s9_pred.py"""
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD

mp = mpmath.mpf
U = 2.0**-53
L, st = TD.LDB, TD.st
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)


def cands(t, n):
    X, N = L(t), L(n)
    tt = L(st(X * X))
    tte = X * X  # extended product
    h = (n + 1) / 2
    T2, Nm = mp(t) ** 2, mp(n)
    return {  # name: (value as mpf, exact value, exponent)
        "y=fl(n/fl(n+tt))": (mp(st(N / L(st(N + tt)))), Nm / (Nm + T2), h),
        "y=fl(n/(n+tt)e)": (mp(st(N / (N + tt))), Nm / (Nm + T2), h),
        "y=fl(n/(n+tte)e)": (mp(st(N / (N + tte))), Nm / (Nm + T2), h),
        "w=fl(1+fl(tt/n))": (mp(st(L(1) + L(st(tt / N)))), 1 + T2 / Nm, -h),
        "w=fl(1+tt/n e)": (mp(st(L(1) + tt / N)), 1 + T2 / Nm, -h),
        "u=fl(n+tt)": (mp(st(N + tt)), Nm + T2, -h),
        "w=fl(fl(n+tt)/n)": (mp(st(L(st(N + tt)) / N)), 1 + T2 / Nm, -h),
        "q=fl(tt/n) log1p": (mp(st(tt / N)), T2 / Nm, -h * (T2 / Nm) / (1 + T2 / Nm)),
        "tt only": (mp(float(tt)), T2, -h * T2 / (Nm + T2)),
    }


res = defaultdict(dict)
for key in sorted(by):
    rr = by[key]
    n = key[0]
    errs, preds = [], defaultdict(list)
    for r in rr:
        ex = mpmath.exp(TD.exact_logpdf(r["t"], n))
        errs.append(float((r["excel"] - ex) / ex) / U)
        for k, (v, e, w) in cands(r["t"], n).items():
            preds[k].append(float(w * (v - e) / e) / U)
    errs = np.array(errs)
    for k, pv in preds.items():
        d = errs - np.array(pv)
        res[k][key] = d.std()
    res["(none)"][key] = errs.std()
keys = sorted(by)
print("block".ljust(20), " ".join(f"{k[1][4:]:>11s}" for k in keys))
for k, d in res.items():
    print(f"{k:20s}", " ".join(f"{d[kk]:11.2f}" for kk in keys))
