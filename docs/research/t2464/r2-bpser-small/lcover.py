"""Ceiling of any constant-L model with w3's series: per shape, the best L (sweep over the cells' implied L intervals,
lscan.l_interval) and how many cells it covers; then where the candidate L forms sit relative to that best L.
usage: lcover.py [sets=runs] [min=50] [series=order:form (lib.series)] [--brief]"""
import sys
from collections import defaultdict

import mpmath

import lib
import lscan

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
sets = tuple(kw.get("sets", "runs").split(","))
old = [s for s in sets if s not in lib.PROBES]
rows = (lib.sl.load(tuple(old)) if old else []) + lib.load_probe(tuple(s for s in sets if s in lib.PROBES))
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], lib.V())
shapes = defaultdict(list)
for r, q in zip(rows, res):
    if q[2] == "w_bpser" and r["side"] == "lower":
        fn, a, b, x = q[3][-1]
        shapes[(a, b)].append((x, r["excel"]))
for (a, b), cells in sorted(shapes.items(), key=lambda kv: -len(kv[1])):
    if len(cells) < int(kw.get("min", "50")):
        continue
    ivs = [iv for iv in (lscan.l_interval(a, b, x, e, kw.get("series")) for x, e in cells) if iv]
    ev = sorted([(lo, 0) for lo, _ in ivs] + [(hi, 1) for _, hi in ivs])
    cur = best = 0
    bl = None
    for i, (v, t) in enumerate(ev):
        cur += 1 if t == 0 else -1
        if cur > best:
            best, bl = cur, (v, ev[i + 1][0])
    c = lscan.candidates(a, b)
    acc = c["acc"]
    U = lscan.U
    mid = (bl[0] + bl[1]) / 2
    cov = {k: sum(lo <= v <= hi for lo, hi in ivs) for k, v in c.items()}
    print(f"{lib.sub_branch('bpser', a, b):13s} a={a} b={b} n={len(cells)} inverted={len(ivs)} best L covers {best} "
          f"at L-acc={float((mid - acc) / U):+.3f} (width {float((bl[1] - bl[0]) / U):.3f})")
    if "--brief" not in sys.argv:
        print("    candidate L-acc / cells covered: " + "  ".join(f"{k}={float((v - acc) / U):+.2f}/{cov[k]}" for k, v in c.items()))
