"""verify-r3-loader-const: is Excel's density in the image of EXP over doubles?

For a stored (double) exponent E, Excel's value y must equal EXP(E) for some double E. For each a, b > 1 pdf cell this
scans the doubles E within +-SPAN steps of fl(ln y) with the engine harness's EXP (bit-exact model V6) and reports,
binned by |ln y|, how many cells are reachable. Controls: y' = EXP(nextafter(fl(ln y))) (always reachable); y + j ulp,
j = +-1..8 (the local image density: chance level); and the M4 model's own predictions with extended E (what an
extended exponent gives through EXP V6, which is not the same as chance).
Usage: vimage.py [CASE_PREFIX]
"""

from __future__ import annotations

import math
import sys
from collections import defaultdict

import vdata
import vscore

SPAN = 40
CHAINS = ("M4-Cd", "M4-CdXd", "M4-L5x", "M4-hx", "M4-lcx")


def neighbours(e0: float):
    out, lo, hi = [e0], e0, e0
    for _ in range(SPAN):
        lo, hi = math.nextafter(lo, -math.inf), math.nextafter(hi, math.inf)
        out += [lo, hi]
    return out


def reachable(ys):
    es, owner = [], []
    for i, y in enumerate(ys):
        for e in neighbours(math.log(y)):
            es.append(e)
            owner.append(i)
    got = [False] * len(ys)
    vals = []
    for j in range(0, len(es), 200000):
        vals += vscore.VM.excelmath.excel("EXP", es[j : j + 200000])
    for i, v in zip(owner, vals):
        got[i] |= v == ys[i]
    return got


def shift(y: float, j: int) -> float:
    for _ in range(abs(j)):
        y = math.nextafter(y, math.inf if j > 0 else -math.inf)
    return y


def main():
    pre = sys.argv[1] if len(sys.argv) > 1 else ""
    rows, _ = vdata.load_all()
    rows, _, _ = vscore.dedup(rows)
    rows = [r for r in rows if r["case"].startswith(pre) and r["excel"] > 0]
    ys = [r["excel"] for r in rows]
    ctl = vscore.VM.excelmath.excel("EXP", [math.nextafter(math.log(y), math.inf) for y in ys])
    g_y, g_c = reachable(ys), reachable(ctl)
    g_n = [0.0] * len(ys)  # image density: share of y + j ulp (j = +-1..8) that are reachable
    for j in [*range(1, 9), *range(-8, 0)]:
        for i, ok in enumerate(reachable([shift(y, j) for y in ys])):
            g_n[i] += ok / 16
    P = vscore.predict(rows)
    g_m = reachable([P[(r["x"], r["a"], r["b"])]["M4"] for r in rows])
    extra = {nm: reachable([P[(r["x"], r["a"], r["b"])][nm] for r in rows]) for nm in CHAINS}
    bins = defaultdict(lambda: [0, 0, 0, 0, 0] + [0] * len(CHAINS))
    for i, (r, a, c, n, m) in enumerate(zip(rows, g_y, g_c, g_n, g_m)):
        L = abs(math.log(r["excel"]))
        key = "<1" if L < 1 else "1-2" if L < 2 else "2-4" if L < 4 else "4-8" if L < 8 else ">=8"
        t = bins[key]
        t[0] += 1
        t[1] += a
        t[2] += c
        t[3] += n
        t[4] += m
        for j, nm in enumerate(CHAINS):
            t[5 + j] += extra[nm][i]
    print(
        f"cells {len(rows)} (prefix {pre!r}); per |ln y| bin: n, Excel y reachable, control EXP(double) reachable, "
        "chance, M4 prediction reachable"
    )
    for key in ("<1", "1-2", "2-4", "4-8", ">=8"):
        if key in bins:
            n, a, c, d, m, *ex = bins[key]
            more = "  ".join(f"{nm} {v}" for nm, v in zip(CHAINS, ex))
            print(f"  |E| {key:4s} n {n:6d}  Excel {a:6d}  control {c:6d}  chance {d:8.1f}  M4-extended {m:6d}  {more}")
    sel = [(r, a) for r, a in zip(rows, g_y) if (r["a"], r["b"]) == (45.2, 60.9)]
    print(f"base (45.2, 60.9) exactly: {len(sel)} cells, reachable {sum(a for _, a in sel)}")


if __name__ == "__main__":
    main()
