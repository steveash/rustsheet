"""TOMS 708 bratio branch map with Excel's own bpser (w3 model) in the bpser branches.

For every BETA.DIST cdf cell (0 < x < 1, closed forms a = 1 / b = 1 excluded) follow bratio's dispatch
(R toms708.c, Didonato & Morris 1992) and evaluate:
  L_w_bpser  : w = bpser(a0, b0, x0)                        (w3 model: EXP(z) * s, s accumulated from 1/a)
  L_w1_bpser : w1 = bpser(b0, a0, y0), w = 0.5 - w1 + 0.5   (x87 stored)
  other branches (fpser, apser, bup+bgrat, bgrat, bfrac, basym): correctly rounded value (not modelled here)
then undo the swap (w <-> w1). y = fl(1 - x). Scores per branch, against the correctly rounded value.
Usage: python bratio_map.py [z53|z64]
"""

import collections
import sys

from lib import *
from model import model

M = mpmath.mpf
ZM = sys.argv[1] if len(sys.argv) > 1 else "z53"
from dispatch import EPS, branch  # noqa: E402


rows = [r for r in load(cum=True) if r["a"] != 1.0 and r["b"] != 1.0 and max(r["a"], r["b"]) <= 1e4]
E = np.array([r["excel"] for r in rows])
cr = np.array([float(exact_cdf(r["x"], r["a"], r["b"])) for r in rows])
save_exact()
info = []
for r in rows:
    y = float(sub(1.0, r["x"]))
    info.append(branch(r["a"], r["b"], r["x"], y))
pred = cr.copy()
for kind in ("w_bpser", "w1_bpser"):
    idx = [i for i, t in enumerate(info) if t[0] == kind]
    if not idx:
        continue
    if kind == "w_bpser":
        aa, bb, xx = (np.array([info[i][k] for i in idx]) for k in (1, 2, 3))
    else:
        aa, bb, xx = (np.array([info[i][k] for i in idx]) for k in (2, 1, 4))
    t = model(xx, aa, bb, big=8.0, zmode=ZM)
    comp = add(sub(0.5, t), 0.5)  # 0.5 - t + 0.5, x87 stored
    w, w1 = (t, comp) if kind == "w_bpser" else (comp, t)
    sw = np.array([info[i][5] for i in idx])
    pred[idx] = np.where(sw, w1, w)  # bratio swaps w and w1 back at the end
tune, hold = split_half(rows)
tid = {id(r) for r in tune}
g = collections.defaultdict(list)
for i, (r, t) in enumerate(zip(rows, info)):
    g[(t[0], "swapped" if t[5] else "direct", "min<=1" if min(r["a"], r["b"]) <= 1 else "a,b>1")].append(i)
lines = [
    f"bratio branch map, Excel bpser = w3 model ({ZM}); other branches = correctly rounded. "
    f"exact/within4/within15 (hold-out half in brackets)"
]
for k in sorted(g):
    idx = np.array(g[k])
    ih = np.array([i for i in idx if id(rows[i]) not in tid])
    sm, sc = score(pred[idx], E[idx]), score(cr[idx], E[idx])
    hm = score(pred[ih], E[ih]) if len(ih) else (0, 0, 0, 0)
    hc = score(cr[ih], E[ih]) if len(ih) else (0, 0, 0, 0)
    lines.append(
        f"  {k[0]:10s} {k[1]:8s} {k[2]:7s} n {len(idx):6d}  model {fmt(sm):26s} [{hm[0]}/{hm[3]}]   CR {fmt(sc):26s} [{hc[0]}/{hc[3]}]"
    )
allm, allc = score(pred, E), score(cr, E)
lines.append(f"  ALL cdf cells: model {fmt(allm)}   CR {fmt(allc)}")
txt = "\n".join(lines)
print(txt)
open(HERE / f"bratio_map_{ZM}.out", "w").write(txt + "\n")
np.save(HERE / f"bratio_pred_{ZM}.npy", pred)
