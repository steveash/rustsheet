"""Per r-bin feasible band for the relative error eps of the correction (in units of 2^-64), from resid.rows_d:
eps_lo = max over rows of lower bounds, eps_hi = min of upper bounds (a constant-in-bin eps would need lo<=hi)."""
import math
import sys

import resid

rs = resid.rows_d(sys.argv[1:] or ["runs", "grid", "hold"])
U = 2.0**-64
nb = 24
bins = [[-1e9, 1e9, 0, 0] for _ in range(nb)]
for p, r, l, d, nm in rs:
    x = -r
    if x <= 0:
        continue
    j = min(nb - 1, int(x / 0.2308 * nb))
    s = math.ulp(l) / abs(l) / U
    lo, hi = (-0.5 - d) * s, (0.5 - d) * s
    # relative error defined as (computed - exact)/exact with exact negative: sign flip for l<0
    lo, hi = -hi, -lo
    b = bins[j]
    b[0] = max(b[0], lo)
    b[1] = min(b[1], hi)
    b[2] += 1
    b[3] += abs(d) >= 0.5
for j, (lo, hi, n, m) in enumerate(bins):
    print(f"-r in [{j*0.2308/nb:.4f},{(j+1)*0.2308/nb:.4f}) n={n:5d} miss={m:3d}  eps band [{lo:+9.1f},{hi:+9.1f}] x2^-64 {'CONFLICT' if lo>hi else ''}")
