"""Correlation of V - H (V = ln(Excel x) - P, H = -hL: the part a stored first sum must explain, rounding included)
with the stored-product rounding errors of s14_regs.  Usage: s15_corr.py SET tag"""
import sys

import numpy as np

import r5lib as CL
import r5lib as R
import s14_regs as S

src, tag = sys.argv[1], sys.argv[2]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    toks, arr, meas, K0 = CL.prepare(rs, m, n, "C+H!+P!")
    Y0 = meas[0]
    Y = np.array([float((v - Y0) * R.U) for v in meas])
    H = (arr["H"] - arr["H"][0]).astype(float) * 2 ** 53
    P = (arr["P"] - arr["P"][0]).astype(float) * 2 ** 53
    V = Y - P - H
    rg = S.regs(rs, m, n)
    c = sorted(((abs(np.corrcoef(V, z)[0, 1]) if z.std() > 0 else 0, k, np.polyfit(z, V, 1)[0] if z.std() > 0 else 0)
                for k, z in rg.items()), reverse=True)
    print(f"{blk:28s} sd {V.std():5.1f}", [(round(a, 2), k, round(b, 2)) for a, k, b in c[:5]])
