"""Lattice of Excel's D = ln(Excel) - ln(exact) within large-shape runs (units 2^-53), with ln F exact and the run's
x part sizes, to locate the stored term behind the noise."""
import sys

import mpmath

import fblib as B

SET = sys.argv[1] if len(sys.argv) > 1 else "a"
MIN = float(sys.argv[2]) if len(sys.argv) > 2 else 20
rows = B.load(SET)
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
U = mpmath.mpf(2) ** 53
for blk, rr in runs.items():
    if max(rr[0]["d1"], rr[0]["d2"]) < MIN:
        continue
    D = [float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U) for r in rr]
    r0 = rr[0]
    a, b, x = r0["d1"] / 2, r0["d2"] / 2, r0["x"]
    lnF = float(mpmath.log(r0["exact"]))
    mx = r0["d1"] * x
    y = mx / (mx + r0["d2"])
    terms = dict(lnF=lnF, alny=a * mpmath.log(y), blnq=b * mpmath.log(1 - y), hln=(a + b) * mpmath.log(mx + r0["d2"]))
    ds = sorted(D)
    print(f"{blk:22s} " + " ".join(f"{k}={float(v):.2f}" for k, v in terms.items()))
    print("   D sorted:", " ".join(f"{v:.0f}" for v in ds))
