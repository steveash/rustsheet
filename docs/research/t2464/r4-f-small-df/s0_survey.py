"""Survey: per run (blk), Excel's D = (excel - exact)/exact in units (mean, sd), and CR / M-xd counts, small-df sets.
Usage: s0_survey.py SET [SET ...]"""
import statistics as S
import sys

import slib as L

for name in sys.argv[1:]:
    rows = [r for r in L.load(name) if name == "mya" or L.small(r) or r["d1"] <= 4 or r["d2"] <= 4]
    blks: dict = {}
    for r in rows:
        blks.setdefault(r["blk"], []).append(r)
    print(f"# {name}: {len(rows)} rows")
    tot = {"CR": [], "M-xd": []}
    for b, rs in blks.items():
        D = [L.D_units(r["excel"], r) for r in rs]
        cr = [(r["cr"], r["excel"]) for r in rs]
        mx = [(L.mxd(r), r["excel"]) for r in rs]
        tot["CR"] += cr
        tot["M-xd"] += mx
        s = L.side_r(rs[0]["x"], rs[0]["d1"], rs[0]["d2"])
        print(f"{b:28s} side{s[0]} r {s[1]:.3f} n {len(rs):3d} D mean {S.mean(D):+7.2f} sd {S.pstdev(D):6.2f}"
              f" | CR {L.F.fmt(L.F.tally(cr))} | M-xd {L.F.fmt(L.F.tally(mx))}")
    for k, v in tot.items():
        print("TOTAL", k, L.F.fmt(L.F.tally(v)))
