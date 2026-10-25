"""Sub-ulp lnB scan: L = lnb_base + d * ulp(lnb_base) / S for d in [-S*W, S*W], i.e. an extended lnB. Per shape it
prints the d range that maximises the hits; a peak at d != 0 with more hits than d = 0 means Excel's lnB is not the
stored double.  Usage: fscan.py old|probe1|log1p|<own> [S=64] [W=1] [mode=m1] [maxrows=600]"""

import collections
import math
import sys

import m1lib as M
from lscan import rows_of


def fscan(rows, S=64, W=1, mode="m1", maxrows=600, lnb=M.lnb_base):
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["a"], r["b"])].append(r)
    res = {}
    for sh, rs in sorted(by.items()):
        rs = rs[:: max(1, len(rs) // maxrows)]
        L0 = lnb(*sh)
        u = M.mpf(math.ulp(L0)) / S
        pc = []
        for r in rs:
            if "l" in r:
                pc.append((0.0, M.sx("*", r["b"] - 1.0, r["l"])))
            else:
                pc.append(M.pieces(r["x"], r["a"], r["b"]))
        hits = {}
        for d in range(-S * W, S * W + 1):
            L = M.mpf(L0) + d * u
            hits[d] = sum(
                M.combine(tx, ty, L, sh[0], sh[1], mode) == r["excel"] for (tx, ty), r in zip(pc, rs)
            )
        res[sh] = (len(rs), hits)
    return res


if __name__ == "__main__":
    a = sys.argv[1:]
    S = int(a[1]) if len(a) > 1 else 64
    W = int(a[2]) if len(a) > 2 else 1
    mode = a[3] if len(a) > 3 else "m1"
    mx = int(a[4]) if len(a) > 4 else 600
    for sh, (n, hits) in fscan(rows_of(a[0]), S, W, mode, mx).items():
        best = max(hits.values())
        ds = [d for d, h in hits.items() if h == best]
        print(
            f"a={sh[0]!r:20} b={sh[1]!r:20} n={n:4d} at0={hits[0]:4d} best={best:4d} d in [{min(ds)}, {max(ds)}]/{S}"
        )
