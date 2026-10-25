"""s5: residual sequence of one run after the s2 fit (rx, eLx, eL)."""
import sys

import numpy as np

import plib as P
import s2_reg

src, want = sys.argv[1], sys.argv[2]
rows = [r for r in P.load(src) if r["blk"] == want]
m, n = rows[0]["d1"], rows[0]["d2"]
D = np.array([P.D_units(r) for r in rows])
F0 = np.array([[s2_reg.feats(r["x"], m, n)[0][k] for k in ("rx", "eLx", "eL")] for r in rows])
A = np.column_stack([F0, np.ones(len(D))])
cf, *_ = np.linalg.lstsq(A, D, rcond=None)
print(cf)
res = D - A @ cf
print(" ".join(f"{v:.0f}" for v in res))
print(" ".join(f"{v:.0f}" for v in np.diff(res)))
