"""Upper-side compositions: T.DIST(t > 0, TRUE) and T.DIST.RT(t < 0) from the kernel's (w, w1) at |t|.
Candidates (I = the kernel's lower tail I_y(n/2, 1/2), J = its computed complement 1 - I as the kernel returns it):
  1-L      fl(1 - I/2)
  .5-L+.5  (0.5 - I/2) + 0.5
  .5+J/2   fl(0.5 + J/2)
usage: compose.py [spec]  (default: the r2-tdist best spec)"""
import sys
from collections import defaultdict

import tails
import tlib
import tscore
import xlk

SPEC = sys.argv[1] if len(sys.argv) > 1 else "args=t2,alnrel=xl,erfc=xl,bg_lnx=cut0.01,bg=ideal"
args, cfg = tscore.parse(SPEC)
rows = [r for r in tlib.load(("rt", "cdf")) if r["t"] != 0.0 and (r["kind"] == "cdf") == (r["t"] > 0)]
for r in rows:
    r["cr"] = tlib.exact_cell(r)
pts = [tails.kargs(args, abs(r["t"]), r["n"]) for r in rows]
res = xlk.run([p for p, _ in pts], cfg)
out = defaultdict(list)
for r, (p, fl), (w, w1, b) in zip(rows, pts, res):
    I, J = (w1, w) if fl else (w, w1)
    c = {"1-L": 1.0 - I / 2, ".5-L+.5": (0.5 - I / 2) + 0.5, ".5+J/2": 0.5 + J / 2, "CR": r["cr"]}
    for k, v in c.items():
        out[(k, "all")].append((v, r["excel"]))
        out[(k, b)].append((v, r["excel"]))
for k in sorted(out, key=str):
    print(tlib.line(str(k), out[k]))
