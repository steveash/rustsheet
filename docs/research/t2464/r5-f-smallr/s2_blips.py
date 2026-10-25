"""Per-cell view of the HKP11 best fit on one run: residual, S1 = fl(C + H) in grid units, S1 + P frac on the S2 grid,
P's own rounding error eP = P - c*LN(x) (units), LN(x) change flags.  Usage: s2_blips.py SRC tag"""
import sys

import numpy as np

import r5lib as R
import s1_scan as S

LD = R.LD
src, tag = sys.argv[1], sys.argv[2]
for blk, rs, m, n in R.PL.runs(src, 0, tag):
    A = R.run_arrays(rs, m, n)
    Hk = S.H_forms(A)["hL"]
    sc, d, c2, res = S.scan(A, Hk, "HKP11")
    Kh = LD(float(A["Kp"]))
    Klo = float((A["Kp"] - R.mpf(float(A["Kp"]))) * R.U)
    C = Kh + LD(d + Klo) * LD(2.0) ** -53
    S1 = R.f64(C + Hk)
    g1 = R.ulpu(S1[0])
    P = A["P"]
    cc = A["a"] if A["side1"] else -A["b"]
    eP = (P.astype(LD) - LD(cc) * A["Lx"].astype(LD)).astype(np.float64) * 2.0 ** 53
    S2x = S1.astype(LD) + P.astype(LD)
    g2 = R.ulpu(float(S2x[0]))
    ext = (C + Hk).astype(np.float64)
    print(blk, sc, "g1", g1, "g2", g2)
    for k in range(len(res)):
        fr1 = float(((C + Hk)[k] - LD(np.floor((C + Hk)[k] / LD(g1 * 2.0**-53)) * LD(g1 * 2.0**-53))) * LD(2.0**53) / LD(g1))
        q2 = float(S2x[k] / LD(g2 * 2.0 ** -53)) % 4
        lxch = "L" if k and A["Lx"][k] != A["Lx"][k - 1] else " "
        pch = "P" if k and P[k] != P[k - 1] else " "
        print(f"{k:3d} res {res[k]:+5.0f} S1/g1 frac {fr1:.3f} (S1+P)/g2 mod4 {q2:.2f} eP {eP[k]:+7.1f} {lxch}{pch} r {A['r'][k]:.6f}")
