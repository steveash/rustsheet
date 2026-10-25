"""Double-double exponent with product errors: E_full = (a-1)*lnx + (b-1)*ly - L computed exactly from the stored
doubles lnx, ly, L (the products' rounding errors kept), hi = a double, lo = E_full - hi, density = fl(EXP(hi)*(1+lo)).
Usage: dd2.py [sets]"""
import collections
import sys

import m1lib as M

mpf, R53, W1 = M.mpf, M.fl53, M.W1


def ingred(r):
    a, b, x = r["a"], r["b"], r["x"]
    am1, bm1 = mpf(a - 1.0), mpf(b - 1.0)
    lnx = mpf(M.BE.LN(x)) if a != 1.0 else mpf(0)
    ly = mpf(r["l"]) if "l" in r else mpf(M.ly_of(x))
    return am1, bm1, lnx, ly, mpf(M.lnb_base(a, b))


def d(v):
    return mpf(R53(v))


def forms(am1, bm1, lnx, ly, L, b):
    tx, ty = d(am1 * lnx), d(bm1 * ly)
    ex, ey = am1 * lnx - tx, bm1 * ly - ty  # product errors
    Ed = tx + ty - L  # exact sum of the rounded pieces
    out = {}
    for hn, hi in (("(tx+ty)-L", d(d(tx + ty) - L)), ("tx+(ty-L)", d(tx + d(ty - L))), ("fl(E)", d(Ed))):
        P = mpf(W1.exp_v6(hi))
        for ln, lo in (("sum", Ed - hi), ("sum+ex+ey", Ed + ex + ey - hi), ("sum+ey", Ed + ey - hi),
                       ("sum+ex", Ed + ex - hi)):
            out[f"{hn}|{ln}"] = R53(P * (1 + lo)) if b != 1.0 else float(mpf(W1.exp_v6(d(tx - L))))
    return out


if __name__ == "__main__":
    which = sys.argv[1:] or ["old", "probe1"]
    for nm in which:
        rows = {"old": M.load_old, "probe1": M.load_probe1, "log1p": lambda: M.load_log1p_rows()[::4]}[nm]()
        by = collections.defaultdict(collections.Counter)
        for r in rows:
            g = "b=1" if r["b"] == 1.0 else ("a=1" if r["a"] == 1.0 else ("a<1" if r["a"] < 1 else "b<1"))
            g += ",big" if max(r["a"], r["b"]) > 8 else ""
            for k, v in forms(*ingred(r), r["b"]).items():
                by[g][k] += v == r["excel"]
                by["ALL"][k] += v == r["excel"]
            by[g]["n"] += 1
            by["ALL"]["n"] += 1
        for g, c in sorted(by.items()):
            n = c.pop("n")
            top = sorted(c.items(), key=lambda kv: -kv[1])
            print(nm, g, n, " ".join(f"{k}={v}" for k, v in top))
