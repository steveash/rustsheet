"""Per-shape consistency of the x part's value: mean residual (D - dX, units 2^-53) per block, plus a linear trend
across each run (units per 256 cells).  A right x part gives the same mean on every block of a shape.
Usage: cmean.py [--src S] BLOCKREGEX 'knobs' ..."""
import re
import sys

import numpy as np

import mreg as MR
import r3lib as L
import scan as S

args = sys.argv[1:]
src = "old"
if args[0] == "--src":
    src, args = args[1], args[2:]
rows = S.load(src)
pat = re.compile(args[0])
for spec in args[1:] or [""]:
    print(f"== [{spec}]")
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk):
            continue
        idx = sorted(idx, key=lambda i: rows[i]["x"])
        r = MR.resid(rows, idx, S.parse(spec))
        t = np.arange(len(r)) / 256.0
        sl = np.polyfit(t, r, 1)[0]
        b1, b2, br = L.xpart(rows[idx[0]]["x"], rows[idx[0]]["a"], rows[idx[0]]["b"], S.parse(spec))
        print(f"  {blk:24s} {br} mean {np.mean(r):+9.2f}  trend {sl:+8.2f}/256  sd {np.std(r):6.2f}")
