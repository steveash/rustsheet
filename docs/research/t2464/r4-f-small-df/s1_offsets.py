"""Per (shape, side, run) offset of Excel against M-xd (exact K): median and sd of (excel - M-xd)/excel in units.
A flat offset per (shape, side) is an error in Excel's constant K_side.  Usage: s1_offsets.py SET"""
import statistics as S
import sys

import slib as L

name = sys.argv[1]
rows = L.load(name)
blks: dict = {}
for r in rows:
    if max(r["d1"], r["d2"]) > 100:
        continue
    s, rr = L.side_r(r["x"], r["d1"], r["d2"])
    blks.setdefault((r["d1"], r["d2"], s, r["blk"]), []).append((r, rr))
for (m, n, s, b), rs in sorted(blks.items()):
    d = [float((L.mpf(r["excel"]) - L.mpf(L.mxd(r))) / L.mpf(r["excel"]) / L.U) for r, _ in rs]
    print(f"({m:g},{n:g}) side{s} {b:24s} r {rs[0][1]:.3f} n {len(d):3d} off med {S.median(d):+7.2f} sd {S.pstdev(d):6.2f}")
