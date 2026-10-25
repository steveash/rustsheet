"""Per-block scores of the prefactor x exact-S candidate for direct cdf cells (closed forms excluded)."""
import collections, sys
from lib import *
M = mpmath.mpf
rows = [r for r in load(cum=True, branch="direct") if r["a"] != 1.0 and r["b"] != 1.0]
sel = sys.argv[1] if len(sys.argv) > 1 else "a<1"
if sel == "a<1": rows = [r for r in rows if r["a"] < 1]
elif sel == "b<1": rows = [r for r in rows if r["b"] < 1 <= r["a"]]
else: rows = [r for r in rows if min(r["a"], r["b"]) > 1]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
Sx = [i / (M(xi) ** M(ai) / (M(ai) * mpmath.beta(M(ai), M(bi)))) for i, xi, ai, bi in zip(I, x, a, b)]
apb = add(a, b); ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", apb)
alx = mul(a, xl("LN", x))
P = div(xl("EXP", sub(alx, add(ga, sub(gb, gab)))), a)
m = np.array([float(M(p) * s) for p, s in zip(P, Sx)])
cr = np.array([float(v) for v in I])
g = collections.defaultdict(list)
for i, r in enumerate(rows):
    key = r["block"] if r["block"] != "be" else f"be a={r['a']:g} b={r['b']:g}" if sel != "x" else "be"
    if r["block"] == "be" and sel == "a<1": key = f"be a={r["a"]:g} b={r["b"]:g}"
    g[key].append(i)
for k in sorted(g, key=lambda k: -len(g[k])):
    idx = g[k]
    s1, s0 = score(m[idx], E[idx]), score(cr[idx], E[idx])
    print(f"  {k:28s} n {len(idx):5d}  model {s1[0]:5d}/{s1[1]:5d}/{s1[2]:5d}   CR {s0[0]:5d}/{s0[1]:5d}/{s0[2]:5d}")
