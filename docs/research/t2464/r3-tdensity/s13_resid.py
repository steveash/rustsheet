"""In-sample a1/a2: Excel's error minus a model's error (model = loader_dt with options, constant = the best shared
constant from the interval scan), forward-selected on extra rounding features (s11_fsel.feats).  Prints the slope of
Excel's error on the model's own error (1 = the model's roundings are Excel's).  usage: s13_resid.py '<opts>' blk"""
import json
import sys
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD
from s11_fsel import feats

L = TD.LDB
U = 2.0**-53
o = json.loads(sys.argv[1])
flt = sys.argv[2]
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if flt in r["blk"]]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)
for key in sorted(by):
    rr = by[key]
    n = key[0]
    nb, pos, ivs, C0 = TD.fit_free_C(rr, n, o)
    Y, M, F = [], [], []
    for r in rr:
        ex = mpmath.exp(TD.exact_logpdf(r["t"], n))
        Y.append(float((r["excel"] - ex) / ex) / U)
        m = TD.loader_dt(r["t"], n, C0 + L(pos) * TD.U53, o)
        M.append(float((m - ex) / ex) / U)
        F.append(feats(r["t"], n))
    Y, M = np.array(Y), np.array(M)
    names = list(F[0])
    Xm = np.array([[d[k] for k in names] for d in F])
    A = np.column_stack([M, np.ones(len(Y))])
    c, *_ = np.linalg.lstsq(A, Y, rcond=None)
    R = Y - M
    line = f"n={n:<3g} {key[1]:16s} exact {nb}/{len(rr)} sd(Y) {Y.std():.2f} slope {c[0]:+.2f} sd(Y-M) {R.std():.2f}"
    chosen = []
    for _ in range(3):
        best = None
        for j, nm in enumerate(names):
            if j in chosen or Xm[:, j].std() < 1e-9:
                continue
            A = np.column_stack([Xm[:, chosen + [j]], np.ones(len(Y))])
            cc, *_ = np.linalg.lstsq(A, R, rcond=None)
            s = (R - A @ cc).std()
            if best is None or s < best[0]:
                best = (s, j, cc)
        chosen.append(best[1])
        line += f" | +{names[best[1]]}({best[2][-2]:+.2f})->{best[0]:.2f}"
    print(line, flush=True)
