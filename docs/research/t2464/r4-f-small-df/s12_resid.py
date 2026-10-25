"""Per cell, Excel's admissible E interval under F = EXP(E) (units, absolute, relative to exact ln F), minus the
candidate xn64 chain's own offset; next to the rounding error of storing that chain's E to a double (eEst) and
the stored-P rounding (eP). Usage: s12_resid.py d1 d2 tag [form]"""
import sys

import numpy as np

import slib as L

m, n, tag = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
form = sys.argv[4] if len(sys.argv) > 4 else "xn64"
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (m, n) and r["blk"].endswith(tag + ")")]
U = L.LDB(2.0) ** -53
grid = np.arange(-16, 16.0001, 0.01).astype(L.LDB)


def fr(v):
    p, q = L.LDB(v).as_integer_ratio()
    return L.mpf(p) / q


out = []
for r in rows:
    x = r["x"]
    Es = L.mpmath.log(r["exact"])
    E0 = L.ld(Es)
    v = L.exp_v6_vec(E0 + grid * U)
    ok = grid[v == r["excel"]]
    A, K, _ = L.chain(x, m, n, form)
    Em = L.ld(K) + A
    em = float((fr(Em) - Es) / L.U)
    eEst = float((L.mpf(L.st(Em)) - fr(Em)) / L.U)
    lo, hi = (float(ok.min()) - em, float(ok.max()) - em) if len(ok) else (np.nan, np.nan)
    out.append((lo, hi, eEst))
    print(f"{x!r:24s} need [{lo:+6.2f},{hi:+6.2f}] eEst {eEst:+6.2f} {'IN' if lo <= eEst <= hi else '  '}")
