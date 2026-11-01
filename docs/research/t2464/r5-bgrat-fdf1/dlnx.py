"""T-3567 r5-bgrat-fdf1 dlnx: does D at b != 1/2 take another ln x than FINAL's (LN(x) above y = 0.01)?
bgld.bgrat_ld (Q: grat1 CF in extended, eps 1e-15) on the split points with lnx from LN(x), Excel's log1p(-y) and
TOMS alnrel(-y), y = 1 - x (exact here, x > 1/2)."""
import sys
from collections import defaultdict
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import bgld  # noqa: E402
from common import split_cells, table, xlk  # noqa: E402
bx = xlk.bx
KX = xlk.XKernel(xlk.XCfg(alnrel="xl"))
KT = xlk.XKernel(xlk.XCfg(alnrel="toms"))
L = {"LN(x)": lambda x: bx.LN(x), "Excel log1p(-y)": lambda x: KX.alnrel(-(1 - x)), "TOMS alnrel(-y)": lambda x: KT.alnrel(-(1 - x))}
g = defaultdict(list)
for a, b0, x, _case, d in split_cells():
    for name, f in L.items():
        g[(name, "y<=0.375" if 1 - x <= 0.375 else "y>0.375")].append((bgld.bgrat_ld(a, b0, f(x), "cf", 1e-15), d))
table(g, ("lnx", "y"))
