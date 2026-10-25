"""List M-sd's misses (not within 15 digits) on a set, with Excel's and M-sd's error against the exact density.
Usage: s1_misses.py SET [REGION]"""
import sys

import elib as E

rows = E.L.load(sys.argv[1])
reg = sys.argv[2] if len(sys.argv) > 2 else None
for r in sorted(rows, key=lambda r: (r["d1"], r["d2"], r["x"])):
    if reg and E.region(r) != reg:
        continue
    v = E.SD.value(r["x"], r["d1"], r["d2"])
    if not E.L.w15(v, r["excel"]):
        side, rr = E.L.side_r(r["x"], r["d1"], r["d2"])
        print(f"{r['d1']:g},{r['d2']:g} x={r['x']!r:24} s{side} r={rr:.3g} xl={E.L.D_units(r['excel'], r):9.1f} msd={E.L.D_units(v, r):9.1f}  {r['blk']}")
