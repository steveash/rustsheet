"""b10: is Excel's stirlerr at large v an extended-precision (64-bit) difference of lgamma and the Stirling terms?
At the x = 1 cells, E = -1/2 L(r) + 1/2 ln(w/2pi) + st(h) - st(a) - st(b) with st(v) exact for v <= 15 (Loader's
table) and, above, one of the variants below with every operation rounded to 64 bits (R64, round to nearest):
  g1: R64(R64(R64(lgamma(v+1)) - R64(R64(v+1/2) * R64(ln v))) + v) - R64(ln sqrt(2pi))
  g0: lgamma(v) - (v-1/2) ln v + v - ln sqrt(2pi), same rounding
  g1d / g0d: as g1 / g0 but the final value rounded to double
Also a planted control: Excel replaced by g1's own F (EXP V6), to show the count reaches the whole set.
Usage: b10_stx.py SET"""

from __future__ import annotations

import sys

import numpy as np

import b6_x1 as X
import blib as B

mp, U, F = B.mp, B.U, B.F
R64 = F.R64
LS2PI = mp.log(mp.sqrt(2 * mp.pi))


def st_var(v, kind):
    if v <= 15:
        return X.stx(v)
    V = mp.mpf(v)
    if kind.startswith("g1"):
        lg, c = R64(mp.loggamma(V + 1)), R64(V + mp.mpf(0.5))
    else:
        lg, c = R64(mp.loggamma(V)), R64(V - mp.mpf(0.5))
    t = R64(lg - R64(c * R64(mp.log(V))))
    t = R64(R64(t + V) - R64(LS2PI))
    return mp.mpf(float(t)) if kind.endswith("d") else t


def E_of(m, n, kind):
    a, b, h, side, w, o = X.shape(m, n)
    r = B.st(B.LDB(w) / B.LDB(o))
    base = -B.C.ex(B.SL.L_xl(r)) / 2 + mp.log(mp.mpf(w) / (2 * mp.pi)) / 2
    if kind == "exact":
        return base + X.stx(h) - X.stx(a) - X.stx(b)
    return base + st_var(h, kind) - st_var(a, kind) - st_var(b, kind)


if __name__ == "__main__":
    x1 = [r for r in B.load(sys.argv[1]) if r["x"] == 1.0]
    for kind in ("exact", "g1", "g0", "g1d", "g0d"):
        hits, miss = 0, []
        for r in x1:
            lo, hi = X.target(r)
            E = E_of(r["d1"], r["d2"], kind)
            ok = lo <= E <= hi
            hits += ok
            miss.append(0.0 if ok else float((E - (lo if E < lo else hi)) * U))
        ms = np.array(miss)
        print(f"{kind:6s} hits {hits}/{len(x1)} rms miss {np.sqrt(np.mean(ms**2)):.2f}")
