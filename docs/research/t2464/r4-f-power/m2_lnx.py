"""m2: which ln x feeds the stored power product and the separate -ln x term.
lnF = K' + sg*fl(c*LX1) - LX2 - h*L(r) (L = Excel log1p, sums extended); LX1, LX2 in:
  'LN'   Excel LN (FLDLN2/FYL2X stored to a double);  'L64' the same unstored (64-bit);  'ex' exact ln x;
  'cr' the correctly rounded double ln x (the engine's POWER uses libm ln).
Prints the per-run residual sd (Excel - model) for each (LX1, LX2).  Usage: m2_lnx.py SRC MINMAX"""
import collections
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U
R3 = P.F.R3


def lx_of(x, mode):
    if mode == "LN":
        return C.LNx(x)
    if mode == "L64":
        return R3.LN64(x)
    if mode == "cr":
        return LDB(float(mpmath.log(mpmath.mpf(x))))
    return R3._ld_of(mpmath.log(mpmath.mpf(x)))


def lnF(x, m, n, m1, m2):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    Kp = s3_order.kprime(m, n, side1)
    Pp = S(LDB(c) * lx_of(x, m1))
    rest = sg * Pp - lx_of(x, m2) - LDB(h) * M.L_form(s, side1, "xl", n)
    return Kp + ex(rest)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    combos = [("LN", "LN"), ("cr", "LN"), ("cr", "cr"), ("cr", "L64"), ("L64", "LN")]
    tot = collections.defaultdict(list)
    for blk, rs, m, n in P.runs(src, minmax):
        D = np.array([P.D_units(r) for r in rs])
        out = []
        for c1, c2 in combos:
            Dm = np.array([float((lnF(r["x"], m, n, c1, c2) - mpmath.log(r["exact"])) * U) for r in rs])
            sd = (D - Dm).std()
            tot[(c1, c2)].append(sd)
            out.append(f"{c1}/{c2} {sd:6.1f}")
        print(f"{blk:20s} | " + " | ".join(out))
    print("median:", {f"{k[0]}/{k[1]}": round(float(np.median(v)), 2) for k, v in tot.items()})


if __name__ == "__main__":
    main()
