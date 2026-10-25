"""The exponent as a double plus a correction: density = fl(EXP(hi) * (1 + lo)) (Excel's EXP of the double hi).
Variants of how hi and lo are formed from the stored pieces tx, ty, L.  Usage: dd.py [sets] (old, probe1, log1p)"""
import collections
import sys

import mpmath

import m1lib as M

mpf, R64, R53, W1 = M.mpf, M.fl64, M.fl53, M.W1


def parts(r):
    a, b = r["a"], r["b"]
    if "l" in r:
        tx, ty = 0.0, M.sx("*", b - 1.0, r["l"])
    else:
        tx, ty = M.pieces(r["x"], a, b)
    return mpf(tx), mpf(ty), mpf(M.lnb_base(a, b))


def forms(tx, ty, L, b):
    E = tx + ty - L
    out = {}
    out["m1"] = W1.exp_v6(mpf(R53(tx - L)) if b == 1.0 else R64(tx + R64(ty - L)))
    hiE = mpf(R53(E))
    P = mpf(W1.exp_v6(hiE))
    out["hi=fl(E),lo exact"] = R53(P * (1 + (E - hiE)))
    # hi from double ops (each op rounded to double), lo = exact remainder
    for nm, hi in (
        ("tx+(ty-L)", mpf(R53(tx + mpf(R53(ty - L))))),
        ("(tx+ty)-L", mpf(R53(mpf(R53(tx + ty)) - L))),
        ("(tx-L)+ty", mpf(R53(mpf(R53(tx - L)) + ty))),
    ):
        P = mpf(W1.exp_v6(hi))
        out["hi=" + nm + ",lo exact"] = R53(P * (1 + (E - hi)))
        out["hi=" + nm + ",no lo"] = float(P)
    return out


if __name__ == "__main__":
    which = sys.argv[1:] or ["old", "probe1", "log1p"]
    for nm in which:
        rows = {"old": M.load_old, "probe1": M.load_probe1, "log1p": lambda: M.load_log1p_rows()[::4]}[nm]()
        by = collections.defaultdict(collections.Counter)
        for r in rows:
            tx, ty, L = parts(r)
            g = "b=1" if r["b"] == 1.0 else ("a=1" if r["a"] == 1.0 else ("a<1" if r["a"] < 1 else "b<1"))
            g += ",big" if max(r["a"], r["b"]) > 8 else ""
            for k, v in forms(tx, ty, L, r["b"]).items():
                by[g][k] += v == r["excel"]
            by[g]["n"] += 1
        for g, c in sorted(by.items()):
            print(nm, g, dict(c))
