"""Under the dd exponent: variants of the stored pieces tx = (a-1) ln x and ty = (b-1) ln(1-x): ln x stored (LN) or
extended (FYL2X unstored), ln(1-x) above 0.375 stored or extended, products stored or extended (the dd hi/lo split
then works on the extended values).  Usage: txty.py [sets]"""
import collections
import sys

import m1lib as M
from lscan import rows_of
from m1score import region

mpf, R53, R64, W1 = M.mpf, M.fl53, M.fl64, M.W1


def dd(T, Y, L, b):
    if b == 1.0:
        return W1.exp_v6(mpf(R53(T - L)))
    hi = mpf(R53(mpf(R53(T + Y)) - L))
    return R53(mpf(W1.exp_v6(hi)) * (1 + ((T + Y - L) - hi)))


def variants(r):
    x, a, b = r["x"], r["a"], r["b"]
    am1, bm1 = mpf(a - 1.0), mpf(b - 1.0)
    lnx_s, lnx_e = mpf(M.BE.LN(x)), W1.ln64(x)
    if x <= 0.375:
        ly_s = ly_e = mpf(M.ly_of(x))
    else:
        ly_s, ly_e = mpf(M.BE.LN(1.0 - x)), W1.ln64(1.0 - x)
    L = mpf(M.lnb_base(a, b))
    out = {}
    for nx, lx in (("lnx_s", lnx_s), ("lnx_e", lnx_e)):
        for ny, ly in (("ly_s", ly_s), ("ly_e", ly_e)):
            for np_, rp in (("prod_s", lambda v: mpf(R53(v))), ("prod_e", R64)):
                out[f"{nx},{ny},{np_}"] = dd(rp(am1 * lx), rp(bm1 * ly), L, b)
    return out


if __name__ == "__main__":
    for s in sys.argv[1:] or ["probe1", "a"]:
        by = collections.defaultdict(collections.Counter)
        for r in rows_of(s):
            g = r.get("blk", region(r))
            for k, v in variants(r).items():
                by[g][k] += v == r["excel"]
                by["ALL"][k] += v == r["excel"]
            by[g]["n"] += 1
            by["ALL"]["n"] += 1
        for g, c in sorted(by.items()):
            n = c.pop("n")
            print(s, g, n, " ".join(f"{k}={v}" for k, v in sorted(c.items(), key=lambda kv: -kv[1])[:5]))
