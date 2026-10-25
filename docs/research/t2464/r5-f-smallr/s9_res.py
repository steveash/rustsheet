"""Residual sequence of the best free-constant fit of one chain on matching runs.  Usage: s9_res.py SET tag chain"""
import sys

import r5lib as CL
import r5lib as R

src, tag, ch = sys.argv[1:4]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    sc, d, res = CL.scan(rs, m, n, ch, ret=True)
    print(blk, ch, sc, len(rs), "d", d)
    print("  " + " ".join(f"{v:.0f}" for v in res))
