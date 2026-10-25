"""Visibly-off (not w15) cells of LT1: hold-out probe (frozen offsets from lt1_offsets.json) and old cells.tsv, with
ulps from Excel, CR's ulps and the branch; plus read-back checks of the hold-out's t.  usage: s18_miss.py"""
import json
import math
from collections import Counter

import tdlib as TD
from s17_final import branch

TD.LT1_OFF.update({float(k): v for k, v in json.loads((TD.HERE / "lt1_offsets.json").read_text()).items()})
hold = TD.load_probe("hold")
print("hold read-back mismatches:", sum(r["t_rb"] != r["t"] for r in hold))
for name, rows in (("hold", hold), ("old", [r for r in TD.load_old() if r["n"] >= 3])):
    c = Counter()
    for r in rows:
        m = TD.lt1(r["t"], r["n"])
        if not TD.w15(m, r["excel"]):
            cr = TD.exact_pdf(r["t"], r["n"])
            u = math.ulp(r["excel"])
            c[(r.get("blk", r["case"][-30:]), r["n"])] += 1
            if c[(r.get("blk", r["case"][-30:]), r["n"])] <= 2:
                print(f"{name} {r.get('blk', r['case'][-30:])} n={r['n']:g} t={r['t']!r} br={branch(r)} "
                      f"LT1 {(m - r['excel']) / u:+.0f} ulp, CR {(cr - r['excel']) / u:+.0f} ulp")
    print(name, dict(c))
