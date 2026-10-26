"""Per-block detail for one model: exact / w15 counts, ln(out) magnitude and the spread of the admissible C
midpoints (units 2^-53) after subtracting the model's X; a wide spread means unmodelled x-dependence.
Usage: vblock.py CASE_SUFFIX MODEL"""
from __future__ import annotations

import sys

import numpy as np

import vdata as V
import vmodel as M
import vscore as S


def main(suf, model):
    rows, res, pb, br = S.run([suf])
    for blk, idx in V.blocks(rows).items():
        r0 = rows[idx[0]]
        X = [M.xpart(rows[i]["x"], r0["a"], r0["b"], S.MODELS[model])[0] for i in idx]
        ivs = [S.e_interval(rows[i]["excel"]) for i in idx]
        c = np.array([float((iv[0] - x) / S.U) + (iv[1] + iv[2]) / 2 for iv, x in zip(ivs, X) if iv])
        lo = np.percentile(c, 5)
        hi = np.percentile(c, 95)
        w = np.mean([iv[2] - iv[1] for iv in ivs if iv])
        print(f"{blk:8s} ({r0['a']:g},{r0['b']:g}) {br[idx[0]]} lnE {float(ivs[0][0]):+8.3f} reach {len(c)}/{len(idx)} "
              f"exact {pb[(model, blk)]['x']}/{pb[(model, blk)]['n']}  C mid sd {c.std():6.2f} 5-95% {hi - lo:6.2f}"
              f"  mean width {w:.2f}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
