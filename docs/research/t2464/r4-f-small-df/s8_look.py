"""Print one run: x, Excel's D (units relative to exact), and local rounding errors of candidate pieces
(units relative to each piece's ulp... here absolute in 2^-53): e1 = fl(1+r)-(1+r), eL = L_xl(r) - log1p(r) (exact r),
er = fl(r) - r_exact, eLN = LN(x) - ln x. Usage: s8_look.py d1 d2 tag"""
import sys

import slib as L

m, n, tag = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (m, n) and r["blk"].endswith(tag + ")")]
U = L.U
for r in rows:
    x = r["x"]
    s, rr = L.side_r(x, m, n)
    rex = L.mpf(m) * x / n if s == 1 else n / (L.mpf(m) * x)
    D = L.D_units(r["excel"], r)
    Dm = L.D_units(L.mxd(r), r)
    p, q = L.L_xl(rr).as_integer_ratio()
    eL = float((L.mpf(p) / q - L.mpmath.log1p(rex)) / U)
    er = float((L.mpf(rr) - rex) / U)
    e1 = float((L.mpf(L.st(L.LDB(1) + L.LDB(rr))) - 1 - rex) / U)
    eLN = float((L.mpf(float(L.LN(x))) - L.mpmath.log(x)) / U)
    print(f"{x!r:24s} D {D:+6.2f} M-xd {Dm:+6.2f} | er {er:+6.3f} e1 {e1:+6.3f} eL {eL:+6.3f} eLN {eLN:+6.3f}")
