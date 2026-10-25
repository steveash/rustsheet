"""Required shift of the first sum: at the HKP11 best constant, for each cell the set of shifts j (units, -120..120 step 1)
of (C + H_k) that reproduce Excel's S2 (|res| <= 3).  Prints [lo, hi] of the admissible j per cell with eP, eLN(x),
LN/P change flags.  Usage: s4_jitter.py SRC tag [k0 k1]"""
import sys

import numpy as np

import r5lib as R
import s1_scan as S

LD = R.LD


def jitter(A, Hk, C):
    P = A["P"].astype(LD)
    js = np.arange(-120, 121, 1.0)
    ok = []
    for j in js:
        Ym = R.units_rel(S.chain("HKP11", C + LD(j) * LD(2.0) ** -53, Hk, P) - P, A["Y0"])
        ok.append(Ym)
    return js, np.array(ok)


if __name__ == "__main__":
    src, tag = sys.argv[1], sys.argv[2]
    k0, k1 = (int(sys.argv[3]), int(sys.argv[4])) if len(sys.argv) > 4 else (0, 80)
    for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
        A = R.run_arrays(rs, m, n)
        Hk = S.H_forms(A)["hL"]
        sc, d, c2, res = S.scan(A, Hk, "HKP11")
        Kh = LD(float(A["Kp"]))
        Klo = float((A["Kp"] - R.mpf(float(A["Kp"]))) * R.U)
        C = Kh + LD(d + Klo) * LD(2.0) ** -53
        js, Ym = jitter(A, Hk, C)
        good = np.abs(A["Y"][None, :] - Ym - c2) <= 3
        cc = A["a"] if A["side1"] else -A["b"]
        eP = (A["P"].astype(LD) - LD(cc) * A["Lx"].astype(LD)).astype(np.float64) * 2.0 ** 53
        print(blk, sc, len(res))
        for k in range(k0, min(k1, len(res))):
            jj = js[good[:, k]]
            lo, hi = (jj.min(), jj.max()) if len(jj) else (np.nan, np.nan)
            fl = ("L" if k and A["Lx"][k] != A["Lx"][k - 1] else " ") + ("P" if k and A["P"][k] != A["P"][k - 1] else " ")
            print(f"{k:3d} res {res[k]:+5.0f} j [{lo:+5.0f},{hi:+5.0f}] eP {eP[k]:+7.1f} {fl}")
