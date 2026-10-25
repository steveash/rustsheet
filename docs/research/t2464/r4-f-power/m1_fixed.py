"""m1: fixed-coefficient log-space model, residual sd per run (no fitting except a per-run mean):
  lnF = K' + sg*fl(c*LN(x)) - LN(x) - h*L(r)    (c = a, sg = + for mx <= n; c = b, sg = - otherwise; sums extended)
L forms: 'xl' Excel log1p (series to 0.375, LN(fl(1+r)) above); 'm' exact log1p of the stored r; 'ln' LN(fl(1+r));
'dd' LN(fl(1 + big)) - LN(big), big = 1/r stored ratio of the other direction.
Usage: m1_fixed.py SRC MINMAX L..."""
import sys

import mpmath
import numpy as np

import cands as C
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
mpf = mpmath.mpf
U = P.U


def L_form(s, side1, form, n):
    r = s["r2"] if side1 else s["r1"]
    if form == "xl":
        return C.L1P(r)
    if form == "m":
        return C._ld(mpmath.log1p(mpf(r))) if hasattr(C, "_ld") else P.F.R3._ld_of(mpmath.log1p(mpf(r)))
    if form == "ln":
        return C.LNx(C.st(1 + LDB(r)))
    if form == "dd":
        big = s["r1"] if side1 else s["r2"]
        return C.LNx(C.st(1 + LDB(big))) - C.LNx(big)
    raise ValueError(form)


def lnF_model(x, m, n, form):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    Kp = s3_order.kprime(m, n, side1)
    Pp = S(LDB(c) * s["Lx"])
    rest = sg * Pp - s["Lx"] - LDB(h) * L_form(s, side1, form, n)
    return Kp + ex(rest)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    forms = sys.argv[3:] or ["xl"]
    for blk, rs, m, n in P.runs(src, minmax):
        D = np.array([P.D_units(r) for r in rs])
        out = []
        for f in forms:
            Dm = np.array([float((lnF_model(r["x"], m, n, f) - mpmath.log(r["exact"])) * U) for r in rs])
            res = D - Dm
            out.append(f"{f} {res.std():6.1f} (mean {res.mean():+6.1f})")
        print(f"{blk:20s} Dsd {D.std():6.1f} | " + " | ".join(out))


if __name__ == "__main__":
    main()
