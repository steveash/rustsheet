"""T-2464: exact I_x(a,b) and beta density (mpmath, 60 digits) for the BETA.DIST/BETADIST rows of cells.tsv.

Writes beta_exact.tsv: case, cell, x, a, b, cum, excel, exact (repr of the correctly rounded double),
err_ulp (excel - exact in ulps of exact), rel (relative error), branch (direct if x <= a/(a+b) else reflected).
Only 0 < x < 1 with A=0, B=1 bounds (BETA.DIST's optional bounds rescale x first).
"""
import math
import sys
from pathlib import Path

import mpmath

mpmath.mp.dps = 60
HERE = Path(__file__).parent
rows = []
for line in (HERE / "cells.tsv").read_text().splitlines()[1:]:
    fn, case, cell, args, excel = line.split("\t")
    if fn not in ("BETA.DIST", "BETADIST"):
        continue
    vals = args.split("|")
    try:
        nums = [v for v in vals]
        x, a, b = float(nums[0]), float(nums[1]), float(nums[2])
        if fn == "BETA.DIST":
            cum = nums[3] in ("True", "1.0", "1")
            if not (nums[3] in ("True", "False") or nums[3].replace(".", "").replace("-", "").isdigit()):
                continue
            cum = nums[3] == "True" or (nums[3] not in ("False",) and float(nums[3]) != 0)
            lo = float(nums[4]) if len(nums) > 4 else 0.0
            hi = float(nums[5]) if len(nums) > 5 else 1.0
        else:
            cum = True
            lo = float(nums[3]) if len(nums) > 3 else 0.0
            hi = float(nums[4]) if len(nums) > 4 else 1.0
    except ValueError:
        continue
    if (lo, hi) != (0.0, 1.0) or not (0 < x < 1) or a <= 0 or b <= 0:
        continue
    e = float(excel)
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    try:
        if cum:
            v = mpmath.betainc(A, B, 0, X, regularized=True)
        else:
            v = X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B)
    except Exception:
        continue
    ex = float(v)
    err = (e - ex) / math.ulp(ex) if ex else math.inf
    rel = float((mpmath.mpf(e) - v) / v) if v else math.inf
    branch = "direct" if x <= a / (a + b) else "reflected"
    rows.append((case, cell, repr(x), repr(a), repr(b), str(int(cum)), repr(e), repr(ex), f"{err:.3f}", f"{rel:.3e}", branch))
with (HERE / "beta_exact.tsv").open("w") as f:
    f.write("case\tcell\tx\ta\tb\tcum\texcel\texact\terr_ulp\trel\tbranch\n")
    for r in rows:
        f.write("\t".join(r) + "\n")
print(len(rows))
