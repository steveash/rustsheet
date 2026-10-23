"""Score log1p candidates against pinned l (pins.tsv) per region: bit-exact / within 1 ulp / n.
Usage: score_impls.py name... (candidates from cands.py)."""
import math
import sys
from collections import defaultdict

import cands
import pin

REG = [(0, 1e-6), (1e-6, 1e-3), (1e-3, 0.0625), (0.0625, 0.125), (0.125, 0.25), (0.25, 0.3), (0.3, 0.375 + 1e-17),
       (0.375 + 1e-17, 0.6)]


def score(rows, names=None, regs=REG):
    names = names or list(cands.IMPLS)
    res = {}
    for nm in names:
        f = cands.IMPLS[nm]
        per = defaultdict(lambda: [0, 0, 0])
        for p, n, k, l in rows:
            if k in ("none", "multi"):
                continue
            for lo, hi in regs:
                if lo <= p < hi:
                    v = f(-p)
                    c = per[(lo, hi)]
                    c[2] += 1
                    c[0] += v == l
                    c[1] += abs(v - l) <= math.ulp(l)
        res[nm] = per
    return res


def show(res, regs=REG):
    print("%-22s" % "cand" + "".join("%17s" % f"[{lo:.3g},{hi:.3g})" for lo, hi in regs))
    for nm, per in res.items():
        print("%-22s" % nm + "".join("%17s" % ("%d/%d/%d" % tuple(per[r]) if per[r][2] else "-") for r in regs))


if __name__ == "__main__":
    rows = [r for nm in sys.argv[1:] for r in pin.load(nm)]
    show(score(rows))
