"""v1 vs v2 variants: alx from extended ln (fl53(a*ln64 x)), accurate lnB for max(a,b) > 8. Per group scores."""
import sys, collections
from lib import *
from model import model
rows = [r for r in load(cum=True, branch="direct") if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
cr = np.array([float(exact_cdf(xi, ai, bi)) for xi, ai, bi in zip(x, a, b)])
groups = collections.defaultdict(list)
for i, r in enumerate(rows):
    k = r["block"] if r["block"] != "be" else ("be a<1" if r["a"] < 1 else "be b<1") + (" max<=8" if max(r["a"], r["b"]) <= 8 else " max>8")
    groups[k].append(i)
V = {"LN53+big8": dict(big=8.0), "z64+big8": dict(big=8.0, zmode="z64"), "z64 ln64 big8": dict(big=8.0, zmode="z64", alx_mode="ln64")}
res = {k: model(x, a, b, **kw) for k, kw in V.items()}
tune, hold = split_half(rows); tid = {id(r) for r in tune}
ih = np.array([i for i, r in enumerate(rows) if id(r) not in tid])
print(f"{'group':22s} {'n':>5s}  " + "  ".join(f"{v:>14s}" for v in list(V) + ["CR"]))
for g, idx in sorted(groups.items(), key=lambda t: -len(t[1])):
    idx = np.array(idx)
    print(f"{g:22s} {len(idx):5d}  " + "  ".join(f"{score(res[v][idx], E[idx])[0]:5d}/{score(res[v][idx], E[idx])[2]:5d}   " for v in V) + f"  {score(cr[idx], E[idx])[0]:5d}/{score(cr[idx], E[idx])[2]:5d}")
for v in V:
    print(f"  {v:12s} all {fmt(score(res[v], E))}   hold {fmt(score(res[v][ih], E[ih]))}")
print(f"  {'CR':12s} all {fmt(score(cr, E))}   hold {fmt(score(cr[ih], E[ih]))}")
save_exact()
