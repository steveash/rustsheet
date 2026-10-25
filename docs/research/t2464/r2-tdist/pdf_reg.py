"""Forensics on T.DIST(t, n, FALSE) where tt = t*t and n + tt are exact (t0969-t-df-ratio, the Exact-sheet runs
t = 1 + k/2^20 and 0.25 + k/2^22, n = 3, 4, 5, 7). Excel's error e (units 2^-53, relative to the exact density) is
regressed on the effect, in the same units, of each candidate intermediate's rounding:
  dLNu   -h * (LN(u) - ln u)          (if Excel used Excel's LN(u), u = n + tt)
  dLN1q  -h * (LN(fl(1+fl(tt/n))) - ln(1 + tt/n))
  dq     -h * (fl(tt/n) - tt/n) / (1 + tt/n)   (rounding of q = tt/n feeding an exact log)
  dE_u   fl(h*LN(u)) - h*LN(u) (product rounding, sign -)
  dEXPu  Excel's EXP(-fl(h LN u)) relative error vs exact exp of the same argument
usage: pdf_reg.py"""
import math

import mpmath
import numpy as np

import tlib

bx = tlib.bx
U = 2.0**-53
rows = [r for r in tlib.load(("pdf",)) if "df-ratio" in r["case"] and r["n"] in (3, 4, 5, 7)]
rows = [r for r in rows if r["t"] * r["t"] == mpmath.mpf(r["t"]) ** 2]
print("cells with exact tt:", len(rows))
for n in (3.0, 4.0, 5.0, 7.0):
    rr = [r for r in rows if r["n"] == n]
    h = (n + 1) / 2
    X, Y = [], []
    for r in rr:
        t = r["t"]
        tt = t * t
        u = n + tt
        assert mpmath.mpf(u) == n + mpmath.mpf(t) ** 2
        ex = tlib.exact_pdf(t, n)
        e = (r["excel"] - ex) / ex / U
        with mpmath.workdps(40):
            lu = mpmath.log(mpmath.mpf(u))
            Lu = bx.LN(u)
            q = tt / n
            l1q = mpmath.log1p(mpmath.mpf(tt) / n)
            L1q = bx.LN(1.0 + q)
            p = bx.sx("*", h, Lu)
            ee = bx.EXP(-p)
            cols = [
                float(-h * (Lu - lu)) / U,
                float(-h * (L1q - l1q)) / U,
                float(-h * (mpmath.mpf(q) - mpmath.mpf(tt) / n) / (1 + mpmath.mpf(tt) / n)) / U,
                float(-(p - h * mpmath.mpf(Lu))) / U,
                float((ee - mpmath.exp(-mpmath.mpf(p))) / mpmath.exp(-mpmath.mpf(p))) / U,
            ]
        X.append(cols)
        Y.append(e)
    X, Y = np.array(X), np.array(Y)
    names = ["dLNu", "dLN1q", "dq", "dE_u", "dEXPu"]
    print(f"n={n:g} cells={len(Y)} excel err mean {Y.mean():+.2f} sd {Y.std():.2f}")
    for j, nm in enumerate(names):
        c = np.corrcoef(X[:, j], Y)[0, 1] if X[:, j].std() > 0 else float("nan")
        print(f"   {nm}: sd {X[:, j].std():.2f} corr {c:+.3f}")
    A = np.column_stack([X, np.ones(len(Y))])
    coef, *_ = np.linalg.lstsq(A, Y, rcond=None)
    res = Y - A @ coef
    print("   joint coef", " ".join(f"{nm} {c:+.2f}" for nm, c in zip(names + ["1"], coef)), f"resid sd {res.std():.2f}")
