"""m3: forms of the x-power on the mx > n side (and side 1 for reference), all products of Excel's stored LN(x):
  cur : -fl(b*Lx) - Lx            (m1)
  ah  : +fl(a*Lx) - fl(h*Lx) - Lx (from (a-1) ln x - h ln(mx) - h L(n/mx))
  a1h : +fl((a-1)*Lx) - fl(h*Lx)
  b1  : -fl((b+1)*Lx)
  hmx : +fl(a*Lx) - Lx - fl(h*LN(mx)) + h ln m  (h ln(mx) via LN(mx); constant part exact)
Side 1 forms: cur fl(a*Lx) - Lx; a1 fl((a-1)*Lx).  Usage: m3_xform.py SRC MINMAX"""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

mpf, LDB, S, ex, U = mpmath.mpf, C.LDB, C.S, C.ex, P.U


def xpart(s, x, m, n, form):
    a, b, h = s["a"], s["b"], s["h"]
    Lx = s["Lx"]
    if s["mx"] <= n:
        if form == "a1":
            return S(LDB(a - 1) * Lx), 0
        return S(LDB(a) * Lx) - Lx, 0
    if form == "ah":
        return S(LDB(a) * Lx) - S(LDB(h) * Lx) - Lx, 0
    if form == "a1h":
        return S(LDB(a - 1) * Lx) - S(LDB(h) * Lx), 0
    if form == "b1":
        return -S(LDB(b + 1) * Lx), 0
    if form == "hmx":
        return S(LDB(a) * Lx) - Lx - S(LDB(h) * s["Lmx"]), mpf(h) * mpmath.log(mpf(m))
    return -S(LDB(b) * Lx) - Lx, 0


def lnF(x, m, n, form):
    s, _ = C.pieces(x, m, n)
    side1 = s["mx"] <= n
    Kp = s3_order.kprime(m, n, side1)
    xp, kadd = xpart(s, x, m, n, form)
    rest = xp - LDB(s["h"]) * M.L_form(s, side1, "xl", n)
    return Kp + kadd + ex(rest)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    forms = sys.argv[3:] or ["cur", "ah", "a1h", "b1", "hmx", "a1"]
    for blk, rs, m, n in P.runs(src, minmax):
        D = np.array([P.D_units(r) for r in rs])
        out = []
        for f in forms:
            Dm = np.array([float((lnF(r["x"], m, n, f) - mpmath.log(r["exact"])) * U) for r in rs])
            out.append(f"{f} {(D - Dm).std():6.1f}")
        print(f"{blk:20s} | " + " | ".join(out))


if __name__ == "__main__":
    main()
