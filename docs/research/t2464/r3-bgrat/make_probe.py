"""Oracle probe t2512-r3-bgrat-qgamma (probes-numeric): Excel's incomplete gamma at every bgrat call's (b, z).

For each distinct (b, z) over the recorded bgrat calls with b != 1/2 (calls.pkl; z = fl(-nu lnx) as TOMS forms it),
two cells: A = GAMMA.DIST(z, b, 1, TRUE) (Excel's P) and B = CHISQ.DIST.RT(2z, 2b) (Excel's directly computed Q;
statistics.md T-1778: CHISQ.DIST.RT(x, k) = Q(k/2, x/2), 2z and 2b exact). Doubles are written exactly (w2-forensics
enc). Meta: t2512-r3-bgrat-qgamma.meta.json = [[row, b, z], ...].
usage: make_probe.py [--write]"""
from __future__ import annotations

import pickle
import sys

from common import HERE, w2p, write_probe
from exact import zdouble

NAME = "t2512-r3-bgrat-qgamma"

d = pickle.load(open(HERE / "calls.pkl", "rb"))
pairs = sorted({(c[1], zdouble(c[0], c[1], c[5])) for c in d["calls"].values() if c[1] != 0.5})
cells, meta = {}, []
for i, (b, z) in enumerate(pairs, 1):
    cells[f"A{i}"] = f"=GAMMA.DIST({w2p.num(z)},{w2p.num(b)},1,TRUE)"
    cells[f"B{i}"] = f"=CHISQ.DIST.RT({w2p.num(2 * z)},{w2p.num(2 * b)})"
    meta.append((i, b, z))
write_probe(NAME, "T-2512 r3-bgrat: Excel's incomplete gamma P and Q at the (b, z) of every bgrat call with b != 1/2", cells, meta, "--write" in sys.argv, extra_tags=("incomplete-gamma",))
