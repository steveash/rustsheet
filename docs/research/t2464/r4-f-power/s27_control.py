"""s27: positive control for the K-phase scan (m9).  Synthetic 'Excel' values from the HKP11 structure with a known
K_in = K' + delta_true (extended) and F = EXP_V6(E)/x, on the real x runs of probe a; m9's scan must recover the
structure (sd ~ 0) at delta_true modulo the first sum's grid, and the wrong order (KPH) must not fit.
Usage: s27_control.py BLK DELTA_TRUE"""
import sys

import mpmath

import m7_flex as M7
import m9_kscan as M9
import plib as P
import s3_order

want, dtrue = sys.argv[1], float(sys.argv[2])
rows = [dict(r) for r in P.load("a") if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
K0 = s3_order.kprime(m, n, m * rows[0]["x"] <= n)
Kd = P.F.R3._ld_of(K0 + mpmath.mpf(dtrue) / P.U)
for r in rows:
    t, lx = M7.terms(r["x"], m, n, 0, 0)
    E = M7.value(dict(t, K=Kd), 0, "HKP", (1, 1))  # without the -ln x (lnx = 0 here)
    r["excel"] = float(P.F.R3.exp_v6(P.F.R3._ld_of(E), True) / P.F.LDB(r["x"]))
for order in ("HKP", "KPH"):
    out = sorted(M9.run_scan(rows, m, n, order, (1, 1), 2.0, 260))[:3]
    print(f"control {want} true delta {dtrue:+.0f} | {order}11 best " + " | ".join(f"delta {o[1]:+.0f} sd {o[0]:.2f}" for o in out))
