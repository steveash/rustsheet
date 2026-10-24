"""s26: on r3's sw sweeps at x = 1, Excel's error (units) beside the dd model's, per sweep: sd and the largest
|error|, with the size of the first sum's ulp (fl(K_hi + H)) for reference."""
import collections

import mpmath
import numpy as np

import fmodel as FM
import plib as P

rows = [r for r in P.load("r3:sw") if r["x"] == 1.0]
g = collections.defaultdict(list)
for r in rows:
    g[r["blk"]].append(r)
for blk, rs in g.items():
    eE = np.array([P.D_units(r) for r in rs])
    eM = np.array([float((mpmath.log(mpmath.mpf(FM.value(r["x"], r["d1"], r["d2"], "dd"))) - mpmath.log(r["exact"])) * P.U) for r in rs])
    big = [(r["d1"], r["d2"]) for r, e in zip(rs, eM) if abs(e) > 8]
    print(f"{blk:18s} Excel sd {eE.std():5.2f} max {np.abs(eE).max():5.1f} | dd sd {eM.std():6.2f} max {np.abs(eM).max():6.1f} | dd-Excel sd {(eM - eE).std():6.2f} | dd |err|>8 at {big[:4]}")
