"""Misses of the forward power-sum model: position of the model's exact pre-rounding value r + s (doubled)
relative to Excel's l, in ulps of l; and the same for the exact 2 atanh(r).  Usage: miss_fwd.py"""
import math

import mpmath

import coef_scan as C
import gen_scan as G
import r2lib as R

M, A_ = G.M, G.A_


def pre(r, N=14):
    t2 = M(r, r)
    p = r
    s = None
    for k in range(1, N + 1):
        p = M(p, t2)
        tm = M(p, C.BASE[k])
        s = tm if s is None else A_(s, tm)
    return 2 * (R.mpf(r) + R.mpf(s)), s


f = C.make(C.BASE)
for p, r, l, h in G.load_rows():
    if f(r) != l:
        v, s = pre(r)
        u = math.ulp(l)
        ex = 2 * mpmath.atanh(R.mpf(r))
        print(f"p={p:.10f} -r={-r:.5f} model-pre d={float((v - R.mpf(l)) / u):+.4f}  exact d={float((ex - R.mpf(l)) / u):+.4f}"
              f"  s err (ulp s) {float((R.mpf(s) - (mpmath.atanh(R.mpf(r)) - R.mpf(r))) / R.mpf(math.ulp(s))):+.2f}")
