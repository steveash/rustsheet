"""General sheets: brcomp-style log choice (x <= 0.375: ln y = log1p(-x); 0.375 < x < 0.625: LN x, LN y;
x >= 0.625: ln x = log1p(-y)) against plain LN, times groupings/storage of E = tx + ty - lnB. Per x zone.

Usage: gen1.py probe1C0 [probe1C3 ...]   (or 'old' for the 2,035 beta_exact pdf cells, region a<1,b<=8)
"""

import collections
import itertools
import sys

import mpmath

from forms import ingredients
from lib import D, R64, exp_v6, load_pdf, load_probe, mpf

ST = {"d": D, "e": R64}


def zone(x):
    return "a<=.375" if x <= 0.375 else ("b<.5" if x < 0.5 else ("c<.625" if x < 0.625 else "d>=.625"))


def run(rows):
    ing = ingredients(rows)
    z = collections.Counter()
    n = collections.Counter()
    for r, g in zip(rows, ing):
        x = r["x"]
        zn = zone(x)
        n[zn] += 1
        y = mpf(1.0 - x)
        l1p_y = mpf(D(mpmath.log1p(-y)))  # accurate ln x from y (x >= 0.625)
        lxs = {"LNx": g["LNd"], "brc_lx": l1p_y if x >= 0.625 else g["LNd"]}
        lys = {"brc_ly": mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"], "log1p_all": mpf(D(g["l1p"]))}
        ga, gb, gab = g["ga"], g["gb"], g["gab"]
        Ls = {
            "L1d": mpf(D(ga + D(gb - gab))),
            "L1e": R64(ga + R64(gb - gab)),
            "L2d": mpf(D(D(ga + gb) - gab)),
        }
        for (klx, lx), (kly, ly), (kL, L) in itertools.product(lxs.items(), lys.items(), Ls.items()):
            tx = mpf(D(g["am1"] * lx))
            ty = mpf(D(g["bm1"] * ly))
            for s1, s2 in itertools.product("de", repeat=2):
                Es = {
                    "tx+(ty-L)": ST[s2](tx + ST[s1](ty - L)),
                    "(tx+ty)-L": ST[s2](ST[s1](tx + ty) - L),
                    "(tx-L)+ty": ST[s2](ST[s1](tx - L) + ty),
                }
                for kg, E in Es.items():
                    z[(klx, kly, kL, kg, s1 + s2, zn)] += exp_v6(E) == r["excel"]
    zones = sorted(n)
    keys = {k[:5] for k in z}
    res = sorted(((sum(z[k + (zz,)] for zz in zones), k) for k in keys), reverse=True)
    print("zones", [(zz, n[zz]) for zz in zones], "total", sum(n.values()))
    for t, k in res[:int(__import__("os").environ.get("TOP","15"))]:
        print(t, " ".join(k), [z[k + (zz,)] for zz in zones])


if __name__ == "__main__":
    for sh in sys.argv[1:]:
        if sh == "old":
            rows = [r for r in load_pdf() if r["a"] < 1 and 1 <= r["b"] <= 8]
        else:
            rows = [r for r in load_probe("probe1") if r["case"] == sh]
        print("==", sh, rows[0]["a"], rows[0]["b"])
        run(rows)
