"""For the s4 chain at its best h: per cell, Excel's admissible stored-E doubles vs the model's pre-rounding
L5 + g (in ulps of E), and the model's lc and g roundings, to locate the missing rounding.  Usage: BLOCK knobs N"""
import math
import sys
import mpmath
import dlib as L
import s4_chain as C4

blk, spec = sys.argv[1], sys.argv[2]
N = int(sys.argv[3]) if len(sys.argv) > 3 else 40
pre = C4.prep(blk, spec, "x")
best, ho, nod, hb = C4.run(blk, spec, idx_lc=pre, quiet=True)
idx, a, b, k, n, lc = pre
L5 = mpmath.mpf(L.LN(a + b - 1))
rows = C4.rows
miss = 0
for i in sorted(idx, key=lambda i: rows[i]["x"])[:N]:
    r = rows[i]
    gx = lc[i] - hb
    g = mpmath.mpf(L.D(gx))
    pe = L5 + g
    e0 = float(mpmath.log(r["excel"]))
    u = math.ulp(e0)
    ok = [j for j in range(-8, 9) if L.exp_v6(mpmath.mpf(e0 + j * u)) == r["excel"]]
    pos = float((pe - mpmath.mpf(e0)) / u)
    gerr = float((g - gx) / math.ulp(float(g)))
    hit = min(ok) - 0.5 <= pos <= max(ok) + 0.5
    print(f"x={r['x']!r:22s} g={float(g):+.6f} gerr={gerr:+.2f}ulp(g) L5+g at {pos:+6.2f} ulp(E), ok {ok} {'' if hit else 'MISS'}")
