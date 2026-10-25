"""Bin a block's residual r = D - dX (rmsfit, knobs) by the phase of a candidate quantity on a grid: phase =
frac(Q/g) for Q in {2kv_s, T = k ln(k/M), s = d v_s, dv on the 2kv grid}.  Usage: phase.py BLOCK knobs side"""
import math
import sys

import mpmath
import numpy as np

import mreg as MR
import r3lib as L
import scan as S

rows = S.load("old")
blk, spec, side = sys.argv[1], sys.argv[2], sys.argv[3]
idx = L.blocks(rows)[blk]
r = MR.resid(rows, idx, S.parse(spec))
r = r - r.mean()
LD = L.LDB
qs = {}
for j, i in enumerate(idx):
    x, a, b = rows[i]["x"], rows[i]["a"], rows[i]["b"]
    k, n, nk, ab = L.shape(a, b)
    q = L.st(1 - LD(x))
    M = L.st(LD(n) * LD(x)) if side == "p" else L.st(LD(n) * LD(q))
    kk = k if side == "p" else nk
    d = kk - M
    sm = L.st(LD(kk) + LD(M))
    v = L.st(LD(d) / LD(sm))
    e2 = 2 * mpmath.mpf(kk) * v
    u = math.ulp(float(e2))
    T = mpmath.mpf(kk) * mpmath.log(mpmath.mpf(kk) / M)
    lnr = mpmath.mpf(L.LN(L.st(LD(kk) / LD(M))))
    dd = mpmath.mpf(d) * d
    for nm, Q, g in (("2kv", e2, u), ("T", T, u), ("Tv", 2 * mpmath.mpf(kk) * mpmath.atanh(v), u),
                     ("kLN", kk * lnr, math.ulp(float(kk * lnr))), ("dd", dd, math.ulp(float(dd))),
                     ("dv", mpmath.mpf(d) * v, math.ulp(float(mpmath.mpf(d) * v))),
                     ("4kv", 2 * e2, math.ulp(float(2 * e2)))):
        qs.setdefault(nm, []).append(float((Q / g) % 1))
for nm, ph in qs.items():
    ph = np.array(ph)
    bins = np.minimum((ph * 8).astype(int), 7)
    print(nm, " ".join(f"{np.mean(r[bins == b_]):+6.2f}" for b_ in range(8)))
