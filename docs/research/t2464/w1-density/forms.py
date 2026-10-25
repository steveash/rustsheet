"""Density candidate forms shared by the probe scorers.

Every form takes precomputed per-row ingredients (dict) and returns a double. Ingredients:
x, a, b (mpf of doubles), am1 = fl(a-1), bm1 = fl(b-1), LNd = Excel LN(x) (double), LNe = FYL2X extended ln x,
ly_e = extended ln(1-x) with 1-x exact, ly_yd = Excel LN(fl(1-x)), l1p = exact log1p(-x),
ga, gb, gab = Excel GAMMALN(a), (b), (a+b); Ga, Gb, Gab = Excel GAMMA.
"""

import itertools
import math

import mpmath

from lib import D, R64, exp_v6, exp_v6_ext, ln64, mpf, xl

ST = {"d": D, "e": R64}


def ingredients(rows):
    X = [r["x"] for r in rows]
    A = [r["a"] for r in rows]
    B = [r["b"] for r in rows]
    AB = [a + b for a, b in zip(A, B)]
    LN = xl("LN", X)
    LNy = xl("LN", [1.0 - x for x in X])
    ga, gb, gab = xl("GAMMALN", A), xl("GAMMALN", B), xl("GAMMALN", AB)
    out = []
    for i in range(len(rows)):
        x, a, b = mpf(X[i]), mpf(A[i]), mpf(B[i])
        out.append(
            dict(
                x=x,
                a=a,
                b=b,
                am1=mpf(D(a - 1)),
                bm1=mpf(D(b - 1)),
                LNd=mpf(LN[i]),
                LNe=ln64(x),
                ly_e=ln64(1 - x),
                ly_yd=mpf(LNy[i]),
                l1p=mpmath.log1p(-x),
                ga=mpf(ga[i]),
                gb=mpf(gb[i]),
                gab=mpf(gab[i]),
            )
        )
    return out


def lnB_variants(g):
    ga, gb, gab = g["ga"], g["gb"], g["gab"]
    return {
        "L1d": mpf(D(ga + D(gb - gab))),
        "L1e": R64(ga + R64(gb - gab)),
        "L2d": mpf(D(D(ga + gb) - gab)),
        "L3d": mpf(D(gb + D(ga - gab))),
        "Lx": mpmath.log(mpmath.beta(g["a"], g["b"])),
    }


def exp_forms(g):
    """EXP(E) forms with E = tx + ty - lnB over storage variants. Returns name -> double."""
    out = {}
    txs = {
        "txD(am1*LNd)": mpf(D(g["am1"] * g["LNd"])),
        "txE(am1*LNd)": R64(g["am1"] * g["LNd"]),
        "txE(am1*LNe)": R64(g["am1"] * g["LNe"]),
        "txD(am1*LNe)": mpf(D(g["am1"] * g["LNe"])),
    }
    tys = {
        "tyE(bm1*lye)": R64(g["bm1"] * g["ly_e"]),
        "tyD(bm1*lye)": mpf(D(g["bm1"] * g["ly_e"])),
        "tyD(bm1*D(lye))": mpf(D(g["bm1"] * mpf(D(g["ly_e"])))),
        "tyD(bm1*LNy)": mpf(D(g["bm1"] * g["ly_yd"])),
        "tyE(bm1*LNy)": R64(g["bm1"] * g["ly_yd"]),
    }
    for (kl, L), (kx, tx), (ky, ty) in itertools.product(lnB_variants(g).items(), txs.items(), tys.items()):
        if kl == "Lx":
            continue
        for s in "de":
            E = R64(tx + ST[s](ty - L))
            out[f"EXP {kx} {ky} {kl} tx+({s}(ty-L))"] = exp_v6(E)
            E = R64(ST[s](tx + ty) - L)
            out[f"EXP {kx} {ky} {kl} ({s}(tx+ty))-L"] = exp_v6(E)
            E = mpf(D(tx + ST[s](ty - L)))
            out[f"EXP {kx} {ky} {kl} D(tx+({s}(ty-L)))"] = exp_v6(E)
    return out
