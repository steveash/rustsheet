"""Oracle probe t2512-r3-bgrat-split (probes-numeric): Excel's bgrat increment alone at every L140 bup+bgrat call.

L140 bup+bgrat forms I_x(a, b) = RN(w_bup + bgrat(a, b0, x)) with b0 = b - int(b) in (0, 1). BETA.DIST(x, a, b0, TRUE)
dispatches (min(a, b0) <= 1, swapped, b0' = a > 15) to bratio's direct bgrat branch with w = 0 and returns that
same increment as the lower tail, rounded once (where the model says the direct cell takes bgrat; others are
skipped). Column A = BETA.DIST(x, a, b0, TRUE). Meta: [[row, a, b0, x, src_case, src_cell], ...]
usage: make_probe2.py [--write]"""
from __future__ import annotations

import pickle
import sys

import branches
from common import HERE, w2p, write_probe, xlk

NAME = "t2512-r3-bgrat-split"

d = pickle.load(open(HERE / "calls.pkl", "rb"))
seen, cells, meta = set(), {}, []
for i, c in d["calls"].items():
    a, b0, x, y, w, lnx, g = c
    br = d["res"][i][2]
    if not br.startswith("L140 bup+bgrat") or b0 == 0.5 or b0 == 1.0 or (a, b0, x) in seen:
        continue
    seen.add((a, b0, x))
    if y != 1.0 - x:  # the direct cell's y must be the call's y
        continue
    _, _, br2 = xlk.bx.kernel(a, b0, x, 1.0 - x, branches.FINAL)
    if not br2.startswith("bgrat"):
        continue
    k = len(cells) + 1
    cells[f"A{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b0)},TRUE)"
    r = d["rows"][i]
    meta.append((k, a, b0, x, r["case"], r["cell"]))
write_probe(NAME, "T-2512 r3-bgrat: BETA.DIST(x, a, frac(b)) at every L140 bup+bgrat cell (Excel's bgrat increment alone)", cells, meta, "--write" in sys.argv)
