"""T-3567 r5-bgrat-fdf1 dvar: Excel's direct bgrat increment D at b != 1/2 (probe t2512-r3-bgrat-split, 2,908 points).
r3-bgrat's x87 bgrat (bgld.bgrat_ld: stored lnx and z, the rest extended) with the prefactor E = exp(-(algdiv(b, a)
+ b ln nu)) formed in other ways. Q: grat1's CF in extended at eps 1e-15 (r3's best) or exact. Hold-out = r3's md5 half.
"""
import sys
from collections import defaultdict
from pathlib import Path
import numpy as np
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import bgld  # noqa: E402
import bscore  # noqa: E402
from common import split_cells, table, xlk  # noqa: E402
bx = xlk.bx
K = bx._KERNELS.get(xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ideal")) or xlk.XKernel(xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ideal"))
LD = np.longdouble
import mpmath
def mpq(v):
    p, q = v.as_integer_ratio()
    with mpmath.workdps(40):
        return mpmath.mpf(p) / q
def arg_ld(a, b, nu):
    return xlk._algdiv_ld(b, a) + LD(b) * np.log(nu)
E = {
    "E extended (r3)": None,
    "fl(E)": lambda a, b, nu: float(np.exp(-arg_ld(a, b, nu))),
    "EXP(fl(-arg_ld))": lambda a, b, nu: bx.EXP(float(-arg_ld(a, b, nu))),
    "EXPX(-arg_ld)": lambda a, b, nu: bx.EXPX(mpq(-arg_ld(a, b, nu))),
    "EXP(-fl(algdiv_d + fl(b LN(nu_d))))": lambda a, b, nu: bx.EXP(-(K.algdiv(b, a, True) + b * bx.LN(float(nu)))),
    "EXP(-fl(algdiv_ld + b LN(nu_d)))": lambda a, b, nu: bx.EXP(float(-(xlk._algdiv_ld(b, a) + LD(b) * LD(bx.LN(float(nu))))))
}
C = {(c["a"], c["b"], c["x"]): c for c in bscore.cells()}
g = defaultdict(list)
for a, b0, x, _case, d in split_cells():
    c = C[(a, b0, x)]
    half = "hold" if c["row"]["hold"] else "tune"
    for name, f in E.items():
        for qm, eps in (("cf", 1e-15), ("exact", 1e-15)):
            m = bgrat_v = bgld.bgrat_ld(a, b0, c["lnx"], qm, eps, f)
            for h in (half, "ALL"):
                g[(name, f"Q {qm}", h)].append((m, d))
table(g, ("E", "Q", "half"))
