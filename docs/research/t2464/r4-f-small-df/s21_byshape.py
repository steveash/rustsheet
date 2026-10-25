"""Per (shape, side, run) counts of the frozen M-sd and CR on one set (diagnostic). Usage: s21_byshape.py SET"""
import sys

import sdmodel as SD
import slib as L

rows = L.load(sys.argv[1])
g: dict = {}
for r in rows:
    g.setdefault(r["blk"], []).append(r)
for b, rs in g.items():
    t = L.F.tally([(SD.value(r["x"], r["d1"], r["d2"]), r["excel"]) for r in rs])
    c = L.F.tally([(r["cr"], r["excel"]) for r in rs])
    s, rr = L.side_r(rs[0]["x"], rs[0]["d1"], rs[0]["d2"])
    print(f"{b:24s} side{s} r {rr:.3f} M-sd {L.F.fmt(t)} | CR {L.F.fmt(c)}")
