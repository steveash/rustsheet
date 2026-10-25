"""F at d1 = 2 (a = 1) and d2 = 2 (b = 1): does the closed form hold, as a function of the small argument?"""
import math, sys
sys.path.insert(0, '.')
import w5lib as L
from dispatch import bratio_branch
rows = []
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    x, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not x > 0 or (d1 != 2 and d2 != 2) or d1 == d2: continue
    p = d1 * x; w = 1 / (1 + d2 / p); v = 1 / (1 + p / d2)
    a, b = d1 / 2, d2 / 2
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    br, ind, comp = bratio_branch(a, b, w, v)
    # closed forms on the original problem: a = 1: lower = 1 - y^b, upper = y^b ; b = 1: lower = x^a
    if d1 == 2:
        P = L.EXP(b * L.LN(v)); low, up = (0.5 - P) + 0.5, P
        P2 = L.EXP(b * L.LN(1 - w)); low2, up2 = (0.5 - P2) + 0.5, P2
    else:
        P = L.EXP(a * L.LN(w)); low, up = P, (0.5 - P) + 0.5
        low2, up2 = low, up
    val = c["excel"]
    m = low if tail == "lower" else up
    m2 = low2 if tail == "lower" else up2
    ex = float(L.ibeta(w, a, b)) if w <= 0.5 else float(1 - L.ibeta(v, b, a))
    exv = ex if tail == "lower" else (float(1 - L.ibeta(w, a, b)) if w <= 0.5 else float(L.ibeta(v, b, a)))
    rows.append((d1 == 2, w, tail, comp, br, ind, (m - val) / math.ulp(val), (m2 - val) / math.ulp(val), (exv - val) / math.ulp(val), d1, d2, x))
L.save_cache()
for r in sorted(rows):
    print(("a=1" if r[0] else "b=1"), f"w={r[1]:.4g} {r[2]:5s} computed={r[3]:5s} {r[4]:20s} ind={r[5]} cf(v)={r[6]:+.0f} cf(1-w)={r[7]:+.0f} exact={r[8]:+.1f}  d={r[9]:g},{r[10]:g} x={r[11]}")
L.save_cache()
