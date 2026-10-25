"""Is Excel's density EXP of some double?  For each in-sample block, the share of cells whose value is in the image
of Excel's EXP (model V6) over doubles (r3-loader-bd0 eset), and the spread of the admissible E around the exact
log density (units of 2^-53 absolute).  usage: s10_eset.py"""
from collections import defaultdict

import numpy as np

import tdlib as TD

rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)
for key in sorted(by):
    rr = by[key][::2]
    emp = 0
    d = []
    for r in rr:
        es = TD.R3.eset(r["excel"])
        if not es:
            emp += 1
            continue
        lp = float(TD.exact_logpdf(r["t"], key[0]))
        d.append((np.mean(es) - lp) / 2**-53)
    print(f"{key[1]:18s} cells {len(rr)} not-EXP-of-a-double {emp}  E-offset mean {np.mean(d):+.1f} sd {np.std(d):.1f}")
TD.R3.save_cache = lambda: None
