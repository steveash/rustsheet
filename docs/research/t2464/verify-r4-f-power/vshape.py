"""Per-shape within-4-ulp / within-15-digit counts of CR, m1, dd, dd-S1x on a source.  Usage: vshape.py SRC"""
import collections
import sys

import vlib as V
import vscore as S

rs = S.rows(sys.argv[1])
S.prebatch(rs)
g = collections.defaultdict(lambda: collections.defaultdict(list))
for r in rs:
    for var in ("CR", "m1", "dd", "dd-S1x"):
        g[(r["d1"], r["d2"])][var].append((V.model(r["x"], r["d1"], r["d2"], var), r["excel"]))
for k, d in g.items():
    print(f"{k!s:14s} " + " | ".join(f"{v} %d/%d/%d of %d" % V.tally(p) for v, p in d.items()))
