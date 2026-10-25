"""Region map in x for the w3 model on min(a,b)<1 direct cells (all sources): exact / within-4 by x bin and by
which shape is small; z53 and z64."""
import collections
from lib import *
from model import model
rows = [r for r in load(cum=True, branch="direct") if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1 and max(r["a"], r["b"]) <= 1e3]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
m53 = model(x, a, b, big=8.0); m64 = model(x, a, b, big=8.0, zmode="z64")
cr = np.array([float(exact_cdf(xi, ai, bi)) for xi, ai, bi in zip(x, a, b)])
bins = [0, 1e-6, 0.01, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0]
for side in ("a<1", "b<1"):
    for lo, hi in zip(bins[:-1], bins[1:]):
        idx = np.array([i for i, r in enumerate(rows) if lo <= r["x"] < hi and ((r["a"] < 1) == (side == "a<1"))])
        if len(idx) == 0: continue
        s53, s64, sc = score(m53[idx], E[idx]), score(m64[idx], E[idx]), score(cr[idx], E[idx])
        print(f"  {side} x in [{lo:g},{hi:g}) n {len(idx):5d}  z53 {s53[0]:5d}/{s53[1]:5d}  z64 {s64[0]:5d}/{s64[1]:5d}  CR {sc[0]:5d}/{sc[1]:5d}")
