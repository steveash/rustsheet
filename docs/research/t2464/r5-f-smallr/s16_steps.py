"""Steps of V = ln(Excel x) - P against the steps of H and P (units), and the P / LN(x) change flags.
Usage: s16_steps.py SET tag [n]"""
import sys

import numpy as np

import r5lib as CL
import r5lib as R

src, tag = sys.argv[1], sys.argv[2]
N = int(sys.argv[3]) if len(sys.argv) > 3 else 60
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    toks, arr, meas, K0 = CL.prepare(rs, m, n, "C+H!+P!+X")
    Y0 = meas[0]
    Y = np.array([float((v - Y0) * R.U) for v in meas])  # ln(Excel) (X in chain): add back ln x exactly below
    lx = np.array([float((CL.mpmath.log(CL.mpmath.mpf(q["x"])) - CL.mpmath.log(CL.mpmath.mpf(rs[0]["x"]))) * R.U)
                   for q in rs])
    H = (arr["H"] - arr["H"][0]).astype(float) * 2 ** 53
    P = (arr["P"] - arr["P"][0]).astype(float) * 2 ** 53
    V = Y + lx - P
    print(blk, "x0", rs[0]["x"])
    print(" k   dV     dH     dP   V-H")
    for k in range(1, min(N, len(rs))):
        print(f"{k:3d} {V[k] - V[k - 1]:6.0f} {H[k] - H[k - 1]:6.1f} {P[k] - P[k - 1]:6.0f} {V[k] - H[k]:6.0f}")
