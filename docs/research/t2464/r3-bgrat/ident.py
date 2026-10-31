"""T-2512 r3-bgrat: r2-tdist's b = 1/2 identity, generalised, on Excel's direct bgrat cells (probe -split).

D_model = RN(F * Qm / Q): F the exact I at the double lnx, Q the exact Q(b0, z) at the stored z = fl(-nu lnx), Qm a
candidate for the Q Excel's bgrat uses:
  exact      Qm = Q (= CR)
  grat1:eps  TOMS grat1 in doubles with Excel's EXP/LN (r2-bratio's port), r as TOMS bgrat forms it, at eps
Also rho = D/F - 1 in units of 2^-53 against z, split at grat1's series/CF switch z = 1.1."""
from __future__ import annotations

import json
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

import branches  # noqa: E402
import bscore as score  # noqa: E402
import score_lib as sl  # noqa: E402

NAME = "t2512-r3-bgrat-split"
out = json.load(open(HERE.parents[3] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))["sheets"]["Calc"]
meta = json.load(open(HERE / f"{NAME}.meta.json"))
xlk.run([(a, b0, x, 1.0 - x) for _k, a, b0, x, _c, _e in meta], branches.FINAL)
K = xlk.bx._KERNELS[branches.FINAL]
C = {(c["a"], c["b"], c["x"]): c for c in score.cells()}


def toms_r(a, b, lnx, z):
    bm1 = b - 0.5 - 0.5
    r = b * (1.0 + K.gam1(b)) * K.exp(b * K.log(z))
    return r * K.exp(a * lnx) * K.exp(0.5 * bm1 * lnx)


mpmath.mp.dps = 40
g = defaultdict(list)
rho = defaultdict(list)
for k, a, b0, x, case, _cell in meta:
    v = out.get(f"A{k}")
    if not v or v.get("t") != "n":
        continue
    c = C[(a, b0, x)]
    dx, F, Q, z = v["v"], c["F"], c["Q"], c["z"]
    band = "z<1.1" if z < 1.1 else "z>=1.1"
    cand = {"exact": Q}
    for e in (15e-15, 1e-15):
        cand[f"grat1:{e:g}"] = K.grat1(b0, z, toms_r(a, b0, c["lnx"], z), e)
    for m, q in cand.items():
        mv = float(F * mpmath.mpf(q) / Q)
        g[(band, m)].append((mv, dx))
        g[("ALL", m)].append((mv, dx))
    rho[band].append(float((mpmath.mpf(dx) / F - 1) * 2**53))
print("| z band | Qm | n | exact/w4/w15 |")
print("|---|---|---|---|")
for k in sorted(g):
    n, e, f, w = sl.tally(g[k])
    print(f"| {k[0]} | {k[1]} | {n} | {e}/{f}/{w} |")
for b, v in sorted(rho.items()):
    print(f"rho {b}: n {len(v)} median {st.median(v):.1f} sd {st.pstdev(v):.1f}")
