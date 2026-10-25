"""Per x-run block (fixed a, b): scan lnB over doubles within +-K ulps of the exact lnB, with the exponent
E = ((a-1)*LN(x) + (b-1)*L1) - lnB stored x87 and pdf = Excel EXP(E). If one lnB double makes the block bit-exact,
the x-dependent part of the density is solved there and only lnB(a,b) is left. L1 = LN(fl(1-x)) or log1p(-x)."""
import collections, math, sys
import numpy as np, mpmath
from common import *
from pdf_cands import blocks
g = blocks()
K = int(sys.argv[1]) if len(sys.argv) > 1 else 40
keys = [k for k in g if k.startswith("xruns")]
for k in sorted(keys):
    pts = g[k]; a, b = pts[0][1], pts[0][2]
    xs = [p[0] for p in pts]; ex = [p[3] for p in pts]
    LNx = xl("LN", xs); LN1 = xl("LN", [1 - x for x in xs])
    lb0 = float(exact_lnbeta(a, b))
    cands = [lb0]
    v = lb0
    for _ in range(K): v = math.nextafter(v, math.inf); cands.append(v)
    v = lb0
    for _ in range(K): v = math.nextafter(v, -math.inf); cands.append(v)
    best = []
    for l1name in ("LN(fl(1-x))", "log1p"):
        T = [x87("+", x87("*", a - 1, lx), x87("*", b - 1, (l1 if l1name != "log1p" else math.log1p(-x)))) for x, lx, l1 in zip(xs, LNx, LN1)]
        E = [x87("-", t, lb) for lb in cands for t in T]
        V = xl("EXP", E)
        n = len(xs)
        for i, lb in enumerate(cands):
            c = sum(V[i * n + j] == ex[j] for j in range(n))
            best.append((c, l1name, (lb - lb0) / math.ulp(lb0)))
    best.sort(reverse=True)
    print(f"{k:12s} a={a} b={b} n={len(xs)} best: " + " | ".join(f"{c} {l} lnB=exact{d:+.0f}ulp" for c, l, d in best[:3]))
