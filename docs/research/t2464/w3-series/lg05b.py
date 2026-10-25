"""Force Excel's internal lnGamma(0.5) to the double above the correctly rounded one (0.5723649429247002) and
score z53/z64 on the blocks and the beta_exact cells that involve 0.5 as a or b."""
import collections
import model as Mo
from lib import *
rows = [r for r in load(cum=True, branch="direct") if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1 and 0.5 in (r["a"], r["b"])]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
groups = collections.defaultdict(list)
for i, r in enumerate(rows):
    groups[r["block"] if r["block"] != "be" else ("be a=.5" if r["a"] == 0.5 else "be b=.5")].append(i)
res = {}
for ov in (None, 0.5723649429247002, 0.5723649429247001):
    Mo.OVERRIDE.clear()
    if ov: Mo.OVERRIDE[0.5] = ov
    for zm in ("z53", "z64"):
        res[(ov, zm)] = Mo.model(x, a, b, big=8.0, zmode=zm)
for g, idx in groups.items():
    idx = np.array(idx)
    print(f"{g:14s} n {len(idx):5d} " + "  ".join(f"{str(k[0])[-4:] if k[0] else 'ex64'}/{k[1]}: {int((v[idx] == E[idx]).sum())}" for k, v in res.items()))
print("total", {f"{str(k[0])[-4:] if k[0] else 'ex64'}/{k[1]}": int((v == E).sum()) for k, v in res.items()}, "of", len(E))
