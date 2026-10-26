"""Pooled per (shape, side) offset of Excel against M-xd (exact K) on probe mya (64 points: both runs of the side),
next to the exact K pieces. Usage: s3_ktable.py [maxd]"""
import statistics as S
import sys

import slib as L

maxd = float(sys.argv[1]) if len(sys.argv) > 1 else 10
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]
g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s), []).append(float((L.mpf(r["excel"]) - L.mpf(L.mxd(r))) / L.mpf(r["excel"]) / L.U))
for (m, n, s), d in sorted(g.items()):
    K = L.K_exact(m, n, s)
    lnb = L.lnB_exact(m, n)
    print(f"({m:g},{n:g}) s{s} n {len(d):3d} off {S.median(d):+6.2f} mean {S.mean(d):+6.2f} sd {S.pstdev(d):5.2f} |"
          f" K {float(K):+.6f} -lnB {float(-lnb):+.6f} B {float(L.mpmath.exp(lnb)):.6f}")
