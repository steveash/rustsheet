"""Excel's constant at x = 1 (sw sweeps; M-xd's E = K - h L(r) there): offset = (excel - M-xd)/excel in units, next
to candidate constant errors (cand - exact K, units absolute). Usage: s2_const.py [d1 ...]"""
import sys

import slib as L

U = L.U
rows = [r for r in L.load("sw") if r["x"] == 1.0]
want = {float(v) for v in sys.argv[1:]} or None
TK = L.B.BX.Kernel(L.B.BX.TOMS_XL)
L.xl("GAMMALN", sorted({v for r in rows for v in (r["d1"] / 2, r["d2"] / 2, (r["d1"] + r["d2"]) / 2)}))
L.xl("GAMMA", sorted({v for r in rows for v in (r["d1"] / 2, r["d2"] / 2, (r["d1"] + r["d2"]) / 2)}))
seen = set()
for r in sorted(rows, key=lambda r: (r["d1"], r["d2"])):
    m, n = r["d1"], r["d2"]
    if (m, n) in seen or (want and m not in want and n not in want):
        continue
    seen.add((m, n))
    a, b, h = m / 2, n / 2, (m + n) / 2
    side, _ = L.side_r(1.0, m, n)
    off = float((L.mpf(r["excel"]) - L.mpf(L.mxd(r))) / L.mpf(r["excel"]) / U)
    lnb = L.lnB_exact(m, n)
    e_gl = float((L.mpf(L.GL(a)) + L.GL(b) - L.GL(h) - lnb) / U)
    e_toms = float((L.mpf(TK.betaln(a, b)) - lnb) / U)
    e_ga = float((L.mpmath.log(L.mpf(L.GA(a)) * L.GA(b) / L.GA(h)) - lnb) / U)
    print(f"({m:g},{n:g}) s{side} off {off:+7.2f} | -lnB errs: GL {-e_gl:+8.2f} TOMS {-e_toms:+8.2f} GAMMA {-e_ga:+8.2f}")
