import math, sys
sys.path.insert(0, '.')
import w5lib as L
from closed_final import items, closed_value
for src, c, x, y, a, b, side, ex in items():
    m = closed_value(a, b, x, y, side)
    if m is None or m == ex: continue
    swap = x > 0.5
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    alt = {}
    if a0 == 1:
        for nm, l in (("LN(y0)", L.LN(y0)), ("LN(1-x0)", L.LN(1 - x0))):
            w1 = L.EXP(b0 * l)
            for cn, w in (("1-w1", 1 - w1), ("(0.5-w1)+0.5", (0.5 - w1) + 0.5)):
                lower, upper = (w1, w) if swap else (w, w1)
                alt[nm + "," + cn] = (lower if side == "lower" else upper)
    print(src, c["fn"], c["args"], f"x0={x0!r} a0={a0} b0={b0} swap={swap} {side} ex={ex!r} model={(m-ex)/math.ulp(ex):+.0f}",
          {k: f"{(v-ex)/math.ulp(ex):+.0f}" for k, v in alt.items()})
