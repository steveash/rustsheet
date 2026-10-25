"""Per-block mean/sd of Excel's T density error (units 2^-53 relative to exact) on the in-sample probes, and the
df-step blocks' error per df.  usage: s5_blocks.py"""
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD

rows = TD.load_probe("a1") + TD.load_probe("a2")
blk = defaultdict(list)
for r in rows:
    ex = mpmath.exp(TD.exact_logpdf(r["t"], r["n"]))
    r["err"] = float((r["excel"] - ex) / ex) / 2**-53
    blk[(r["blk"], r["n"])].append(r["err"])
for k in sorted(blk, key=lambda k: (k[0][:2], k[1], k[0])):
    v = np.array(blk[k])
    if len(v) > 5:
        print(f"{k[0]:18s} n={k[1]:<4g} N={len(v):3d} mean {v.mean():+6.2f} sd {v.std():5.2f}")
for t in ("0.5", "0.9", "2"):
    e = sorted((k[1], blk[k][0]) for k in blk if k[0] == f"df(t{t})")
    print(f"df(t{t}):", " ".join(f"{n:g}:{x:+.1f}" for n, x in e))
