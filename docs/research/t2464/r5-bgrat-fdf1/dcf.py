"""T-3567 r5-bgrat-fdf1 dcf: is D's white noise grat1's continued fraction rounded in doubles? sd of (Excel - model)
in units of 2^-53 relative on the split probe's two 1,024-double runs, and exact/w4/w15 on all 2,908 split points,
for bgld.bgrat_ld with Q from the CF in extended ("cf") or in doubles ("cfd"), per eps."""
import sys
from collections import defaultdict
from pathlib import Path
import numpy as np
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import bgld  # noqa: E402
import bscore  # noqa: E402
from common import split_cells, table  # noqa: E402
C = {(c["a"], c["b"], c["x"]): c for c in bscore.cells()}
RUNS = ((30.7, 0.7), (40.0, 0.030000000000000027))
g = defaultdict(list)
sd = defaultdict(list)
for a, b0, x, _case, d in split_cells():
    c = C[(a, b0, x)]
    for qm in ("cf", "cfd"):
        for eps in (15e-15, 1e-15, 1e-16):
            m = bgld.bgrat_ld(a, b0, c["lnx"], qm, eps)
            g[(qm, eps)].append((m, d))
            if (a, b0) in RUNS:
                sd[(qm, eps, (a, b0))].append((d - m) / d * 2.0 ** 53)
table(g, ("Q", "eps"))
for k, v in sorted(sd.items(), key=str):
    v = np.array(v)
    print(k, f"mean {v.mean():+.2f} sd {v.std():.2f}")
