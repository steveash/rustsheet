"""verify-r5-f-smallr: per-run tallies (shape, side, r band) of the frozen variants on given cases."""
from __future__ import annotations

import collections
import sys

import vparse as V
import vscore as S

if __name__ == "__main__":
    res = collections.defaultdict(S.Tally)
    for cid in sys.argv[1:]:
        for cell, x, m, n, e in V.rows(cid)[0]:
            side, r = S.side_r(x, m, n)
            key = (int(m), int(n), side, round(r, 2))
            for var in ("msd", "xs1", "dd"):
                res[(key, var)].add(S.grade(S.value(var, x, m, n), e))
    keys = sorted({k for k, _ in res})
    for k in keys:
        print(k, "  ".join(f"{v} {res[(k, v)]}" for v in ("msd", "xs1", "dd")))
