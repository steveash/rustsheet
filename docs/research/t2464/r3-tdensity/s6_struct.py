"""Free-constant structure scan on the in-sample probes a1/a2 (runs and exact blocks): one constant per df shared by
all t blocks.  For each structure (tdlib.struct_out) and df: deepest overlap of the per-cell constant intervals.
usage: s6_struct.py name [name ...]"""
import sys
from collections import defaultdict

import tdlib as TD

L = TD.LDB
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
by = defaultdict(list)
for r in rows:
    by[r["n"]].append(r)
for name in sys.argv[1:]:
    tot = [0, 0]
    out = []
    for n in sorted(by):
        rr = by[n]
        C0 = TD.ld_of(TD.struct_C0(name, n))
        val = name.split("_")[1] in ("K", "Ke")
        ivs = []
        blks = []
        for r in rr:
            p = TD.pieces(r["t"], n)
            if val:
                F = lambda u, p=p: TD.struct_out(name, p, C0 * (L(1) + L(u) * TD.U53))  # noqa: E731
            else:
                F = lambda u, p=p: TD.struct_out(name, p, C0 + L(u) * TD.U53)  # noqa: E731
            ivs.append(TD.interval(F, r["excel"], W=48.0))
            blks.append(r["blk"])
        nb, pos = TD.best_overlap(ivs)
        per = defaultdict(lambda: [0, 0])
        for iv, b in zip(ivs, blks):
            per[b][1] += 1
            per[b][0] += any(lo <= pos <= hi for lo, hi in iv)
        tot[0] += nb
        tot[1] += len(rr)
        out.append(f"n={n:g} {nb}/{len(rr)} @{pos:+.2f} [" + " ".join(f"{v[0]}" for b, v in sorted(per.items())) + "]")
    print(f"{name}: total {tot[0]}/{tot[1]} | " + "; ".join(out), flush=True)
