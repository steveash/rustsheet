"""s10: residual sequence (Excel - m1 'xl' model, units) along one run, with the s3 stored-intermediate values."""
import sys

import mpmath
import numpy as np

import m1_fixed as M
import plib as P

src, want = sys.argv[1], sys.argv[2]
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
D = np.array([P.D_units(r) for r in rows])
Dm = np.array([float((M.lnF_model(r["x"], m, n, "xl") - mpmath.log(r["exact"])) * P.U) for r in rows])
res = D - Dm
print("x0", rows[0]["x"].hex(), "res:", " ".join(f"{v:.0f}" for v in res))
print("diff:", " ".join(f"{v:.0f}" for v in np.diff(res)))
