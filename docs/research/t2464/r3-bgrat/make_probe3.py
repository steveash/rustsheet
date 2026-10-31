"""Oracle probe t2512-r3-bgrat-bupdens (probes-numeric): Excel's density at every L140 bup+bgrat cell with one bup term.

L140 at 1 < b < 2 (n = 1): w = bup(b0, a, y, x, 1) = x^a y^b0 / (b0 B(b0, a)) = x y pdf/b0, with pdf the Beta(a, b0)
density at x (r2-tdist bup_lab: Excel's bup prefactor is its density times x times y). Columns (x, a, b0 the
bgrat call's; y = fl(1 - x)): A = BETA.DIST(x, a, b0, FALSE), B = BETA.DIST(y, b0, a, FALSE).
Meta: [[row, a, b0, x], ...]. usage: make_probe3.py [--write]"""
from __future__ import annotations

import pickle
import sys

from common import HERE, w2p, write_probe
NAME = "t2512-r3-bgrat-bupdens"

d = pickle.load(open(HERE / "calls.pkl", "rb"))
seen, cells, meta = set(), {}, []
for i, c in d["calls"].items():
    a, b0, x, y, w, lnx, g = c
    r = d["rows"][i]
    if not d["res"][i][2] == "L140 bup+bgrat" or b0 in (0.5, 1.0) or (a, b0, x) in seen:
        continue
    if not 1 < r["b"] < 2 or y != 1.0 - x:
        continue
    seen.add((a, b0, x))
    k = len(meta) + 1
    cells[f"A{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b0)},FALSE)"
    cells[f"B{k}"] = f"=BETA.DIST({w2p.num(y)},{w2p.num(b0)},{w2p.num(a)},FALSE)"
    meta.append((k, a, b0, x))
write_probe(NAME, "T-2512 r3-bgrat: Excel's density at every one-term L140 bup+bgrat cell (the bup prefactor)", cells, meta, "--write" in sys.argv)
