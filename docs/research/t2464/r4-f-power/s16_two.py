"""s16: second stored term that is a function of LN(x).  Data: per LN(x)-plateau mean of est (s13: Excel minus the
m1 model with the power product's own rounding removed).  Model: est_k = t1*rho_mine_k + t2*rho_G(c2*L_k + phi)
+ b0 + b1*k, with t1 in {0, 1} (the fl(c*LN(x)) term kept or not), c2 a half-integer in (0, 2h], G a power of two
(units) and phi in [0, G) (step G/256).  Prints the 8 best by residual sd.  Usage: s16_two.py SRC BLK"""
import collections
import sys
from fractions import Fraction

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex


def plateaus(src, want):
    rows = [r for r in P.load(src) if r["blk"] == want]
    m, n = rows[0]["d1"], rows[0]["d2"]
    pl = collections.OrderedDict()
    for r in rows:
        x = r["x"]
        s, e = C.pieces(x, m, n)
        side1 = s["mx"] <= n
        c, sg = (s["a"], 1) if side1 else (s["b"], -1)
        p = LDB(c) * s["Lx"]
        rho = sg * float((ex(S(p)) - ex(p)) * P.U)
        res = P.D_units(r) - float((M.lnF_model(x, m, n, "xl") - mpmath.log(r["exact"])) * P.U)
        pl.setdefault(float(s["Lx"]), []).append((res + rho, rho))
    Ls = list(pl)
    est = np.array([np.mean([v[0] for v in pl[k]]) for k in Ls])
    rm = np.array([np.mean([v[1] for v in pl[k]]) for k in Ls])
    return m, n, Ls, est, rm


def main():
    src, want = sys.argv[1], sys.argv[2]
    m, n, Ls, est, rm = plateaus(src, want)
    h = (m + n) / 2
    K = len(Ls)
    kk = np.arange(K)
    base = np.column_stack([np.ones(K), kk])
    out = []
    Lf = [Fraction(L) for L in Ls]
    for t1 in (0, 1):
        y = est - t1 * rm  # est = Excel's total rounding; t1 = 1 keeps the fl(c*LN(x)) term
        for p2 in range(1, int(4 * h) + 1):
            c2 = Fraction(p2, 2)
            vu = [c2 * L * 2**53 for L in Lf]
            v0 = vu[0]
            d = np.array([float(v - v0) for v in vu])
            for kG in range(5, 11):
                G = 2.0**kG
                for j in range(256):
                    ph = G * j / 256
                    t = (d + ph) / G
                    rho2 = (np.round(t) - t) * G
                    for t2 in (1, -1):
                        yy = y - t2 * rho2
                        cf, *_ = np.linalg.lstsq(base, yy, rcond=None)
                        sd = (yy - base @ cf).std()
                        out.append((sd, t1, float(c2), kG, j, t2))
    out.sort()
    yb = est - (base @ np.linalg.lstsq(base, est, rcond=None)[0])
    print(f"{want} plateaus {K}  est sd {est.std():.1f} (detrended {yb.std():.1f})")
    for o in out[:10]:
        print(f"  sd {o[0]:6.2f} t1={o[1]} c2={o[2]:g} G=2^{o[3]} phase {o[4]}/256 t2={o[5]:+d}")


if __name__ == "__main__":
    main()
