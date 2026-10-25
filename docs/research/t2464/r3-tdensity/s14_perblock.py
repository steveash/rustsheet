"""In-sample a1/a2: free constant fitted per block (not per df) for one loader_dt option set; prints each block's
best count, its offset (units of 2^-53 from the exact Loader constant) and the bd0 branch mix.  usage:
s14_perblock.py '<opts>'"""
import json
import sys
from collections import defaultdict


import tdlib as TD

L = TD.LDB
o = json.loads(sys.argv[1])
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)
tot = 0
for key in sorted(by):
    rr = by[key]
    n = key[0]
    nb, pos, ivs, C0 = TD.fit_free_C(rr, n, o)
    tot += nb
    br = defaultdict(int)
    for r in rr:
        br[TD.cache_u(r["t"], n, o)[2]] += 1
    print(f"n={n:<3g} {key[1]:18s} {nb:3d}/{len(rr)} @{pos:+7.2f}  branch {dict(br)}")
print("total", tot)
