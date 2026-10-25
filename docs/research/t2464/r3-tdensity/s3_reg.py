"""In-sample a1/a2 consecutive-double runs: regress Excel's density error (units 2^-53, relative to the exact density
at the double t) on the relative effect of each input rounding (t*t, t*t/n, 1+q, n+t*t, y = n/(n+tt)) and on the
errors of Excel's own inner columns (POWER, F.DIST) at their actual arguments.  usage: s3_reg.py [blk-filter]"""
import sys
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD

mp = mpmath.mpf
U = 2.0**-53
flt = sys.argv[1] if len(sys.argv) > 1 else "run"
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if r["blk"].startswith(flt)]
by = defaultdict(list)
for r in rows:
    by[r["n"]].append(r)
names = ["tt", "q", "1q", "u", "y", "eD", "eE", "eI", "eG"]
for n in sorted(by):
    rr = by[n]
    h = mp(n + 1) / 2
    X, Y = [], []
    for r in rr:
        t = r["t"]
        with mpmath.workdps(40):
            T, N = mp(t), mp(n)
            tt = t * t
            q = TD.st(TD.LDB(tt) / TD.LDB(n))
            q1 = TD.st(TD.LDB(1) + TD.LDB(q))
            u = TD.st(TD.LDB(n) + TD.LDB(tt))
            y = TD.st(TD.LDB(n) / TD.LDB(u))
            ex = mp(TD.exact_pdf(t, n))
            ex = mpmath.exp(TD.exact_logpdf(t, n))
            e = float((r["excel"] - ex) / ex) / U
            f_tt = -h * (mp(tt) - T * T) / (N + T * T)
            f_q = -h * (mp(q) - mp(tt) / N) / (1 + mp(tt) / N)
            f_1q = -h * (mp(q1) - 1 - mp(q)) / mp(q1)
            f_u = -h * (mp(u) - N - mp(tt)) / mp(u)
            f_y = h * (mp(y) - N / mp(u)) / mp(y)
            pD = mpmath.power(mp(q1), -h)
            pE = mpmath.power(mp(u), -h)
            pI = mpmath.power(mp(y), h)
            fG = mpmath.sqrt(N ** N * mp(tt) / (mp(tt) + N) ** (N + 1)) / (mp(tt) * mpmath.beta(mp(0.5), N / 2))
            cols = [f_tt, f_q, f_1q, f_u, f_y, (r["pow1q"] - pD) / pD, (r["pownu"] - pE) / pE,
                    (r["powy"] - pI) / pI, (r["fdist"] - fG) / fG]
        X.append([float(c) / U for c in cols])
        Y.append(e)
    X, Y = np.array(X), np.array(Y)
    print(f"n={n:g} cells={len(Y)} excel err mean {Y.mean():+.2f} sd {Y.std():.2f}")
    print("   " + " ".join(f"{nm}:sd{X[:, j].std():.2f},r{np.corrcoef(X[:, j], Y)[0, 1]:+.2f}" for j, nm in enumerate(names)
                         if X[:, j].std() > 0))
    sel = [j for j in range(5) if X[:, j].std() > 1e-9]
    A = np.column_stack([X[:, sel], np.ones(len(Y))])
    coef, *_ = np.linalg.lstsq(A, Y, rcond=None)
    print("   inputs joint:", " ".join(f"{names[j]} {c:+.2f}" for j, c in zip(sel, coef)), f"c {coef[-1]:+.2f}",
          f"resid sd {(Y - A @ coef).std():.2f}")
