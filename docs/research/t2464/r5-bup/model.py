"""T-3568 r5-bup: the frozen model of L140 bup+bgrat with n >= 2 bup terms (frozen before the hold-out was scored).

At the oriented call (a0, b0, x0, y0), n = int(b0), c = b0 - n (c = 1, n - 1 at an integer b0):
    ret  = x87(P * fl(x0 * y0) / c)                 P = Excel's BETA.DIST(y0, c, a0, FALSE) (r3-bgrat, n = 1)
    S    = 1 + sum_{i=1}^{n-1} t_i (doubles, from i = 1 up, TOMS's k and eps = 1e-15 break)
    t_i  = fl(R_i * EXP(fl(i * LN(y0))))            Excel's EXP and LN (r2-bratio); y0^i through an exponent
    R_i  = fl(R_{i-1} * fl(fl(apb + l) / fl(ap1 + l))), l = i - 1, apb = fl(c + a0), ap1 = fl(c + 1)
    W    = fl(ret * S)  (one rounding of the extended product)
    cell = RN(W + D)                                D = Excel's BETA.DIST(x0, a0, c, TRUE)
Chosen on t3568-r5-bup-tune's BETA.DIST rows (D by bgrat) among the forms in score.py, powforms.py, powforms2.py;
structural (no constant fitted). The callers' cells (F, BINOM, NEGBINOM) use the same composition at their own
oriented call."""
from __future__ import annotations

import mpmath
import numpy as np

import score as s
from common import xlk

bx = xlk.bx
LD = np.longdouble


def bup_sum(c: float, a0: float, y0: float, x0: float, n: int) -> float:
    apb, ap1 = c + a0, c + 1.0
    lny = bx.LN(y0)
    k = s.k_terms(c, a0, y0, x0, n)
    r, w = 1.0, 1.0
    for i in range(1, n):
        l = i - 1
        r = r * ((apb + l) / (ap1 + l))
        t = r * bx.EXP(i * lny)
        w = w + t
        if l >= k and t <= s.EPS * w:
            break
    return w


def cell(r: dict, d: float | None = None) -> float:
    ret = LD(r["P"]) * LD(r["x0"] * r["y0"]) / LD(r["c"])
    w = float(ret * LD(bup_sum(r["c"], r["a0"], r["y0"], r["x0"], r["n"])))
    return float(mpmath.mpf(w) + (r["D"] if d is None else d))
