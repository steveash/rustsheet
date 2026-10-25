"""NEGBINOM.DIST(n-k,k,p,FALSE) against BINOM.DIST(k,n,p,FALSE) at the same p (probe n's NEG rows vs probe a's BIN
rows): sd of the exponent errors and of their difference per block, and how often NEG == fl(fl(k/n) * BIN) etc."""
import math

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
binr = {(r["k"], r["n"], r["p"]): r for r in B.load_probe("a") if r["fn"] == "BIN"}
neg = [r for r in B.load_probe("n") if r["fn"] == "NEG"]
blk: dict = {}
for r in neg:
    b = binr.get((r["k"], r["n"], r["p"]))
    if b:
        blk.setdefault(r["blk"], []).append((r, b))
tot = dict(mul=0, kn=0, ext=0, n=0)
for name, prs in sorted(blk.items()):
    dn = np.array([float((mpmath.log(r["excel"]) - B.exact_ln(r)) * 2**53) for r, _ in prs])
    db = np.array([float((mpmath.log(b["excel"]) - B.exact_ln(b)) * 2**53) for _, b in prs])
    c = dict(mul=0, kn=0, ext=0)
    for r, b in prs:
        k, n, y = r["k"], r["n"], LDB(b["excel"])
        c["mul"] += B.st(LDB(B.st(LDB(k) / LDB(n))) * y) == r["excel"]
        c["kn"] += B.st(LDB(B.st(LDB(k) * y)) / LDB(n)) == r["excel"]
        c["ext"] += B.st(LDB(k) * y / LDB(n)) == r["excel"]
    for kk in c:
        tot[kk] += c[kk]
    tot["n"] += len(prs)
    print(f"{name:24s} n={len(prs)} sd D_neg {dn.std():7.2f} sd D_bin {db.std():7.2f} sd diff {(dn - db).std():5.2f} "
          f"(mean {np.mean(dn - db):+6.2f})  from-BIN: {c}")
print("totals", tot)
