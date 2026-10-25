"""Do BINOM.DIST(k,n,p,FALSE) and BETA.DIST(p,k+1,n-k+1,FALSE) share their p-dependent roundings?  Per block:
sd of Excel's exponent error D for each, and sd of D_beta - D_binom (shared roundings cancel).
Usage: b2_share.py <probe> [second fn, default BETA]"""
import math
import sys

import mpmath

import blib as B

name = sys.argv[1] if len(sys.argv) > 1 else "a"
other = sys.argv[2] if len(sys.argv) > 2 else "BETA"
rows = B.load_probe(name)
by = {}
for r in rows:
    by.setdefault((r["blk"], r["p"]), {})[r["fn"]] = r


def D(r):
    return float((mpmath.log(r["excel"]) - B.exact_ln(r)) * 2**53)


def sd(v):
    m = sum(v) / len(v)
    return math.sqrt(sum((x - m) ** 2 for x in v) / len(v)), m


blks = {}
for (blk, p), d in by.items():
    if "BIN" in d and other in d:
        blks.setdefault(blk, []).append((D(d["BIN"]), D(d[other]), d["BIN"]["excel"]))
for blk in sorted(blks):
    v = blks[blk]
    s1, m1 = sd([a for a, b, _ in v])
    s2, m2 = sd([b for a, b, _ in v])
    s3, m3 = sd([b - a for a, b, _ in v])
    print(f"{blk:28s} n={len(v)} pmf~{v[0][2]:.3g}  sd D_bin {s1:6.2f} (mean {m1:+7.2f})  sd D_{other} {s2:6.2f} "
          f"(mean {m2:+7.2f})  sd diff {s3:6.2f} (mean {m3:+7.2f})")
