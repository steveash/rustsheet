"""Log-form blocks: does the residual jump by one ulp of a stored intermediate?  For each cell, prints the C
midpoint residual (units 2^-53, minus the block median) next to ulp(t), ulp(LN(r)) * k and ulp(M-k) per side.
Usage: vtsplit.py CASE_SUFFIX BLOCK"""
from __future__ import annotations

import math
import sys
from collections import Counter

import numpy as np

import vdata as V
import vmodel as M
import vscore as S


def main(suf, blk):
    rows = V.load_case(f"t2513-{suf}" if suf.startswith("verify") else f"t2513-r3-loader-bd0-{suf}")
    idx = V.blocks(rows)[blk]
    r0 = rows[idx[0]]
    M.LN.need([r for i in idx for r in M.ratios_needed(rows[i]["x"], r0["a"], r0["b"])])
    c = []
    for i in idx:
        iv = S.e_interval(rows[i]["excel"])
        X = M.xpart(rows[i]["x"], r0["a"], r0["b"], S.MODELS["r3"])[0]
        c.append(float((iv[0] - X) / S.U) + (iv[1] + iv[2]) / 2)
    c = np.array(c) - np.median(c)
    print("residual histogram (rounded):", sorted(Counter(np.round(c).astype(int)).items()))
    for side, (k, Mm) in enumerate(M.means(rows[idx[0]]["x"], r0["a"], r0["b"])):
        r = float(M.LD(k) / M.LD(Mm))
        ln = M.LN(r)
        t = float(M.LD(k) * M.LD(ln))
        print(f"side {side}: k {k} M {Mm:.6f} r {r:.6f} LN {ln:.6f} t {t:.6f}  ulp(t) {math.ulp(t) * 2**53:.1f}u"
              f"  k*ulp(LN) {k * math.ulp(ln) * 2**53:.1f}u  ulp(M-k) {math.ulp(Mm - k) * 2**53:.1f}u")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
