"""m4: side-2 (mx > n) alternatives with the large ratio R = fl(mx/n) and S = fl(1 + R):
  cur : K2 - fl(b*Lx) - Lx - h*L(n/mx)                     (m1)
  nv1 : K1 + fl((a-1)*Lx) - h*LN(S)                         (no flip, naive)
  nv2 : K1 + fl(a*Lx) - Lx - h*LN(S)
  nv3 : K1 + fl(a*Lx) - Lx - fl(h*LN(S))
  nv4 : K1 + fl((a-1)*Lx) - fl(h*LN(S))
  K1 = lnG(h) - lnG(a) - lnG(b) + a ln(m/n), K2 = ... - b ln(m/n) (exact).  Usage: m4_naive.py SRC MINMAX"""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

LDB, S, ex, U = C.LDB, C.S, C.ex, P.U


def lnF(x, m, n, form):
    s, _ = C.pieces(x, m, n)
    a, _b, h = s["a"], s["b"], s["h"]
    if form == "cur" or s["mx"] <= n:
        return M.lnF_model(x, m, n, "xl")
    K1 = s3_order.kprime(m, n, True)
    Lx = s["Lx"]
    LS = C.LNx(C.st(1 + LDB(s["r2"])))
    if form == "nv1":
        v = S(LDB(a - 1) * Lx) - LDB(h) * LS
    elif form == "nv2":
        v = S(LDB(a) * Lx) - Lx - LDB(h) * LS
    elif form == "nv3":
        v = S(LDB(a) * Lx) - Lx - S(LDB(h) * LS)
    else:
        v = S(LDB(a - 1) * Lx) - S(LDB(h) * LS)
    return K1 + ex(v)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    for blk, rs, m, n in P.runs(src, minmax):
        if not all(m * r["x"] > n for r in rs):
            continue
        D = np.array([P.D_units(r) for r in rs])
        out = []
        for f in ("cur", "nv1", "nv2", "nv3", "nv4"):
            Dm = np.array([float((lnF(r["x"], m, n, f) - mpmath.log(r["exact"])) * U) for r in rs])
            out.append(f"{f} {(D - Dm).std():6.1f}")
        print(f"{blk:20s} | " + " | ".join(out))


if __name__ == "__main__":
    main()
