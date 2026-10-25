"""Per sheet: scan a constant exponent offset c (units 2^-53) on E = tx + ty - L1d (extended sum), brcomp logs.
A sharp 100% peak at c != 0 means the composition is right and Excel's lnB differs from L1d by -c.
Usage: cscan.py sheet [sheet...]; prints best c per x zone and overall.
"""

import collections
import sys

import mpmath
import numpy as np

from forms import ingredients
from gen1 import zone
from lib import D, R64, exp_v6, load_probe, mpf, xl

U = mpf(2) ** -53


def prep(rows, lnb_override=None):
    ing = ingredients(rows)
    out = []
    for r, g in zip(rows, ing):
        x = r["x"]
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        tx = mpf(D(g["am1"] * g["LNd"]))
        ty = mpf(D(g["bm1"] * ly))
        L = mpf(D(g["ga"] + D(g["gb"] - g["gab"]))) if lnb_override is None else lnb_override
        out.append((R64(tx + R64(ty - L)), r["excel"], zone(x), r))
    return out


def scan(pre, cs):
    res = []
    for c in cs:
        h = collections.Counter()
        for E, v, zn, _ in pre:
            h[zn] += exp_v6(R64(E + c * U)) == v
        res.append((c, h))
    return res


if __name__ == "__main__":
    cs = np.arange(-3, 3.001, 0.1)
    for sh in sys.argv[1:]:
        rows = [r for r in load_probe("probe1") if r["case"] == sh]
        pre = prep(rows)
        n = collections.Counter(p[2] for p in pre)
        res = scan(pre, cs)
        best = max(res, key=lambda t: sum(t[1].values()))
        c0 = [t for t in res if abs(t[0]) < 1e-9][0]
        print(
            sh,
            rows[0]["a"],
            rows[0]["b"],
            "n",
            dict(n),
            "| c=0:",
            sum(c0[1].values()),
            dict(c0[1]),
            "| best c %.1f:" % best[0],
            sum(best[1].values()),
            dict(best[1]),
        )
