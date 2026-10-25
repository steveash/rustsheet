"""How much of H rides in the stored sum?  Model: fl(fl(C + alpha H) + P) + (1 - alpha) H (the rest extended,
outside the roundings).  alpha scanned; best free C per alpha.  Usage: s17_alpha.py SET tag"""
import sys

import numpy as np

import r5lib as CL
import r5lib as R

LD = R.LD
src, tag = sys.argv[1], sys.argv[2]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    toks, arr, meas, K0 = CL.prepare(rs, m, n, "C+H!+P!")
    a, b, h = m / 2, n / 2, (m + n) / 2
    out = []
    for al in [0, 0.1, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.9, 1.0, 1.1, a / h, b / h, (a - 1) / h, (b - 1) / h,
               (a + 1) / h, (b + 1) / h]:
        arr2 = {"H": arr["H"] * LD(al), "P": arr["P"], "X": arr["H"] * LD(1 - al)}
        tk = CL.parse("C+H!+P!+X")
        Y0 = meas[0]
        Y = np.array([float((v - Y0) * R.U) for v in meas])
        ds = np.arange(-320, 320, 0.25)
        C = (LD(float(K0)) + ds.astype(LD) * LD(2.0) ** -53)[:, None]
        Ym = R.units_rel(CL.evaluate(tk, arr2, C), Y0)
        res = Y[None, :] - Ym
        res = res - np.median(res, axis=1)[:, None]
        sc = np.sum(np.abs(res) <= 3, axis=1).max()
        out.append(f"{al:.3f}:{sc}")
    print(blk, f"a/h={a / h:.3f} b/h={b / h:.3f}", " ".join(out))
