"""s1: per large-shape run, correlation of D with each candidate stored term's total error (single regressor)."""
import sys

import numpy as np

import cands as C
import plib as P

src = sys.argv[1] if len(sys.argv) > 1 else "a"
minmax = float(sys.argv[2]) if len(sys.argv) > 2 else 50
names = None
for blk, rs, m, n in P.runs(src, minmax):
    D = np.array([P.D_units(r) for r in rs])
    E = [C.errs(r["x"], m, n) for r in rs]
    names = list(E[0])
    out = []
    for k in names:
        e = np.array([d[k] for d in E])
        if e.std() < 1:
            continue
        c = np.corrcoef(D, e)[0, 1]
        sl = np.polyfit(e, D, 1)[0]
        out.append((abs(c), k, c, sl, e.std()))
    out.sort(reverse=True)
    print(f"{blk:22s} Dsd {D.std():6.1f} | " + " | ".join(f"{k} r={c:+.2f} sl={sl:+.2f} sd={sd:.0f}" for _, k, c, sl, sd in out[:4]))
