"""apser cells (probe tiny): what bratio does when apser is skipped (a0 = b tiny, b0 = a, x0 = y):
x0 >= 0.3 -> w1 = bpser(b0, a0, y0); b0 > 15 -> bgrat; else bup(b0, a0, y0, x0, 20) + bgrat(b0 + 20, ...).
Prints relative errors against Excel for TOMS apser (BEST), this no-apser dispatch, and CR."""
from collections import Counter

import lib

rows = [r for r in lib.load_probe(("tiny",)) if r["blk"] == "apser"]
mn = lib.Model(lib.bx.BEST, lib.V(noapser=True))
eps = lib.EPS
c = Counter()
for r in sorted(rows, key=lambda r: (r["a"], r["x"])):
    a, b, x, e = r["a"], r["b"], r["x"], r["excel"]
    x0 = 1.0 - x
    v = mn.apser(b, a, x0, eps)  # lib.Model with noapser=True: bratio's fall-through dispatch
    path = "w1_bpser" if x0 >= 0.3 else ("bgrat" if a > 15 else "bup+bgrat")
    t = lib.kernel(a, b, x, 1.0 - x)[0]
    band = "y<0.04" if x0 < 0.04 else "y<0.25" if x0 < 0.25 else "y>=0.25"
    for nm, val in (("apser", t), ("noapser", v), ("CR", r["cr"])):
        c[(band, nm)] += lib.sl.w15(val, e)
    c[(band, "n")] += 1
    print(f"a={a:<7g} b={b:<9.3g} y={x0:<7.4f} {path:9s} rel(noapser-e)={(v - e) / e:+.2e} rel(apser-e)={(t - e) / e:+.2e} "
          f"rel(cr-e)={(r['cr'] - e) / e:+.2e}")
print("w15 counts:", dict(sorted(c.items())))
