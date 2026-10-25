"""a0 = 1, x0 < 0.1: does the closed form hold when bratio would not take bpser (x0*b0 > 0.7)?  And the cut near 0.1."""
import math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L
from closed_final import items
res = defaultdict(list); near = []
for src, c, x, y, a, b, side, ex in items():
    swap = x > 0.5
    a0, b0, x0 = (b, a, y) if swap else (a, b, x)
    if a0 != 1 or b0 == 1: continue
    w1 = L.EXP(b0 * L.LN(1 - x0)); w = (0.5 - w1) + 0.5
    lower, upper = (w1, w) if swap else (w, w1)
    m = lower if side == "lower" else upper
    exact = float(L.ibeta(x, a, b) if x <= 0.5 else 1 - L.ibeta(y, b, a))
    exact = exact if side == "lower" else (float(1 - L.ibeta(x, a, b)) if x <= 0.5 else float(L.ibeta(y, b, a)))
    if x0 < 0.1:
        k = ("x0<0.1", "x0*b0>0.7" if x0 * b0 > 0.7 else "x0*b0<=0.7")
        res[k + ("closed",)].append((m, ex)); res[k + ("exact",)].append((exact, ex))
    if 0.08 < x0 < 0.13:
        near.append((x0, src, m == ex, exact == ex, b0))
for k in sorted(res): print(k, L.fmt(L.score(res[k])))
xs_closed = [r[0] for r in near if r[2] and not r[3]]
xs_not = [r[0] for r in near if not r[2]]
print("largest x0 < 0.1 that misses the closed form:", max([v for v in xs_not if v < 0.1], default=None),
      " smallest x0 >= 0.1 closed-only:", min([v for v in xs_closed if v >= 0.1], default=None))
print("x0 < 0.1 that match closed but not exact:", sorted(v for v in xs_closed if v < 0.1)[:10])
L.save_cache()
print("--- 0.0999 < x0 < 0.1")
for src, c, x, y, a, b, side, ex in items():
    swap = x > 0.5
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if a0 != 1 or b0 == 1 or not (0.0995 < x0 < 0.1005): continue
    w1 = L.EXP(b0 * L.LN(1 - x0)); w = (0.5 - w1) + 0.5
    lower, upper = (w1, w) if swap else (w, w1)
    m = lower if side == "lower" else upper
    print(src, c["fn"], c["args"], f"x0={x0!r} y0={y0!r} b0={b0} closed={'ok' if m == ex else f'{(m-ex)/math.ulp(ex):+.0f}'}")
