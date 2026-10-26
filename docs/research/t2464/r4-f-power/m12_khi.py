"""m12: K_hi from pieces.  lnF = fl(fl(K_hi + H) + P) + (K' - K_hi) - ln x, the constant's remainder added exactly
(a double-double constant, as M2's hi/lo split).  K_hi candidates:
  'near'  nearest double of K'
  'G3'    fl(fl(Gh - Ga) - Gb) + R  (G = nearest doubles of the exact lnGamma; R the side's ratio term, nearest double,
          added last, stored)
  'G3r'   R added first: fl(fl(fl(Gh + R) - Ga) - Gb)
  'Gab'   fl(Gh - fl(Ga + Gb)) then + R
Prints per run: sd, cells off the mode, mode.  Usage: m12_khi.py SRC MINMAX"""
import sys

import mpmath
import numpy as np

import cands as C
import m7_flex as M7
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
mpf = mpmath.mpf


def khi(m, n, side1, form):
    a, b, h = mpf(m) / 2, mpf(n) / 2, (mpf(m) + mpf(n)) / 2
    Gh, Ga, Gb = (LDB(float(mpmath.loggamma(v))) for v in (h, a, b))
    R = LDB(float((a if side1 else -b) * mpmath.log(mpf(m) / mpf(n))))
    if form == "near":
        return LDB(float(s3_order.kprime(m, n, side1)))
    if form == "G3":
        return S(S(S(Gh - Ga) - Gb) + R)
    if form == "G3r":
        return S(S(S(Gh + R) - Ga) - Gb)
    if form == "Gab":
        return S(S(Gh - S(Ga + Gb)) + R)
    raise ValueError(form)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    forms = ("near", "G3", "G3r", "Gab")
    for blk, rs, m, n in P.runs(src, minmax):
        side1 = m * rs[0]["x"] <= n
        K0 = s3_order.kprime(m, n, side1)
        D = np.array([P.D_units(r) for r in rs])
        line = []
        for f in forms:
            Kh = khi(m, n, side1, f)
            lo = K0 - ex(Kh)
            d = []
            for r, Dv in zip(rs, D):
                t, lx = M7.terms(r["x"], m, n, 0, 0)
                v = M7.value(dict(t, K=Kh), lx, "HKP", (1, 1)) + lo
                d.append(Dv - float((v - mpmath.log(r["exact"])) * P.U))
            d = np.array(d)
            vals, cnt = np.unique(np.round(d), return_counts=True)
            mv = vals[np.argmax(cnt)]
            line.append(f"{f} sd {d.std():5.1f} off {int(np.sum(np.abs(d - mv) > 3)):3d} mode {mv:+4.0f}")
        print(f"{blk:20s} s{1 if side1 else 2} | " + " | ".join(line))


if __name__ == "__main__":
    main()
