"""T-2512 r3-bgrat: r2-tdist's b = 1/2 identity, generalised, on Excel's direct bgrat cells (probe -split).

D_model = RN(F * Qm / Q): F the exact I at the double lnx, Q the exact Q(b0, z) at the stored z = fl(-nu lnx), Qm a
candidate for the Q Excel's bgrat uses:
  exact      Qm = Q (= CR)
  grat1:eps  TOMS grat1 in doubles with Excel's EXP/LN (r2-bratio's port), r as TOMS bgrat forms it, at eps
Also rho = D/F - 1 in units of 2^-53 against z, split at grat1's series/CF switch z = 1.1."""
from __future__ import annotations

import statistics as st
from collections import defaultdict

import mpmath

import bscore as score
from common import split_kernel, table

K, rows = split_kernel()
C = {(c["a"], c["b"], c["x"]): c for c in score.cells()}


def toms_r(a, b, lnx, z):
    bm1 = b - 0.5 - 0.5
    r = b * (1.0 + K.gam1(b)) * K.exp(b * K.log(z))
    return r * K.exp(a * lnx) * K.exp(0.5 * bm1 * lnx)


mpmath.mp.dps = 40
g = defaultdict(list)
rho = defaultdict(list)
for a, b0, x, _case, dx in rows:
    c = C[(a, b0, x)]
    F, Q, z = c["F"], c["Q"], c["z"]
    band = "z<1.1" if z < 1.1 else "z>=1.1"
    cand = {"exact": Q}
    for e in (15e-15, 1e-15):
        cand[f"grat1:{e:g}"] = K.grat1(b0, z, toms_r(a, b0, c["lnx"], z), e)
    for m, q in cand.items():
        mv = float(F * mpmath.mpf(q) / Q)
        g[(band, m)].append((mv, dx))
        g[("ALL", m)].append((mv, dx))
    rho[band].append(float((mpmath.mpf(dx) / F - 1) * 2**53))
table(g, ("z band", "Qm"))
for b, v in sorted(rho.items()):
    print(f"rho {b}: n {len(v)} median {st.median(v):.1f} sd {st.pstdev(v):.1f}")
