"""Under density = fl(EXP(hi) * fl(1 + lo)), lo = E - hi exactly: which double hi (and does b = 1 take the split).
Usage: hivar.py [sets]"""
import collections
import sys

import m1lib as M
from lscan import rows_of
from m1score import region

mpf, R53, R64, W1 = M.mpf, M.fl53, M.fl64, M.W1


def fin(hi, E):
    return R53(mpf(W1.exp_v6(hi)) * mpf(R53(1 + (E - hi))))


def variants(r):
    a, b = r["a"], r["b"]
    tx, ty = map(mpf, M.pieces(r["x"], a, b))
    L = mpf(M.lnb_base(a, b))
    E = tx + ty - L
    d = lambda v: mpf(R53(v))  # noqa: E731
    his = {"(tx+ty)-L": d(d(tx + ty) - L), "fl(E)": d(E), "tx+(ty-L)": d(tx + d(ty - L)),
           "(tx-L)+ty": d(d(tx - L) + ty), "fl(R64(tx+ty)-L)": d(R64(tx + ty) - L), "fl(R64 m1 E)": d(R64(tx + R64(ty - L)))}
    out = {k: fin(h, E) for k, h in his.items()}
    out["b1-stored"] = W1.exp_v6(d(tx - L)) if b == 1.0 else out["(tx+ty)-L"]
    return out


if __name__ == "__main__":
    for s in sys.argv[1:] or ["probe1", "old", "a"]:
        by = collections.defaultdict(collections.Counter)
        for r in rows_of(s):
            g = region(r)
            for k, v in variants(r).items():
                by[g][k] += v == r["excel"]
                by["ALL"][k] += v == r["excel"]
            by[g]["n"] += 1
            by["ALL"]["n"] += 1
        for g, c in sorted(by.items()):
            n = c.pop("n")
            print(s, g, n, " ".join(f"{k}={v}" for k, v in sorted(c.items(), key=lambda kv: -kv[1])))
