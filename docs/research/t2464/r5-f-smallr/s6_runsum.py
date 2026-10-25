"""Per-run HKP11 free-constant score (s1_scan.scan, H = -hL extended) with the grids of S1, P, S2.
Usage: s6_runsum.py SET"""
import sys

import r5lib as R
import s1_scan as S

src = sys.argv[1]
tot = N = 0
for blk, rs, m, n in R.runs(src):
    A = R.run_arrays(rs, m, n)
    sc, d, c2, res = S.scan(A, S.H_forms(A)["hL"], "HKP11")
    S1 = float(A["Y0"])
    P = A["P"][0]
    tot += sc
    N += len(rs)
    print(f"{blk:28s} {sc:3d}/{len(rs)} S1={S1:8.2f} g{R.ulpu(S1):4.0f} P={P:8.2f} g{R.ulpu(P):4.0f} "
          f"S2={S1 + P:8.2f} g{R.ulpu(S1 + P):4.0f} r={A['r'][0]:.3f}")
    sys.stdout.flush()
print("total", tot, "of", N)
