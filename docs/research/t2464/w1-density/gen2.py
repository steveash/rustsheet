"""Product forms: density = EXP(e1) * EXP(e2) with the lnB attached to either factor, factors stored or
extended, versus the single EXP. brcomp-style logs. Per x zone. Usage: gen2.py probe1C0 ...
"""

import collections
import itertools
import sys

import mpmath

from forms import ingredients
from gen1 import zone
from lib import D, R64, exp_v6, exp_v6_ext, load_pdf, load_probe, mpf, xl

ST = {"d": D, "e": R64}


def run(rows):
    ing = ingredients(rows)
    LNa = xl("LN", [r["a"] for r in rows])
    LNb = xl("LN", [r["b"] for r in rows])
    z = collections.Counter()
    n = collections.Counter()
    for r, g, lna, lnb in zip(rows, ing, LNa, LNb):
        x = r["x"]
        zn = zone(x)
        n[zn] += 1
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        lx = g["LNd"]
        ga, gb, gab = g["ga"], g["gb"], g["gab"]
        Ls = {
            "L1d": mpf(D(ga + D(gb - gab))),
            "L2d": mpf(D(D(ga + gb) - gab)),
            "L3d": mpf(D(gb + D(ga - gab))),
        }
        if r["a"] == 1:
            Ls["-LNb"] = -mpf(lnb)
        if r["b"] == 1:
            Ls["-LNa"] = -mpf(lna)
        tx = mpf(D(g["am1"] * lx))
        ty = mpf(D(g["bm1"] * ly))
        for kL, L in Ls.items():
            for se in "de":
                F = {}
                F["single tx+(ty-L)"] = exp_v6(ST[se](tx + R64(ty - L)))
                F["single (tx-L)+ty"] = exp_v6(ST[se](R64(tx - L) + ty))
                for sp in "de":
                    # x factor carries L
                    px = exp_v6_ext(ST[se](tx - L))
                    py = exp_v6_ext(ty)
                    PX = ST[sp](px)
                    PY = ST[sp](py)
                    F[f"EXP(tx-L)*EXP(ty) f{sp}"] = D(PX * PY)
                    px2 = exp_v6_ext(tx)
                    py2 = exp_v6_ext(ST[se](ty - L))
                    F[f"EXP(tx)*EXP(ty-L) f{sp}"] = D(ST[sp](px2) * ST[sp](py2))
                    Bv = exp_v6_ext(L)
                    F[f"EXP(tx)*EXP(ty)/EXP(L) f{sp}"] = D(ST[sp](ST[sp](px2) * ST[sp](py)) / ST[sp](Bv))
                    F[f"EXP(tx)/EXP(L)*EXP(ty) f{sp}"] = D(ST[sp](ST[sp](px2) / ST[sp](Bv)) * ST[sp](py))
                    F[f"EXP(ty)/EXP(L)*EXP(tx) f{sp}"] = D(ST[sp](ST[sp](py) / ST[sp](Bv)) * ST[sp](px2))
                for k, v in F.items():
                    z[(kL, f"E{se}", k, zn)] += v == r["excel"]
    zones = sorted(n)
    keys = {k[:3] for k in z}
    res = sorted(((sum(z[k + (zz,)] for zz in zones), k) for k in keys), reverse=True)
    print("zones", [(zz, n[zz]) for zz in zones], "total", sum(n.values()))
    for t, k in res[:12]:
        print(t, " | ".join(k), [z[k + (zz,)] for zz in zones])


if __name__ == "__main__":
    for sh in sys.argv[1:]:
        rows = [r for r in load_probe("probe1") if r["case"] == sh]
        print("==", sh, rows[0]["a"], rows[0]["b"])
        run(rows)
