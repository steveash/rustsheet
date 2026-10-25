"""Reflected branch (x > a/(a+b)) for min(a,b) < 1: 1 - I_y(b, a) with y = fl(1 - x) and the w3 series model for the
small tail, final subtraction x87 stored; vs correctly rounded. Also w2 found a switch at x = 0.5 for min < 1."""
import collections
from lib import *
from model import model
rows = [r for r in load(cum=True, branch="reflected") if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1 and max(r["a"], r["b"]) <= 1e3]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
y = sub(1.0, x)
cr = np.array([float(exact_cdf(xi, ai, bi)) for xi, ai, bi in zip(x, a, b)])
out = {}
for zm in ("z53", "z64"):
    t = model(y, b, a, big=8.0, zmode=zm)
    out[f"1 - w3(y,b,a) {zm}"] = sub(1.0, t)
    out[f"w3(x,a,b) direct formula {zm}"] = model(x, a, b, big=8.0, zmode=zm)
out["CR"] = cr
g = collections.defaultdict(list)
for i, r in enumerate(rows):
    g[(r["block"] if r["block"] != "be" else "be") + (" x<=0.5" if r["x"] <= 0.5 else " x>0.5")].append(i)
for k, idx in sorted(g.items()):
    idx = np.array(idx)
    print(f"  {k:22s} n {len(idx):5d}  " + "  ".join(f"{nm}: {score(v[idx], E[idx])[0]}/{score(v[idx], E[idx])[2]}" for nm, v in out.items()))
for nm, v in out.items():
    print(f"  all {nm:28s} {fmt(score(v, E))}")
